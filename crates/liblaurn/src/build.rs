// nix-store --query --tree $(nix-store --query --outputs --force-realize $(nix-instantiate ./tests/basic/default.nix) 2>/dev/null )
//
// $ nix-build -E '(import <nixpkgs> {}).bashInteractive'
// /nix/store/v9i377m22afk3xybpwbq50yz29jark1r-bash-interactive-4.4-p23
//
// # nix-store --query -b python /nix/store/0ylzsywl6ybv9b7m99qdj0h742pvkyhg-llvm-7.1.0.drv
// /nix/store/6ax7mqlqn24wh48krna8wiinh2qycvzs-llvm-7.1.0-python
//
//

use std::ffi::OsStr;
use std::io::{Error as IoError, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Create a nix derivation that will add the dependencies from the `laurn.nix` derivation as well
/// as bash and bunch of other tools directly from nixpkgs.
///
/// This is to be consumed by nix-instanciate
fn source_input(laurn_shell_nix: &Path) -> String {
    format!(
        r#"
{{ system ? builtins.currentSystem }}:

let
  pkgs = import <nixpkgs> {{ inherit system; }};
  bash = (import <nixpkgs> {{}}).bashInteractive;
  origShell = (import {source_path});
in pkgs.stdenv.mkDerivation rec {{
  name = "laurn-shell";

  buildInputs = origShell.buildInputs;

  src = pkgs.writeScriptBin "start" ''
#!/bin/bash

export PATH=@binpath@

if [ $# -gt 0 ]; then
    exec @bashShell@/bin/bash -c "$*"
else
    exec @bashShell@/bin/bash -i
fi
'';

  binpath = pkgs.lib.makeBinPath (origShell.buildInputs ++ [
    pkgs.coreutils
    pkgs.procps
    pkgs.iproute
    pkgs.mount
    pkgs.which
    bash
  ]);
  bashShell = bash;

  buildPhase = "";
  installPhase = ''
    cp -r ./bin/start $out
    chmod +x $out
    substituteAllInPlace $out
  '';
}}
"#,
        source_path = laurn_shell_nix.display()
    )
}

#[derive(Debug)]
pub enum InstantiationFailed {
    Truncated,
    Code { exit_status: i32 },
    Exec(IoError),
    Write(IoError),
    StdinNotCaptured,
}

pub struct Instantiate(PathBuf);

impl Instantiate {
    pub fn new(source: &Path) -> Result<Self, InstantiationFailed> {
        let mut child = Command::new("nix-instantiate")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(InstantiationFailed::Exec)?;

        let stdin = (&mut child.stdin)
            .as_mut()
            .ok_or(InstantiationFailed::StdinNotCaptured)?;

        stdin
            .write_all(source_input(source).as_ref())
            .map_err(InstantiationFailed::Write)?;

        let output = child
            .wait_with_output()
            .map_err(InstantiationFailed::Exec)?;

        if !output.status.success() {
            Err(InstantiationFailed::Code {
                exit_status: output.status.code().unwrap(),
            })

        // output.stdout looks like "/nix/store/hash-foo\n"
        } else if let Some((_last, head_slice)) = output.stdout.split_last() {
            let os_path = OsStr::from_bytes(head_slice);
            let path = PathBuf::from(os_path);
            Ok(Self(path))
        } else {
            Err(InstantiationFailed::Truncated)
        }
    }
}

#[derive(Debug)]
pub enum BuildFailed {
    Code { exit_status: i32 },
    Exec(IoError),
    Truncated,
}

pub struct Build {
    pub(crate) output: PathBuf,
}

impl Build {
    pub fn realize(instantiation: Instantiate) -> Result<Self, BuildFailed> {
        let output = Command::new("nix-store")
            .arg("--query")
            .arg("--outputs")
            .arg("--force-realize")
            .arg(instantiation.0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .map_err(BuildFailed::Exec)?;

        if !output.status.success() {
            Err(BuildFailed::Code {
                exit_status: output.status.code().unwrap(),
            })

        // output.stdout looks like "/nix/store/hash-foo\n"
        } else if let Some((_last, head_slice)) = output.stdout.split_last() {
            let os_path = OsStr::from_bytes(head_slice);
            let path = PathBuf::from(os_path);
            Ok(Self { output: path })
        } else {
            Err(BuildFailed::Truncated)
        }
    }
}
