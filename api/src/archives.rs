use std::fmt::Display;
use std::path::Path;
use either::Either;
use oxrdf::{Subject, Triple};
use crate::formats::FormatId;
#[cfg(feature="fs")]
use crate::formats::FormatStore;
#[cfg(feature="fs")]
use crate::utils::problems::ProblemHandler;
use crate::{Seq, Str};
use crate::uris::{ArchiveURIRef, DomURI, DomURIRef};
use crate::utils::HMap;

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveId(Str);
impl ArchiveId {
    #[inline]
    pub fn as_str(&self) -> &str { &self.0 }
    #[inline]
    pub fn as_ref(&self) -> ArchiveIdRef<'_> { ArchiveIdRef(&self.0) }
    #[inline]
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    #[inline]
    pub fn steps(&self) -> impl DoubleEndedIterator<Item=&str> {
        self.0.split('/')
    }
    pub fn new<S:Into<Str>>(s:S) -> Self {
        Self(s.into())
    }
}
impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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
    #[inline]
    pub fn steps(&self) -> impl Iterator<Item=&str> {
        self.0.split('/')
    }
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
    fn new_from<P:ProblemHandler>(data:ArchiveData,path:&Path,handler:&P,formats: &FormatStore) -> Self;
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
    fn new(id:ArchiveId) -> Self;
    fn base(&self) -> &ArchiveGroupBase<A,Self>;
    fn base_mut(&mut self) -> &mut ArchiveGroupBase<A,Self>;
    #[inline]
    fn id<'a>(&'a self) -> ArchiveIdRef where A:'a {
        self.base().id.as_ref()
    }
    #[inline]
    fn meta(&self) -> Option<&A> {
        self.base().meta.as_ref()
    }
    fn archives<'a>(&'a self) -> impl Iterator<Item=&'a A> where A:'a {
        ArchiveIter::new(self.meta(),&self.base().archives)
    }
    fn iter_all<'a>(&'a self) -> impl Iterator<Item=Either<&'a Self,&'a A>> where A:'a {
        FullIter {
            stack:Vec::new(),
            curr:self.base().archives.iter()
        }
    }
    #[cfg(feature = "pariter")]
    fn archives_par<'a>(&'a self) -> impl rayon::iter::ParallelIterator<Item=&'a A> where A:'a+Sync,Self:Sync {
        pariter::ParArchiveIter::new(self.meta(),&self.base().archives)
    }
    #[cfg(feature="fs")]
    #[inline]
    fn load_dir<P:ProblemHandler>(path:&Path,formats:&FormatStore,handler:&P) -> Vec<Either<Self,A>> where A:std::fmt::Debug {
        load_dir::ArchiveLoader {
            archives:Vec::new(),
            formats,
            handler,
            mh:path
        }.run()
    }
    #[inline]
    fn uri<'a>(&'a self) -> Option<ArchiveURIRef<'a>> where A:'a {
        self.meta().map(|m| ArchiveURIRef{
            base:m.dom_uri(),
            archive:self.id()
        })
    }

    fn all_archive_triples<'a>(&'a self) -> impl Iterator<Item=Triple> where A:'a {
        TripleIter {
            curr:self.base().archives.iter(),
            buf1:None,
            buf2:None,
            buf3:None,
            currtop:None,
            stack:Vec::new()
        }
    }
    //fn iter(&self) -> LeafIterator<&Either<Self,A>> { self.iter_leafs() }
}

struct TripleIter<'a,A:ArchiveT,G:ArchiveGroupT<A>> {
    curr:std::slice::Iter<'a,Either<G,A>>,
    buf1:Option<Triple>,
    buf2:Option<Triple>,
    buf3:Option<Triple>,
    currtop:Option<ArchiveURIRef<'a>>,
    stack:Vec<(&'a G,Option<ArchiveURIRef<'a>>)>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> Iterator for TripleIter<'a,A,G> {
    type Item = Triple;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::ontology::rdf::ulo2::{LIBRARY,LIBRARY_GROUP,CONTAINS};
        if let Some(t) = std::mem::replace(&mut self.buf1,std::mem::replace(&mut self.buf2,self.buf3.take())) { return Some(t) }
        loop {
            match self.curr.next() {
                Some(Either::Right(a)) => {
                    let next = Triple {
                        subject: Subject::NamedNode(a.uri().to_iri()),
                        predicate: oxrdf::vocab::rdf::TYPE.into_owned(),
                        object: oxrdf::Term::NamedNode(LIBRARY.into_owned())
                    };
                    if let Some(currtop) = self.currtop {
                        self.buf1 = Some(Triple {
                            subject: Subject::NamedNode(currtop.to_iri()),
                            predicate: CONTAINS.into_owned(),
                            object: oxrdf::Term::NamedNode(a.uri().to_iri())
                        });
                    }
                    return Some(next)
                }
                Some(Either::Left(g)) => {
                    if let Some(uri) = g.uri() {
                        self.stack.push((g,Some(uri)));
                        let next = Triple {
                            subject: Subject::NamedNode(uri.to_iri()),
                            predicate: oxrdf::vocab::rdf::TYPE.into_owned(),
                            object: oxrdf::Term::NamedNode(LIBRARY_GROUP.into_owned())
                        };
                        let meta = g.meta().unwrap();
                        self.buf1 = Some(Triple {
                            subject: Subject::NamedNode(meta.uri().to_iri()),
                            predicate: oxrdf::vocab::rdf::TYPE.into_owned(),
                            object: oxrdf::Term::NamedNode(LIBRARY.into_owned())
                        });
                        self.buf2 = Some(Triple {
                            subject: Subject::NamedNode(uri.to_iri()),
                            predicate: CONTAINS.into_owned(),
                            object: oxrdf::Term::NamedNode(meta.uri().to_iri())
                        });
                        if let Some(currtop) = self.currtop {
                            self.buf3 = Some(Triple {
                                subject: Subject::NamedNode(currtop.to_iri()),
                                predicate: CONTAINS.into_owned(),
                                object: oxrdf::Term::NamedNode(uri.to_iri())
                            });
                        }
                        return Some(next)
                    } else {
                        self.stack.push((g,self.currtop));
                    };
                }
                None => match self.stack.pop() {
                    Some((g,currtop)) => {
                        self.currtop = currtop;
                        self.curr = g.base().archives.iter();
                    }
                    None => return None
                }
            }
        }
    }
}

#[cfg(feature="fs")]
mod load_dir {
    use std::path::Path;
    use either::Either;
    use tracing::{debug, span, trace, trace_span};
    use crate::archives::{ArchiveData, ArchiveGroupT, ArchiveId, ArchiveT, IgnoreSource};
    use crate::formats::{FormatId, FormatStore};
    use crate::utils::problems::ProblemHandler;
    use std::fmt::Debug;
    use std::io::BufRead;
    use crate::Str;
    use crate::uris::DomURI;
    use crate::utils::HMap;

    pub struct ArchiveLoader<'a,P:ProblemHandler,A:ArchiveT+Debug,G:ArchiveGroupT<A>> {
        pub archives: Vec<Either<G,A>>,
        pub formats:&'a FormatStore,
        pub handler:&'a P,
        pub mh:&'a Path
    }
    impl<'a,P:ProblemHandler+'a,A:ArchiveT+Debug,G:ArchiveGroupT<A>> ArchiveLoader<'a,P,A,G> {
        pub fn run(mut self) -> Vec<Either<G,A>> {
            let mut stack = vec!(vec!());
            let mut curr = match std::fs::read_dir(self.mh) {
                Ok(rd) => rd,
                _ => {
                    self.handler.add("archives",format!("Could not read directory {}",self.mh.display()));
                    return self.archives
                }
            };
            'top: loop {
                macro_rules! next {
                    () => {
                        loop {
                            match stack.last_mut() {
                                None => break 'top,
                                Some(s) => {
                                    match s.pop() {
                                        Some(e) => {
                                            curr = std::fs::read_dir(&e).unwrap();
                                            stack.push( Vec::new() );
                                            continue 'top
                                        }
                                        None => { stack.pop();}
                                    }
                                }
                            }
                        }
                    }
                }
                let d = match curr.next() {
                    None => next!(),
                    Some(Ok(d)) => d,
                    _ => continue
                };
                let md = match d.metadata() {
                    Ok(md) => md,
                    _ => continue
                };
                let path = d.path();
                let _span = trace_span!(target:"archives","checking","{}",path.display()).entered();
                //std::thread::sleep(std::time::Duration::from_secs_f32(0.01));
                if md.is_dir() {
                    if d.file_name().to_str().map_or(true,|s| s.starts_with('.')) {continue}
                    if d.file_name().eq_ignore_ascii_case("meta-inf") {
                        match self.get_manifest(&path) {
                            Some(Ok(m)) => {
                                let p = path.parent().unwrap();
                                self.add(A::new_from(m,p,self.handler,self.formats));
                                stack.pop();
                                next!();
                            }
                            Some(_) => {
                                stack.pop();
                                next!();
                            }
                            _ => ()
                        }
                    }
                    stack.last_mut().unwrap().push(path);
                }
            }
            self.archives
        }

        fn add(&mut self,a:A) {
            let mut id = a.id().steps().map(|s| s.to_string()).collect::<Vec<_>>();
            assert!(!id.is_empty());
            if id.len() == 1 {
                self.archives.push(Either::Right(a));
                return
            }
            for c in &mut self.archives {
                match c {
                    Either::Left(ref mut g) if g.id().steps().last().map_or(false, |x| x == *id.first().unwrap()) => {
                        id.remove(0);
                        return Self::add_i(a,g,id,1);
                    }
                    _ => ()
                }
            }
            let g = G::new(ArchiveId::new(id.first().unwrap().to_string()));
            self.archives.push(Either::Left(g));
            id.remove(0);
            Self::add_i(a,self.archives.last_mut().unwrap().as_mut().unwrap_left(),id,1)
        }


        fn add_i(a:A,curr:&mut G,mut id:Vec<String>,mut depth:usize) {
            if id.len() <= 1 {
                if a.data().is_meta {
                    curr.base_mut().meta= Some(a);
                } else {
                    curr.base_mut().archives.push(Either::Right(a));
                }
                return
            }
            depth += 1;
            let head = id.remove(0);
            for g in curr.base_mut().archives.iter_mut().filter_map(|g| g.as_mut().left()) {
                if g.id().steps().last().map_or(false, |x| x == head) {
                    return Self::add_i(a,g,id,depth)
                }
            }
            let g = G::new(ArchiveId::new(a.id().steps().take(depth).collect::<Vec<_>>().join("/")));
            curr.base_mut().archives.push(Either::Left(g));
            Self::add_i(a,curr.base_mut().archives.last_mut().unwrap().as_mut().unwrap_left(),id,depth)
        }

        fn get_manifest(&self,metainf:&Path) -> Option<Result<ArchiveData,()>> {
            trace!("Checking for manifest");
            match std::fs::read_dir(metainf) {
                Ok(rd) => {
                    for d in rd {
                        let d = match d {
                            Err(_) => {
                                self.handler.add("archives",format!("Could not read directory {}",metainf.display()));
                                continue
                            },
                            Ok(d) => d
                        };
                        if !d.file_name().eq_ignore_ascii_case("manifest.mf") {continue}
                        let path = d.path();
                        if !path.is_file() { continue }
                        return Some(self.do_manifest(&path))
                    }
                    trace!("not found");
                    None
                }
                _ => {
                    self.handler.add("archives",format!("Could not read directory {}",metainf.display()));
                    None
                }
            }
        }
        fn do_manifest(&self,path:&Path) -> Result<ArchiveData,()> {
            let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
            let mut id:Str = Str::default();
            let mut formats = Vec::new();
            let mut dom_uri:Str = "".into();
            let mut dependencies = Vec::new();
            let mut ignores = IgnoreSource::default();
            let mut is_meta = false;
            let mut attrs = HMap::default();
            for line in reader.lines() {
                let line = match line {
                    Err(_) => continue,
                    Ok(l) => l
                };
                let (k,v) = match line.split_once(':') {
                    Some((k,v)) => (k.trim(),v.trim()),
                    _ => continue
                };
                match k {
                    "id" => { id = v.into() }
                    "format" => { formats = v.split(',').flat_map(FormatId::try_from).collect() }
                    "url-base" => { dom_uri = v.into() }
                    "dependencies" => {
                        for d in v.split(',') {
                            dependencies.push(ArchiveId::new(d))
                        }
                    }
                    "ignore" => {
                        ignores = IgnoreSource::new(v,&path.parent().unwrap().parent().unwrap().join("source"));//Some(v.into());
                    }
                    _ => {attrs.insert(k.into(),v.into());}
                }
            }
            let id = ArchiveId::new(id);
            if id.steps().last().is_some_and(|s| s.eq_ignore_ascii_case("meta-inf") ) {
                is_meta = true;
            }
            if formats.is_empty() && !is_meta {
                self.handler.add("archives",format!("No formats found for archive {}",id));
                return Err(())
            }
            if id.is_empty() {
                self.handler.add("archives","No id found for archive");
                return Err(())
            }
            let checks_out = {
                let mut ip = path.parent().unwrap().parent().unwrap();
                let ids = id.steps().rev().collect::<Vec<_>>();
                let mut checks_out = true;
                for name in ids {
                    if ip.file_name().map_or(false,|f| f == name) {
                        ip = ip.parent().unwrap();
                    } else {
                        checks_out = false; break
                    }
                }
                checks_out && ip == self.mh
            };
            if !checks_out {
                self.handler.add("archives",format!("Archive {}'s id does not match its location ({})",id,path.display()));
                return Err(())
            }
            if dom_uri.is_empty() {
                self.handler.add("archives",format!("Archive {} has no URL base",id));
                return Err(())
            }
            let id: ArchiveId = id.into();
            dependencies.retain(|d:&ArchiveId| !d.is_empty() && d.as_str() != id.as_str());
            let a = ArchiveData {
                id, formats:formats.into(),
                dom_uri:DomURI::new(dom_uri),
                dependencies:dependencies.into(),
                ignores,attrs,is_meta
            };
            debug!("Archive found: {}",a.id);
            trace!("{:?}",a);
            Ok(a)
        }
    }
}

struct FullIter<'a,A:ArchiveT,G:ArchiveGroupT<A>> {
    stack:Vec<std::slice::Iter<'a,Either<G,A>>>,
    curr:std::slice::Iter<'a,Either<G,A>>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<A>> Iterator for FullIter<'a, A, G> {
    type Item = Either<&'a G,&'a A>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.curr.next() {
                Some(Either::Left(g)) => {
                    let old = std::mem::replace(&mut self.curr, g.base().archives.iter());
                    self.stack.push(old);
                    return Some(Either::Left(g))
                }
                Some(Either::Right(a)) => return Some(Either::Right(a)),
                None => match self.stack.pop() {
                    Some(s) => { self.curr = s; }
                    None => return None
                }
            }
        }
    }
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