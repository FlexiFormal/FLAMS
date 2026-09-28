use std::sync::Arc;

use flams_math_archives::backend::AnyBackend;
use flams_utils::{
    change_listener::{ChangeListener, ChangeSender},
    parking_lot::RwLock,
};
use tracing::instrument;

use crate::{
    queue::{
        entry::QueueMessage,
        queueid::QueueId,
        running::{FinishedQueue, RunningQueue},
    },
    taskmap::{TaskMap, buildtask::BuildTask},
};

pub mod entry;
pub mod queueid;
pub mod running;

#[derive(Debug)]
pub enum QueueState {
    Running(RunningQueue),
    Idle,
    Finished(FinishedQueue),
}

#[derive(Debug, Clone)]
pub enum QueueName {
    Global,
    Sandbox { name: std::sync::Arc<str>, idx: u16 },
}
impl std::fmt::Display for QueueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => f.write_str("global"),
            Self::Sandbox { name, idx } => {
                f.write_str(name)?;
                idx.fmt(f)
            }
        }
    }
}

#[derive(Debug)]
pub struct QueueI {
    backend: AnyBackend,
    name: QueueName,
    pub id: QueueId,
    span: tracing::Span,
    pub map: RwLock<TaskMap>,
    pub sender: ChangeSender<QueueMessage>,
    pub state: RwLock<QueueState>,
}

#[derive(Debug, Clone)]
pub struct Queue(pub Arc<QueueI>);

impl Queue {
    pub fn new(id: QueueId, name: QueueName, backend: AnyBackend) -> Self {
        Self(Arc::new(QueueI {
            id,
            name,
            backend,
            span: tracing::Span::current(),
            map: RwLock::default(),
            sender: ChangeSender::new(32),
            state: RwLock::new(QueueState::Idle),
        }))
    }

    #[inline]
    #[must_use]
    pub fn backend(&self) -> &AnyBackend {
        &self.0.backend
    }

    #[must_use]
    pub fn listener(&self) -> ChangeListener<QueueMessage> {
        self.0.sender.listener()
    }

    #[instrument(level="info",parent=&self.0.span,skip_all,name="Collecting queue state")]
    pub fn state_message(&self) -> QueueMessage {
        match &*self.0.state.read() {
            QueueState::Running(RunningQueue {
                running,
                queue,
                blocked,
                failed,
                done,
                ..
            }) => QueueMessage::Started {
                running: running.iter().map(BuildTask::as_message).collect(),
                queue: queue.iter().map(BuildTask::as_message).collect(),
                blocked: blocked.iter().map(BuildTask::as_message).collect(),
                failed: failed.iter().map(BuildTask::as_message).collect(),
                done: done.iter().map(BuildTask::as_message).collect(),
            },
            QueueState::Idle => QueueMessage::Idle(
                self.0
                    .map
                    .read()
                    .map
                    .values()
                    .map(BuildTask::as_message)
                    .collect(),
            ),
            QueueState::Finished(FinishedQueue { done, failed }) => QueueMessage::Finished {
                failed: failed.iter().map(BuildTask::as_message).collect(),
                done: done.iter().map(BuildTask::as_message).collect(),
            },
        }
    }

    #[inline]
    #[must_use]
    pub fn name(&self) -> &QueueName {
        &self.0.name
    }
}
