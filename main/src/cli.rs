use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(propagate_version = true, version, about, long_about = Some(
"iMᴍᴛ - Generic knowledge management system for flexiformal knowledge\n\
--------------------------------------------------------------------\n\
See the \u{1b}]8;;https://github.com/UniFormal/MMT\u{1b}\\documentation\u{1b}]8;;\u{1b}\\ for details"
))]
pub struct Cli {
    /// a comma-separated list of MathHub paths (if not given, the default paths are used
    /// as determined by the MATHHUB system variable or ~/.mathhub/mathhub.path)
    #[arg(short, long)]
    pub(crate) mathhubs: Option<String>,

    /// whether to enable debug logging
    #[arg(short, long)]
    pub(crate) debug:Option<bool>,

    #[arg(short, long)]
    /// The toml config file to use
    pub(crate) config_file:Option<PathBuf>,

    #[arg(short, long)]
    /// The log directory to use
    pub(crate) log_dir:Option<PathBuf>,

    #[arg(short, long)]
    /// The admin password to use for the server
    pub(crate) admin_pwd:Option<String>,

    /// Network port to use for the server
    #[arg(long,value_parser = clap::value_parser!(u16).range(1..))]
    pub(crate) port: Option<u16>,

    /// Network address to use for the server
    #[arg(long)]
    pub(crate) ip: Option<String>,

    #[arg(long)]
    /// The database file to use for account management etc.
    pub(crate) db:Option<PathBuf>
}
impl Cli {
    pub fn get() -> Cli { Cli::parse() }
}