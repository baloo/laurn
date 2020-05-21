/// parse the `laurn.nix` config file
///
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use serde_derive::Deserialize;

#[derive(Debug)]
pub enum ConfigError {
    IO(io::Error),
    Parsing(toml::de::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub laurn: LaurnConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            laurn: LaurnConfig::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct LaurnConfig {
    pub mode: Mode,
}

impl Default for LaurnConfig {
    fn default() -> Self {
        LaurnConfig { mode: Mode::None }
    }
}

#[serde(rename_all = "lowercase")]
#[derive(Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Mode {
    None,
    Rust,
}

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let mut file = File::open(path).map_err(ConfigError::IO)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(ConfigError::IO)?;
    load_config_str(&contents)
}

fn load_config_str(contents: &str) -> Result<Config, ConfigError> {
    toml::from_str(&contents).map_err(ConfigError::Parsing)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        let config = load_config_str(
            r#"
[laurn]
mode = "rust"
"#,
        );

        let config = config.expect("unable to parse config");

        assert_eq!(config.laurn.mode, Mode::Rust);
    }
}
