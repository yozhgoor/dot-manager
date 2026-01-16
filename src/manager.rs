use anyhow::{Result, bail};
use std::{
    env, fs,
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

            let local_file_path = {
                if !local_path.starts_with("/") {
                    if let Some(ref home_path) = config.home_path {
                        home_path.join(&local_path)
                    } else if let Ok(home_path) = env::var("HOME") {
                        PathBuf::from(home_path).join(&local_path)
                    } else {
                        bail!("failed to determine `HOME` path");
                    }
                } else {
                    local_path.clone()
                }
            };

            match (local_file_path.exists(), remote_file_path.exists()) {
                (false, false) => {
                    statuses.push(Status::Absent(local_path, local_file_path));
                }
                (true, false) => {
                    let local_content = read_content(&local_file_path)?;
                    statuses.push(Status::Upload(remote_path, remote_file_path, local_content));
                }
                (false, true) => {
                    let remote_content = read_content(&remote_file_path)?;
                    statuses.push(Status::Download(local_path, local_file_path, remote_content));
                }
                (true, true) => {
                    let local_content = read_content(&local_file_path)?;
                    let remote_content = read_content(&remote_file_path)?;

                    if local_content == remote_content {
                        statuses.push(Status::UpToDate(local_path, local_file_path));
                    } else {
                        statuses.push(Status::Update(
                            (local_path, local_file_path, local_content),
                            (remote_path, remote_file_path, remote_content),
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
                Status::UpToDate(path, _) => up_to_date.push(path.display()),
                Status::Update((local_path, _,  _), (remote_path, _, _)) => {
                    to_update.push((local_path.display(), remote_path.display()));
                }
                Status::Upload(path, _, _) => to_upload.push(path.display()),
                Status::Download(path, _, _) => to_download.push(path.display()),
                Status::Absent(path, _) => absent.push(path.display()),
            }
        }

        if !up_to_date.is_empty() {
            println!("Up to date:");
            for path in up_to_date {
                println!("* {}", path);
            }
            println!();
        }

        if !to_update.is_empty() {
            println!("To update:");
            for (local, remote) in to_update {
                println!("* {} - {}", local, remote);
            }
            println!();
        }

        if !to_upload.is_empty() {
            println!("To upload:");
            for path in to_upload {
                println!("* {}", path);
            }
            println!();
        }

        if !to_download.is_empty() {
            println!("To download:");
            for path in to_download {
                println!("* {}", path);
            }
            println!();
        }

        if !absent.is_empty() {
            println!("Does not exists:");
            for path in absent {
                println!("* {}", path);
            }
            println!();
        }
    }

    pub fn run(&self, cli: Cli) -> Result<()> {
        for status in &self.0 {
            match status {
                Status::Update((_, local_path, local_content), (_, remote_path, remote_content))
                    if cli.update.is_some() =>
                {
                    match cli.update.as_ref().expect("update is some") {
                        UpdateMode::Local => {
                            write_content(local_path, remote_content)?;
                            println!("`{}`: Updated", local_path.display());
                        }
                        UpdateMode::Remote => {
                            write_content(remote_path, local_content)?;
                            println!("`{}`: Updated", remote_path.display());
                        }
                    }
                }
                Status::Upload(_, path, content) if cli.upload => {
                    write_content(path, content)?;
                    println!("`{}`: Uploaded", path.display());
                }
                Status::Download(_, path, content) if cli.download => {
                    write_content(path, content)?;
                    println!("`{}`: Downloaded", path.display());
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Status {
    UpToDate(PathBuf, PathBuf),
    Update((PathBuf, PathBuf, String), (PathBuf, PathBuf, String)),
    Upload(PathBuf, PathBuf, String),
    Download(PathBuf, PathBuf, String),
    Absent(PathBuf, PathBuf),
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

fn write_content(path: &Path, content: &str) -> Result<()> {
    if !path.exists()
        && let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    if let Err(err) = fs::write(path, content) {
        bail!("failed to write content in `{}`: {}", path.display(), err);
    } else {
        Ok(())
    }
}
