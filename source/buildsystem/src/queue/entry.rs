use flams_math_archives::formats::BuildTargetId;
use flams_utils::vecmap::VecMap;
use ftml_ontology::utils::time::Eta;
use ftml_uris::{ArchiveId, UriPath};

use crate::taskmap::buildtask::{buildtaskid::BuildTaskId, taskstate::TaskState};

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub id: BuildTaskId,
    pub archive: ArchiveId,
    pub rel_path: UriPath,
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
    TaskBlocked {
        id: BuildTaskId,
        target: BuildTargetId,
        eta: Eta,
    },
}
