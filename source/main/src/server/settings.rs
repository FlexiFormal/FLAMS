use clap::Parser;
use immt_utils::settings::GitlabSettings;
use core::panic;
use immt_system::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};
use std::path::{Path, PathBuf};

#[derive(Parser,Debug)]
#[command(propagate_version = true, version, about, long_about = Some(
"iMᴍᴛ - Generic knowledge management system for flexiformal knowledge\n\
--------------------------------------------------------------------\n\
See the \u{1b}]8;;https://github.com/UniFormal/MMT\u{1b}\\documentation\u{1b}]8;;\u{1b}\\ for details"
))]
struct Cli {
    /// a comma-separated list of `MathHub` paths (if not given, the default paths are used
    /// as determined by the MATHHUB system variable or ~/.mathhub/mathhub.path)
    #[arg(short, long)]
    pub(crate) mathhubs: Option<String>,

    /// whether to enable debug logging
    #[arg(short, long)]
    pub(crate) debug: Option<bool>,

    #[arg(short, long)]
    /// The toml config file to use
    pub(crate) config_file: Option<PathBuf>,

    #[arg(short, long)]
    /// The log directory to use
    pub(crate) log_dir: Option<PathBuf>,

    #[arg(short, long)]
    /// The admin password to use for the server
    pub(crate) admin_pwd: Option<String>,

    /// Network port to use for the server
    #[arg(long,value_parser = clap::value_parser!(u16).range(1..))]
    pub(crate) port: Option<u16>,

    /// Network address to use for the server
    #[arg(long)]
    pub(crate) ip: Option<String>,

    #[arg(long)]
    /// The database file to use for account management etc.
    pub(crate) db: Option<PathBuf>,

    /// The number of threads to use for the buildqueue
    #[arg(short, long)]
    pub(crate) threads: Option<u8>,

    /// enter lsp mode
    #[arg(long)]
    pub(crate) lsp: bool,

    #[arg(long)]
    pub(crate) gitlab_url: Option<String>,

    #[arg(long)]
    pub(crate) gitlab_token: Option<String>,
    #[arg(long)]
    pub(crate) gitlab_app_id: Option<String>,
    #[arg(long)]
    pub(crate) gitlab_app_secret: Option<String>,
    #[arg(long)]
    pub(crate) gitlab_redirect_url: Option<String>
}
impl From<Cli> for (Option<PathBuf>, SettingsSpec) {
    fn from(cli: Cli) -> Self {
        let settings = SettingsSpec {
            mathhubs: cli
                .mathhubs
                .map(|s| {
                    s.split(',')
                        .map(|s| PathBuf::from(s.trim()).into_boxed_path())
                        .collect()
                })
                .unwrap_or_default(),
            debug: cli.debug,
            log_dir: cli.log_dir.map(PathBuf::into_boxed_path),
            server: ServerSettings {
                port: cli.port.unwrap_or_default(),
                ip: cli.ip.map(|s| s.parse().expect("Illegal ip")),
                admin_pwd: cli.admin_pwd,
                database: cli.db.map(PathBuf::into_boxed_path),
            },
            buildqueue: BuildQueueSettings {
                num_threads: cli.threads,
            },
            gitlab: GitlabSettings {
                url: cli.gitlab_url.map(Into::into),
                token: cli.gitlab_token.map(Into::into),
                app_id: cli.gitlab_app_id.map(Into::into),
                app_secret: cli.gitlab_app_secret.map(Into::into),
                redirect_url: cli.gitlab_redirect_url.map(Into::into),
            },
            lsp: cli.lsp
        };
        (cli.config_file, settings)
    }
}

impl Cli {
    #[must_use]#[inline]
    fn get() -> Self {
        Self::parse()
    }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn get_settings() -> SettingsSpec {
    fn from_file(cfg_file:&Path) -> SettingsSpec {
        let cfg = std::fs::read_to_string(cfg_file).unwrap_or_else(|e| {
            panic!("Could not read config file {}: {e}", cfg_file.display())
        });
        let cfg: SettingsSpec = toml::from_str(&cfg).unwrap_or_else(|e| {
            panic!("Could not parse config file {}: {e}", cfg_file.display())
        });
        cfg
    }
    let cli = Cli::get();
    let (cfg, mut settings) = cli.into();
    settings += SettingsSpec::from_envs();
    if let Some(cfg_file) = cfg {
        if cfg_file.exists() {
            settings += from_file(&cfg_file);
        } else {
            panic!("Could not find config file {}", cfg_file.display());
        }
    } else if let Ok(path) = std::env::current_exe() {
        if let Some(path) = path.parent() {
            let path = path.join("settings.toml");
            if path.exists() {
                settings += from_file(&path);
            }
        }
    }
    settings
}
