use anyhow::{Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::{Cli, UpdateMode},
    config::Config,
    dotfiles::Dotfile,
};

const IGNORE_LINE: &str = "dot-manager: ignore after this";

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

        for Dotfile {
            local_path,
            remote_path,
        } in config.files
        {
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
                    let local_content = read_content(&local_path)?;
                    statuses.push(Status::Upload(remote_file_path, local_content));
                }
                (false, true) => {
                    if log {
                        log::warn!("`{}` can be downloaded from remote", local_path.display());
                    }
                    let remote_content = read_content(&remote_file_path)?;
                    statuses.push(Status::Download(local_path, remote_content));
                }
                (true, true) => {
                    let local_content = read_content(&local_path)?;
                    let remote_content = read_content(&remote_file_path)?;

                    if local_content == remote_content {
                        if log {
                            log::info!("`{}` is up to date", local_path.display());
                        }
                    } else {
                        if log {
                            log::warn!("`{}` can be updated", local_path.display());
                        }
                        statuses.push(Status::Update(
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
        println!();

        for status in &self.0 {
            match status {
                Status::Update((local_path, local_content), (remote_path, remote_content))
                    if cli.update.is_some() =>
                {
                    match cli.update.as_ref().expect("update is some") {
                        UpdateMode::Local => {
                            fs::write(local_path, remote_content)?;
                            log::info!("`{}`: Updated", local_path.display());
                        }
                        UpdateMode::Remote => {
                            fs::write(remote_path, local_content)?;
                            log::info!("`{}`: Updated", remote_path.display());
                        }
                    }
                }
                Status::Upload(path, content) if cli.upload => {
                    fs::write(path, content)?;
                    log::info!("`{}`: Uploaded", path.display());
                }
                Status::Download(path, content) if cli.download => {
                    fs::write(path, content)?;
                    log::info!("`{}`: Downloaded", path.display());
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn read_content(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(s) => {
            let content = if s.contains(IGNORE_LINE) {
                s.lines()
                    .take_while(|line| !line.contains(IGNORE_LINE))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                s
            };

            Ok(content)
        }
        Err(err) => bail!("failed to read content at `{}`: {}", path.display(), err),
    }
}

#[derive(Debug)]
enum Status {
    Update((PathBuf, String), (PathBuf, String)),
    Upload(PathBuf, String),
    Download(PathBuf, String),
}
