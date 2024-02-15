use immt_api::archives::ArchiveId;
use immt_api::source_files::BuildState;
use crate::backend::archive_manager::ArchiveManager;
use rayon::prelude::*;
use immt_api::archives::ArchiveT;
use immt_api::CloneStr;

#[derive(Default)]
pub struct BuildQueue {
    pub stale: Vec<(ArchiveId,CloneStr,u64)>,
    pub new: Vec<(ArchiveId,CloneStr)>,
    pub deleted: Vec<(ArchiveId,CloneStr)>
}
impl BuildQueue {
    pub(crate) fn init(&mut self,mgr:&ArchiveManager) {
       let (stale,new,deleted) = mgr.par_iter().fold(|| (vec!(),vec!(),vec!()), |(stale,new,deleted),a| {
            a.iter_sources((stale,new,deleted),|f,(stale,new,deleted)| {
                match f.state {
                    BuildState::Stale {last_built,..} => stale.push((a.id().to_owned(),f.rel_path.clone(),last_built)),
                    BuildState::New => new.push((a.id().to_owned(),f.rel_path.clone())),
                    BuildState::Deleted => deleted.push((a.id().to_owned(),f.rel_path.clone())),
                    _ => ()
                }
            })
        }).reduce(|| (vec!(),vec!(),vec!()),|(mut stale,mut new,mut deleted),(s,n,d)| {
            stale.extend(s);
            new.extend(n);
            deleted.extend(d);
            (stale,new,deleted)
        });
        self.stale = stale;
        self.new = new;
        self.deleted = deleted;
    }
}