use std::fmt::Debug;
use std::path::Path;
use either::Either;
use tracing::{event, info, instrument};
use crate::backend::archives::{Archive, ArchiveGroup};
use immt_api::formats::FormatStore;
use immt_api::Str;
use immt_api::archives::{ArchiveId,ArchiveGroupT,ArchiveT};

pub struct ArchiveManager {
    top:ArchiveGroup
}
use crate::utils::problems::ProblemHandler;

impl ArchiveManager {
    pub fn new(mh:&Path,handler:&ProblemHandler,formats:&FormatStore) -> Self {
        let top = ArchiveGroup::new(ArchiveId::new(Str::from("")));
        let mut manager = Self{ top };
        manager.load(mh,handler,formats);
        manager
    }
    pub fn iter(&self) -> impl Iterator<Item=&Archive> {
        self.top.archives()
    }
    pub fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item=&Archive> {
        self.top.archives_par()
    }

    pub fn get_top(&self) -> &ArchiveGroup { &self.top }
    pub fn num_archives(&self) -> usize { self.top.archives().count() }

    pub fn find<Id:for<'a>Into<ArchiveId>>(&self,id:Id) -> Option<Either<&ArchiveGroup,&Archive>> {
        let id = id.into();
        let steps = id.steps().collect::<Vec<_>>();
        self.find_i(steps)
    }

    fn find_i(&self,mut id:Vec<&str>) -> Option<Either<&ArchiveGroup,&Archive>> {
        macro_rules! id { ($e:expr) => { match $e.as_ref() {
            Either::Left(g) => g.id(),
            Either::Right(a) => a.id()
        }};}
        if id.is_empty() { return None }
        let mut curr = &self.top.base().archives;
        loop {
            let head = id.remove(0);
            if id.is_empty() {
                return curr.binary_search_by_key(&head, |g| id!(g).last_name()).ok().map(|i|curr[i].as_ref())
            }
            let g = if let Ok(i) = curr.binary_search_by_key(&head, |g| id!(g).last_name()) {
                if let Either::Left(g) = curr[i].as_ref() {g} else {return None}
            } else { return None };

            if id.len() == 1 && id.last().unwrap().eq_ignore_ascii_case("meta-inf") {
                return g.meta().map(Either::Right)
            }
            curr = &g.base().archives;
        }
    }

    #[instrument(level = "info",name = "Loading archives", target = "backend", skip(self,handler,formats), fields(found) )]
    fn load(&mut self, in_path:&Path,handler:&ProblemHandler,formats:&FormatStore) {
        info!("Searching for archives");
        self.top.base_mut().archives = ArchiveGroupT::load_dir(in_path,formats,handler).into();
        tracing::Span::current().record("found", self.num_archives());
        info!("Done");
    }
}
impl Debug for ArchiveManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"ArchiveManager")
    }
}