use std::{path::PathBuf, str::FromStr};

use immt_utils::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};

use crate::{backend::AnyBackend, building::{BuildTask,BuildResult},build_result, build_target, formats::CHECK, source_format};


#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test() {
  fn get_dependencies(backend: &AnyBackend, task: &BuildTask) {}

  fn run_build_target_1(_:&AnyBackend,task:&BuildTask) -> BuildResult {
    BuildResult::empty()
  }
  fn run_build_target_2(_:&AnyBackend,task:&BuildTask) -> BuildResult {
    BuildResult::empty()
  }

  source_format!(my_file_format ["ext1","ext2"] 
    [BUILD_TARGET_1 => BUILD_TARGET_2 => CHECK]
    @ "Some File Format with extensions .ext1 and .ext2"
    = get_dependencies
  );

  build_target!(
    build_target_1 [] => [FOO]
    @ "Some Build Target producing a Foo"
    = run_build_target_1
  );

  build_target!(
    build_target_2 [] => [BAR]
    @ "Some Build Target producing a Bar"
    = run_build_target_2
  );

  build_result!(foo @ "Some build result");
  build_result!(bar @ "Some other build result");

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