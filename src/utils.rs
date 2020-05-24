use std::path::{Path, PathBuf};

pub(crate) trait PathMerge {
    fn merge(self, other: &Path) -> PathBuf;
}

impl PathMerge for &'_ Path {
    fn merge(self, other: &Path) -> PathBuf {
        match other.strip_prefix("/") {
            Ok(rel) => self.join(rel),
            Err(_) => self.join(other),
        }
    }
}

impl PathMerge for PathBuf {
    fn merge(self, other: &Path) -> PathBuf {
        match other.strip_prefix("/") {
            Ok(rel) => self.join(rel),
            Err(_) => self.join(other),
        }
    }
}
