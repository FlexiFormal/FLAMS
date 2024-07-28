use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use immt_core::building::buildstate::QueueMessage;
use immt_core::building::formats::{BuildJobSpec, FormatOrTarget};
use immt_core::prelude::DirLike;
use immt_core::uris::archives::ArchiveId;
use immt_core::utils::filetree::{FileLike, SourceDirEntry};
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::{Archive, Storage};
use crate::building::queue::Queue;
use crate::building::tasks::TaskSpec;
use crate::controller::Controller;
use crate::utils::asyncs::{ChangeListener, ChangeSender};
use crate::utils::settings::Settings;


#[derive(Clone,Debug)]
pub(crate) enum Semaphore {
    Stepwise {
        sender: tokio::sync::watch::Sender<bool>,
        recv: tokio::sync::watch::Receiver<bool>,
        last:Option<PathBuf>
    },
    Counting {
        inner: std::sync::Arc<tokio::sync::Semaphore>,
        num: Arc<parking_lot::RwLock<u8>>,
    }
}

#[derive(Debug)]
struct BuildQueueI {
    inner: parking_lot::RwLock<Vec<Queue>>,
    change:ChangeSender<QueueMessage>,
    num_threads:Semaphore,
}
impl BuildQueueI {
   fn do_spec<Ctrl:Controller+'static>(&self,spec:BuildJobSpec,ctrl:&Ctrl) {
        match spec {
            BuildJobSpec::Group {id,target,stale_only} => {
                self.enqueue_group(id,target,!stale_only,ctrl)
            },
            BuildJobSpec::Archive {id,target,stale_only} => {
                self.enqueue_archive(id,target,!stale_only,ctrl);
            }
            BuildJobSpec::Path {id, rel_path, target,
                stale_only} =>
            self.enqueue_path(id,target,&rel_path,!stale_only,ctrl),
        }
   }

    fn enqueue_path<Ctrl:Controller+'static>(&self,id:ArchiveId,target:FormatOrTarget,rel_path:&str,all:bool,ctrl:&Ctrl) {
        use spliter::ParallelSpliterator;
        use rayon::iter::*;
        let format = match target {
            FormatOrTarget::Format(f) => f,
            FormatOrTarget::Target(_) => {
                todo!()
            }
        };
        let tree = ctrl.archives().get_tree();
        let mut queue = self.inner.write();
        let q = if queue.is_empty() {
            queue.push(Queue::new("global".into(),self.change.clone()));
            queue.last_mut().unwrap()
        } else {
            queue.last_mut().unwrap()
            // TODO
        };

        match tree.find_archive(&id) {
            Some(Archive::Physical(ma)) => {
                if let Some(sd) = ma.source_files() {
                    if let Some(dirorfile) = sd.find_entry(rel_path) {
                         match dirorfile {
                            SourceDirEntry::File(f) => {
                                let spec = TaskSpec {
                                    archive: ma.uri(),base_path: ma.path(),rel_path: rel_path.into(),target};
                                q.enqueue(std::iter::once(spec).par_bridge().into_par_iter(),ctrl);
                            },
                            SourceDirEntry::Dir(d) => {
                                let files = d.children.dir_iter().par_split().into_par_iter().filter_map(|fd| {
                                    if let SourceDirEntry::File(f) = fd {
                                        if f.format == format {
                                            if all { Some(f.relative_path()) }
                                            else { todo!() }
                                        } else { None }
                                    } else {None}
                                }).map(|rp| TaskSpec {
                                    archive: ma.uri(),base_path: ma.path(),rel_path: rp,target});
                                q.enqueue(files,ctrl);
                            }
                        }
                    }
                }
            }
            None => (),
            _ => todo!()
        }
    }

    fn enqueue_archive<Ctrl:Controller+'static>(&self,id:ArchiveId,target:FormatOrTarget,all:bool,ctrl:&Ctrl) {
        use spliter::ParallelSpliterator;
        use rayon::iter::*;
        let format = match target {
            FormatOrTarget::Format(f) => f,
            FormatOrTarget::Target(_) => {
                todo!()
            }
        };
        let tree = ctrl.archives().get_tree();
        let mut queue = self.inner.write();
        let q = if queue.is_empty() {
            queue.push(Queue::new("global".into(),self.change.clone()));
            queue.last_mut().unwrap()
        } else {
            queue.last_mut().unwrap()
            // TODO
        };

        match tree.find_archive(&id) {
            Some(Archive::Physical(ma)) => {
                if let Some(sd) = ma.source_files() {
                    let files = sd.dir_iter().par_split().into_par_iter().filter_map(|fd| {
                        if let SourceDirEntry::File(f) = fd {
                            if f.format == format {
                                if all { Some(f.relative_path()) }
                                else { todo!() }
                            } else { None }
                        } else {None}
                    }).map(|rp| TaskSpec {
                        archive: ma.uri(),base_path: ma.path(),rel_path: rp,target});
                    q.enqueue(files,ctrl);
                }
            }
            None => (),
            _ => todo!()
        }
    }

    fn enqueue_group<Ctrl:Controller+'static>(&self,id:ArchiveId,target:FormatOrTarget,all:bool,ctrl:&Ctrl) {
        use spliter::ParallelSpliterator;
        use rayon::iter::*;
        let format = match target {
            FormatOrTarget::Format(f) => f,
            FormatOrTarget::Target(_) => {
                todo!()
            }
        };
        let tree = ctrl.archives().get_tree();
        let mut queue = self.inner.write();
        let q = if queue.is_empty() {
            queue.push(Queue::new("global".into(),self.change.clone()));
            queue.last_mut().unwrap()
        } else {
            queue.last_mut().unwrap()
            // TODO
        };

        let files = tree.archives().par_iter().filter_map(|a|
            if a.id().as_str().starts_with(id.as_str()) {
                match a {
                    Archive::Physical(ma) => {
                        ma.source.as_ref().map(|sd| {
                            sd.dir_iter().par_split().into_par_iter().filter_map(|fd| {
                                if let SourceDirEntry::File(f) = fd {
                                    if f.format == format {
                                        if all { Some(f.relative_path()) }
                                        else { todo!() }
                                    } else { None }
                                } else {None}
                            }).map(|rp| TaskSpec {
                                archive: ma.uri(),base_path: ma.path(),rel_path: rp,target})
                        })
                    },
                    _ => None
                }
            } else {None}
        ).flatten();
        q.enqueue(files,ctrl);
    }
}

#[derive(Debug)]
pub struct BuildQueue(Arc<BuildQueueI>);
impl BuildQueue {

    pub fn enqueue<Ctrl:Controller+'static>(&self,job:BuildJobSpec,ctrl:&Ctrl) {
        self.0.do_spec(job,ctrl)
    }

    pub fn queues(&self) -> impl Deref<Target=Vec<Queue>> + '_ {
        self.0.inner.read()
    }

    pub fn new(settings:&Settings) -> Self {
        Self(Arc::new(BuildQueueI {
            inner: parking_lot::RwLock::new(Vec::new()),
            change:ChangeSender::new(64),
            num_threads:match settings.num_threads.get() {
                0 => {
                    let (sender,recv) = tokio::sync::watch::channel(false);
                    Semaphore::Stepwise { sender,recv,last:None }
                },
                i => {
                    Semaphore::Counting {
                        inner: std::sync::Arc::new(tokio::sync::Semaphore::new(*i as usize)),
                        num: Arc::new(parking_lot::RwLock::new(*i)),
                    }
                }
            },
        }))
    }

    pub fn start<Ctrl:Controller+Clone+'static>(&self,id:&str,ctrl:Ctrl) {
        if let Some(q) = self.0.inner.read().iter().find(|q| q.id() == id) {
            q.run(&self.0.num_threads,ctrl)
        }
    }

    pub fn listener(&self) -> ChangeListener<QueueMessage> {
        self.0.change.listener()
    }
}
