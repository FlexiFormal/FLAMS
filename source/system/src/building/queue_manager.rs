use std::{fmt::Display, num::NonZeroU32, sync::atomic::AtomicU8};
use flams_utils::vecmap::VecMap;
use crate::backend::{AnyBackend, Backend, GlobalBackend, SandboxedBackend, SandboxedRepository};

use super::queue::{Queue, QueueName, QueueState, RunningQueue};

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct QueueId(NonZeroU32);
impl QueueId {
  #[must_use]
  pub fn global() -> Self {
    Self(NonZeroU32::new(1).unwrap_or_else(|| unreachable!()))
  }
}
impl Display for QueueId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f,"queue {}",self.0)
  }
}
impl From<QueueId> for NonZeroU32 {
  #[inline]
  fn from(id:QueueId) -> Self {
    id.0
  }
}
impl From<NonZeroU32> for QueueId {
  #[inline]
  fn from(id:NonZeroU32) -> Self {
    Self(id)
  }
}

#[derive(Debug)]
pub struct QueueManager {
    inner: parking_lot::RwLock<VecMap<QueueId,Queue>>,
    threads:Semaphore,
}

static QUEUE_MANAGER : std::sync::OnceLock<QueueManager> = std::sync::OnceLock::new();

impl QueueManager {
  #[inline]
  pub fn clear() {
    if let Some(m) = QUEUE_MANAGER.get() {
      m.inner.write().0.clear();
    }
  }
  pub fn initialize(num_threads:u8) {
    QUEUE_MANAGER.get_or_init(|| {
      let init = if crate::settings::Settings::get().admin_pwd.is_some() {
        VecMap::new()
      } else {
        vec![(QueueId::global(),
          Queue::new(QueueId::global(),QueueName::Global,GlobalBackend::get().to_any())
        )].into()
      };
      Self {
        inner: parking_lot::RwLock::new(init),
        threads: {
          #[cfg(feature="tokio")] 
          { match num_threads {
            0 => Semaphore::Linear,
            i => Semaphore::Counting {
              inner: std::sync::Arc::new(tokio::sync::Semaphore::new(i as usize)),
              num: flams_utils::triomphe::Arc::new(AtomicU8::new(i))
            }
          }}
          #[cfg(not(feature="tokio"))] 
          { Semaphore::Linear }
        }
      }
    });
  }
  /// ### Panics
  pub fn get() -> &'static Self {
    QUEUE_MANAGER.get().expect("Queue manager not initialized")
  }

  pub fn new_queue(&self,queue_name:&str) -> QueueId {
    let mut inner = self.inner.write();
    let mut count = 0;
    loop {
      if inner.0.iter().any(|(_,q)| matches!(q.name(),QueueName::Sandbox{name,idx} if &**name == queue_name && *idx == count)) {
        count += 1;
      } else {break}
    }
    let sbname = format!("{queue_name}_{count}");
    let id = QueueId(NonZeroU32::new(inner.0.iter().map(|(k,_)| k.0.get()).max().unwrap_or_default() + 1).unwrap_or_else(|| unreachable!()));
    let backend = AnyBackend::Sandbox(SandboxedBackend::new(&sbname));
    inner.insert(id,
      Queue::new(id,
        QueueName::Sandbox{name:queue_name.to_string().into(),idx:count},
        backend
      )
    );
    id
  }

  pub fn all_queues(&self) -> Vec<(QueueId,QueueName,Option<Vec<SandboxedRepository>>)> {
    let inner = self.inner.read();
    inner.iter().map(|(k,v)| (*k,v.name().clone(),
      if let AnyBackend::Sandbox(sb) = v.backend() {
        Some(sb.get_repos())
      } else { None }
    )).collect()
  }

  pub fn with_all_queues<R>(&self,f:impl FnOnce(&[(QueueId,Queue)]) -> R) -> R {
    f(&self.inner.read().0)
  }

  pub fn queues_for_user(&self,user_name:&str) -> Vec<(QueueId,QueueName,Option<Vec<SandboxedRepository>>)> {
    let inner = self.inner.read();
    inner.iter().filter_map(|(k,v)| {
      if let QueueName::Sandbox{name,idx} = v.name() {
        if &**name == user_name {Some((*k,v.name().clone(),
          if let AnyBackend::Sandbox(sb) = v.backend() {
            Some(sb.get_repos())
          } else { None }
        ))} else {None}
      } else {None}
    }).collect()
  }

  pub fn with_queue<R>(&self,id:QueueId,f:impl FnOnce(Option<&Queue>) -> R) -> R {
    let inner = self.inner.read();
    f(inner.get(&id))
  }

  pub fn migrate<R,E:ToString>(&self,id:QueueId,then:impl FnOnce(&SandboxedBackend) -> Result<R,E>) -> Result<(R,usize),String> {
    let mut inner = self.inner.write();
    let r = if let Some(queue) = inner.get(&id) {
      if !matches!(&*queue.0.state.read(),QueueState::Finished{..}) {
        return Err(format!("Queue {id} not finished"))
      }
      if !matches!(queue.backend(),AnyBackend::Sandbox(_)) {
        return Err(format!("Global Queue can not be migrated"))
      }
      let AnyBackend::Sandbox(sandbox) = queue.backend() else {unreachable!()};
      then(&sandbox).map_err(|e| e.to_string())?
    } else {
      return Err(format!("No queue {id} found"))
    };
    let Some(queue) = inner.remove(&id) else {unreachable!()};
    let AnyBackend::Sandbox(sandbox) = queue.backend() else {unreachable!()};
    Ok((r,sandbox.migrate()))
  }

  pub fn delete(&self,id:QueueId) {
    let mut inner = self.inner.write();
    if let Some(q) = inner.remove(&id) {
      let mut s = q.0.state.write();
      match &mut *s {
        QueueState::Running(RunningQueue{queue,blocked,..}) => {
          queue.clear();
          blocked.clear();
        }
        _ => ()
      }
      if matches!(q.name(),QueueName::Global) {
        inner.insert(id, Queue::new(id,QueueName::Global,GlobalBackend::get().to_any()));
      }
    }
  }

  /// ### Errors 
  /// if no queue with that id exists
  #[allow(clippy::result_unit_err)]
  pub fn start_queue(&self,id:QueueId) -> Result<(),()> {
    let sem = self.threads.clone();
    self.with_queue(id, |q| {
      let Some(q) = q else {return Err(())};
      q.start(sem);
      Ok(())
    })
  }

  /// ### Errors 
  /// if no queue with that id exists
  #[allow(clippy::result_unit_err)]
  pub fn requeue_failed(&self,id:QueueId) -> Result<(),()> {
    self.with_queue(id, |q| q.map_or(Err(()),|q| {
      q.requeue_failed();
      Ok(())
    }))
  }

  pub fn with_global<R>(&self,f:impl FnOnce(&Queue) -> R) -> R {
    let inner = self.inner.read();
    f(inner.get(&QueueId::global()).unwrap_or_else(|| unreachable!()))
  }

}

#[derive(Debug,Clone)]
pub(crate) enum Semaphore {
  Linear,
  #[cfg(feature="tokio")]
  Counting {
    inner: std::sync::Arc<tokio::sync::Semaphore>,
    num: flams_utils::triomphe::Arc<AtomicU8>
  }
}