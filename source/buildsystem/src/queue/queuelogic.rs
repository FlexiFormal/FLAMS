use tokio::sync::Semaphore;
use tracing::instrument;

use crate::{
    graph::scheduler::Scheduler,
    queue::{Queue, QueueState, entry::QueueMessage, running::RunningQueue},
    taskmap::buildtask::BuildTask,
};

impl Queue {
    #[instrument(level = "info",
    parent=&self.0.span,
    target = "buildqueue",
    name = "Running buildqueue",
    skip_all
  )]
    pub fn start(&self, sem: Semaphore) {
        let mut state = self.0.state.write();
        if matches!(&*state, QueueState::Running(_)) {
            return;
        }
        let map = self.0.map.read();
        let mut running = RunningQueue::new(map.total);
        let scheduler = Scheduler::init(&mut running, &map);
        self.0.sender.lazy_send(|| QueueMessage::Started {
            running: Vec::new(),
            queue: running.queue.iter().map(BuildTask::as_message).collect(),
            blocked: Vec::new(),
            failed: Vec::new(),
            done: Vec::new(),
        });
        *state = QueueState::Running(running);
        drop(map);
        drop(state);
        // match sem {
        //     Semaphore::Linear => self.run_sync(),
        //     #[cfg(feature = "tokio")]
        //     Semaphore::Counting { inner: sem, .. } => {
        //         tokio::task::spawn(self.clone().run_async(sem).in_current_span());
        //     } //.in_current_span());}
        // }
    }
    async fn run_async(self, sem: std::sync::Arc<tokio::sync::Semaphore>) {
        loop {
            let Ok(permit) = tokio::sync::Semaphore::acquire_owned(sem.clone()).await else {
                break;
            };
            let Some((task, id)) = self.get_next_async().await else {
                break;
            };
            let selfclone = self.clone();
            let span = tracing::Span::current();
            tokio::task::spawn_blocking(move || {
                span.in_scope(move || selfclone.run_task_async(&task, id, permit));
            });
        }
        loop {
            if matches!(&*self.0.state.read(),QueueState::Running(RunningQueue{running,..}) if !running.is_empty())
            {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            } else {
                break;
            }
        }
        self.finish();
    }
}
