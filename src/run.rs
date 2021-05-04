use std::ffi::{CStr, CString, NulError, OsStr};
use std::fs;
use std::io::Error as IOError;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use nix::errno::Errno;
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::stat::{self, mknod, stat, Mode, SFlag};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, getgid, getuid, Gid, Pid, Uid};
use nix::Error as SysError;

use tempfile::Builder as TempBuilder;

use home::home_dir;

use crate::config::{Config, NetworkConfig};
use crate::container::{Container, Error as ContainerError};
use crate::strategy::{ExposedPath, Strategy};
use crate::utils::PathMerge;

#[derive(Debug)]
pub enum RunError {
    Dependencies(ContainerError),
    System(SysError),
    TempDir(IOError),
    Mount(SysError),
    Wait(SysError),
    Chroot(SysError),
    Fork(SysError),
    Stat(SysError),
    Exec(SysError),

    Collect(IOError),
    AsRoot(IOError),

    Mkdir,
    GuessHomeDir,

    /// Command includes null bytes in the middle
    Nul(NulError),
}

pub fn run<'e, I: Iterator<Item = &'e str>>(
    container: Container,
    config: Config,
    command: Option<&mut I>,
    run_as_root: bool,
) -> Result<i32, RunError> {
    // Here is how it's gonna go:
    //   - First process is going to do the workdir
    //   - Second process is going to unshare
    //   - Third is going to mount and then exit
    //
    // The purpose is for the second process to exit, and remove its mount via namespace deletion,
    // just so the first process to be able to collect the tempdir
    let working_dir = TempBuilder::new()
        .prefix("laurn.")
        .tempdir()
        .map_err(RunError::TempDir)?;

    let working_dir_path = working_dir.path();

    // First fork
    match unistd::fork() {
        Ok(unistd::ForkResult::Parent { child, .. }) => {
            let res = wait_child(child);

            // The temp directory should only be collected once our child process returned and the
            // namespace is deleted
            drop(working_dir);
            res
        }
        Ok(unistd::ForkResult::Child) => {
            let res = run_unshare(container, working_dir_path, config, command, run_as_root);

            // This is not our responsability to destroy working_directory
            std::mem::forget(working_dir);

            res
        }
        Err(e) => {
            eprintln!("Fork failed");
            Err(RunError::Fork(e))
        }
    }
}

type UidGid = (Uid, Gid);

fn get_outside_id() -> UidGid {
    let uid = getuid();
    let gid = getgid();

    (uid, gid)
}

fn fake_root(ug: UidGid) -> Result<(), RunError> {
    let (uid, gid) = ug;

    fs::write("/proc/self/setgroups", b"deny").map_err(RunError::AsRoot)?;
    fs::write(
        "/proc/self/uid_map",
        format!("0 {} 1", uid.as_raw()).as_bytes(),
    )
    .map_err(RunError::AsRoot)?;
    fs::write(
        "/proc/self/gid_map",
        format!("0 {} 1", gid.as_raw()).as_bytes(),
    )
    .map_err(RunError::AsRoot)?;

    Ok(())
}

fn run_unshare<'e, I: Iterator<Item = &'e str>>(
    container: Container,
    working_dir: &Path,
    config: Config,
    command: Option<&mut I>,
    run_as_root: bool,
) -> Result<i32, RunError> {
    let ug = if run_as_root {
        Some(get_outside_id())
    } else {
        None
    };

    let mut flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWUSER
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWIPC;
    // TODO(baloo): do we need a NEWUTS here?

    if config.laurn.network == NetworkConfig::Isolated {
        flags |= CloneFlags::CLONE_NEWNET;
    }
    unshare(flags).map_err(RunError::System)?;

    // Should we rewrite uids?
    if let Some(ug) = ug {
        fake_root(ug)?;
    }

    // Second fork
    match unistd::fork() {
        Ok(unistd::ForkResult::Parent { child, .. }) => wait_child(child),
        Ok(unistd::ForkResult::Child) => run_child(container, working_dir, config, command),
        Err(e) => {
            eprintln!("Fork failed");
            Err(RunError::Fork(e))
        }
    }
}

fn wait_child(child: Pid) -> Result<i32, RunError> {
    loop {
        match waitpid(child, None).map_err(RunError::Wait)? {
            WaitStatus::Exited(_pid, res) => {
                return Ok(res);
            }
            e => unimplemented!("unimplemented status check: {:?}", e),
        }
    }
}

fn run_child<'e, I: Iterator<Item = &'e str>>(
    container: Container,
    working_dir: &Path,
    config: Config,
    command: Option<&mut I>,
) -> Result<i32, RunError> {
    let project_dir = container.laurn_expr.parent().ok_or(RunError::Mkdir)?;

    let data: Option<&str> = None;

    let mode = stat::Mode::S_IRWXU
        | stat::Mode::S_IRGRP
        | stat::Mode::S_IXGRP
        | stat::Mode::S_IROTH
        | stat::Mode::S_IXOTH;

    let fmode =
        stat::Mode::S_IRUSR | stat::Mode::S_IWUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH;

    let dependencies = container.references().map_err(RunError::Dependencies)?;

    // First mount the nix dependencies (and the main shell "entrypoint"), read-only
    for dep in dependencies.iter() {
        let dep = NixPath(dep.as_path());

        dep.mount(working_dir, project_dir, mode, fmode, MountMode::RO)?;
    }

    let resolv = PathBuf::from("/etc/resolv.conf");
    let dep = NixPath(resolv.as_path()); // Meh, hackish
                                         // TODO(baloo): on github, resolv.conf can't be be remounted, mount it RW for now as it's out
                                         // of reach anyway
    dep.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;

    // Then mount the project itself
    let project = ProjectPath(project_dir);
    project.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;

    // Depending on the configuration, we want to expose things from $HOME or project other things
    // (the laurn config itself, git, ...)
    let protected_paths = Strategy::from(config.laurn.mode);
    for ro_path in protected_paths.ro_paths.iter() {
        ro_path.mount(working_dir, project_dir, mode, fmode, MountMode::RO)?;
    }
    for rw_path in protected_paths.rw_paths.iter() {
        rw_path.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;
    }

    // Mount things required to run processes
    let filesystems = vec![
        working_dir.join("proc"),
        working_dir.join("sys"),
        working_dir.join("dev"),
        working_dir.join("dev/pts"),
        working_dir.join("dev/shm"),
    ];
    for fs in filesystems {
        unistd::mkdir(&fs, mode).map_err(RunError::Mount)?;
    }

    let devices = vec![
        Dev("/dev/null"),
        Dev("/dev/zero"),
        Dev("/dev/full"),
        Dev("/dev/random"),
        Dev("/dev/urandom"),
        Dev("/dev/tty"),
        Dev("/dev/console"),
    ];
    for dev in devices.iter() {
        // /dev items needs to be bind-mounted from host because we run in user-namespace and we
        // can't mknod.
        dev.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;
    }

    // Only root can mount sysfs, we need to bindmount that
    let sysfs = Dev("/sys");
    // TODO: we get EPERM when trying to remount readonly, maybe worth figuring out why
    sysfs.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;

    // And then just chroot and run from there
    unistd::chroot(working_dir).map_err(RunError::Chroot)?;
    unistd::chdir(project_dir).map_err(RunError::Chroot)?;

    let mount_flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    let proc_dir = Path::new("/proc");
    mount(Some("proc"), proc_dir, Some("proc"), mount_flags, data).map_err(RunError::Mount)?;

    let mount_flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    let devpts_dir = Path::new("/dev/pts");
    let devpts_data = Some("mode=620,ptmxmode=666");
    mount(
        Some("devpts"),
        devpts_dir,
        Some("devpts"),
        mount_flags,
        devpts_data,
    )
    .map_err(RunError::Mount)?;

    // `/dev/ptmx`. A bind-mount or symlink of the container's /dev/pts/ptmx.
    mknod("/dev/ptmx", SFlag::S_IFREG, fmode, 0).map_err(RunError::Mount)?;
    let mount_flags = MsFlags::MS_BIND | MsFlags::MS_PRIVATE | MsFlags::MS_REC;
    let empty_fs: Option<&str> = None;
    mount(
        Some("/dev/pts/ptmx"),
        "/dev/ptmx",
        empty_fs,
        mount_flags,
        data,
    )
    .map_err(RunError::Mount)?;

    let mount_flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    let devshm_dir = Path::new("/dev/shm");
    let devshm_data = Some("size=65536k");
    mount(
        Some("shm"),
        devshm_dir,
        Some("tmpfs"),
        mount_flags,
        devshm_data,
    )
    .map_err(RunError::Mount)?;

    // Adapt the nix-shell wrapper
    let shell_wrapper: &OsStr = container.output.output.as_path().as_ref();
    let shell_wrapper = shell_wrapper.as_bytes();

    // Adapt the optional command to run
    let command: Vec<CString> = match command {
        Some(iter) => {
            let mut out = Vec::new(); // need with_capacity / size_hint?
            out.push(CString::new("laurn-shell").map_err(RunError::Nul)?);
            for el in iter {
                out.push(CString::new(el).map_err(RunError::Nul)?);
            }
            out
        }
        None => vec![], // Technically this is wrong, we should prefix with the shell wrapper here too, but bash is forgoving (and I'm lazy)
    };
    let command: Vec<&CStr> = command.iter().map(|s| s.as_c_str()).collect();

    unistd::execv(
        CString::new(shell_wrapper)
            .map_err(RunError::Nul)?
            .as_c_str(),
        &command,
    )
    .map_err(RunError::Exec)?;

    unreachable!("exec returned?");
}

fn mkdirp(target: &Path, mode: Mode) -> Result<(), RunError> {
    let mut cur = PathBuf::new();

    for part in target.components() {
        match part {
            Component::RootDir => cur.push("/"),
            Component::Normal(path) => {
                cur.push(path);
                match unistd::mkdir(cur.as_path(), mode) {
                    Ok(_) => continue,
                    Err(e) if e == SysError::Sys(Errno::EEXIST) => continue,
                    Err(e) => return Err(RunError::Mount(e)),
                }
            }
            _ => panic!("should not yield curdir or whatever"),
        }
    }

    Ok(())
}

trait Mount {
    fn mount(
        &self,
        root_dir: &Path,
        project_dir: &Path,
        mode: Mode,
        fmode: Mode,
        mount_mode: MountMode,
    ) -> Result<(), RunError>;
}

impl Mount for ExposedPath {
    fn mount(
        &self,
        root_dir: &Path,
        project_dir: &Path,
        mode: Mode,
        fmode: Mode,
        mount_mode: MountMode,
    ) -> Result<(), RunError> {
        let (source_path, target_path) = match *self {
            ExposedPath::Project(ref pp) => Ok((
                project_dir.merge(pp.as_path()),
                root_dir.merge(project_dir).merge(pp.as_path()),
            )),
            ExposedPath::UserHome(ref up) => {
                let home = home_dir().ok_or(RunError::GuessHomeDir)?;
                Ok((
                    home.as_path().merge(up),
                    root_dir.merge(home.as_path()).merge(up.as_path()),
                ))
            }
        }?;

        if !source_path.exists() {
            return Ok(());
        }

        if let Some(p) = target_path.parent() {
            mkdirp(p, mode)?;
        }

        mount_target(
            source_path.as_path(),
            target_path.as_path(),
            mode,
            fmode,
            mount_mode,
        )
    }
}

struct NixPath<'p>(&'p Path);

impl<'p> Mount for NixPath<'p> {
    fn mount(
        &self,
        root_dir: &Path,
        _project_dir: &Path,
        mode: Mode,
        fmode: Mode,
        mount_mode: MountMode,
    ) -> Result<(), RunError> {
        let (source_path, target_path) = (self.0, root_dir.merge(self.0));

        if let Some(p) = target_path.parent() {
            mkdirp(p, mode)?;
        }

        mount_target(source_path, target_path.as_path(), mode, fmode, mount_mode)
    }
}

struct ProjectPath<'p>(&'p Path);

impl<'p> Mount for ProjectPath<'p> {
    fn mount(
        &self,
        root_dir: &Path,
        _project_dir: &Path,
        mode: Mode,
        fmode: Mode,
        mount_mode: MountMode,
    ) -> Result<(), RunError> {
        let (source_path, target_path) = (self.0, root_dir.merge(self.0));

        if let Some(p) = target_path.parent() {
            mkdirp(p, mode)?;
        }

        mount_target(source_path, target_path.as_path(), mode, fmode, mount_mode)
    }
}

struct Dev<'p>(&'p str);

impl<'p> Mount for Dev<'p> {
    fn mount(
        &self,
        root_dir: &Path,
        _project_dir: &Path,
        mode: Mode,
        fmode: Mode,
        mount_mode: MountMode,
    ) -> Result<(), RunError> {
        let p = PathBuf::from(self.0);
        let (source_path, target_path) = (p.as_path(), root_dir.merge(p.as_path()));

        if let Some(p) = target_path.parent() {
            mkdirp(p, mode)?;
        }

        mount_target(source_path, target_path.as_path(), mode, fmode, mount_mode)
    }
}

fn mount_target(
    source_path: &Path,
    target_path: &Path,
    mode: Mode,
    fmode: Mode,
    mount_mode: MountMode,
) -> Result<(), RunError> {
    let info = stat(source_path).map_err(RunError::Stat)?;

    match SFlag::from_bits_truncate(info.st_mode) {
        SFlag::S_IFDIR => {
            match unistd::mkdir(target_path, mode) {
                Ok(_) => {}
                Err(e) if e == SysError::Sys(Errno::EEXIST) => {}
                Err(e) => return Err(RunError::Mount(e)),
            };
        }
        SFlag::S_IFREG | SFlag::S_IFCHR => {
            // mknod(2) can be used to create (empty) files, no need to open/close
            // in which case, dev is to be ignored (hence 0)
            match mknod(target_path, SFlag::S_IFREG, fmode, 0) {
                Ok(_) => {}
                // If target path already exist, then fine
                // if it's a directly we won't be able to mount a file atop of it
                // and it will fail on the mount below
                Err(e) if e == SysError::Sys(Errno::EEXIST) => {}
                Err(e) => return Err(RunError::Mount(e)),
            };
        }
        mode => {
            unimplemented!("unimplemented dependency of type {:?}", mode);
        }
    }

    match mount_mode {
        MountMode::RW => {
            let data: Option<&str> = None;
            let empty_fs: Option<&str> = None;

            let mount_flags = MsFlags::MS_BIND | MsFlags::MS_PRIVATE | MsFlags::MS_REC;
            mount(Some(source_path), target_path, empty_fs, mount_flags, data)
                .map_err(RunError::Mount)?;
        }
        MountMode::RO => {
            let data: Option<&str> = None;
            let empty_fs: Option<&str> = None;

            let mount_flags = MsFlags::MS_BIND | MsFlags::MS_PRIVATE | MsFlags::MS_REC;
            mount(Some(source_path), target_path, empty_fs, mount_flags, data)
                .map_err(RunError::Mount)?;

            // Mount need to be issued twice for readonly
            let mount_flags =
                MsFlags::MS_RDONLY | MsFlags::MS_REMOUNT | MsFlags::MS_PRIVATE | MsFlags::MS_BIND;
            let empty_source: Option<&str> = None;
            mount(empty_source, target_path, empty_fs, mount_flags, data)
                .map_err(RunError::Mount)?;
        }
    }

    Ok(())
}

enum MountMode {
    RW,
    RO,
}
