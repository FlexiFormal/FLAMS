use std::sync::Arc;

use flams_math_archives::formats::BuildTargetId;
use flams_utils::{parking_lot::RwLock, vecmap::VecSet};

use crate::taskmap::buildtask::{
    buildtaskid::BuildTaskId, dependency::Dependency, taskstate::AtomicTaskState,
};

#[derive(Debug)]
pub struct BuildStepI {
    //task:std::sync::Weak<BuildTaskI>,
    pub target: BuildTargetId,
    pub state: AtomicTaskState,
    //yields:RwLock<Vec<ModuleUri>>,
    pub requires: RwLock<VecSet<Dependency>>,
    pub dependents: RwLock<Vec<(BuildTaskId, BuildTargetId)>>,
}
impl PartialEq for BuildStepI {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl Eq for BuildStepI {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStep(pub(crate) Arc<BuildStepI>);
impl BuildStep {
    pub fn add_dependency(&self, dep: Dependency) {
        self.0.requires.write().insert(dep);
    }
    /*
    #[must_use]
    pub fn get_task(&self) -> BuildTask {
        BuildTask(self.0.task.upgrade().unwrap_or_else(|| unreachable!()))
    }
    */
}

impl std::ops::Deref for BuildStep {
    type Target = BuildStepI;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
