use std::{ops::{Add, AddAssign}, path::{Path, PathBuf}};


#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SettingsSpec {
    #[cfg_attr(feature = "serde", serde(default))]
    pub mathhubs: Vec<Box<Path>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub debug: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub server: ServerSettings,
    #[cfg_attr(feature = "serde", serde(default))]
    pub log_dir: Option<Box<Path>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub buildqueue: BuildQueueSettings,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lsp:bool
}
impl Add for SettingsSpec {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            mathhubs: if self.mathhubs.is_empty() {
                rhs.mathhubs
            } else {
                self.mathhubs
            }, //self.mathhubs.into_iter().chain(rhs.mathhubs.into_iter()).collect(),
            debug: self.debug.or(rhs.debug),
            server: self.server + rhs.server,
            log_dir: self.log_dir.or(rhs.log_dir),
            buildqueue: self.buildqueue + rhs.buildqueue,
            lsp:self.lsp || rhs.lsp
        }
    }
}
impl AddAssign for SettingsSpec {
    fn add_assign(&mut self, rhs: Self) {
        if self.mathhubs.is_empty() {
            self.mathhubs = rhs.mathhubs;
        }
        self.debug = self.debug.or(rhs.debug);
        self.lsp = self.lsp || rhs.lsp;
        self.server += rhs.server;
        if self.log_dir.is_none() {
            self.log_dir = rhs.log_dir;
        }
        self.buildqueue += rhs.buildqueue;
    }
}

impl SettingsSpec {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn from_envs() -> Self {
        Self {
            mathhubs: Vec::new(),
            debug: std::env::var("IMMT_DEBUG")
                .ok()
                .and_then(|s| s.parse().ok()),
            log_dir: std::env::var("IMMT_LOG_DIR")
                .ok()
                .map(|s| PathBuf::from(s).into_boxed_path()),
            server: ServerSettings {
                port: std::env::var("IMMT_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
                ip: std::env::var("IMMT_IP").ok().map(|s| {
                    s.parse()
                        .expect("Could not parse IP address (environment variable IMMT_IP)")
                }),
                admin_pwd: std::env::var("IMMT_ADMIN_PWD").ok(),
                database: std::env::var("IMMT_DATABASE")
                    .ok()
                    .map(|s| PathBuf::from(s).into_boxed_path()),
            },
            buildqueue: BuildQueueSettings {
                num_threads: std::env::var("IMMT_NUM_THREADS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            lsp:false
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerSettings {
    #[cfg_attr(feature = "serde", serde(default))]
    pub port: u16,
    #[cfg_attr(feature = "serde", serde(default))]
    pub ip: Option<std::net::IpAddr>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub admin_pwd: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub database: Option<Box<Path>>,
}
impl Add for ServerSettings {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            port: if self.port == 0 { rhs.port } else { self.port },
            ip: self.ip.or(rhs.ip),
            admin_pwd: self.admin_pwd.or(rhs.admin_pwd),
            database: self.database.or(rhs.database),
        }
    }
}
impl AddAssign for ServerSettings {
    fn add_assign(&mut self, rhs: Self) {
        if self.port == 0 {
            self.port = rhs.port;
        }
        if self.ip.is_none() {
            self.ip = rhs.ip;
        }
        if self.admin_pwd.is_none() {
            self.admin_pwd = rhs.admin_pwd;
        }
        if self.database.is_none() {
            self.database = rhs.database;
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BuildQueueSettings {
    #[cfg_attr(feature = "serde", serde(default))]
    pub num_threads: Option<u8>,
}
impl Add for BuildQueueSettings {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_threads: self.num_threads.or(rhs.num_threads),
        }
    }
}
impl AddAssign for BuildQueueSettings {
    fn add_assign(&mut self, rhs: Self) {
        if self.num_threads.is_none() {
            self.num_threads = rhs.num_threads;
        }
    }
}