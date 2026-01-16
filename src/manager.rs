use anyhow::{Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::{Cli, UpdateMode},
    config::{Config, Dotfile},
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
            let (remote_path, remote_file_path) = {
                let config_remote_path = if !config.remote_path.starts_with("/") {
                    if let Some(ref home_path) = config.home_path {
                        home_path.join(&config.remote_path)
                    } else if let Ok(home_path) = env::var("HOME") {
                        PathBuf::from(home_path).join(&config.remote_path)
                    } else {
                        bail!("failed to determine `HOME` path");
                    }
                } else {
                    config.remote_path.clone()
                };

                if let Some(path) = remote_path {
                    (path.to_path_buf(), config_remote_path.join(&path))
                } else {
                    if !local_path.is_dir() {
                        if let Some(file_name) = local_path.file_name() {
                            (PathBuf::from(file_name), config_remote_path.join(&file_name))
                        } else {
                            bail!("failed to determine remote path");
                        }
                    } else {
                        bail!("remote path cannot be a directory");
                    }
                }
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
                    statuses.push(Status::Absent(File::new(local_path, local_file_path)));
                }
                (true, false) => {
                    let local_content = read_content(&local_file_path)?;
                    statuses.push(Status::Upload(FileWithContent::new(remote_path, remote_file_path, local_content)));
                }
                (false, true) => {
                    let remote_content = read_content(&remote_file_path)?;
                    statuses.push(Status::Download(FileWithContent::new(
                        local_path,
                        local_file_path,
                        remote_content,
                    )));
                }
                (true, true) => {
                    let local_content = read_content(&local_file_path)?;
                    let remote_content = read_content(&remote_file_path)?;

                    if local_content == remote_content {
                        statuses.push(Status::UpToDate(File::new(local_path, local_file_path)));
                    } else {
                        statuses.push(Status::Update(
                            FileWithContent::new(local_path, local_file_path, local_content),
                            FileWithContent::new(remote_path, remote_file_path, remote_content),
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
                Status::UpToDate(file) => up_to_date.push(file.short_path.display()),
                Status::Update(local_file, remote_file) => {
                    to_update.push((local_file.short_path.display(), remote_file.short_path.display()));
                }
                Status::Upload(file) => to_upload.push(file.short_path.display()),
                Status::Download(file) => to_download.push(file.short_path.display()),
                Status::Absent(file) => absent.push(file.short_path.display()),
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
                Status::Update(local_file, remote_file) if cli.update.is_some() => {
                    match cli.update.as_ref().expect("update is some") {
                        UpdateMode::Local => {
                            write_content(&local_file.full_path, &remote_file.content)?;
                            println!("`{}`: Updated", local_file.short_path.display());
                        }
                        UpdateMode::Remote => {
                            write_content(&remote_file.full_path, &local_file.content)?;
                            println!("`{}`: Updated", remote_file.short_path.display());
                        }
                    }
                }
                Status::Upload(file) if cli.upload => {
                    write_content(&file.full_path, &file.content)?;
                    println!("`{}`: Uploaded", file.short_path.display());
                }
                Status::Download(file) if cli.download => {
                    write_content(&file.full_path, &file.content)?;
                    println!("`{}`: Downloaded", file.short_path.display());
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Status {
    UpToDate(File),
    Update(FileWithContent, FileWithContent),
    Upload(FileWithContent),
    Download(FileWithContent),
    Absent(File),
}

#[derive(Debug, Eq, PartialEq)]
struct File {
    short_path: PathBuf,
    full_path: PathBuf,
}

impl File {
    fn new(short_path: PathBuf, full_path: PathBuf) -> Self {
        Self {
            short_path,
            full_path,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct FileWithContent {
    short_path: PathBuf,
    full_path: PathBuf,
    content: String,
}

impl FileWithContent {
    fn new(short_path: PathBuf, full_path: PathBuf, content: String) -> Self {
        Self {
            short_path,
            full_path,
            content,
        }
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
