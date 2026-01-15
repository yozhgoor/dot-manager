#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    /// Check the current status of the dotfiles
    #[arg(long, short)]
    check: bool,
    /// Update the dotfiles
    #[arg(long, short)]
    update: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
