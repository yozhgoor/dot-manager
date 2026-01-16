use anyhow::{Result, bail};
use std::fs;
use std::path::PathBuf;

use crate::{cli::{UpdateMode, Cli}, config::Config, dotfiles::Dotfile};

#[derive(Debug)]
pub struct Manager(Vec<Status>);

impl Manager {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, item: Status) {
        self.0.push(item)
    }

    pub fn check(config: Config, log: bool) -> Result<Self> {
        let mut statuses = Self::new();

        for Dotfile { local_path, remote_path } in config.files {
            let remote_file_path = if let Some(ref remote_root_path) = config.remote_path {
                remote_root_path.join(&remote_path)
            } else {
                bail!("`remote_path` configuration not provided");
            };

            match (local_path.exists(), remote_file_path.exists()) {
                (false, false) => {
                    if log {
                        log::error!(
                            "`{}` does not exists and is not available on remote",
                            local_path.display()
                        );
                    }
                }
                (true, false) => {
                    if log {
                        log::warn!("`{}` can be uploaded on remote", local_path.display());
                    }
                    let local_content = fs::read_to_string(&local_path)?;
                    statuses.push(Status::ToUpload(remote_file_path, local_content));
                }
                (false, true) => {
                    if log {
                        log::warn!("`{}` can be downloaded from remote", local_path.display());
                    }
                    let remote_content = fs::read_to_string(&remote_file_path)?;
                    statuses.push(Status::ToDownload(local_path, remote_content));
                }
                (true, true) => {
                    let local_content = fs::read_to_string(&local_path)?;
                    let remote_content = fs::read_to_string(&remote_file_path)?;

                    if local_content == remote_content {
                        if log {
                            log::info!("`{}` is up to date", local_path.display());
                        }
                    } else {
                        if log {
                            log::warn!("`{}` can be updated", local_path.display());
                        }
                        statuses.push(Status::ToUpdate(
                            (local_path, local_content),
                            (remote_file_path, remote_content),
                        ));
                    }
                }
            }
        }

        Ok(statuses)
    }

    pub fn run(&self, cli: Cli) -> Result<()> {
        for status in &self.0 {
            match status {
                Status::ToUpdate((local_path, local_content), (remote_path, remote_content)) if cli.update.is_some() => {
                    match cli.update.expect("update is some") {
                        UpdateMode::Local => todo!(),
                        UpdateMode::Remote => todo!(),
                    }
                }
                Status::ToUpload(local_path, local_content) if cli.upload => {
                    todo!()
                }
                Status::ToDownload(remote_path, remote_content) if cli.download => {
                    todo!()
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Status {
    ToUpdate((PathBuf, String), (PathBuf, String)),
    ToUpload(PathBuf, String),
    ToDownload(PathBuf, String),
}
