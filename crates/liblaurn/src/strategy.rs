use std::ffi::OsStr;
use std::path::PathBuf;

use crate::config::Mode;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum ExposedPath {
    Project(PathBuf),
    UserHome(PathBuf),
}

impl ExposedPath {
    fn project<T: ?Sized + AsRef<OsStr>>(input: &T) -> Self {
        ExposedPath::Project(PathBuf::from(input))
    }
    fn user_home<T: ?Sized + AsRef<OsStr>>(input: &T) -> Self {
        ExposedPath::UserHome(PathBuf::from(input))
    }
}

#[derive(Debug)]
pub(crate) struct Strategy {
    pub ro_paths: Vec<ExposedPath>,
    pub rw_paths: Vec<ExposedPath>,
}

impl Strategy {
    fn new(mut ro_paths: Vec<ExposedPath>, rw_paths: Vec<ExposedPath>) -> Self {
        ro_paths.push(ExposedPath::project(".git"));
        ro_paths.push(ExposedPath::project(".laurnrc"));
        ro_paths.push(ExposedPath::project("laurn.nix"));
        ro_paths.push(ExposedPath::project("nix"));

        Self { ro_paths, rw_paths }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy {
            ro_paths: vec![],
            rw_paths: vec![],
        }
    }
}

impl From<Mode> for Strategy {
    fn from(mode: Mode) -> Strategy {
        match mode {
            Mode::None => Strategy::default(),
            Mode::Rust => Strategy::new(vec![], vec![ExposedPath::user_home(".cargo")]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Mode;
    #[test]
    fn rust() {
        let strategy = Strategy::from(Mode::Rust);
        assert_eq!(
            strategy.ro_paths,
            vec![
                ExposedPath::project(".git"),
                ExposedPath::project(".laurnrc"),
                ExposedPath::project("laurn.nix"),
                ExposedPath::project("nix"),
            ]
        );
        assert_eq!(strategy.rw_paths, vec![ExposedPath::user_home(".cargo")]);
    }
}
