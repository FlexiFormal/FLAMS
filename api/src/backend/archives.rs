use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use either::Either;
use crate::InputFormat;
use crate::utils::MMTURI;


#[derive(Debug)]
pub struct Archive {
    pub(in crate::backend) manifest:ArchiveManifest,
    path:PathBuf
}

impl Archive {
    pub fn id(&self) -> &ArchiveId { &self.manifest.id }
    pub fn formats(&self) -> &[InputFormat] { &self.manifest.formats }
    pub fn path(&self) -> &Path { &self.path }
    pub fn new(manifest:ArchiveManifest,path:PathBuf) -> Self {
        Self { manifest, path }
    }
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
    type Item = &'a Archive;
    type Iter = ParSpliter<ParArchiveGroupIter<'a>>;
    fn into_par_iter(self) -> Self::Iter {
        ParArchiveGroupIter::new(self.meta.as_ref(), &self.archives)
    }
}

#[derive(Debug)]
pub struct ArchiveManifest {
    pub id:ArchiveId,
    pub formats:Box<[InputFormat]>,
    pub is_meta:bool,
    pub content_uri:MMTURI,
    pub narrative_uri:MMTURI,
    pub url_base:Box<str>,
    pub dependencies:Box<[ArchiveId]>,
    pub ignores:Option<Box<str>>,
    pub attrs:BTreeMap<String,String>
}


#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
#[derive(Clone,Hash,PartialEq,Eq)]
pub struct ArchiveId(pub(in crate::backend) Box<[Box<str>]>);
impl ArchiveId {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn steps(&self) -> &[Box<str>] { &self.0 }
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
impl From<Vec<Box<str>>> for ArchiveId {
    fn from(v:Vec<Box<str>>) -> Self {
        Self(v.into_boxed_slice())
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
                        match meta {
                            Some(a) => {
                                self.len -= 1;
                                return Some(a)
                            },
                            _ => ()
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
                        match meta {
                            Some(a) => {
                                return Some(a)
                            },
                            _ => ()
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