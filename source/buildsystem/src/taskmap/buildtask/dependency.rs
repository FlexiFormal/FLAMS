use flams_math_archives::formats::{BuildTargetId, TaskDependency, TaskRef};
use ftml_uris::ModuleUri;

use crate::taskmap::buildtask::BuildTask;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    Physical {
        task: TaskRef,
        strict: bool,
    },
    Logical {
        uri: ModuleUri,
        strict: bool,
    },
    Resolved {
        task: BuildTask,
        step: BuildTargetId,
        strict: bool,
    },
}
impl From<TaskDependency> for Dependency {
    fn from(value: TaskDependency) -> Self {
        match value {
            TaskDependency::Logical { uri, strict } => Self::Logical { uri, strict },
            TaskDependency::Physical { task, strict } => Self::Physical { task, strict },
        }
    }
}
