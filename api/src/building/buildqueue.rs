use std::collections::VecDeque;
use std::sync::atomic::AtomicU16;
use futures::stream::StreamFuture;
use futures::StreamExt;
use immt_core::utils::triomphe::Arc;
use crate::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};
use crate::utils::asyncs::{ChangeListener, ChangeSender};

#[derive(Debug)]
struct QueuedTask();

#[derive(Debug)]
struct Queue {
    queue: VecDeque<QueuedTask>,
}
#[derive(Debug)]
struct BuildQueueI {
    source_formats:Box<[SourceFormat]>,
    build_data_formats: Box<[BuildDataFormat]>,
    build_targets: Box<[BuildTarget]>,
    inner: Vec<Queue>,
    #[cfg(feature = "tokio")]
    num_threads:Arc<(AtomicU16,tokio::sync::Semaphore)>,
    #[cfg(feature = "tokio")]
    listener:ChangeSender<u16>
}
#[derive(Debug)]
pub struct BuildQueue(BuildQueueI);
impl BuildQueue {
    pub fn new(source_formats:Box<[SourceFormat]>,build_data_formats: Box<[BuildDataFormat]>,build_targets: Box<[BuildTarget]>) -> Self {
        #[cfg(feature="tokio")]
        let num_threads = tokio::runtime::Handle::current().metrics().num_workers() / 2;
        let ret = Self(BuildQueueI {
            source_formats,
            build_data_formats,
            build_targets,
            inner: Vec::new(),
            #[cfg(feature="tokio")]
            num_threads: Arc::new((AtomicU16::new(num_threads as u16),tokio::sync::Semaphore::new(num_threads))),
            #[cfg(feature = "tokio")]
            listener:ChangeSender::new(8)
        });
        let listener = ret.0.listener.listener();
        let threads = ret.0.num_threads.clone();
        #[cfg(feature = "tokio")]
        {
            tokio::spawn(async move {
                loop {
                    let listener = listener.inner.clone().into_future();
                    if let Some(i) = listener.await.0 {
                        let old = match threads.0.fetch_update(std::sync::atomic::Ordering::Relaxed,std::sync::atomic::Ordering::Relaxed,|_| Some(i)) {
                            Ok(i) | Err(i) => i
                        };
                        if old < i {
                            threads.1.add_permits(i as usize - old as usize);
                        } else if old > i {
                            threads.1.forget_permits(old as usize - i as usize);
                        }
                    }
                }
            });
        }
        ret
    }
    pub fn formats(&self) -> &[SourceFormat] {
        &self.0.source_formats
    }

    pub fn num_threads(&self) -> u16 {
        self.0.num_threads.0.load(std::sync::atomic::Ordering::Relaxed)
    }
}
