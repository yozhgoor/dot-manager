use anyhow::Result;
use std::io::Write;
use clap::Parser;

mod cli;
mod config;
mod dotfiles;
mod manager;

use config::Config;
use cli::Cli;

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
    manager::run(cli, config)?;

    Ok(())
}
