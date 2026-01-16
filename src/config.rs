use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub remote_path: PathBuf,
    #[serde(default)]
    pub home_path: Option<PathBuf>,
    #[serde(default)]
    pub files: Dotfiles,
}

impl Config {
    pub fn get() -> Result<Self> {
        let config_dir = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
        let config_file_path = config_dir.place_config_file("config.toml")?;

        match fs::read_to_string(&config_file_path) {
            Ok(file) => Ok(
                toml::de::from_str(&file).context("failed to deserialize configuration file")?
            ),
            Err(err) => {
                if !config_file_path.exists() {
                    bail!("configuration file does not exists");
                } else {
                    bail!("failed to read configuration file: {}", err);
                }
            }
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Dotfiles(Vec<Dotfile>);

impl std::iter::IntoIterator for Dotfiles {
    type Item = Dotfile;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dotfile {
    #[serde(rename = "local")]
    pub local_path: PathBuf,
    #[serde(default, rename = "remote")]
    pub remote_path: Option<PathBuf>,
}
