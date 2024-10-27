#![allow(clippy::ref_option)]

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

pub use immt_utils::settings::{SettingsSpec,BuildQueueSettings, ServerSettings};
use lazy_static::lazy_static;

static SETTINGS: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();

pub struct Settings {
    pub mathhubs: Box<[Box<Path>]>,
    pub mathhubs_is_default:bool,
    pub debug: bool,
    pub log_dir: Box<Path>,
    pub port: u16,
    pub ip: std::net::IpAddr,
    pub admin_pwd: Option<Box<str>>,
    pub database: Box<Path>,
    pub num_threads: u8,
    pub lsp:bool
}
impl Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Settings")
    }
}

impl Settings {
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn initialize(settings: SettingsSpec) {
        SETTINGS
            .set(settings.into())
            .expect("Error initializing settings");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get() -> &'static Self {
        SETTINGS.get().expect("Settings not initialized")
    }

    #[must_use]
    pub fn as_spec(&self) -> SettingsSpec {
        let spec = SettingsSpec {
            mathhubs: self.mathhubs.to_vec(),
            debug: Some(self.debug),
            log_dir: Some(self.log_dir.clone()),
            server: ServerSettings {
                port: self.port,
                ip: Some(self.ip),
                admin_pwd: self.admin_pwd.as_ref().map(ToString::to_string),
                database: Some(self.database.clone()),
            },
            buildqueue: BuildQueueSettings {
                num_threads: Some(self.num_threads),
            },
            lsp: self.lsp
        };
        spec
    }
}
impl From<SettingsSpec> for Settings {
    #[allow(clippy::cast_possible_truncation)]
    fn from(spec: SettingsSpec) -> Self {
        let (mathhubs,mathhubs_is_default) = if spec.mathhubs.is_empty() {
            (MATHHUB_PATHS.clone(),true)
        } else {
            let mhs = spec.mathhubs.into_boxed_slice();
            let is_def = mhs == *MATHHUB_PATHS;
            (mhs,is_def)
        };
        Self {
            mathhubs,mathhubs_is_default,
            debug: spec.debug.unwrap_or(cfg!(debug_assertions)),
            log_dir: spec.log_dir.unwrap_or_else(|| {
                CONFIG_DIR
                    .as_ref()
                    .expect("could not determine config directory")
                    .join("log")
                    .into_boxed_path()
            }),
            port: if spec.server.port == 0 {
                8095
            } else {
                spec.server.port
            },
            ip: spec
                .server
                .ip
                .unwrap_or_else(|| "127.0.0.1".parse().unwrap_or_else(|_| unreachable!())),
            admin_pwd: spec.server.admin_pwd.map(String::into_boxed_str),
            database: spec.server.database.unwrap_or_else(|| {
                CONFIG_DIR
                    .as_ref()
                    .expect("could not determine config directory")
                    .join("users.sqlite")
                    .into_boxed_path()
            }),
            num_threads: spec.buildqueue.num_threads.unwrap_or_else(|| {
                #[cfg(feature = "tokio")]
                {
                    (tokio::runtime::Handle::current().metrics().num_workers() / 2) as u8
                }
                #[cfg(not(feature = "tokio"))]
                {
                    1
                }
            }),
            lsp: spec.lsp
        }
    }
}

lazy_static! {
    static ref MATHHUB_PATHS: Box<[Box<Path>]> = mathhubs().into();
    static ref CONFIG_DIR: Option<Box<Path>> =
        simple_home_dir::home_dir().map(|d| d.join(".immt").into_boxed_path());
    static ref EXE_DIR: Option<Box<Path>> = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Into::into));
}

fn mathhubs() -> Vec<Box<Path>> {
    if let Ok(f) = std::env::var("MATHHUB") {
        return f
            .split(',')
            .map(|s| PathBuf::from(s.trim()).into_boxed_path())
            .collect();
    }
    if let Some(d) = simple_home_dir::home_dir() {
        let p = d.join(".mathhub").join("mathhub.path");
        if let Ok(f) = std::fs::read_to_string(p) {
            return f
                .split('\n')
                .map(|s| PathBuf::from(s.trim()).into_boxed_path())
                .collect();
        }
        return vec![d.join("MathHub").into_boxed_path()];
    }
    panic!(
    "No MathHub directory found and default ~/MathHub not accessible!\n\
    Please set the MATHHUB environment variable or create a file ~/.mathhub/mathhub.path containing \
    the path to the MathHub directory."
  )
}
