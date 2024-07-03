use std::path::PathBuf;
use immt_api::exe_dir;
use immt_core::utils::settings::SettingsSpec;

pub fn get(cli: crate::cli::Cli) -> SettingsSpec {
    let cfg_file = if let Ok(cfg) = std::env::var("IMMT_CONFIG_FILE") {
        PathBuf::from(cfg)
    } else if let Some(cfg) = cli.config_file {
        cfg
    } else {
        exe_dir().expect("Could not find config directory").join("settings.toml")
    };
    let mut set = if cfg_file.exists() {
        let cfg = std::fs::read_to_string(cfg_file).unwrap();
        toml::from_str(&cfg).unwrap()
    } else {
        SettingsSpec::default()
    };

    if let Some(mhs) = cli.mathhubs {
        let mhs = mhs.split(',').map(PathBuf::from).collect();
        set.mathhubs = mhs;
    } else if let Ok(mh) = std::env::var("MATHHUB") {
        let mhs = mh.split(',').map(PathBuf::from).collect();
        set.mathhubs = mhs;
    }

    if let Some(d) = cli.debug {
        set.debug = Some(d);
    } else if let Ok(s) = std::env::var("IMMT_DEBUG") {
        if let Ok(s) = s.parse() {
            set.debug = Some(s);
        }
    }

    if let Some(s) = cli.admin_pwd {
        set.server.admin_pwd = Some(s);
    } else if let Ok(s) = std::env::var("IMMT_ADMIN_PWD") {
        set.server.admin_pwd = Some(s);
    }

    if let Some(s) = cli.port {
        set.server.port = s;
    } else if let Ok(s) = std::env::var("IMMT_PORT") {
        if let Ok(s) = s.parse() {
            set.server.port = s;
        }
    }

    if let Some(s) = cli.ip {
        if let Ok(s) = s.parse() {
            set.server.ip = Some(s);
        }
    } else if let Ok(s) = std::env::var("IMMT_IP") {
        if let Ok(s) = s.parse() {
            set.server.ip = Some(s);
        }
    }

    if let Some(s) = cli.db {
        set.server.database = Some(s);
    } else if let Ok(s) = std::env::var("IMMT_DATABASE") {
        set.server.database = Some(PathBuf::from(s));
    }

    if let Some(s) = cli.log_dir {
        set.log_dir = Some(s);
    } else if let Ok(s) = std::env::var("IMMT_LOG_DIR") {
        set.log_dir = Some(PathBuf::from(s));
    }
    set
}