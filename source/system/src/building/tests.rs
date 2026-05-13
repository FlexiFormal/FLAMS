/*
#![allow(unused_variables)]

 use std::{path::PathBuf, str::FromStr};

 use flams_math_archives::{backend::AnyBackend, formats::BuildResult, source_format};
 use flams_utils::settings::{BuildQueueSettings, ServerSettings, SettingsSpec};

 use crate::building::BuildTask;

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

    build_result!(foo @ "Some build result");
    build_result!(bar @ "Some other build result");

    crate::initialize(TEST_SETTINGS.clone());
}

lazy_static::lazy_static! {
  static ref TEST_SETTINGS : SettingsSpec = SettingsSpec {
    mathhubs:vec![PathBuf::from("/insert/your/path/here/MathHub").into()],
    lsp:false,
    debug: Some(true),
    temp_dir:None,
    database:None,
    gitlab:flams_utils::settings::GitlabSettings::default(),
    // irrelevant, because no server involved anyway
    server: ServerSettings {
      port:3000,
      ip:Some(std::net::IpAddr::from_str("127.0.0.1").expect("This is a valid IP")),
      external_url:None,
      admin_pwd:None,
    },
    log_dir:None,
    buildqueue:BuildQueueSettings {
      num_threads:Some(4)
    }
  };
}
 */

use crate::building::{BuildTask, queue::TaskMap};

 pub fn find_cycles(map: &TaskMap, task_n: &BuildTask) {
     // cycles are in the buildstep Individual build steps
     // Here the taskrefs are the buildstep that in this function exist and it is responsible for finding cylces
     // TaskRef {
     //             archive: self.0.uri.archive_id().clone(),
     //             rel_path: self.0.rel_path.clone(),
     //             target,
     //         }
     let mut paths = task_n
         .steps()
         .iter()
         .map(|b| {
             (
                 task_n.as_task_ref(b.target),
                 vec![task_n.as_task_ref(b.target)],
             )
         })
         .collect::<HashMap<TaskRef, Vec<TaskRef>>>();
     let mut visited = HashSet::new();
     // This is depth first search stack
     let mut stack = paths.keys().cloned().collect::<Vec<_>>();
     let mut cycles = HashSet::new();
     while let Some(x) = stack.pop() {
         // Just a check to see whether the buildtask exist
         visited.insert(x.clone());
         let keys = paths
             .iter()
             .filter_map(|(k, v)| {
                 if let Some(ss) = v.last() {
                     if ss == &x { Some(k.clone()) } else { None }
                 } else {
                     None
                 }
             })
             .collect::<Vec<_>>();
         if let Some(b_task) = map.map.get(&(x.archive, x.rel_path)) {
             let deps = b_task
                 .steps()
                 .iter()
                 .flat_map(|b| {
                     let deps = b.requires.read();
                     let mut dp = Vec::new();
                     for i in deps.0.iter() {
                         if let Dependency::Resolved { task, step, strict } = i
                             && *strict
                         {
                             let ref_ta = task.as_task_ref(*step);

                             if !visited.contains(&ref_ta) {
                                 for i in &keys {
                                     let ent = paths.get_mut(i).unwrap();
                                     ent.push(ref_ta.clone());
                                 }
                                 dp.push(ref_ta)
                             } else {
                                 cycles.insert(ref_ta);
                             }
                         }
                     }
                     dp
                 })
                 .collect::<Vec<_>>();
             stack.extend_from_slice(&deps);
         }
     }
     paths.retain(|k, v| cycles.contains(k));
     for i in paths {
         let mut store = String::new();
         for j in i.1.iter().rev() {
             let store2 = format!(
                 "[archiveId : {} , rel_path : {}, target : {}] -> ",
                 j.archive, j.rel_path, j.target
             );
             store.push_str(&store2);
         }
         store.pop();
         store.pop();
         store.pop();
         info!("The cylce is as follows {}", store);
     }
 }
