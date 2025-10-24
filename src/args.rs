use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Args {
    /// Use a local .tar file instead of downloading one
    #[arg(long)]
    pub tar: Option<PathBuf>,
    /// Install into specific directory
    #[arg(long)]
    pub install_dir: Option<PathBuf>,
    /// Verbose logs (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Skip check update when starting
    #[arg(long)]
    pub skip_check_update: bool,
    /// How often do you need to check for updates
    #[arg(long)]
    pub check_update_interval: Option<usize>,
    /// Check for the latest .tar.gz and print its url
    #[arg(long)]
    pub print_tar_url: bool,
    /// Run the install/update code but don't actually run the final binary
    #[arg(long)]
    pub no_exec: bool,
    /// The timeout to use for http connections and requests
    #[arg(long)]
    pub timeout: Option<usize>,
    /// How often to try to resume the download until giving up (0 for unlimited)
    #[arg(long)]
    pub download_attempts: Option<usize>,
}
