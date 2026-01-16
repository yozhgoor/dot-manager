use anyhow::{Result, bail};
use std::fs;

use crate::{cli::Cli, config::Config, dotfiles::Dotfile};

pub fn run(cli: Cli, config: Config) -> Result<()> {
    let mut to_update = Vec::new();

    if cli.check || cli.update {
        for Dotfile { origin, remote } in config.files {
            if !origin.exists() {
                log::warn!("`{}`: original file does not exists", origin.display());
                continue;
            }

            let remote_file_path = if let Some(ref remote_path) = config.remote_path {
                remote_path.join(&remote)
            } else {
                bail!("`remote_path` configuration not provided");
            };

            let origin_content = fs::read_to_string(&origin)?;

            if !remote_file_path.exists() {
                log::warn!("`{}`: remote file does not exists", remote.display());
                to_update.push((remote, remote_file_path, origin_content));
                continue;
            }

            let remote_content = fs::read_to_string(&remote_file_path)?;

            if origin_content != remote_content {
                log::info!("`{}`: can be updated", remote.display());
                to_update.push((remote, remote_file_path, origin_content));
            }
        }
    }

    if cli.update {
        println!();

        for (short_path, full_path, content) in to_update {
            if !full_path.exists()
                && let Some(parent) = full_path.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }

            match fs::write(&full_path, content) {
                Ok(()) => log::info!("`{}`: Updated", short_path.display()),
                Err(err) => bail!("failed to write `{}`: {}", short_path.display(), err),
            }
        }
    }

    Ok(())
}
