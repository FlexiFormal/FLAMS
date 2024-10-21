use std::{fmt::Display, num::NonZeroU32, sync::atomic::AtomicU8};
use immt_utils::vecmap::VecMap;
use crate::backend::{Backend, GlobalBackend};

use super::queue::Queue;

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
  pub fn initialize(num_threads:u8) {
    QUEUE_MANAGER.get_or_init(|| {
      let global = QueueId::global();
      let init = vec![(global,Queue::new(global,"Global".into(),GlobalBackend::get().to_any()))].into();
      Self {
        inner: parking_lot::RwLock::new(init),
        threads: {
          #[cfg(feature="tokio")] 
          { match num_threads {
            0 => Semaphore::Linear,
            i => Semaphore::Counting {
              inner: std::sync::Arc::new(tokio::sync::Semaphore::new(i as usize)),
              num: immt_utils::triomphe::Arc::new(AtomicU8::new(i))
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

  pub fn all_queues(&self) -> Vec<(QueueId,std::sync::Arc<str>)> {
    let inner = self.inner.read();
    inner.iter().map(|(k,v)| (*k,v.name().clone())).collect()
  }

  pub fn get_queue<R>(&self,id:QueueId,f:impl FnOnce(Option<&Queue>) -> R) -> R {
    let inner = self.inner.read();
    f(inner.get(&id))
  }

  /// ### Errors 
  /// if no queue with that id exists
  #[allow(clippy::result_unit_err)]
  pub fn start_queue(&self,id:QueueId) -> Result<(),()> {
    let sem = self.threads.clone();
    self.get_queue(id, |q| {
      let Some(q) = q else {return Err(())};
      q.start(sem);
      Ok(())
    })
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
    num: immt_utils::triomphe::Arc<AtomicU8>
  }
}