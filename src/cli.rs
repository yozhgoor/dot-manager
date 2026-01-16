#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    /// Check the current status of the dotfiles
    #[arg(long, short)]
    pub check: bool,
    /// Upload local files
    #[arg(long, short)]
    pub upload: bool,
    /// Download remote files
    #[arg(long, short)]
    pub download: bool,
    /// Update content from local or remote files
    #[arg(long, value_enum)]
    pub update: Option<UpdateMode>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum UpdateMode {
    /// Update local files from remote files
    Local,
    /// Update remote files from local files
    Remote,
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
