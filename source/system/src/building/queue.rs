use crate::{backend::backend, FlamsExtension};

use super::{
    queue_manager::{QueueId, Semaphore},
    BuildTask, BuildTaskId, Eta, QueueMessage, TaskRef, TaskState,
};
use flams_math_archives::{
    backend::{AnyBackend, LocalBackend},
    formats::{BuildResult, BuildTargetId, FormatOrTargets},
    manager::ArchiveOrGroup,
    source_files::{SourceEntry, SourceEntryRef},
    utils::path_ext::RelPath,
    Archive, LocallyBuilt, MathArchive,
};
use flams_utils::{
    change_listener::{ChangeListener, ChangeSender},
    prelude::HMap,
    triomphe::Arc,
};
use ftml_ontology::utils::{time::Timestamp, RefTree};
use ftml_uris::{ArchiveId, UriPath, UriWithArchive};
use parking_lot::RwLock;
use std::{collections::VecDeque, num::NonZeroU32};
use tracing::{instrument, Instrument};

#[derive(Debug)]
pub(super) struct TaskMap {
    pub(super) map: HMap<(ArchiveId, UriPath), BuildTask>,
    pub(super) dependents: HMap<TaskRef, Vec<(BuildTask, BuildTargetId)>>,
    pub(super) counter: NonZeroU32,
    pub(super) total: usize,
}

impl Default for TaskMap {
    fn default() -> Self {
        Self {
            map: HMap::default(),
            dependents: HMap::default(),
            counter: NonZeroU32::new(1).unwrap_or_else(|| unreachable!()),
            total: 0,
        }
    }
}

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
pub(super) struct QueueI {
    backend: AnyBackend,
    name: QueueName,
    pub id: QueueId,
    span: tracing::Span,
    pub(super) map: RwLock<TaskMap>,
    pub(super) sender: ChangeSender<QueueMessage>,
    pub(super) state: RwLock<QueueState>,
}

#[derive(Debug, Clone)]
pub struct Queue(pub(super) Arc<QueueI>);

impl Queue {
    pub(crate) fn new(id: QueueId, name: QueueName, backend: AnyBackend) -> Self {
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
        tracing::info_span!("sorting...").in_scope(|| {
            Self::sort(&map, &mut running);
            tracing::info!("Done");
        });
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
        match sem {
            Semaphore::Linear => self.run_sync(),
            #[cfg(feature = "tokio")]
            Semaphore::Counting { inner: sem, .. } => {
                tokio::task::spawn(self.clone().run_async(sem).in_current_span());
            } //.in_current_span());}
        }
    }

    #[inline]
    fn run_sync(&self) {
        while let Some((task, id)) = self.get_next() {
            self.run_task(&task, id);
        }
        self.finish();
    }

    #[cfg(feature = "tokio")]
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

    fn finish(&self) {
        let state = &mut *self.0.state.write();
        let QueueState::Running(RunningQueue { done, failed, .. }) = state else {
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
        map.total = failed.iter().map(|t| t.0.steps.len()).sum();
        map.map.clear();
        for t in failed {
            for s in &t.0.steps {
                s.0.state.set(TaskState::None);
            }
            map.map.insert(
                (t.archive().archive_id().clone(), t.0.rel_path.clone()),
                BuildTask(Arc::new(super::BuildTaskI {
                    id: BuildTaskId(map.counter),
                    rel_path: t.0.rel_path.clone(),
                    uri: t.0.uri.clone(),
                    steps: t.0.steps.clone(),
                    source: t.0.source.clone(),
                })),
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
            id: task.0.id,
            target,
        });
        let spec = task.as_build_spec(&self.0.backend);
        //println!("Running task {target}");
        let BuildResult { log, result } = tracing::info_span!(target:"buildqueue","Running task",
          archive = %task.0.uri.archive_id(),
          rel_path = %task.0.rel_path,
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
        let QueueState::Running(ref mut state) = &mut *lock else {
            unreachable!()
        };
        state.running.retain(|t| t != task);
        let eta = state.timer.update(1);

        match result {
            Err(deps) => {
                /*
                let mut block = false;
                for d in deps {
                    match d {
                        flams_math_archives::formats::TaskDependency::Physical { task, strict } => {
                            if state.running.iter().chain(state.blocked.iter()).any(|t| task.archive == t.archive().id && task.rel_path == *t.rel_path() && t.get_step(task.target).is_some()) {
                                block = true;
                            }
                        }
                        flams_math_archives::formats::TaskDependency::Logical { uri, strict } => {
                            self.backend().with_local_archive(uri.archive_id(), |a| if let Some(a) = a {
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
                    self.0.sender.lazy_send(|| QueueMessage::TaskFailed {
                        id: task.0.id,
                        target,
                        eta,
                    });
                } else {
                    // TODO: handle dependencies
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
                    self.0.sender.lazy_send(|| QueueMessage::TaskBlocked {
                        id: task.0.id,
                        target,
                        eta,
                    });
                }
                drop(lock);

                let _ = self.0.backend.save(
                    task.document_uri(),
                    Some(task.rel_path()),
                    log,
                    target,
                    None,
                );
            }
            Ok(data) => {
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
                } else {
                    state.done.push(task.clone());
                }
                drop(lock);
                if let Some(data) = data.as_ref() {
                    for e in inventory::iter::<FlamsExtension>() {
                        (e.on_build_result)(
                            &self.0.backend,
                            task.document_uri(),
                            task.rel_path(),
                            &**data,
                        );
                    }
                }

                let _ = self.0.backend.save(
                    task.document_uri(),
                    Some(task.rel_path()),
                    log,
                    target,
                    data,
                );

                self.0.sender.lazy_send(|| QueueMessage::TaskSuccess {
                    id: task.0.id,
                    target,
                    eta,
                });
            }
        }
    }

    fn maybe_restart(&self) {
        let mut state = self.0.state.write();
        if let QueueState::Finished(_) = &mut *state {
            drop(state);
            self.requeue_failed();
        }
    }

    #[instrument(level = "info",
    parent=&self.0.span,
    target = "buildqueue",
    name = "Queueing tasks",
    skip_all
  )]
    #[deprecated(note = "needs refactoring: archive need not be local, etc.")]
    pub fn enqueue_all(&self, target: FormatOrTargets<'_>, stale_only: bool, clean: bool) -> usize {
        self.maybe_restart();
        if let AnyBackend::Sandbox(b) = &self.0.backend {
            b.clear();
            backend().with_archives(|archives| {
                for a in archives {
                    let Archive::Local(archive) = a else { continue };
                    b.maybe_copy(archive);
                    if clean {
                        let _ = std::fs::remove_dir_all(b.path_for(archive.id()).join(".flams"));
                    }
                }
                b.load_all();
            });
        };
        let mut acc = 0;
        self.0.backend.with_archives(|archives| {
            for a in archives {
                let Archive::Local(archive) = a else { continue };
                acc += archive.with_sources(|d| {
                    let d = d.dfs();
                    let map = &mut *self.0.map.write();
                    Self::enqueue(
                        map,
                        &self.0.backend,
                        a,
                        target,
                        stale_only,
                        d.filter_map(|e| match e {
                            SourceEntry::Dir(_) => None,
                            SourceEntry::File(f) => Some(f),
                        }),
                    )
                });
            }
        });
        acc
    }

    #[instrument(level = "info",
    parent=&self.0.span,
    target = "buildqueue",
    name = "Queueing tasks",
    skip_all
  )]
    #[deprecated(note = "needs refatoring: assumes LocalArchive everywhere")]
    pub fn enqueue_group(
        &self,
        id: &ArchiveId,
        target: FormatOrTargets,
        stale_only: bool,
        clean: bool,
    ) -> usize {
        self.maybe_restart();
        if let AnyBackend::Sandbox(b) = &self.0.backend {
            b.require(id, false);
        }
        self.0.backend.with_archive_or_group(id, |g| match g {
            None => 0,
            Some(ArchiveOrGroup::Archive(id)) => self.0.backend.with_archive(id, |a| {
                let Some(archive) = a else { return 0 };
                if clean {
                    if let AnyBackend::Sandbox(b) = &self.0.backend {
                        let _ = std::fs::remove_dir_all(b.path_for(archive.id()).join(".flams"));
                    } else if let Archive::Local(a) = archive {
                        let _ = std::fs::remove_dir_all(a.out_dir());
                    }
                }
                if let Archive::Local(a) = archive {
                    a.with_sources(|d| {
                        let map = &mut *self.0.map.write();
                        Self::enqueue(
                            map,
                            &self.0.backend,
                            archive,
                            target,
                            stale_only,
                            d.dfs().filter_map(|e| match e {
                                SourceEntry::Dir(_) => None,
                                SourceEntry::File(f) => Some(f),
                            }),
                        )
                    })
                } else {
                    0
                }
            }),
            Some(ArchiveOrGroup::Group(g)) => {
                let map = &mut *self.0.map.write();
                let mut ret = 0;
                for id in g.dfs().filter_map(|e| match e {
                    ArchiveOrGroup::Archive(id) => Some(id),
                    ArchiveOrGroup::Group(_) => None,
                }) {
                    ret += self.0.backend.with_archive(id, |a| {
                        let Some(archive) = a else { return 0 };

                        if clean {
                            if let AnyBackend::Sandbox(b) = &self.0.backend {
                                let _ = std::fs::remove_dir_all(
                                    b.path_for(archive.id()).join(".flams"),
                                );
                            } else if let Archive::Local(a) = archive {
                                let _ = std::fs::remove_dir_all(a.out_dir());
                            }
                        }
                        if let Archive::Local(a) = archive {
                            a.with_sources(|d| {
                                Self::enqueue(
                                    map,
                                    &self.0.backend,
                                    archive,
                                    target,
                                    stale_only,
                                    d.dfs().filter_map(|e| match e {
                                        SourceEntry::Dir(_) => None,
                                        SourceEntry::File(f) => Some(f),
                                    }),
                                )
                            })
                        } else {
                            0
                        }
                    });
                }
                ret
            }
        })
    }

    #[instrument(level = "info",
    parent=&self.0.span,
    target = "buildqueue",
    name = "Queueing tasks",
    skip_all
  )]
    #[deprecated(note = "needs refatoring: assumes LocalArchive everywhere")]
    pub fn enqueue_archive(
        &self,
        id: &ArchiveId,
        target: FormatOrTargets,
        stale_only: bool,
        rel_path: Option<RelPath<'_>>,
        clean: bool,
    ) -> usize {
        self.maybe_restart();
        if let AnyBackend::Sandbox(b) = &self.0.backend {
            b.require(id, true);
        }
        self.0.backend.with_archive(id, |archive| {
            let Some(archive) = archive else { return 0 };
            if clean {
                if let AnyBackend::Sandbox(b) = &self.0.backend {
                    let _ = std::fs::remove_dir_all(b.path_for(archive.id()).join(".flams"));
                } else if let Archive::Local(a) = archive {
                    let _ = std::fs::remove_dir_all(a.out_dir());
                }
            }
            if let Archive::Local(a) = archive {
                a.with_sources(|d| match rel_path {
                    None => {
                        let map = &mut *self.0.map.write();
                        Self::enqueue(
                            map,
                            &self.0.backend,
                            archive,
                            target,
                            stale_only,
                            d.dfs().filter_map(|e| match e {
                                SourceEntry::Dir(_) => None,
                                SourceEntry::File(f) => Some(f),
                            }),
                        )
                    }
                    Some(p) => {
                        let Some(d) = d.find(p) else { return 0 };
                        match d {
                            SourceEntryRef::Dir(d) => {
                                let map = &mut *self.0.map.write();
                                Self::enqueue(
                                    map,
                                    &self.0.backend,
                                    archive,
                                    target,
                                    stale_only,
                                    d.dfs().filter_map(|e| match e {
                                        SourceEntry::Dir(_) => None,
                                        SourceEntry::File(f) => Some(f),
                                    }),
                                )
                            }
                            SourceEntryRef::File(f) => {
                                let map = &mut *self.0.map.write();
                                Self::enqueue(
                                    map,
                                    &self.0.backend,
                                    archive,
                                    target,
                                    stale_only,
                                    std::iter::once(f),
                                )
                            }
                        }
                    }
                })
            } else {
                0
            }
        })
    }
}

#[derive(Debug)]
pub struct RunningQueue {
    pub(super) queue: VecDeque<BuildTask>,
    pub(super) blocked: Vec<BuildTask>,
    pub(super) done: Vec<BuildTask>,
    pub(super) failed: Vec<BuildTask>,
    pub(super) running: Vec<BuildTask>,
    timer: Timer,
}
impl RunningQueue {
    fn new(total: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            failed: Vec::new(),
            blocked: Vec::new(),
            done: Vec::new(),
            running: Vec::new(),
            timer: Timer::new(total),
        }
    }
}

#[derive(Debug)]
pub struct FinishedQueue {
    pub(super) done: Vec<BuildTask>,
    pub(super) failed: Vec<BuildTask>,
}

#[derive(Debug)]
struct Timer {
    started: Timestamp,
    steps: usize,
    done: usize,
}
impl Timer {
    fn new(total: usize) -> Self {
        Self {
            started: Timestamp::now(),
            steps: total,
            done: 0,
        }
    }
    #[allow(clippy::cast_precision_loss)]
    fn update(&mut self, dones: u8) -> Eta {
        self.done += dones as usize;
        let avg = self.started.since_now() * (1.0 / (self.done as f64));
        let time_left = avg * ((self.steps - self.done) as f64);
        Eta {
            time_left,
            done: self.done,
            total: self.steps,
        }
    }
}
