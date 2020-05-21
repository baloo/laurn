use std::ffi::{CString, OsStr};
use std::io::Error as IOError;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::stat::{self, stat, Mode, SFlag};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, Pid};
use nix::Error as SysError;

use tempfile::Builder as TempBuilder;

use home::home_dir;

use crate::config::Config;
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

    Mkdir,
    GuessHomeDir,
}

pub fn run(container: Container, config: Config) -> Result<i32, RunError> {
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
            let res = run_unshare(container, working_dir_path, config);

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

fn run_unshare(container: Container, working_dir: &Path, config: Config) -> Result<i32, RunError> {
    let flags = CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWPID;
    unshare(flags).map_err(RunError::System)?;

    // Second fork
    match unistd::fork() {
        Ok(unistd::ForkResult::Parent { child, .. }) => wait_child(child),
        Ok(unistd::ForkResult::Child) => run_child(container, working_dir, config),
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
                println!("child exited: {}", res);
                return Ok(res);
            }
            e => unimplemented!("unimplemented status check: {:?}", e),
        }
    }
}

fn run_child(container: Container, working_dir: &Path, config: Config) -> Result<i32, RunError> {
    let project_dir = container.laurn_expr.parent().ok_or(RunError::Mkdir)?;

    let data: Option<&str> = None;

    let mode = stat::Mode::S_IRWXU
        | stat::Mode::S_IRGRP
        | stat::Mode::S_IXGRP
        | stat::Mode::S_IROTH
        | stat::Mode::S_IXOTH;

    let fmode =
        stat::Mode::S_IRUSR | stat::Mode::S_IWUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH;

    let mut dependencies = container.references().map_err(RunError::Dependencies)?;
    dependencies.push(PathBuf::from("/etc/resolv.conf")); // Meh, hackish

    // First mount the nix dependencies (and the main shell "entrypoint"), read-only
    for dep in dependencies.iter() {
        let dep = NixPath(dep.as_path());

        dep.mount(working_dir, project_dir, mode, fmode, MountMode::RO)?;
    }

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
    let proc_dir = working_dir.join("proc");
    unistd::mkdir(&proc_dir, mode).map_err(RunError::Mount)?;

    let dev_dir = working_dir.join("dev");
    unistd::mkdir(&dev_dir, mode).map_err(RunError::Mount)?;

    let dev_dir = working_dir.join("tmp");
    unistd::mkdir(&dev_dir, mode).map_err(RunError::Mount)?;

    let devices = vec![
        Dev("/dev/null"),
        Dev("/dev/console"),
        Dev("/dev/random"),
        Dev("/dev/urandom"),
        Dev("/dev/tty"),
        Dev("/dev/zero"),
    ];
    for dev in devices.iter() {
        // /dev items needs to be bind-mounted from host because we run in user-namespace and we
        // can't mknod.
        dev.mount(working_dir, project_dir, mode, fmode, MountMode::RW)?;
    }

    // And then just chroot and run from there
    unistd::chroot(working_dir).map_err(RunError::Chroot)?;
    unistd::chdir(project_dir).map_err(RunError::Chroot)?;

    let mount_flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    let proc_dir = Path::new("/proc");
    mount(Some("proc"), proc_dir, Some("proc"), mount_flags, data).map_err(RunError::Mount)?;

    let command: &OsStr = container.output.output.as_path().as_ref();
    let command = command.as_bytes();
    unistd::execv(
        CString::new(command).expect("Cstring::failed").as_c_str(),
        &[],
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
            let fd = open(
                target_path,
                OFlag::O_CREAT | OFlag::O_CLOEXEC | OFlag::O_EXCL | OFlag::O_WRONLY,
                fmode,
            );

            match fd {
                Ok(fd) => {
                    unistd::close(fd).map_err(RunError::Mount)?;
                }
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
