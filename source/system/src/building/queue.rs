use std::{collections::VecDeque, num::NonZeroU32};
use either::Either;
use immt_ontology::uris::{ArchiveId,ArchiveURITrait};
use immt_utils::{change_listener::{ChangeListener, ChangeSender}, prelude::{HMap, TreeLike}, time::Delta, triomphe::Arc};
use parking_lot::RwLock;
use tracing::{instrument, Instrument};
use crate::{backend::{archives::{source_files::SourceEntry, ArchiveOrGroup}, AnyBackend, Backend}, formats::{BuildTargetId, FormatOrTargets}};
use super::{queue_manager::{QueueId, Semaphore}, BuildResult, BuildTask, Eta, QueueMessage, TaskRef, TaskState };
use immt_utils::time::Timestamp;

#[derive(Debug)]
pub(super) struct TaskMap {
  pub(super)map:HMap<(ArchiveId,std::sync::Arc<str>),BuildTask>,
  pub(super)dependents: HMap<TaskRef,Vec<(BuildTask,BuildTargetId)>>,
  pub(super)counter:NonZeroU32,
  pub(super) total:usize
}

impl Default for TaskMap {
  fn default() -> Self {
    Self { map:HMap::default(),dependents:HMap::default(),counter:NonZeroU32::new(1).unwrap_or_else(|| unreachable!()),total:0 }
  }
}

#[derive(Debug)]
pub enum QueueState {
  Running(RunningQueue),
  Idle
}

#[derive(Debug)]
pub(super) struct QueueI {
  backend:AnyBackend,
  name:std::sync::Arc<str>,
  pub id:QueueId,
  pub(super) map:RwLock<TaskMap>,
  sender:ChangeSender<QueueMessage>,
  pub(super) state:RwLock<QueueState>
}

#[derive(Debug,Clone)]
pub struct Queue(pub(super) Arc<QueueI>);

impl Queue {
  pub(crate) fn new(id:QueueId,name:std::sync::Arc<str>,backend:AnyBackend) -> Self {
    Self(Arc::new(QueueI { 
      id,name,
      backend,
      map:RwLock::default(),
      sender:ChangeSender::new(32),
      state:RwLock::new(QueueState::Idle) 
    }))
  }

  #[must_use]
  pub fn listener(&self) -> ChangeListener<QueueMessage> { self.0.sender.listener() }

  pub fn state_message(&self) -> QueueMessage {
    match &*self.0.state.read() {
      QueueState::Running(RunningQueue{running,queue,blocked,failed,done,..}) => 
        QueueMessage::Started { 
          running: running.iter().map(BuildTask::as_message).collect(),
          queue: queue.iter().map(BuildTask::as_message).collect(), 
          blocked: blocked.iter().map(BuildTask::as_message).collect(),
          failed: failed.iter().map(BuildTask::as_message).collect(), 
          done: done.iter().map(BuildTask::as_message).collect(),
        },
      QueueState::Idle =>
        QueueMessage::Idle(self.0.map.read().map.values().map(BuildTask::as_message).collect())
    }
  }

  #[inline]#[must_use]
  pub fn name(&self) -> &std::sync::Arc<str> { &self.0.name }

  #[instrument(level = "info",
    target = "buildqueue",
    name = "Running buildqueue",
    fields(id = %self.0.id),
    skip_all
  )]
  pub fn start(&self,sem:Semaphore) {
    let mut state = self.0.state.write();
    if matches!(&*state,QueueState::Running(_)) { return }
    let map = self.0.map.read();
    let mut running = RunningQueue::new(map.total);
    tracing::info_span!(target:"buildqueue","sorting...").in_scope(|| {
      Self::sort(&map,&mut running);
      tracing::info!("Done");
    });
    self.0.sender.lazy_send(|| QueueMessage::Started { 
      running: Vec::new(), 
      queue: running.queue.iter().map(BuildTask::as_message).collect(), 
      blocked: Vec::new(),
      failed: Vec::new(), 
      done: Vec::new()
    });
    *state = QueueState::Running(running);
    drop(map);
    drop(state);
    match sem {
      Semaphore::Linear => self.run_sync(),
      #[cfg(feature="tokio")]
      Semaphore::Counting { inner:sem, .. } => 
        {tokio::task::spawn(self.clone().run_async(sem).in_current_span());}
    }
  }

  fn run_sync(&self) {
    let mut timer = Timestamp::now();
    while let Some((task,id)) = self.get_next() {
      let old = timer;
      timer = Timestamp::now();
      self.run_task(task, id,old);
    }
  }

  #[cfg(feature="tokio")]
  async fn run_async(self,sem:std::sync::Arc<tokio::sync::Semaphore>) {
    loop {
      let timer = Timestamp::now();
      let Ok(permit) = tokio::sync::Semaphore::acquire_owned(sem.clone()).await else {
        return
      };
      let Some((task,id)) = self.get_next() else {
        return
      };
      let selfclone = self.clone();
      tokio::task::spawn_blocking(move || selfclone.run_task_async(task,id,permit,timer));
    }
  }


  #[cfg(feature="tokio")]
  fn run_task_async(&self,task:BuildTask,target:BuildTargetId,permit:tokio::sync::OwnedSemaphorePermit,timer:Timestamp) {
    self.run_task(task, target, timer);
    drop(permit);
  }

  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::significant_drop_tightening)]
  fn run_task(&self,task:BuildTask,target:BuildTargetId,timer:Timestamp) {
    let started = timer.since_now();
    self.0.sender.lazy_send(|| QueueMessage::TaskStarted{
      id:task.0.id,target
    });
    let BuildResult {log,result} = 
      tracing::info_span!(target:"buildqueue","Running task",
        archive = %task.0.archive.archive_id(),
        rel_path = %task.0.rel_path,
        format = %target
      ).in_scope(|| (target.run())(&self.0.backend,&task));
    let (idx,_) = task.steps().iter().enumerate().find(|(_,s)| s.0.target == target).unwrap_or_else(|| unreachable!());
    let mut lock = self.0.state.write();
    let QueueState::Running(ref mut state) = &mut *lock else {unreachable!()};

    match result {
      Err(_deps) => { // TODO: handle dependencies
        self.0.backend.with_archive(task.archive().archive_id(), |a| {
          let Some(a) = a else {return};
          a.save(task.rel_path(), log, target,None);
        });
        let num = (task.steps().len() - idx) as u8;
        state.running.retain(|t| *t != task);
        let mut found = false;
        for s in task.steps() {
          if s.0.target == target {
            found = true;
          } 
          if found {
            *s.0.state.write() = TaskState::Failed;
          }
        }
        let eta = state.timer.update(started, num);
        self.0.sender.lazy_send(|| QueueMessage::TaskFailed {
          id:task.0.id,target,eta
        });
        state.failed.push(task);
      }
      Ok(data) => {
        self.0.backend.with_archive(task.archive().archive_id(), |a| {
          let Some(a) = a else {return};
          a.save(task.rel_path(), log,target, Some(data));
        });
        state.running.retain(|t| *t != task);

        let mut found = false;
        let mut requeue = false;
        for s in task.steps() {
          if s.0.target == target {
            found = true;
            *s.0.state.write() = TaskState::Done;
          } else if found {
            *s.0.state.write() = TaskState::Queued;
            requeue = true;
            break
          }
        }
        let eta = state.timer.update(timer.since_now(), 1);
        self.0.sender.lazy_send(|| QueueMessage::TaskSuccess {
          id:task.0.id,target,eta
        });
        if requeue { state.queue.push_front(task);}
        else {state.done.push(task);}
      }
    }
    
  }

  #[instrument(level = "info",
    target = "buildqueue",
    name = "Queueing tasks",
    skip_all
  )]
  pub fn enqueue_group(&self,id:&ArchiveId,target:FormatOrTargets,stale_only:bool) -> usize {
    self.0.backend.with_archive_or_group(id, |g| match g {
      None => 0,
      Some(ArchiveOrGroup::Archive(id)) => {
        self.0.backend.with_archive(id, |a| {
          let Some(archive) = a else {return 0};
          archive.with_sources(|d| {
            let Some(d) = d.dfs() else {return 0};
            let map = &mut *self.0.map.write();
            Self::enqueue(map,&self.0.backend,archive,target, stale_only, 
              d.filter_map(|e| match e {
                SourceEntry::Dir(_) => None,
                SourceEntry::File(f) => Some(f)
              })
            )
          })
        })
      }
      Some(ArchiveOrGroup::Group(g)) => {
        let Some(g) = g.dfs() else {return 0};
        let map = &mut *self.0.map.write();
        let mut ret = 0;
        for id in g.filter_map(|e| match e {
          ArchiveOrGroup::Archive(id) => Some(id),
          ArchiveOrGroup::Group(_) => None
        }) {
          ret += self.0.backend.with_archive(id, |a| {
            let Some(archive) = a else {return 0};
            archive.with_sources(|d| {
              let Some(d) = d.dfs() else {return 0};
              Self::enqueue(map,&self.0.backend,archive,target, stale_only, 
                d.filter_map(|e| match e {
                  SourceEntry::Dir(_) => None,
                  SourceEntry::File(f) => Some(f)
                })
              )
            })
          });
        }
        ret
      }
    })
  }

  #[instrument(level = "info",
    target = "buildqueue",
    name = "Queueing tasks",
    skip_all
  )]
  pub fn enqueue_archive(&self,id:&ArchiveId,target:FormatOrTargets,stale_only:bool,rel_path:Option<&str>) -> usize {
    self.0.backend.with_archive(id, |archive| {
      let Some(archive) = archive else { return 0 };
      archive.with_sources(|d| {
        match rel_path {
          None => {
            let Some(d) = d.dfs() else {return 0};
            let map = &mut *self.0.map.write();
            Self::enqueue(map,&self.0.backend,archive,target, stale_only, 
              d.filter_map(|e| match e {
                SourceEntry::Dir(_) => None,
                SourceEntry::File(f) => Some(f)
              })
            )
          }
          Some(p) => {
            let Some(d) = d.find(p) else {return 0};
            match d {
              Either::Left(d) => {
                let Some(d) = d.dfs() else {return 0};
                let map = &mut *self.0.map.write();
                Self::enqueue(map,&self.0.backend,archive,target, stale_only, 
                  d.filter_map(|e| match e {
                    SourceEntry::Dir(_) => None,
                    SourceEntry::File(f) => Some(f)
                  })
                )
              }
              Either::Right(f) => {
                let map = &mut *self.0.map.write();
                Self::enqueue(map,&self.0.backend,archive,target, stale_only, std::iter::once(f))
              }
            }
          }
        }
      })
    })
  }

}

#[derive(Debug)]
pub struct RunningQueue {
  pub(super) queue: VecDeque<BuildTask>,
  pub(super) blocked:Vec<BuildTask>,
  pub(super) done:Vec<BuildTask>,
  pub(super) failed:Vec<BuildTask>,
  pub(super) running:Vec<BuildTask>,
  timer:Timer
}
impl RunningQueue {
  fn new(total:usize) -> Self {
    Self { queue:VecDeque::new(),failed:Vec::new(),blocked:Vec::new(),done:Vec::new(),running:Vec::new(),timer:Timer::new(total) }
  }
}

#[derive(Debug)]
struct Timer {
  average:Delta,
  steps:usize,
  done:usize
}
impl Timer {
  fn new(total:usize) -> Self {
    Self { average:Delta::default(),steps:total,done:0 }
  }
  #[allow(clippy::cast_precision_loss)]
  fn update(&mut self,delta:Delta,dones:u8) -> Eta {
    self.average.update_average(self.done as f64 / (self.done as f64 + f64::from(dones)),delta);
    self.done += dones as usize;
    Eta {
      time_left:self.average * ((self.steps - self.done) as f64),
      done:self.done,
      total:self.steps
    }
  }
}
