use std::{
    any::Any,
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use either::Either;
use flams_ontology::{
    languages::Language,
    uris::{
        ArchiveId, ArchiveURI, ArchiveURIRef, ArchiveURITrait, DocumentURI, ModuleURI, URIRefTrait,
    },
};
use flams_utils::{
    time::Eta,
    triomphe::Arc,
    vecmap::{VecMap, VecSet},
};
use parking_lot::RwLock;

use crate::formats::{BuildArtifactTypeId, BuildTargetId};

mod queue;
pub mod queue_manager;
pub use queue::QueueName;
mod buildtool;
mod queueing;

#[cfg(all(test, feature = "tokio"))]
mod tests;

pub use queue::Queue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskState {
    Running,
    Queued,
    Blocked,
    Done,
    Failed,
    None,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TaskRef {
    pub archive: ArchiveId,
    pub rel_path: std::sync::Arc<str>,
    pub target: BuildTargetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    Physical {
        task: TaskRef,
        strict: bool,
    },
    Logical {
        uri: ModuleURI,
        strict: bool,
    },
    Resolved {
        task: BuildTask,
        step: BuildTargetId,
        strict: bool,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BuildTaskId(NonZeroU32);
impl From<BuildTaskId> for u32 {
    #[inline]
    fn from(id: BuildTaskId) -> Self {
        id.0.get()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BuildTaskI {
    id: BuildTaskId,
    archive: ArchiveURI,
    steps: Box<[BuildStep]>,
    source: Either<PathBuf, String>,
    rel_path: std::sync::Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTask(Arc<BuildTaskI>);
impl BuildTask {
    #[must_use]
    #[inline]
    pub fn document_uri(&self) -> DocumentURI {
        DocumentURI::from_archive_relpath(self.archive().owned(), self.rel_path())
    }
    #[must_use]
    pub fn get_task_ref(&self, target: BuildTargetId) -> TaskRef {
        TaskRef {
            archive: self.0.archive.archive_id().clone(),
            rel_path: self.0.rel_path.clone(),
            target,
        }
    }

    #[inline]
    #[must_use]
    pub fn source(&self) -> Either<&Path, &str> {
        match &self.0.source {
            Either::Left(p) => Either::Left(p),
            Either::Right(s) => Either::Right(s),
        }
    }

    #[inline]
    #[must_use]
    pub fn archive(&self) -> ArchiveURIRef {
        self.0.archive.archive_uri()
    }

    #[inline]
    #[must_use]
    pub fn rel_path(&self) -> &str {
        &self.0.rel_path
    }

    #[inline]
    #[must_use]
    pub fn steps(&self) -> &[BuildStep] {
        &self.0.steps
    }

    #[inline]
    #[must_use]
    pub fn get_step(&self, target: BuildTargetId) -> Option<&BuildStep> {
        self.0.steps.iter().find(|s| s.0.target == target)
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_message(&self) -> QueueEntry {
        /*let idx = self.steps().iter().enumerate().find(|s|
            matches!(&*s.1.0.state.read(),TaskState::Running | TaskState::Queued | TaskState::Blocked | TaskState::Failed)
        );
        let idx = if let Some((idx,_)) = idx {(idx - 1) as u8} else {self.steps().len() as u8};
        */
        QueueEntry {
            id: self.0.id,
            archive: self.0.archive.archive_id().clone(),
            rel_path: self.0.rel_path.clone(),
            steps: self
                .steps()
                .iter()
                .map(|s| (s.0.target, *s.0.state.read()))
                .collect(),
        }
    }
}

#[derive(Debug)]
struct BuildStepI {
    //task:std::sync::Weak<BuildTaskI>,
    target: BuildTargetId,
    state: RwLock<TaskState>,
    yields: RwLock<Vec<ModuleURI>>,
    requires: RwLock<VecSet<Dependency>>,
    dependents: RwLock<Vec<(BuildTaskId, BuildTargetId)>>,
}
impl PartialEq for BuildStepI {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}
impl Eq for BuildStepI {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStep(Arc<BuildStepI>);
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

pub trait BuildArtifact: Any + 'static {
    fn get_type_id() -> BuildArtifactTypeId
    where
        Self: Sized;
    /// #### Errors
    fn load(p: &Path) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn get_type(&self) -> BuildArtifactTypeId;
    /// ### Errors
    fn write(&self, path: &Path) -> Result<(), std::io::Error>;
    fn as_any(&self) -> &dyn Any;
}
/// Build Result Artifact is either a File Constructor which is an output of a build process i.e TEX => PDF
/// Data Constructor takes any Struct that implements a BuildArtifact Trait
pub enum BuildResultArtifact {
    File(BuildArtifactTypeId, PathBuf),
    Data(Box<dyn BuildArtifact>),
    None,
}

pub struct BuildResult {
    pub log: Either<String, PathBuf>,
    pub result: Result<BuildResultArtifact, Vec<Dependency>>,
}
impl BuildResult {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            log: Either::Left(String::new()),
            result: Ok(BuildResultArtifact::None),
        }
    }
    #[must_use]
    pub const fn err() -> Self {
        Self {
            log: Either::Left(String::new()),
            result: Err(Vec::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub id: BuildTaskId,
    pub archive: ArchiveId,
    pub rel_path: std::sync::Arc<str>,
    pub steps: VecMap<BuildTargetId, TaskState>,
}

#[derive(Debug, Clone)]
pub enum QueueMessage {
    Idle(Vec<QueueEntry>),
    Started {
        running: Vec<QueueEntry>,
        queue: Vec<QueueEntry>,
        blocked: Vec<QueueEntry>,
        failed: Vec<QueueEntry>,
        done: Vec<QueueEntry>,
    },
    Finished {
        failed: Vec<QueueEntry>,
        done: Vec<QueueEntry>,
    },
    TaskStarted {
        id: BuildTaskId,
        target: BuildTargetId,
    },
    TaskSuccess {
        id: BuildTaskId,
        target: BuildTargetId,
        eta: Eta,
    },
    TaskFailed {
        id: BuildTaskId,
        target: BuildTargetId,
        eta: Eta,
    },
}
