use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::build::{Build, BuildFailed, Instantiate, InstantiationFailed};

#[derive(Debug)]
pub enum Error {
    Exec(IoError),
    Code { exit_status: i32 },
    Truncated,
    ParsingFailed,
}

#[derive(Debug)]
pub enum BuildError {
    Source(IoError),
    Instantiation(InstantiationFailed),
    Build(BuildFailed),
}

pub struct Container {
    pub(crate) laurn_expr: PathBuf,
    pub(crate) output: Build,
}

impl Container {
    pub fn build(source: &Path) -> Result<Container, BuildError> {
        let laurn_expr = source.canonicalize().map_err(BuildError::Source)?;

        let instantiation =
            Instantiate::new(laurn_expr.as_path()).map_err(BuildError::Instantiation)?;
        let build = Build::realize(instantiation).map_err(BuildError::Build)?;

        Ok(Container {
            output: build,
            laurn_expr,
        })
    }

    pub fn references(&self) -> Result<Vec<PathBuf>, Error> {
        let output = Command::new("nix-store")
            .arg("--query")
            .arg("--requisites")
            .arg(&self.output.output)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .map_err(Error::Exec)?;

        if !output.status.success() {
            Err(Error::Code {
                exit_status: output.status.code().unwrap(),
            })

        // output.stdout looks like "/nix/store/hash-foo\n/nix/store/hash-bar\n"
        } else {
            let (rest, out) =
                parsing::references(&output.stdout[..]).map_err(|_| Error::ParsingFailed)?;

            assert!(rest.is_empty()); // If this trigger, there is a bug/regression in the parser, it should consume the whole content
            Ok(out)
        }
    }
}

mod parsing {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    use nom::bytes::complete::take_while1;
    use nom::character::complete::newline;
    use nom::{do_parse, eof, map, named, opt, separated_nonempty_list};

    fn from_slice(lines: Vec<&[u8]>) -> Vec<PathBuf> {
        let mut out = Vec::with_capacity(lines.len());
        for line in lines.iter() {
            let os_path = OsStr::from_bytes(line);
            let path = PathBuf::from(os_path);
            out.push(path);
        }
        out
    }

    named!(
        list_path < &[u8], Vec<&[u8]>>,
        do_parse!(
          paths: separated_nonempty_list!(newline, take_while1(|chr: u8| chr != b'\n')) >>
          // Consume a trailing newline
          opt!(newline) >>
          // Ensure we reached the whole buffer
          eof!() >>
          (paths)
        )
    );

    named!(pub references<&[u8], Vec<PathBuf>>,  map!(list_path, from_slice));

    #[cfg(test)]
    mod test {
        use super::references;
        use std::path::PathBuf;

        #[test]
        fn parse_line_separated() {
            let input = b"/nix/store/hash-foo\n/nix/store/hash-bar";
            let (rest, out) = references(input).expect("parsing failed");
            eprintln!("rest={:?}, out={:?}", rest, out);
            assert!(rest.is_empty());
            assert_eq!(
                out,
                vec![
                    PathBuf::from("/nix/store/hash-foo"),
                    PathBuf::from("/nix/store/hash-bar"),
                ],
            );
        }

        #[test]
        fn parse_line_trailing_newline() {
            let input = b"/nix/store/hash-foo\n/nix/store/hash-bar\n";
            let (_rest, out) = references(input).expect("parsing failed");
            assert_eq!(
                out,
                vec![
                    PathBuf::from("/nix/store/hash-foo"),
                    PathBuf::from("/nix/store/hash-bar"),
                ],
            );
        }

        #[test]
        fn parse_empty() {
            let input = b"\n\n";
            assert!(references(input).is_err());
            let input = b"\n";
            assert!(references(input).is_err());
            let input = b"";
            assert!(references(input).is_err());
        }

        #[test]
        fn parse_empty_line() {
            let input = b"/nix/store/hash-foo\n\n/nix/store/hash-bar\n";
            let out = references(input);
            eprintln!("out={:?}", out);
            assert!(out.is_err());
        }
    }
}
