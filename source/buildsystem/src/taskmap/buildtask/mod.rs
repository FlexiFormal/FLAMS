use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use either::Either;
use flams_math_archives::{
    backend::AnyBackend,
    formats::{BuildSpec, BuildTargetId, TaskRef},
};
use ftml_uris::{ArchiveUri, DocumentUri, UriPath, UriWithArchive};

use crate::{
    queue::entry::QueueEntry,
    taskmap::buildtask::{buildstep::BuildStep, buildtaskid::BuildTaskId},
};
pub mod buildstep;
pub mod buildtaskid;
pub mod dependency;
pub mod taskstate;

#[derive(Debug, PartialEq, Eq)]
struct BuildTaskI {
    id: BuildTaskId,
    uri: DocumentUri,
    steps: Box<[BuildStep]>,
    source: Either<PathBuf, String>,
    rel_path: UriPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTask(pub(crate) Arc<BuildTaskI>);
impl BuildTask {
    #[inline]
    /// # Errors
    pub fn new(
        id: BuildTaskId,
        archive: ArchiveUri,
        steps: Box<[BuildStep]>,
        source: Either<PathBuf, String>,
        rel_path: UriPath,
    ) -> eyre::Result<Self> {
        let uri = DocumentUri::from_archive_relpath(archive, rel_path.as_ref())
            .map_err(eyre::Report::new)?;
        Ok(Self(Arc::new(BuildTaskI {
            uri,
            id,
            steps,
            source,
            rel_path,
        })))
    }

    #[must_use]
    #[inline]
    pub fn document_uri(&self) -> &DocumentUri {
        &self.0.uri
    }

    #[must_use]
    pub fn as_build_spec<'a>(&'a self, backend: &'a AnyBackend) -> BuildSpec<'a> {
        BuildSpec {
            uri: &self.0.uri,
            source: self.source(),
            backend,
            rel_path: self.rel_path(),
        }
    }

    #[must_use]
    pub fn as_task_ref(&self, target: BuildTargetId) -> TaskRef {
        TaskRef {
            archive: self.0.uri.archive_id().clone(),
            rel_path: self.0.rel_path.clone(),
            target,
        }
    }

    pub fn get_id(&self) -> BuildTaskId {
        self.0.id
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
    pub fn archive(&self) -> &ArchiveUri {
        self.0.uri.archive_uri()
    }

    #[inline]
    #[must_use]
    pub fn rel_path(&self) -> &UriPath {
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
            archive: self.0.uri.archive_id().clone(),
            rel_path: self.0.rel_path.clone(),
            steps: self
                .steps()
                .iter()
                .map(|s| (s.0.target, s.0.state.get()))
                .collect(),
        }
    }
}
