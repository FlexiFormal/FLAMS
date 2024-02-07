use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use either::Either;
use immt_api::{Seq,Str,utils::HMap};
use parking_lot::RwLockWriteGuard;
use immt_api::utils::problems::{ProblemHandler as PHandlerT};

#[derive(Debug)]
pub struct Archive {
    pub(in crate::backend) manifest:ArchiveManifest,
    path:PathBuf,
    state:parking_lot::RwLock<ArchiveState>,
    //watcher: Option<RecommendedWatcher>
}

#[derive(Debug)]
struct ArchiveState {
    initialized:bool,
    source_dir:SourceDir
}
impl Default for ArchiveState {
    fn default() -> Self {
        Self {
            initialized:false,
            source_dir:SourceDir{name:"source".into(),children:Vec::new().into()}
        }
    }
}
// use notify::{Watcher, RecommendedWatcher, RecursiveMode};
impl Archive {
    pub fn id(&self) -> &ArchiveId { &self.manifest.id }
    pub fn formats(&self) -> &[FormatId] { &self.manifest.formats }
    pub fn path(&self) -> &Path { &self.path }
    pub fn new(manifest:ArchiveManifest,path:PathBuf) -> Self {
        Self { manifest, path, state:parking_lot::RwLock::new(ArchiveState::default())}//,watcher:None }
    }

    #[instrument(level = "info",name = "initialize", target = "backend::archive", skip(self,handler,formats))]
    pub(in crate::backend) fn initialize(&mut self,handler:&ProblemHandler,formats:&FormatStore) {
        let mut state = self.state.write();
        if state.initialized { return }
        state.initialized = true;
        event!(tracing::Level::DEBUG,"Initializing archive {}",self.manifest.id);
        if !Self::ls_f(&mut state, &self.path, &self.manifest.ignores, handler, formats) && !self.manifest.is_meta {
            handler.add("Missing source", format!("Archive has no source directory: {}",self.manifest.id));
        }
        event!(tracing::Level::DEBUG,"Done");
    }

    fn ls_f(state:&mut RwLockWriteGuard<ArchiveState>, path:&Path,ignore:&IgnoreSource, handler:&ProblemHandler,formats:&FormatStore) -> bool {
        let dirfile = path.join(".out").join("ls_f.db");
        if dirfile.exists() {
            match SourceDir::parse(&dirfile) {
                Ok(v) => state.source_dir.children = v,
                Err(ParseError::DecodingError) => handler.add("ArchiveManager",format!("Error decoding {}",dirfile.display())),
                Err(ParseError::FileError) => handler.add("ArchiveManager",format!("Error reading {}",dirfile.display()))
            }
        }
        let source = path.join("source");
        if source.exists() {
            SourceDir::update(&source, &mut state.source_dir.children, handler,ignore, &|s| formats.from_ext(s));
            match state.source_dir.write_to(&dirfile) {
                Ok(_) => {},
                Err(SerializeError::EncodingError) => handler.add("ArchiveManager",format!("Error encoding {}",dirfile.display())),
                Err(SerializeError::IOError) => handler.add("ArchiveManager",format!("Error writing to {}",dirfile.display()))
            }
            true
        } else {false}
    }
/*
    pub(in crate::backend) fn watch(&mut self,handler:&ProblemHandler) {
        if self.watcher.is_none() {
            if let Ok(watcher) = Self::new_watcher(self.state.clone(), &self.path.join("source"), handler) {
                self.watcher = Some(watcher);
            }
        }
    }
    pub(in crate::backend) fn unwatch(&mut self) {
        self.watcher = None;
    }

    fn new_watcher(state:Arc<parking_lot::RwLock<ArchiveState>>,source:&Path,handler:&ProblemHandler) -> Result<RecommendedWatcher,notify::Error> {
        let ih = handler.clone();
        match notify::recommended_watcher(move |res:Result<notify::Event,notify::Error>| {
            match res {
                Ok(event) => {
                    let state = state.write();
                    match event.kind {
                        notify::EventKind::Create(_) => {
                            todo!()
                        }
                        notify::EventKind::Modify(_) => {
                            todo!()
                        }
                        notify::EventKind::Remove(_) => {
                            todo!()
                        }
                        _ => {}
                    }
                }
                Err(e) => ih.add("file watch",format!("Error: {:?}", e))
            }
        }) {
            Err(e) => {
                handler.add("file watch",format!("Error: {:?}", e));
                Err(e)
            },
            Ok(mut w) => {
                match w.watch(source, RecursiveMode::Recursive) {
                    Err(e) => {
                        handler.add("file watch",format!("Error: {:?}", e));
                        Err(e)
                    },
                    Ok(_) => {
                        event!(tracing::Level::INFO,"Watching {}",source.display());
                        Ok(w)
                    }
                }
            }
        }
    }

    fn iter_sources<R,F:FnMut(&SourceFile,&mut R)>(&self,mut init:R,mut f:F) -> R {
        let state = self.state.read();
        let i = state.source_dir.iter();//TreeIter::new(, |s:&SourceDir| s.children.iter(), |e| e.as_ref());
        for fl in i { f(fl,&mut init) }
        init
    }

 */
}

#[derive(Debug)]
pub struct ArchiveGroup {
    id:ArchiveId,
    meta:Option<Archive>,
    pub(in crate::backend) len:usize,
    pub(in crate::backend) archives:Vec<Either<ArchiveGroup,Archive>>
}
impl ArchiveGroup {
    pub fn set_meta(&mut self,meta:Archive) {
        self.meta = Some(meta)
    }
    pub fn meta(&self) -> Option<&Archive> { self.meta.as_ref() }
    pub fn id(&self) -> &ArchiveId { &self.id }
    pub fn new<S:Into<ArchiveId>>(id:S) -> Self {
        Self {
            id:id.into(),
            meta:None,
            len:0,
            archives:Vec::new()
        }
    }
    pub fn children(&self) -> &[Either<ArchiveGroup,Archive>] { &self.archives }
    pub fn num_archives(&self) -> usize { self.len }
}
impl<'a> IntoIterator for &'a ArchiveGroup {
    type Item = &'a Archive;
    type IntoIter = ArchiveGroupIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ArchiveGroupIter::new(self.meta.as_ref(), &self.archives, self.len)
    }
}

use rayon::prelude::IntoParallelIterator;
use spliter::ParallelSpliterator;

impl<'a> IntoParallelIterator for &'a ArchiveGroup {
    type Iter = ParSpliter<ParArchiveGroupIter<'a>>;
    type Item = &'a Archive;
    fn into_par_iter(self) -> Self::Iter {
        ParArchiveGroupIter::new(self.meta.as_ref(), &self.archives)
    }
}

use immt_api::formats::FormatId;

#[derive(Debug)]
pub struct ArchiveManifest {
    pub id:ArchiveId,
    pub formats:Seq<FormatId>,
    pub is_meta:bool,
    pub url_base:Str,
    pub dependencies:Seq<ArchiveId>,
    pub ignores:IgnoreSource,
    pub attrs:HMap<String,String>
}


#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
#[derive(Clone,Hash,PartialEq,Eq)]
pub struct ArchiveId(pub(in crate::backend) Seq<Str>);
impl ArchiveId {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn steps(&self) -> &[Str] { &self.0 }
}
impl Debug for ArchiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self,f)
    }
}
impl Display for ArchiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i,s) in self.0.iter().enumerate() {
            if i > 0 { write!(f,"/")? }
            write!(f,"{}",s)?
        }
        Ok(())
    }
}
impl From<&str> for ArchiveId {
    fn from(s:&str) -> Self {
        Self(s.split('/').map(|c| c.into()).collect())
    }
}
impl From<String> for ArchiveId {
    fn from(s:String) -> Self {
        Self(s.split('/').map(|c| c.into()).collect())
    }
}
impl From<Vec<Str>> for ArchiveId {
    fn from(v:Vec<Str>) -> Self {
        Self(v.into())
    }
}

pub struct ArchiveGroupIter<'a> {
    stack:Vec<&'a [Either<ArchiveGroup,Archive>]>,
    curr:&'a[Either<ArchiveGroup,Archive>],
    meta:Option<&'a Archive>,
    len:usize
}
impl<'a> ArchiveGroupIter<'a> {
    pub(in crate::backend) fn new(meta:Option<&'a Archive>,group:&'a [Either<ArchiveGroup,Archive>],len:usize) -> Self {
        Self {
            stack:Vec::new(),
            curr:group,
            len,meta
        }
    }
}
impl<'a> ExactSizeIterator for ArchiveGroupIter<'a> {
    fn len(&self) -> usize { self.len }
}
impl<'a> Iterator for ArchiveGroupIter<'a> {
    type Item = &'a Archive;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = std::mem::take(&mut self.meta) {
            self.len -= 1;
            return Some(m)
        }
        loop {
            if self.curr.is_empty() {
                match self.stack.pop() {
                    Some(s) => { self.curr = s; }
                    None => return None
                }
            } else {
                let next = &self.curr[0];
                self.curr = &self.curr[1..];
                match next {
                    Either::Left(g) => {
                        let meta = g.meta.as_ref();
                        let old = std::mem::replace(&mut self.curr,g.archives.as_slice());
                        self.stack.push(old);
                        if let Some(a) = meta {
                            self.len -= 1;
                            return Some(a)
                        }
                    }
                    Either::Right(a) => {
                        self.len -= 1;
                        return Some(a)
                    }
                }
            }
        }
    }
}

pub struct ParArchiveGroupIter<'a> {
    stack:Vec<&'a [Either<ArchiveGroup,Archive>]>,
    curr:&'a[Either<ArchiveGroup,Archive>],
    meta:Option<&'a Archive>,
}
use spliter::ParSpliter;
use tracing::{event, instrument};
use immt_api::source_files::{ParseError, SerializeError, SourceDir};
use immt_api::archives::IgnoreSource;
use crate::formats::FormatStore;
use crate::utils::problems::ProblemHandler;

impl<'a> ParArchiveGroupIter<'a> {
    pub(in crate::backend) fn new(meta:Option<&'a Archive>,group:&'a [Either<ArchiveGroup,Archive>]) -> ParSpliter<Self> {
        Self {
            stack:Vec::new(),
            curr:group, meta
        }.par_split()
    }
}
impl<'a> Iterator for ParArchiveGroupIter<'a> {
    type Item = &'a Archive;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = std::mem::take(&mut self.meta) {
            return Some(m)
        }
        loop {
            if self.curr.is_empty() {
                match self.stack.pop() {
                    Some(s) => { self.curr = s; }
                    None => return None
                }
            } else {
                let next = &self.curr[0];
                self.curr = &self.curr[1..];
                match next {
                    Either::Left(g) => {
                        let meta = g.meta.as_ref();
                        let old = std::mem::replace(&mut self.curr,g.archives.as_slice());
                        self.stack.push(old);
                        if let Some(a) = meta {
                            return Some(a)
                        }
                    }
                    Either::Right(a) => {
                        return Some(a)
                    }
                }
            }
        }
    }
}

impl<'a> spliter::Spliterator for ParArchiveGroupIter<'a> {
    fn split(&mut self) -> Option<Self> {
        if self.curr.len() < 2 && self.stack.len() < 2 { return None }
        let currsplit = self.curr.len()/2;
        let stacksplit = self.stack.len()/2;
        let (leftcurr,rightcurr) = self.curr.split_at(currsplit);
        let rightstack = self.stack.split_off(stacksplit);
        self.curr = leftcurr;
        Some(Self {
            curr:rightcurr,
            stack:rightstack,
            meta:None,
        })
    }
}

// https://geo-ant.github.io/blog/2022/implementing-parallel-iterators-rayon/
// https://tavianator.com/2022/parallel_graph_search.html