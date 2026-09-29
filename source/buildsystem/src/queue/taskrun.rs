use std::{num::NonZeroU32, sync::Arc};

use flams_math_archives::{
    backend::LocalBackend,
    formats::{BuildResult, BuildTargetId},
};
use tracing::instrument;

use crate::{
    queue::{
        Queue, QueueState,
        entry::QueueMessage,
        running::{FinishedQueue, RunningQueue},
    },
    taskmap::buildtask::{
        BuildTask,
        buildtaskid::{BuildTaskI, BuildTaskId},
        taskstate::TaskState,
    },
};

impl Queue {
    // finish function just cleans the failed queue and done queue properly
    fn finish(&self) {
        let state = &mut *self.0.state.write();
        let QueueState::Running(RunningQueue { done, failed, .. }) = &mut *state else {
            unreachable!()
        };
        let done = std::mem::take(done);
        let failed = std::mem::take(failed);
        self.0.sender.lazy_send(|| QueueMessage::Finished {
            failed: failed.iter().map(BuildTask::as_message).collect(),
            done: done.iter().map(BuildTask::as_message).collect(),
        });
        *state = QueueState::Finished(FinishedQueue { done, failed });
    }

    // requeue failed will setup the failed for second run
    #[instrument(level="info",parent=&self.0.span,skip_all,name="Requeueing failed")]
    pub fn requeue_failed(&self) {
        let mut state = self.0.state.write();
        let QueueState::Finished(FinishedQueue { failed, .. }) = &mut *state else {
            return;
        };
        let failed = std::mem::take(failed);
        *state = QueueState::Idle;
        drop(state);
        if failed.is_empty() {
            return;
        }
        let map = &mut *self.0.map.write();
        map.dependents.clear();
        map.counter = unsafe { NonZeroU32::new_unchecked(1) };
        map.total = failed.iter().map(|t| t.steps().len()).sum();
        map.map.clear();
        for t in failed {
            for s in t.steps().iter() {
                s.0.state.set(TaskState::None);
            }
            map.map.insert(
                (t.archive().id.clone(), t.rel_path().clone()),
                BuildTask::new(
                    BuildTaskId(map.counter),
                    t.archive().clone(),
                    t.steps().clone().into(),
                    match t.source() {
                        either::Either::Left(l) => either::Either::Left(l.into()),
                        either::Either::Right(r) => either::Either::Right(r.into()),
                    },
                    t.rel_path().clone(),
                )
                .unwrap(),
            );
            map.counter = map.counter.saturating_add(1);
        }
        self.0.sender.lazy_send(|| {
            QueueMessage::Idle(map.map.values().map(BuildTask::as_message).collect())
        });
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn run_task_async(
        &self,
        task: &BuildTask,
        target: BuildTargetId,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        self.run_task(task, target);
        drop(permit);
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::significant_drop_tightening)]
    fn run_task(&self, task: &BuildTask, target: BuildTargetId) {
        self.0.sender.lazy_send(|| QueueMessage::TaskStarted {
            id: task.get_id(),
            target,
        });
        tracing::info!(target:"buildqueue","building [{}]{{{}}} :: {target}",
            task.archive().id, task.rel_path());
        let spec = task.as_build_spec(&self.0.backend);
        //println!("Running task {target}");
        let BuildResult { log, result } = tracing::info_span!(target:"buildqueue","Running task",
          archive = %task.archive().id,
          rel_path = %task.rel_path(),
          format = %target
        )
        .in_scope(|| (target.run)(spec));
        //println!("Finished running task {target}");
        /*let (idx, _) = task
        .steps()
        .iter()
        .enumerate()
        .find(|(_, s)| s.0.target == target)
        .unwrap_or_else(|| unreachable!());*/
        let mut lock = self.0.state.write();
        let QueueState::Running(state) = &mut *lock else {
            unreachable!()
        };
        state.running.retain(|t| t != task);

        let eta = state.timer.update(1);

        let (mut err, res) = match result {
            Ok(res) => (None, res),
            Err(e) => (Some(e), None),
        };

        if let Err(e) =
            self.0
                .backend
                .save(task.document_uri(), Some(task.rel_path()), log, target, res)
        {
            tracing::error!("Error saving build result: {e}");
            if err.is_none() {
                err = Some(Vec::new());
            }
        }

        match err {
            None => {
                let mut found = false;
                let mut requeue = false;
                for s in task.steps() {
                    if s.0.target == target {
                        found = true;
                        s.0.state.set(TaskState::Done);
                    } else if found {
                        s.0.state.set(TaskState::Queued);
                        requeue = true;
                        break;
                    }
                }
                if requeue {
                    state.queue.push_front(task.clone());
                    tracing::info!(target:"buildqueue","done [{}]{{{}}} :: {target}, next pipeline step queued",
                        task.archive().id, task.rel_path());
                } else {
                    state.done.push(task.clone());
                    tracing::info!(target:"buildqueue","done [{}]{{{}}} :: {target}, task complete",
                        task.archive().id, task.rel_path());
                }
                drop(lock);

                self.0.sender.lazy_send(|| QueueMessage::TaskSuccess {
                    id: task.get_id(),
                    target,
                    eta,
                });
            }
            Some(deps) => {
                /*
                let mut block = false;
                for d in deps {
                    match d {
                        flams_math_archives::formats::TaskDependency::Physical { task, strict } => {
                            if state.running.iter().chain(state.blocked.iter()).any(|t| task.archive == t.archive().id.id && task.rel_path == *t.rel_path() && t.get_step(task.target).is_some()) {
                                block = true;
                            }
                        }
                        flams_math_archives::formats::TaskDependency::Logical { uri, strict } => {
                            self.backend().with_local_archive(uri, |a| if let Some(a) = a {
                                a.do
                            })
                        }

                    }
                } */
                let mut found = false;
                if deps.is_empty() || state.queue.is_empty() {
                    for s in task.steps() {
                        if s.0.target == target {
                            found = true;
                        }
                        if found {
                            s.0.state.set(TaskState::Failed);
                        }
                    }
                    state.failed.push(task.clone());
                    tracing::info!(target:"buildqueue","FAILED [{}]{{{}}} :: {target}",
                        task.archive().id, task.rel_path());
                    self.0.sender.lazy_send(|| QueueMessage::TaskFailed {
                        id: task.get_id(),
                        target,
                        eta,
                    });
                } else {
                    for i in deps {
                        match i {
                            flams_math_archives::formats::TaskDependency::Physical {
                                task,
                                strict,
                            } => {
                                let map = self.0.map.read();
                                let task_in_map = map.map.get(&(task.archive, task.rel_path));
                                if task_in_map.is_some() {
                                    // here you can report some kind of meaning ful error
                                    continue;
                                } else {
                                    // we create one here and add to the graph in the Queue
                                    // here its a failure
                                }
                            }
                            flams_math_archives::formats::TaskDependency::Logical {
                                uri,
                                strict,
                            } => {
                                // here check the modules that are in the backend
                            }
                        }
                    }
                    let mut found = false;
                    for s in task.steps() {
                        if s.0.target == target {
                            found = true;
                        }
                        if found {
                            s.0.state.set(TaskState::Blocked);
                        }
                    }
                    state.blocked.push(task.clone());
                    tracing::info!(target:"buildqueue","blocked (deps outstanding) [{}]{{{}}} :: {target}",
                        task.archive().id, task.rel_path());
                    self.0.sender.lazy_send(|| QueueMessage::TaskBlocked {
                        id: task.get_id(),
                        target,
                        eta,
                    });
                }
                drop(lock);
            }
        }
    }
}
