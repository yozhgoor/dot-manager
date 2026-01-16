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
    pub fn new(config: Config) -> Result<Self> {
        let mut statuses = Vec::new();

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
                    statuses.push(Status::Absent(local_path));
                }
                (true, false) => {
                    let local_content = read_content(&local_path)?;
                    statuses.push(Status::Upload(remote_file_path, local_content));
                }
                (false, true) => {
                    let remote_content = read_content(&remote_file_path)?;
                    statuses.push(Status::Download(local_path, remote_content));
                }
                (true, true) => {
                    let local_content = read_content(&local_path)?;
                    let remote_content = read_content(&remote_file_path)?;

                    if local_content == remote_content {
                        statuses.push(Status::UpToDate(local_path));
                    } else {
                        statuses.push(Status::Update(
                            (local_path, local_content),
                            (remote_file_path, remote_content),
                        ));
                    }
                }
            }
        }

        Ok(Self(statuses))
    }

    pub fn check(&self) {
        let mut up_to_date = Vec::new();
        let mut to_update = Vec::new();
        let mut to_upload = Vec::new();
        let mut to_download = Vec::new();
        let mut absent = Vec::new();

        for status in &self.0 {
            match status {
                Status::UpToDate(path) => up_to_date.push(path.display()),
                Status::Update((local_path, _), (remote_path, _)) => {
                    to_update.push((local_path.display(), remote_path.display()));
                }
                Status::Upload(path, _) => to_upload.push(path.display()),
                Status::Download(path, _) => to_download.push(path.display()),
                Status::Absent(path) => absent.push(path.display()),
            }
        }

        for path in up_to_date {
            println!("`{}` is up to date", path);
        }

        for (local, remote) in to_update {
            println!("`{}` and `{}` are not synced", local, remote);
        }

        for path in to_upload {
            println!("`{}` can be uploaded", path);
        }

        for path in to_download {
            println!("`{}` can be downloaded", path);
        }

        for path in absent {
            println!("`{}` does not exist locally or on remote", path);
        }
    }

    pub fn run(&self, cli: Cli) -> Result<()> {
        if cli.check {
            println!();
        }

        for status in &self.0 {
            match status {
                Status::Update((local_path, local_content), (remote_path, remote_content))
                    if cli.update.is_some() =>
                {
                    match cli.update.as_ref().expect("update is some") {
                        UpdateMode::Local => {
                            fs::write(local_path, remote_content)?;
                            println!("`{}`: Updated", local_path.display());
                        }
                        UpdateMode::Remote => {
                            fs::write(remote_path, local_content)?;
                            println!("`{}`: Updated", remote_path.display());
                        }
                    }
                }
                Status::Upload(path, content) if cli.upload => {
                    fs::write(path, content)?;
                    println!("`{}`: Uploaded", path.display());
                }
                Status::Download(path, content) if cli.download => {
                    fs::write(path, content)?;
                    println!("`{}`: Downloaded", path.display());
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

#[derive(Debug, Eq, PartialEq)]
enum Status {
    UpToDate(PathBuf),
    Update((PathBuf, String), (PathBuf, String)),
    Upload(PathBuf, String),
    Download(PathBuf, String),
    Absent(PathBuf),
}
