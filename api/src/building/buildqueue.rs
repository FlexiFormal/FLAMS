use std::collections::VecDeque;
use std::sync::atomic::AtomicU16;
use futures::stream::StreamFuture;
use futures::StreamExt;
use immt_core::building::formats::{BuildJobSpec, FormatOrTarget};
use immt_core::prelude::DirLike;
use immt_core::uris::archives::ArchiveId;
use immt_core::utils::filetree::{FileLike, SourceDirEntry};
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::Archive;
use crate::backend::manager::ArchiveTree;
use crate::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};
use crate::controller::{Controller, ControllerAsync};
use crate::utils::asyncs::{ChangeListener, ChangeSender};
use crate::utils::settings::SettingsChange;

#[derive(Debug)]
struct QueuedTask{

}

#[derive(Debug)]
struct Queue {
    queue: VecDeque<QueuedTask>,
}

#[cfg(feature = "tokio")]
type Lock<A> = tokio::sync::RwLock<A>;
#[cfg(not(feature = "tokio"))]
type Lock<A> = parking_lot::RwLock<A>;


#[derive(Debug)]
struct BuildQueueI {
    inner: Lock<Vec<Queue>>,
    #[cfg(feature = "tokio")]
    num_threads:tokio::sync::Semaphore,
    #[cfg(feature = "tokio")]
    new_jobs:ChangeSender<BuildJobSpec>
}
impl BuildQueueI {
    #[cfg(feature = "tokio")]
    async fn do_spec<Ctrl:ControllerAsync+'static>(&self,spec:BuildJobSpec,ctrl:&Ctrl) {
        match spec {
            BuildJobSpec::Group {..} => todo!(),
            BuildJobSpec::Archive {id,target,stale_only} => {
                ctrl.archives().with_tree(|tree| self.enqueue_archive(id,target,!stale_only,tree)).await
            }
            BuildJobSpec::Path {..} => todo!(),
        }
    }

    fn enqueue_archive(&self,id:ArchiveId,target:FormatOrTarget,all:bool,tree:&ArchiveTree) {
        let format = match target {
            FormatOrTarget::Format(f) => f,
            FormatOrTarget::Target(_) => {
                todo!()
            }
        };
        match tree.find_archive(&id) {
            Some(Archive::Physical(ma)) => {
                let files = ma.source_files().map(|sd| {
                    sd.dir_iter().filter_map(|fd| {
                        if let SourceDirEntry::File(f) = fd {
                            if f.format == format {
                                if all { Some(f.relative_path().to_string().into_boxed_str()) }
                                else { todo!() }
                            } else { None }
                        } else {None}
                    }).collect()
                }).unwrap_or(Vec::new());
                println!("files: {:?}",files);
                println!("={}",files.len());
            }
            None => (),
            _ => todo!()
        }
    }
    #[cfg(not(feature = "tokio"))]
    fn do_spec<Ctrl:Controller>(&self,spec:BuildJobSpec,ctrl:Ctrl) {
        todo!()
    }
}

#[derive(Debug)]
pub struct BuildQueue(Arc<BuildQueueI>);
impl BuildQueue {

    #[cfg(feature = "tokio")]
    pub fn run_async<Ctrl:ControllerAsync+'static>(&self,ctrl:Ctrl) {
        let inner = self.0.clone();
        tokio::spawn(async move {
            loop {
                let num_threads = ctrl.settings().num_threads.listener().inner;
                let new_jobs = inner.new_jobs.listener().inner;
                tokio::select! {
                    (Some(SettingsChange{old,new}),_) = num_threads.into_future() => {
                        if old < new {
                            inner.num_threads.add_permits(new as usize - old as usize);
                        } else if old > new {
                            inner.num_threads.forget_permits(old as usize - new as usize);
                        }
                    },
                    (Some(job),_) = new_jobs.into_future() => {
                        inner.do_spec(job,&ctrl).await
                    }
                }
            }
        });
    }

    pub fn enqueue(&self,job:BuildJobSpec) {
        #[cfg(feature = "tokio")]
        {self.0.new_jobs.send(job)}
        #[cfg(not(feature = "tokio"))]
        {todo!()}
    }

    pub fn run<Ctrl:Controller>(&self,ctrl:Ctrl) {}
    pub fn new() -> Self {
        Self(Arc::new(BuildQueueI {
            inner: Lock::new(Vec::new()),
            #[cfg(feature="tokio")]
            num_threads:tokio::sync::Semaphore::new(0),
            #[cfg(feature="tokio")]
            new_jobs:ChangeSender::new(64)
        }))
    }
}
