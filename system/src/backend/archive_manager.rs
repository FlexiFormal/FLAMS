use std::fmt::Debug;
use std::io::BufRead;
use std::path::Path;
use either::Either;
use spliter::ParSpliter;
use tracing::{event, instrument, span};
use crate::backend::archives::{Archive, ArchiveGroup, ArchiveGroupIter, ArchiveId, ArchiveManifest, ParArchiveGroupIter};
use immt_api::formats::{FormatId,FormatStore};
use immt_api::{Str,utils::HMap};

pub struct ArchiveManager {
    archives:Vec<Either<ArchiveGroup,Archive>>,
    len:usize
}
impl<'a> IntoIterator for &'a ArchiveManager {
    type Item = &'a Archive;
    type IntoIter = ArchiveGroupIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ArchiveGroupIter::new(None, &self.archives, self.len)
    }
}
use rayon::prelude::IntoParallelIterator;
use immt_api::archives::IgnoreSource;
use crate::utils::problems::ProblemHandler;
use immt_api::utils::problems::{ProblemHandler as PHandlerT};

impl<'a> IntoParallelIterator for &'a ArchiveManager {
    type Item = &'a Archive;
    type Iter = ParSpliter<ParArchiveGroupIter<'a>>;
    fn into_par_iter(self) -> Self::Iter {
        ParArchiveGroupIter::new(None, &self.archives)
    }
}

impl ArchiveManager {
    pub fn new(mh:&Path,handler:&ProblemHandler,formats:&FormatStore) -> Self {
        let mut manager = Self{ archives:Vec::new(),len:0 };
        manager.load(mh,handler,formats);
        manager
    }

    pub fn find<Id:for<'a>Into<ArchiveId>>(&self,id:Id) -> Option<Either<&ArchiveGroup,&Archive>> {
        self.find_i(id.into().0.into())
    }

    pub fn get_top(&self) -> &[Either<ArchiveGroup,Archive>] { &self.archives }
    pub fn num_archives(&self) -> usize { self.len }

    fn find_i(&self,mut id:Vec<Str>) -> Option<Either<&ArchiveGroup,&Archive>> {
        if id.is_empty() { return None }
        let mut curr = &self.archives;
        loop {
            let head = id.remove(0);
            if id.is_empty() {
                return curr.iter().find_map(|g| {
                    match g {
                        Either::Left(g) if g.id().steps().last().map_or(false, |x| *x == head) => Some(Either::Left(g)),
                        Either::Right(a) if a.id().steps().first().map_or(false, |x| *x == head) => Some(Either::Right(a)),
                        _ => None
                    }
                })
            }
            let g = match curr.iter().find_map(|g| {
                match g {
                    Either::Left(g) if g.id().steps().last().map_or(false, |x| *x == head) => Some(g),
                    _ => None
                }
            }) {
                Some(c) => c,
                None => return None
            };
            if id.len() == 1 && id.last().unwrap().eq_ignore_ascii_case("meta-inf") {
                return g.meta().map(Either::Right)
            }
            curr = &g.archives;
        }
    }

    #[instrument(level = "info",name = "initialization", target = "backend", skip(self,handler,formats), fields(found) )]
    fn load(&mut self, in_path:&Path,handler:&ProblemHandler,formats:&FormatStore) {
        event!(tracing::Level::INFO,"Searching for archives");
        self.load_i(in_path,handler,formats);
        tracing::Span::current().record("found", self.len);
        event!(tracing::Level::INFO,"Done");
    }

    fn load_i(&mut self, mh:&Path,handler:&ProblemHandler,formats:&FormatStore) {
        let mut stack = vec!(vec!());
        let mut curr = match std::fs::read_dir(mh) {
            Ok(rd) => rd,
            _ => {
                handler.add("ArchiveManager",format!("Could not read directory {}",mh.display()));
                return
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
            let _span = span!(target:"backend::initialization",tracing::Level::TRACE,"checking","{}",path.display()).entered();
            if md.is_dir() {
                if d.file_name().to_str().map_or(true,|s| s.starts_with('.')) {continue}
                if d.file_name().eq_ignore_ascii_case("meta-inf") {
                    match get_manifest(&path, mh,handler) {
                        Some(Ok(m)) => {
                            self.add(
                                Archive::new(m,path.parent().unwrap().to_path_buf()),
                                handler,formats
                            );
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
    }

    fn add(&mut self,mut a:Archive,handler:&ProblemHandler,formats:&FormatStore) {
        a.initialize(handler,formats);
        if a.id().is_empty() { return }
        if a.id().steps().len() == 1 {
            self.len += 1;
            self.archives.push(Either::Right(a));
            return
        }
        for c in &mut self.archives {
            match c {
                Either::Left(ref mut g) if g.id().steps().last().map_or(false, |x| x == a.id().steps().first().unwrap()) => {
                    let id = a.id().steps().iter().skip(1).cloned().collect();
                    return Self::add_i(a,g,id,1,vec!(&mut self.len));
                }
                _ => ()
            }
        }
        let g = ArchiveGroup::new(&**a.id().steps().first().unwrap());
        self.archives.push(Either::Left(g));
        let id = a.id().steps().iter().skip(1).cloned().collect();
        Self::add_i(a,self.archives.last_mut().unwrap().as_mut().left().unwrap(),id,1,vec!(&mut self.len))
    }

    fn add_i<'a>(a:Archive,curr:&'a mut ArchiveGroup,mut id:Vec<Str>,mut depth:usize,mut lens:Vec<&'a mut usize>) {
        if id.len() <= 1 {
            if a.manifest.is_meta {
                curr.len += 1;
                for len in lens { *len += 1 }
                curr.set_meta(a);
            } else {
                curr.len += 1;
                for len in lens { *len += 1 }
                curr.archives.push(Either::Right(a));
            }
            return
        }
        depth += 1;
        let head = id.remove(0);
        lens.push(&mut curr.len);
        for g in curr.archives.iter_mut().filter_map(|g| g.as_mut().left()) {
            if g.id().steps().last().map_or(false, |x| *x == head) {
                return Self::add_i(a,g,id,depth,lens)
            }
        }
        let g = ArchiveGroup::new(a.id().steps().iter().take(depth).cloned().collect::<Vec<_>>());
        curr.archives.push(Either::Left(g));
        Self::add_i(a,curr.archives.last_mut().unwrap().as_mut().left().unwrap(),id,depth,lens)
    }
}
impl Debug for ArchiveManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"ArchiveManager")
    }
}


fn get_manifest(metainf:&Path,mh:&Path,handler:&ProblemHandler) -> Option<Result<ArchiveManifest,()>> {
    event!(tracing::Level::TRACE,"Checking for manifest");
    match std::fs::read_dir(metainf) {
        Ok(rd) => {
            for d in rd {
                let d = match d {
                    Err(_) => {
                        handler.add("ArchiveManager",format!("Could not read directory {}",metainf.display()));
                        continue
                    },
                    Ok(d) => d
                };
                if !d.file_name().eq_ignore_ascii_case("manifest.mf") {continue}
                let path = d.path();
                if !path.is_file() { continue }
                return Some(do_manifest(&d.path(),mh,handler))
            }
            event!(tracing::Level::TRACE,"not found");
            None
        }
        _ => {
            handler.add("ArchiveManager",format!("Could not read directory {}",metainf.display()));
            None
        }
    }
}

fn do_manifest(path:&Path,mh:&Path,handler:&ProblemHandler) -> Result<ArchiveManifest,()> {
    let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let mut id:Vec<Str> = Vec::new();
    let mut formats = Vec::new();
    let mut url_base:Str = "".into();
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
            "id" => { id = v.split('/').map(|c| c.into()).collect() }
            "format" => { formats = v.split(',').flat_map(FormatId::try_from).collect() }
            "url-base" => { url_base = v.into() }
            "dependencies" => {
                for d in v.split(',') {
                    dependencies.push(d.into())
                }
            }
            "ignore" => {
                ignores = IgnoreSource::new(v,&path.parent().unwrap().parent().unwrap().join("source"));//Some(v.into());
            }
            _ => {attrs.insert(k.into(),v.into());}
        }
    }
    if id.last().is_some_and(|s| s.eq_ignore_ascii_case("meta-inf") ) {
        is_meta = true;
    }
    if formats.is_empty() && !is_meta {
        handler.add("ArchiveManager",format!("No formats found for archive {}",id.join("/")));
        return Err(())
    }
    if id.is_empty() {
        handler.add("ArchiveManager","No id found for archive");
        return Err(())
    }
    // TODO check path
    let checks_out = {
        let mut ip = path.parent().unwrap().parent().unwrap();
        let mut ids = &*id;
        let mut checks_out = true;
        while !ids.is_empty() {
            let name = ids.last().unwrap();
            ids = &ids[..ids.len()-1];
            if ip.file_name().map_or(false,|f| f == &**name) {
                ip = ip.parent().unwrap();
            } else {
                checks_out = false; break
            }
        }
        checks_out && ip == mh
    };
    if !checks_out {
        handler.add("ArchiveManager",format!("Archive {}'s id does not match its location ({})",id.join("/"),path.display()));
        return Err(())
    }
    if url_base.is_empty() {
        handler.add("ArchiveManager",format!("Archive {} has no URL base",id.join("/")));
        return Err(())
    }
    let id: ArchiveId = id.into();
    dependencies.retain(|d:&ArchiveId| !d.is_empty() && *d != id);
    let a = ArchiveManifest {
        id, formats:formats.into(),
        url_base,
        dependencies:dependencies.into(),
        ignores,attrs,is_meta
    };
    event!(tracing::Level::DEBUG,"Archive found: {}",a.id);
    event!(tracing::Level::TRACE,"{:?}",a);
    Ok(a)
}
