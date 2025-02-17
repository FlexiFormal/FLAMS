use std::{num::NonZeroU32, path::PathBuf, str::FromStr};

use either::Either;
use flams_ontology::uris::{ArchiveId, ArchiveURI, BaseURI};
use flams_utils::{
    settings::{BuildQueueSettings, GitlabSettings, ServerSettings, SettingsSpec},
    triomphe::Arc,
};
use parking_lot::lock_api::RwLock;

use crate::{
    backend::AnyBackend,
    build_result, build_target,
    building::{BuildResult, BuildStep, BuildStepI, BuildTask, BuildTaskI, BuildTaskId, TaskState},
    formats::CHECK,
    source_format,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test() {
    const fn get_dependencies(backend: &AnyBackend, task: &BuildTask) {}

    const fn run_build_target_1(_: &AnyBackend, task: &BuildTask) -> BuildResult {
        BuildResult::empty()
    }
    const fn run_build_target_2(_: &AnyBackend, task: &BuildTask) -> BuildResult {
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
    let archive_uri = ArchiveURI::new(
        BaseURI::new_checked("gl.mathhub.info").unwrap(),
        ArchiveId::new("some_id"),
    );
    let build_steps = BuildStep(Arc::new(BuildStepI {
        target: BUILD_TARGET_1,
        state: RwLock::new(TaskState::Running),
        yields: RwLock::new(Vec::new()),
        requires: RwLock::new(flams_utils::vecmap::VecSet::new()),
        dependents: RwLock::new(Vec::new()),
    }));
    let steps = [build_steps];
    let rp = std::sync::Arc::from("some path");
    let build_task = BuildTask(Arc::new(BuildTaskI {
        id: BuildTaskId(NonZeroU32::new(3).unwrap()),
        archive: archive_uri,
        steps: Box::new(steps),
        source: Either::Right("some source".to_string()),
        rel_path: rp,
    }));

    build_result!(foo @ "Some build result");
    build_result!(bar @ "Some other build result");

    crate::initialize(TEST_SETTINGS.clone());
}

lazy_static::lazy_static! {
  static ref TEST_SETTINGS : SettingsSpec = SettingsSpec {
    mathhubs:vec![PathBuf::from("/insert/your/path/here/MathHub").into()],
    lsp:false,
    debug: Some(true),
    // irrelevant, because no server involved anyway
    server: ServerSettings {
      port:3000,
      ip:Some(std::net::IpAddr::from_str("127.0.0.1").expect("This is a valid IP")),
      admin_pwd:None,
      external_url:None
    },
    log_dir:None,
    database : None,
    gitlab : GitlabSettings{ url: None, token: None, app_id: None, app_secret: None, redirect_url:None},
    temp_dir : None,
    buildqueue:BuildQueueSettings {
      num_threads:Some(4)
    }
  };
}
