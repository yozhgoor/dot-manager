use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::dotfiles::Dotfiles;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_path: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Dotfiles::is_empty")]
    pub files: Dotfiles,
}

impl Config {
    fn new() -> Self {
        Self {
            remote_path: None,
            files: Dotfiles::new(),
        }
    }

    pub fn get_or_create() -> Result<Self> {
        let config_dir = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
        let config_file_path = config_dir.place_config_file("config.toml")?;

        let config: Self = match fs::read_to_string(&config_file_path) {
            Ok(file) => toml::de::from_str(&file)?,
            Err(_) => {
                let config = Self::new();
                fs::write(&config_file_path, toml::ser::to_string(&config)?)?;
                println!("Config file created at: {}", config_file_path.display());

                config
            }
        };

        Ok(config)
    }
}
