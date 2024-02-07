use std::fmt::Display;
use std::ops::Deref;
use std::path::Path;
use either::Either;
use crate::formats::FormatId;
use crate::{Seq, Str};
use crate::source_files::FileLike;
use crate::uris::{ArchiveURIRef, DomURI, DomURIRef};
use crate::utils::HMap;
use crate::utils::iter::{HasChildren, HasChildrenRef, LeafIterator};

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveId(Str);
impl ArchiveId {
    #[inline]
    pub fn as_str(&self) -> &str { self.0.as_str() }
    #[inline]
    pub fn as_ref(&self) -> ArchiveIdRef<'_> { ArchiveIdRef(self.0.as_str()) }
    #[inline]
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    #[inline]
    pub fn steps(&self) -> impl Iterator<Item=&str> {
        self.0.split('/')
    }
}
impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[derive(Clone,Copy,Hash,Debug,PartialEq,Eq)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode))]
pub struct ArchiveIdRef<'a>(pub(crate) &'a str);
impl<'a> ArchiveIdRef<'a> {
    #[inline]
    pub fn as_str(&self) -> &str { self.0 }
    pub fn to_owned(&self) -> ArchiveId { ArchiveId(self.0.into()) }
}
impl<'a> Display for ArchiveIdRef<'a> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveData {
    pub id:ArchiveId,
    pub formats:Seq<FormatId>,
    pub is_meta:bool,
    pub dom_uri:DomURI,
    pub dependencies:Seq<ArchiveId>,
    pub ignores:IgnoreSource,
    pub attrs:HMap<String,String>
}
pub trait ArchiveT {
    fn data(&self) -> &ArchiveData;
    #[inline]
    fn attr(&self,key:&str) -> Option<&str> {
        self.data().attrs.get(key).map(|s| s.as_str())
    }
    #[inline]
    fn id(&self) -> ArchiveIdRef {
        self.data().id.as_ref()
    }
    #[inline]
    fn formats(&self) -> &[FormatId] {
        &self.data().formats
    }
    #[inline]
    fn is_meta(&self) -> bool {
        self.data().is_meta
    }
    #[inline]
    fn dom_uri(&self) -> DomURIRef<'_> {
        self.data().dom_uri.as_ref()
    }
    #[inline]
    fn uri(&self) -> ArchiveURIRef<'_> {
        ArchiveURIRef{
            base:self.data().dom_uri.as_ref(),
            archive:self.id()
        }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveGroupBase<A:ArchiveT,G:ArchiveGroupT<A>> {
    pub id:ArchiveId,
    pub meta:Option<A>,
    pub archives:Vec<Either<G,A>>,
}

pub trait ArchiveGroupT<A:ArchiveT>:Sized {
    fn base(&self) -> &ArchiveGroupBase<A,Self>;
    #[inline]
    fn id<'a>(&'a self) -> ArchiveIdRef where A:'a {
        self.base().id.as_ref()
    }
    #[inline]
    fn meta(&self) -> Option<&A> {
        self.base().meta.as_ref()
    }
    fn archives<'a>(&'a self) -> impl Iterator<Item=&'a A> where A:'a {
        ArchiveIter::new(self.meta(),self.base().archives.as_slice())
    }
    #[cfg(feature = "pariter")]
    fn archives_par<'a>(&'a self) -> impl rayon::iter::ParallelIterator<Item=&'a A> where A:'a+Sync,Self:Sync {
        pariter::ParArchiveIter::new(self.meta(),self.base().archives.as_slice())
    }
    //fn iter(&self) -> LeafIterator<&Either<Self,A>> { self.iter_leafs() }
}

struct ArchiveIter<'a,A:ArchiveT,G:ArchiveGroupT<A>> {
    stack:Vec<&'a [Either<G,A>]>,
    curr:&'a[Either<G,A>],
    meta:Option<&'a A>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> ArchiveIter<'a,A,G> {
    fn new(meta:Option<&'a A>,group:&'a [Either<G,A>]) -> Self {
        Self {
            stack:Vec::new(),
            curr:group,meta
        }
    }
}

impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> Iterator for ArchiveIter<'a,A,G> {
    type Item = &'a A;
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
                        let meta = g.meta();
                        let old = std::mem::replace(&mut self.curr,g.base().archives.as_slice());
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

#[cfg(feature = "pariter")]
mod pariter {
    use either::Either;
    use spliter::{ParSpliter, Spliterator,ParallelSpliterator};
    use crate::archives::{ArchiveGroupT, ArchiveT};

    pub(in crate::archives) struct ParArchiveIter<'a,A:ArchiveT,G:ArchiveGroupT<A>> {
        stack:Vec<&'a [Either<G,A>]>,
        curr:&'a[Either<G,A>],
        meta:Option<&'a A>,
    }
    impl<'a,A:ArchiveT+Sync,G:ArchiveGroupT<A>+Sync> ParArchiveIter<'a,A,G> {
        pub(in crate::archives) fn new(meta:Option<&'a A>,group:&'a [Either<G,A>]) -> ParSpliter<Self> {
            Self {
                stack:Vec::new(),
                curr:group, meta
            }.par_split()
        }
    }

    impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> Iterator for ParArchiveIter<'a,A,G> {
        type Item = &'a A;
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
                            let meta = g.meta();
                            let old = std::mem::replace(&mut self.curr,g.base().archives.as_slice());
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

    impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> Spliterator for ParArchiveIter<'a,A,G> {
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
}


// -------------------------------------------------------------------------

#[cfg(target_os = "windows")]
const PATH_SEPARATOR:&str = "\\\\";
#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR:char = '/';

#[derive(Default,Clone,Debug)]
pub struct IgnoreSource (
    Option<regex::Regex>
);
impl Display for IgnoreSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(r) => write!(f,"{}",r),
            None => write!(f,"(None)")
        }
    }
}
impl IgnoreSource {
    pub fn new(regex:&str,source_path:&Path) -> IgnoreSource {
        if regex.is_empty() { return Self::default() }
        #[cfg(target_os = "windows")]
            let regex = regex.replace('/', PATH_SEPARATOR);
        let s = regex.replace('.',r"\.").replace('*',".*");//.replace('/',r"\/");
        let s = s.split('|').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("|");
        let p = source_path.display();//path.to_str().unwrap().replace('/',r"\/");
        #[cfg(target_os = "windows")]
            let p = p.to_string().replace('\\', PATH_SEPARATOR);
        let s= format!("{}{}({})", p, PATH_SEPARATOR, s);
        Self(regex::Regex::new(&s).ok())
    }
    pub fn ignores(&self,p:&Path) -> bool {
        match &self.0 {
            Some(r) => r.is_match(p.to_str().unwrap()),
            None => false
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IgnoreSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
        match &self.0 {
            None => Option::<&str>::None.serialize(serializer),
            Some(r) => Some(r.as_str()).serialize(serializer)
        }
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IgnoreSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
        Ok(IgnoreSource(
            match Option::<&'de str>::deserialize(deserializer)? {
                None => None,
                Some(s) => Some(
                    regex::Regex::new(s).map_err(|_| serde::de::Error::custom("Invalid regex"))?
                )
            }
        ))
    }
}

#[cfg(feature = "bincode")]
impl bincode::Encode for IgnoreSource {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        match &self.0 {
            None => Option::<&str>::None.encode(encoder),
            Some(r) => Some(r.as_str()).encode(encoder)
        }
    }
}
#[cfg(feature = "bincode")]
impl bincode::Decode for IgnoreSource {
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(IgnoreSource(
            match Option::<String>::decode(decoder)? {
                None => None,
                Some(s) => Some(
                    regex::Regex::new(&s).map_err(|_| bincode::error::DecodeError::Other("Invalid regex"))?
                )
            }
        ))
    }
}
#[cfg(feature = "bincode")]
impl<'de> bincode::BorrowDecode<'de> for IgnoreSource {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(IgnoreSource(
            match Option::<&'de str>::borrow_decode(decoder)? {
                None => None,
                Some(s) => Some(
                    regex::Regex::new(s).map_err(|_| bincode::error::DecodeError::Other("Invalid regex"))?
                )
            }
        ))
    }
}