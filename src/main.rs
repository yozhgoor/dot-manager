use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod dotfiles;
mod manager;

use cli::Cli;
use config::Config;
use manager::Manager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::get_or_create()?;

    let manager = Manager::new(config)?;

    if cli.check {
        manager.check();
    }

    manager.run(cli)?;

    Ok(())
}
