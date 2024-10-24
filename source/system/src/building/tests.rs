use std::{path::PathBuf, str::FromStr};

use immt_utils::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};

#[tokio::test]
async fn test() {
  crate::initialize(TEST_SETTINGS.clone());
}

lazy_static::lazy_static! {
  static ref TEST_SETTINGS : SettingsSpec = SettingsSpec {
    mathhubs:vec![PathBuf::from("/insert/your/path/here/MathHub").into()],
    debug: Some(true),
    // irrelevant, because no server involved anyway
    server: ServerSettings {
      port:3000,
      ip:Some(std::net::IpAddr::from_str("127.0.0.1").expect("This is a valid IP")),
      admin_pwd:None,
      database:None
    },
    log_dir:None,
    buildqueue:BuildQueueSettings {
      num_threads:Some(4)
    }
  };
}