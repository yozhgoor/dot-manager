use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod manager;

use cli::Cli;
use config::Config;
use manager::Manager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::get()?;

    let manager = Manager::new(config)?;

    if !cli.upload && !cli.download && !cli.update {
        manager.check();
    }

    manager.run(cli)?;

    Ok(())
}
