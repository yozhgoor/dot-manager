use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dotfiles(Vec<Dotfile>);

impl Dotfiles {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

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
    #[serde(rename = "remote")]
    pub remote_path: PathBuf,
}
