use anyhow::Result;
use clap::Parser;
use std::io::Write;

mod cli;
mod config;
mod dotfiles;
mod manager;

use cli::Cli;
use config::Config;
use manager::Manager;

fn main() -> Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                record.level(),
                env!("CARGO_PKG_NAME"),
                record.args()
            )
        })
        .filter(
            Some(env!("CARGO_PKG_NAME").replace("-", "_").as_ref()),
            log::LevelFilter::Info,
        )
        .init();

    let cli = Cli::parse();
    let config = Config::get_or_create()?;

    let manager = Manager::check(config, cli.check)?;
    manager.run(cli)?;

    Ok(())
}
