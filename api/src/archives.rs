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

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
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
impl From<Vec<Str>> for ArchiveId {
    fn from(v: Vec<Str>) -> Self {
        Self(v.join("/").into())
    }
}
impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone,Copy,Hash,Debug,PartialEq,Eq,PartialOrd,Ord)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode))]
pub struct ArchiveIdRef<'a>(pub(crate) &'a str);
impl<'a> ArchiveIdRef<'a> {
    #[inline]
    pub fn as_str(&self) -> &str { self.0 }
    pub fn to_owned(&self) -> ArchiveId { ArchiveId(self.0.into()) }
    pub fn last_name(&self) -> &'a str {
        self.0.rsplit_once('/').map(|(_,s)| s).unwrap_or(self.0)
    }
    #[inline]
    pub fn steps(&self) -> impl DoubleEndedIterator<Item=&str> {
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
impl ArchiveT for ArchiveData {
    #[inline(always)]
    fn new_from<P:ProblemHandler>(data:ArchiveData,path:&Path,handler:&P,formats: &FormatStore) -> Self { data }
    #[inline(always)]
    fn data(&self) -> &ArchiveData { self }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveGroupBase<A:ArchiveT,G:ArchiveGroupT<Archive=A>> {
    pub id:ArchiveId,
    pub meta:Option<A>,
    pub archives:Vec<Either<G,A>>,
}

pub trait ArchiveGroupT:Sized {
    type Archive:ArchiveT;
    fn new(id:ArchiveId) -> Self;
    fn base(&self) -> &ArchiveGroupBase<Self::Archive,Self>;
    fn base_mut(&mut self) -> &mut ArchiveGroupBase<Self::Archive,Self>;
    #[inline]
    fn id<'a>(&'a self) -> ArchiveIdRef where Self::Archive:'a {
        self.base().id.as_ref()
    }
    #[inline]
    fn meta(&self) -> Option<&Self::Archive> {
        self.base().meta.as_ref()
    }
    fn archives<'a>(&'a self) -> impl Iterator<Item=&'a Self::Archive> where Self::Archive:'a {
        ArchiveIter::new(self.meta(),&self.base().archives)
    }
    fn iter_all<'a>(&'a self) -> impl Iterator<Item=Either<&'a Self,&'a Self::Archive>> where Self::Archive:'a {
        FullIter {
            stack:Vec::new(),
            curr:self.base().archives.iter()
        }
    }
    #[cfg(feature = "pariter")]
    fn archives_par<'a>(&'a self) -> impl rayon::iter::ParallelIterator<Item=&'a Self::Archive> where Self::Archive:'a+Sync,Self:Sync {
        pariter::ParArchiveIter::new(self.meta(),&self.base().archives)
    }
    #[cfg(feature="fs")]
    #[inline]
    fn load_dir<P:ProblemHandler>(path:&Path,formats:&FormatStore,handler:&P) -> Vec<Either<Self,Self::Archive>> where Self::Archive:std::fmt::Debug {
        load_dir::ArchiveLoader {
            archives:Vec::new(),
            formats,
            handler,
            mh:path
        }.run()
    }
    #[cfg(feature="tokio")]
    #[inline]
    async fn load_dir_async<P:ProblemHandler>(path:&Path,formats:&FormatStore,handler:&P) -> Vec<Either<Self,Self::Archive>> where Self::Archive:std::fmt::Debug {
        load_dir::ArchiveLoader {
            archives:Vec::new(),
            formats,
            handler,
            mh:path
        }.run_async().await
    }
    #[inline]
    fn uri<'a>(&'a self) -> Option<ArchiveURIRef<'a>> where Self::Archive:'a {
        self.meta().map(|m| ArchiveURIRef{
            base:m.dom_uri(),
            archive:self.id()
        })
    }

    fn all_archive_triples<'a>(&'a self) -> impl Iterator<Item=Triple> where Self::Archive:'a {
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

struct TripleIter<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> {
    curr:std::slice::Iter<'a,Either<G,A>>,
    buf1:Option<Triple>,
    buf2:Option<Triple>,
    buf3:Option<Triple>,
    currtop:Option<ArchiveURIRef<'a>>,
    stack:Vec<(&'a G,Option<ArchiveURIRef<'a>>)>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> Iterator for TripleIter<'a,A,G> {
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
    use tracing::{debug, trace, trace_span};
    use crate::archives::{ArchiveData, ArchiveGroupT, ArchiveId, ArchiveIdRef, ArchiveT, IgnoreSource};
    use crate::formats::{FormatId, FormatStore};
    use crate::utils::problems::ProblemHandler;
    use std::fmt::Debug;
    use std::io::BufRead;
    use crate::Str;
    use crate::uris::DomURI;
    use crate::utils::HMap;

    macro_rules! id { ($e:expr) => { match $e.as_ref() {
        Either::Left(g) => g.id(),
        Either::Right(a) => a.id()
    }};}

    pub struct ArchiveLoader<'a, P: ProblemHandler, A: ArchiveT + Debug, G: ArchiveGroupT<Archive=A>> {
        pub archives: Vec<Either<G, A>>,
        pub formats: &'a FormatStore,
        pub handler: &'a P,
        pub mh: &'a Path
    }

    macro_rules! do_manifest {
        ($self:ident,$path:expr,$next_line:expr) => {
            let mut id: String = String::new();
            let mut formats = Vec::new();
            let mut dom_uri: String = String::new();
            let mut dependencies = Vec::new();
            let mut ignores = IgnoreSource::default();
            let mut is_meta = false;
            let mut attrs = HMap::default();
            loop {
                let line = $next_line;
                let (k, v) = match line.split_once(':') {
                    Some((k, v)) => (k.trim(), v.trim()),
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
                        ignores = IgnoreSource::new(v, &$path.parent().unwrap().parent().unwrap().join("source"));//Some(v.into());
                    }
                    _ => { attrs.insert(k.into(), v.into()); }
                }
            }
            let id = ArchiveId::new(id);
            if id.steps().last().is_some_and(|s| s.eq_ignore_ascii_case("meta-inf")) {
                is_meta = true;
            }
            if formats.is_empty() && !is_meta {
                $self.handler.add("archives", format!("No formats found for archive {}", id));
                return Err(())
            }
            if id.is_empty() {
                $self.handler.add("archives", "No id found for archive");
                return Err(())
            }
            let checks_out = {
                let mut ip = $path.parent().unwrap().parent().unwrap();
                let ids = id.steps().rev().collect::<Vec<_>>();
                let mut checks_out = true;
                for name in ids {
                    if ip.file_name().map_or(false, |f| f == name) {
                        ip = ip.parent().unwrap();
                    } else {
                        checks_out = false;
                        break
                    }
                }
                checks_out && ip == $self.mh
            };
            if !checks_out {
                $self.handler.add("archives", format!("Archive {}'s id does not match its location ({})", id, $path.display()));
                return Err(())
            }
            if dom_uri.is_empty() {
                $self.handler.add("archives", format!("Archive {} has no URL base", id));
                return Err(())
            }
            let id: ArchiveId = id.into();
            dependencies.retain(|d: &ArchiveId| !d.is_empty() && d.as_str() != id.as_str());
            let a = ArchiveData {
                id,
                formats: formats.into(),
                dom_uri: DomURI::new(dom_uri),
                dependencies: dependencies.into(),
                ignores,
                attrs,
                is_meta
            };
            debug!("Archive found: {}",a.id);
            trace!("{:?}",a);
            Ok(a)
        }
    }

    macro_rules! run {
        ($self:ident,$f:ident => $readdir:expr;$curr:ident => $next:expr;$d:ident => $meta:expr;$path:ident => $manifest:expr) => {
            let mut stack = vec!(vec!());
            let mut $curr =  match {let $f=$self.mh;$readdir} {
                Ok(rd) => rd,
                _ => {
                    $self.handler.add("archives", format!("Could not read directory {}", $self.mh.display()));
                    return $self.archives
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
                                            $curr = {let $f=&e;$readdir}.unwrap();
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
                let $d = $next;
                let md = match $meta {
                    Ok(md) => md,
                    _ => continue
                };
                let $path = $d.path();
                let _span = trace_span!("checking","{}",$path.display()).entered();
                //std::thread::sleep(std::time::Duration::from_secs_f32(0.01));
                if md.is_dir() {
                    if $d.file_name().to_str().map_or(true, |s| s.starts_with('.')) { continue }
                    if $d.file_name().eq_ignore_ascii_case("meta-inf") {
                        match $manifest {
                            Some(Ok(m)) => {
                                let p = $path.parent().unwrap();
                                $self.add(A::new_from(m, p, $self.handler, $self.formats));
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
                    stack.last_mut().unwrap().push($path);
                }
            }
            $self.archives
        }
    }

    impl<'a, P: ProblemHandler + 'a, A: ArchiveT + Debug, G: ArchiveGroupT<Archive=A>> ArchiveLoader<'a, P, A, G> {
        #[cfg(feature = "tokio")]
        pub async fn run_async(mut self) -> Vec<Either<G, A>> {
            run!{self,
                f=>tokio::fs::read_dir(f).await;
                curr => match curr.next_entry().await {
                    Ok(None) => next!(),
                    Ok(Some(d)) => d,
                    _ => continue
                };
                d => d.metadata().await;
                p => self.get_manifest_async(&p).await
            }
        }

        pub fn run(mut self) -> Vec<Either<G, A>> {
            run!{self,
                f => std::fs::read_dir(f);
                curr => match curr.next() {
                    None => next!(),
                    Some(Ok(d)) => d,
                    _ => continue
                };
                d => d.metadata();
                p => self.get_manifest(&p)
            }
        }

        fn add(&mut self, a: A) {
            let mut id = a.id().steps().map(|s| s.to_string()).collect::<Vec<_>>();
            assert!(!id.is_empty());
            if id.len() == 1 {
                match self.archives.binary_search_by_key(&a.id(), |e| id!(e)) {
                    Ok(i) => self.archives[i] = Either::Right(a),
                    Err(i) => self.archives.insert(i, Either::Right(a))
                }
                return
            }
            let i = match self.archives.binary_search_by_key(&id.first().unwrap().as_str(), |e| id!(e).last_name()) {
                Ok(i) => match &mut self.archives[i] {
                    Either::Left(g) => {
                        id.remove(0);
                        return Self::add_i(a, g, id, 1);
                    }
                    _ => unreachable!()
                },
                Err(i) => i
            };
            let g = G::new(ArchiveId::new(id.first().unwrap().to_string()));
            self.archives.insert(i, Either::Left(g));
            id.remove(0);
            Self::add_i(a, self.archives[i].as_mut().unwrap_left(), id, 1)
        }

        fn add_i(a: A, curr: &mut G, mut id: Vec<String>, mut depth: usize) {
            if id.len() <= 1 {
                if a.data().is_meta {
                    curr.base_mut().meta = Some(a);
                } else {
                    match curr.base_mut().archives.binary_search_by_key(&a.id().last_name(), |e| id!(e).last_name()) {
                        Ok(i) => curr.base_mut().archives[i] = Either::Right(a),
                        Err(i) => curr.base_mut().archives.insert(i, Either::Right(a))
                    }
                }
                return
            }
            depth += 1;
            let head = id.remove(0);
            let i = match curr.base_mut().archives.binary_search_by_key(&head.as_str(), |e| id!(e).last_name()) {
                Ok(i) => match &mut curr.base_mut().archives[i] {
                    Either::Left(g) => {
                        return Self::add_i(a, g, id, depth)
                    }
                    _ => unreachable!()
                },
                Err(i) => i
            };
            let g = G::new(ArchiveId::new(a.id().steps().take(depth).collect::<Vec<_>>().join("/")));
            curr.base_mut().archives.insert(i, Either::Left(g));
            Self::add_i(a, curr.base_mut().archives[i].as_mut().unwrap_left(), id, depth)
        }

        #[cfg(feature = "tokio")]
        async fn get_manifest_async(&self, metainf: &Path) -> Option<Result<ArchiveData, ()>> {
            trace!("Checking for manifest");
            match tokio::fs::read_dir(metainf).await {
                Ok(mut rd) => {
                    loop {
                        let d = match rd.next_entry().await {
                            Err(_) => {
                                self.handler.add("archives", format!("Could not read directory {}", metainf.display()));
                                continue
                            },
                            Ok(Some(d)) => d,
                            Ok(None) => break
                        };
                        if !d.file_name().eq_ignore_ascii_case("manifest.mf") { continue }
                        let path = d.path();
                        if !path.is_file() { continue }
                        return Some(self.do_manifest_async(&path).await)
                    }
                    trace!("not found");
                    None
                }
                _ => {
                    self.handler.add("archives", format!("Could not read directory {}", metainf.display()));
                    None
                }
            }
        }

        fn get_manifest(&self, metainf: &Path) -> Option<Result<ArchiveData, ()>> {
            trace!("Checking for manifest");
            match std::fs::read_dir(metainf) {
                Ok(rd) => {
                    for d in rd {
                        let d = match d {
                            Err(_) => {
                                self.handler.add("archives", format!("Could not read directory {}", metainf.display()));
                                continue
                            },
                            Ok(d) => d
                        };
                        if !d.file_name().eq_ignore_ascii_case("manifest.mf") { continue }
                        let path = d.path();
                        if !path.is_file() { continue }
                        return Some(self.do_manifest(&path))
                    }
                    trace!("not found");
                    None
                }
                _ => {
                    self.handler.add("archives", format!("Could not read directory {}", metainf.display()));
                    None
                }
            }
        }

        #[cfg(feature = "tokio")]
        async fn do_manifest_async(&self, path: &Path) -> Result<ArchiveData, ()> {
            use tokio::io::AsyncBufReadExt;
            let reader = tokio::io::BufReader::new(tokio::fs::File::open(path).await.unwrap());
            let mut lines = reader.lines();
            do_manifest!{self,path,match lines.next_line().await {
                Err(_) => continue,
                Ok(Some(l)) => l,
                _ => break
            }}
        }

        fn do_manifest(&self, path: &Path) -> Result<ArchiveData, ()> {
            let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
            let mut lines = reader.lines();
            do_manifest!{self,path,match lines.next() {
                Some(Err(_)) => continue,
                Some(Ok(l)) => l,
                _ => break
            }}
        }
    }
}

struct FullIter<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> {
    stack:Vec<std::slice::Iter<'a,Either<G,A>>>,
    curr:std::slice::Iter<'a,Either<G,A>>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> Iterator for FullIter<'a, A, G> {
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

struct ArchiveIter<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> {
    stack:Vec<&'a [Either<G,A>]>,
    curr:&'a[Either<G,A>],
    meta:Option<&'a A>,
}
impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> ArchiveIter<'a,A,G> {
    fn new(meta:Option<&'a A>,group:&'a [Either<G,A>]) -> Self {
        Self {
            stack:Vec::new(),
            curr:group,meta
        }
    }
}

impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> Iterator for ArchiveIter<'a,A,G> {
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

    pub(in crate::archives) struct ParArchiveIter<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> {
        stack:Vec<&'a [Either<G,A>]>,
        curr:&'a[Either<G,A>],
        meta:Option<&'a A>,
    }
    impl<'a,A:ArchiveT+Sync,G:ArchiveGroupT<Archive=A>+Sync> ParArchiveIter<'a,A,G> {
        pub(in crate::archives) fn new(meta:Option<&'a A>,group:&'a [Either<G,A>]) -> ParSpliter<Self> {
            Self {
                stack:Vec::new(),
                curr:group, meta
            }.par_split()
        }
    }

    impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> Iterator for ParArchiveIter<'a,A,G> {
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

    impl<'a,A:ArchiveT,G:ArchiveGroupT<Archive=A>> Spliterator for ParArchiveIter<'a,A,G> {
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