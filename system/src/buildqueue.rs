use immt_api::archives::ArchiveId;
use immt_api::source_files::BuildState;
use immt_api::Str;
use crate::backend::archive_manager::ArchiveManager;
use rayon::prelude::*;
use immt_api::archives::ArchiveT;

#[derive(Default)]
pub struct BuildQueue {
    pub stale: Vec<(ArchiveId,Str,u64)>,
    pub new: Vec<(ArchiveId,Str)>,
    pub deleted: Vec<(ArchiveId,Str)>
}
impl BuildQueue {
    pub(crate) fn init(&mut self,mgr:&ArchiveManager) {
       let (stale,new,deleted) = mgr.par_iter().fold(|| (vec!(),vec!(),vec!()), |(stale,new,deleted),a| {
            a.iter_sources((stale,new,deleted),|f,(stale,new,deleted)| {
                match f.state {
                    BuildState::Deleted => deleted.push((a.id().to_owned(),f.rel_path.clone())),
                    BuildState::New => new.push((a.id().to_owned(),f.rel_path.clone())),
                    BuildState::Stale {last_built,..} => stale.push((a.id().to_owned(),f.rel_path.clone(),last_built)),
                    _ => ()
                }
            })
        }).reduce(|| (vec!(),vec!(),vec!()),|(mut stale,mut new,mut deleted),(d,e,f)| {
            stale.extend(d);
            new.extend(e);
            deleted.extend(f);
            (stale,new,deleted)
        });
        self.stale = stale;
        self.new = new;
        self.deleted = deleted;
    }
}