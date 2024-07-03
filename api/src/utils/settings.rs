use std::path::PathBuf;
use immt_core::utils::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};
use crate::MATHHUB_PATHS;
use crate::utils::asyncs::{ChangeListener, ChangeSender};

#[derive(Clone,Debug)]
pub struct SettingsChange<T> {
    pub old:T,
    pub new:T
}

#[derive(Debug)]
pub struct SettingsField<T:Clone> {
    value: T,
    sender: ChangeSender<SettingsChange<T>>
}
impl<T:Clone> SettingsField<T> {
    pub fn listener(&self) -> ChangeListener<SettingsChange<T>> {
        self.sender.listener()
    }
}

#[derive(Debug)]
pub struct Settings {
    pub mathhubs: Box<[PathBuf]>,
    pub debug:bool,
    pub log_dir:PathBuf,
    pub port:u16,
    pub ip:std::net::IpAddr,
    pub admin_pwd:Option<Box<str>>,
    pub database:PathBuf,
    pub num_threads:SettingsField<u8>,
}
impl Settings {
    pub fn as_spec(&self) -> SettingsSpec {
        SettingsSpec {
            mathhubs: self.mathhubs.iter().map(|p| p.to_path_buf()).collect(),
            debug: Some(self.debug),
            log_dir: Some(self.log_dir.clone()),
            server: ServerSettings {
                port: self.port,
                ip: Some(self.ip),
                admin_pwd: self.admin_pwd.as_ref().map(|s| s.to_string()),
                database: Some(self.database.clone())
            },
            build_queue: BuildQueueSettings {
                num_threads: Some(self.num_threads.value)
            }
        }
    }
}
impl From<SettingsSpec> for Settings {
    fn from(spec:SettingsSpec) -> Self {
        Self {
            mathhubs: if spec.mathhubs.is_empty() {
                MATHHUB_PATHS.clone()
            } else {spec.mathhubs.into_boxed_slice()},
            debug: spec.debug.unwrap_or(cfg!(debug_assertions)),
            log_dir: spec.log_dir.unwrap_or_else(|| crate::config_dir().expect("could not determine config directory").join("log")),
            port: if spec.server.port==0 {8095} else {spec.server.port},
            ip: spec.server.ip.unwrap_or_else(|| "127.0.0.1".parse().unwrap()),
            admin_pwd: spec.server.admin_pwd.map(|s| s.into_boxed_str()),
            database: spec.server.database.unwrap_or_else(|| crate::config_dir().expect("could not determine config directory").join("users.sqlite")),
            num_threads: SettingsField {
                value: if let Some(u) = spec.build_queue.num_threads {u} else {
                    #[cfg(feature="tokio")]
                    {(tokio::runtime::Handle::current().metrics().num_workers() / 2) as u8}
                    #[cfg(not(feature="tokio"))]
                    {1}
                },
                sender: ChangeSender::new(8)
            }
        }
    }
}