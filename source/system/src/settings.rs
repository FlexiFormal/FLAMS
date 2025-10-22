#![allow(clippy::ref_option)]

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::atomic::AtomicU16,
};

use flams_utils::settings::GitlabSettings;
pub use flams_utils::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};

static SETTINGS: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();

pub struct Settings {
    pub mathhubs_is_default: bool,
    pub debug: bool,
    pub log_dir: Box<Path>,
    pub port: AtomicU16,
    pub ip: std::net::IpAddr,
    pub admin_pwd: Option<Box<str>>,
    pub database: Box<Path>,
    pub stack_size: Option<u8>,
    external_url: Option<Box<str>>,
    temp_dir: parking_lot::RwLock<Option<tempfile::TempDir>>,
    pub num_threads: u8,
    pub gitlab_url: Option<url::Url>,
    pub gitlab_token: Option<Box<str>>,
    pub gitlab_app_id: Option<Box<str>>,
    pub gitlab_app_secret: Option<Box<str>>,
    pub gitlab_redirect_url: Option<Box<str>>,
    pub lsp: bool,
}
impl Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Settings")
    }
}

impl Settings {
    #[inline]
    pub fn mathhubs(&self) -> &'static [&'static Path] {
        flams_math_archives::mathhub::mathhubs()
    }
    pub fn port(&self) -> u16 {
        self.port.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[allow(clippy::missing_panics_doc)]
    pub fn initialize(settings: SettingsSpec) {
        SETTINGS
            .set(settings.into())
            .expect("Error initializing settings");
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get() -> &'static Self {
        SETTINGS.get().expect("Settings not initialized")
    }

    #[inline]
    pub fn external_url(&self) -> &str {
        self.external_url.as_deref().unwrap_or_default()
    }

    /// #### Panics
    pub fn temp_dir(&self) -> PathBuf {
        self.temp_dir
            .read()
            .as_ref()
            .expect("This should never happen!")
            .path()
            .to_path_buf()
    }

    #[allow(clippy::significant_drop_in_scrutinee)]
    pub fn close(&self) {
        if let Some(td) = self.temp_dir.write().take() {
            let _ = td.close();
        }
    }

    /// #### Panics
    #[must_use]
    pub fn as_spec(&self) -> SettingsSpec {
        let port = self.port();
        let spec = SettingsSpec {
            mathhubs: flams_math_archives::mathhub::mathhubs()
                .iter()
                .map(|m| m.to_path_buf())
                .collect(), // self.mathhubs.to_vec(),
            debug: Some(self.debug),
            log_dir: Some(self.log_dir.clone()),
            temp_dir: Some(
                self.temp_dir
                    .read()
                    .as_ref()
                    .expect("This should never happen!")
                    .path()
                    .to_path_buf()
                    .into_boxed_path(),
            ),
            database: Some(self.database.clone()),
            server: ServerSettings {
                port,
                ip: Some(self.ip),
                external_url: self
                    .external_url
                    .as_ref()
                    .map(ToString::to_string)
                    .or_else(|| Some(format!("http://{}:{port}", self.ip))),
                admin_pwd: self.admin_pwd.as_ref().map(ToString::to_string),
            },
            stack_size: self.stack_size,
            buildqueue: BuildQueueSettings {
                num_threads: Some(self.num_threads),
            },
            gitlab: GitlabSettings {
                url: self.gitlab_url.clone(),
                token: self.gitlab_token.clone(),
                app_id: self.gitlab_app_id.clone(),
                app_secret: self.gitlab_app_secret.clone(),
                redirect_url: self.gitlab_redirect_url.clone(),
            },
            lsp: self.lsp,
        };
        spec
    }
}
impl From<SettingsSpec> for Settings {
    #[allow(clippy::cast_possible_truncation)]
    fn from(spec: SettingsSpec) -> Self {
        let mathhubs_is_default = if spec.mathhubs.is_empty() {
            true
        } else {
            let mhs = spec.mathhubs;
            let _ = flams_math_archives::mathhub::set_mathhubs(mhs);
            flams_math_archives::mathhub::mathhubs()
                == flams_math_archives::mathhub::default_mathhubs()
        };
        Self {
            mathhubs_is_default,
            debug: spec.debug.unwrap_or(cfg!(debug_assertions)),
            log_dir: spec.log_dir.unwrap_or_else(|| {
                CONFIG_DIR
                    .as_ref()
                    .expect("could not determine config directory")
                    .join("log")
                    .into_boxed_path()
            }),
            stack_size: spec.stack_size,
            temp_dir: parking_lot::RwLock::new(Some(spec.temp_dir.map_or_else(
                || tempfile::TempDir::new().expect("Could not create temp dir"),
                |p| {
                    let _ = std::fs::create_dir_all(&p);
                    tempfile::Builder::new()
                        .tempdir_in(p)
                        .expect("Could not create temp dir")
                },
            ))),
            external_url: spec.server.external_url.map(String::into_boxed_str),
            port: AtomicU16::new(if spec.server.port == 0 {
                8095
            } else {
                spec.server.port
            }),
            ip: spec
                .server
                .ip
                .unwrap_or_else(|| "127.0.0.1".parse().unwrap_or_else(|_| unreachable!())),
            admin_pwd: if spec.lsp {
                None
            } else {
                spec.server.admin_pwd.map(String::into_boxed_str)
            },
            database: spec.database.unwrap_or_else(|| {
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
            lsp: spec.lsp,
            gitlab_token: spec.gitlab.token,
            gitlab_url: spec.gitlab.url,
            gitlab_app_id: spec.gitlab.app_id,
            gitlab_app_secret: spec.gitlab.app_secret,
            gitlab_redirect_url: spec.gitlab.redirect_url,
        }
    }
}

static CONFIG_DIR: std::sync::LazyLock<Option<Box<Path>>> = std::sync::LazyLock::new(|| {
    simple_home_dir::home_dir().map(|d| d.join(".flams").into_boxed_path())
});

/*
static EXE_DIR: std::sync::LazyLock<Option<Box<Path>>> = std::sync::LazyLock::new(|| {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Into::into))
});
 */
