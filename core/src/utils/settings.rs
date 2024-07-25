use std::path::PathBuf;

#[derive(Debug,Default,Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct SettingsSpec {
    #[cfg_attr(feature = "serde",serde(default))]
    pub mathhubs: Vec<PathBuf>,
    #[cfg_attr(feature = "serde",serde(default))]
    pub debug:Option<bool>,
    #[cfg_attr(feature = "serde",serde(default))]
    pub server:ServerSettings,
    #[cfg_attr(feature = "serde",serde(default))]
    pub log_dir:Option<PathBuf>,
    #[cfg_attr(feature = "serde",serde(default))]
    pub buildqueue:BuildQueueSettings
}

#[derive(Debug,Default,Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct ServerSettings {
    #[cfg_attr(feature = "serde",serde(default))]
    pub port: u16,
    #[cfg_attr(feature = "serde",serde(default))]
    pub ip:Option<std::net::IpAddr>,
    #[cfg_attr(feature = "serde",serde(default))]
    pub admin_pwd:Option<String>,
    #[cfg_attr(feature = "serde",serde(default))]
    pub database:Option<PathBuf>
}

#[derive(Debug,Default,Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct BuildQueueSettings {
    #[cfg_attr(feature = "serde",serde(default))]
    pub num_threads: Option<u8>
}