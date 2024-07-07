use immt_core::building::formats::{BuildJobSpec, FormatOrTarget};
use immt_core::prelude::DirLike;
use immt_core::uris::archives::ArchiveId;
use immt_core::utils::filetree::{FileLike, SourceDirEntry};
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::{Archive, Storage};
use crate::controller::{Controller, ControllerAsync};



#[cfg(feature = "async")]
type Lock<A> = tokio::sync::RwLock<A>;
#[cfg(not(feature = "async"))]
type Lock<A> = parking_lot::RwLock<A>;

use super::queue::{Queue, TaskRef, TaskSpec};

#[derive(Debug)]
struct BuildQueueI {
    inner: Lock<Vec<Queue>>,
    #[cfg(feature = "async")]
    num_threads:Arc<tokio::sync::Semaphore>,
}
impl BuildQueueI {
    immt_core::asyncs!{ALT fn do_spec
        <@s[Ctrl:Controller+'static]>
        <@a[Ctrl:ControllerAsync+Clone+'static]>
        (@s &self,spec:BuildJobSpec,ctrl:&Ctrl)
        (@a &self,spec:BuildJobSpec,ctrl:&Ctrl)
        {
            match spec {
                BuildJobSpec::Group {..} => todo!(),
                BuildJobSpec::Archive {id,target,stale_only} => {
                    wait!(self.enqueue_archive(id,target,!stale_only,ctrl));
                }
                BuildJobSpec::Path {..} => todo!(),
            }
        }
    }

    immt_core::asyncs!{ALT fn enqueue_archive
        <@s[Ctrl:Controller+'static]>
        <@a[Ctrl:ControllerAsync+Clone+'static]>
        (@s &self,id:ArchiveId,target:FormatOrTarget,all:bool,ctrl:&Ctrl)
        (@a &self,id:ArchiveId,target:FormatOrTarget,all:bool,ctrl:&Ctrl) {
        let format = match target {
            FormatOrTarget::Format(f) => f,
            FormatOrTarget::Target(_) => {
                todo!()
            }
        };
        let tree = wait!(ctrl.archives().get_tree());
        let mut queue = wait!(self.inner.write());
        let q = if queue.is_empty() {
            queue.push(Queue::new("global".into()));
            queue.last_mut().unwrap()
        } else {
            queue.last_mut().unwrap()
            // TODO
        };

        match tree.find_archive(&id) {
            Some(Archive::Physical(ma)) => {
                if let Some(sd) = ma.source_files() {
                    let files = sd.dir_iter().filter_map(|fd| {
                        if let SourceDirEntry::File(f) = fd {
                            if f.format == format {
                                if all { Some(f.relative_path()) }
                                else { todo!() }
                            } else { None }
                        } else {None}
                    }).map(|rp| TaskSpec {
                        archive: ma.uri(),base_path: ma.path(),rel_path: rp,target});
                    wait!(q.enqueue(files,ctrl));
                }
            }
            None => (),
            _ => todo!()
        }
    }}
}

#[derive(Debug)]
pub struct BuildQueue(Arc<BuildQueueI>);
impl BuildQueue {

    immt_core::asyncs!{ALT !pub fn enqueue
        <@s[Ctrl:Controller+'static]>
        <@a[Ctrl:ControllerAsync+Clone+'static]>
        (@s &self,job:BuildJobSpec,ctrl:&Ctrl)
        (@a &self,job:BuildJobSpec,ctrl:&Ctrl){
        switch!{
            (todo!())
            (self.0.do_spec(job,ctrl))
        }
    }}

    pub fn new() -> Self {
        Self(Arc::new(BuildQueueI {
            inner: Lock::new(Vec::new()),
            #[cfg(feature="async")]
            num_threads:Arc::new(tokio::sync::Semaphore::new(0)),
        }))
    }
}
