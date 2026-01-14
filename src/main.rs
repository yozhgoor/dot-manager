use anyhow::Result;
use std::io::Write;

mod config;
mod dotfiles;

use config::Config;

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

    let _config = Config::get_or_create()?;

    Ok(())
}
