use std::io::BufRead;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Output;
use futures::TryFutureExt;
use tracing::{instrument, Instrument};
use immt_core::building::formats::{ShortId, SourceFormatId};
use immt_core::ontology::archives::{ArchiveGroup, MathArchiveSpec, StorageSpec};
use immt_core::uris::archives::{ArchiveId, ArchiveURI, ArchiveURIRef};
use immt_core::uris::base::BaseURI;
use immt_core::utils::ignore_regex::IgnoreSource;
use immt_core::utils::{arrayvec, VecMap};
use crate::backend::archives::{Archive, MathArchive, Storage};
use crate::building::targets::SourceFormat;
use crate::utils::asyncs::{ChangeSender, lock};
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::ontology::rdf::terms::Quad;
use immt_core::ulo;
use immt_core::utils::filetree::FileChange;
use immt_core::utils::triomphe::Arc;

#[derive(Clone,Debug)]
pub enum ArchiveChange{
    New(ArchiveURI),
    NewGroup(ArchiveId),
    Update(ArchiveURI),
    Deleted(ArchiveURI)
}

#[derive(Debug)]
pub struct ArchiveTree {
    archives: Vec<Archive>,
    groups: Vec<ArchiveGroup>,
}

#[derive(Debug)]
pub struct ArchiveManager{
    lock:lock::Lock<ArchiveTree>,
    change_sender: ChangeSender<ArchiveChange>,
    filechange_sender: ChangeSender<FileChange>,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self {
            lock:lock::Lock::new(ArchiveTree {
                archives: Vec::new(),
                groups: Vec::new()
            }),
            change_sender: ChangeSender::new(256),
            filechange_sender: ChangeSender::new(256),
        }
    }
}

#[cfg(feature = "tokio")]
#[derive(Debug)]
pub struct ArchiveManagerAsync{
    lock:tokio::sync::RwLock<ArchiveTree>,
    change_sender: ChangeSender<ArchiveChange>,
    filechange_sender: ChangeSender<FileChange>,
}
#[cfg(feature = "tokio")]
impl Default for ArchiveManagerAsync {
    fn default() -> Self {
        Self {
            lock:tokio::sync::RwLock::new(ArchiveTree {
                archives: Vec::new(),
                groups: Vec::new()
            }),
            change_sender: ChangeSender::new(256),
            filechange_sender: ChangeSender::new(256),
        }
    }
}

impl ArchiveManager {
    pub fn load(&self, path:&Path, formats:&[SourceFormat]) -> Vec<Quad> {
        self.lock.write(|s| s.load(path,formats,&self.filechange_sender,&self.change_sender))
    }
    pub fn with_archives<R,F:FnOnce(&[Archive]) -> R>(&self,f:F) -> R {
        self.lock.read(|s| f(s.archives()) )
    }
    pub fn get_archives(&self) -> impl Deref<Target=[Archive]> + '_ {
        parking_lot::RwLockReadGuard::map(self.lock.returnable(),|t| t.archives())
    }
    pub fn load_par(&self, path:&Path, formats:&[SourceFormat]) -> Vec<Quad> {
        self.lock.write(|s| s.load_par(path,formats,&self.filechange_sender,&self.change_sender))
    }
    pub fn with_tree<R>(&self, f:impl FnOnce(&ArchiveTree) -> R) -> R {
        self.lock.read(|s| f(&*s))
    }

    pub fn get_tree(&self) -> impl Deref<Target=ArchiveTree> + '_ {
        self.lock.returnable()
    }

    pub fn find<R,S:AsRef<str>>(&self,id:S,f:impl FnOnce(Option<&Archive>) -> R) -> R {
        self.lock.read(|s| {
            let id = id.as_ref();
            if let Some(a) = ArchiveTree::find_i(s.archives.as_slice(), id) {
                f(Some(a))
            } else {f(None)}
        })
    }
}

#[cfg(feature = "tokio")]
impl ArchiveManagerAsync {
    pub async fn load(&self, path:PathBuf, formats:Box<[SourceFormat]>) -> Vec<Quad> {
        let mut lock = self.lock.write().await;
        lock.load_async(path,formats,self.filechange_sender.clone(),&self.change_sender).await
    }
    pub async fn with_archives<R>(&self,f:impl FnOnce(&[Archive]) -> R) -> R {
        let mut lock = self.lock.read().await;
        f(lock.archives())
    }
    pub async fn get_archives(&self) -> impl Deref<Target=[Archive]> + '_ {
        tokio::sync::RwLockReadGuard::map(self.lock.read().await,|t| t.archives())
    }
    pub async fn with_tree<R>(&self, f:impl FnOnce(&ArchiveTree) -> R) -> R {
        let lock = self.lock.read().await;
        f(&*lock)
    }
    pub async fn get_tree(&self) -> impl Deref<Target=ArchiveTree> + '_ {
        self.lock.read().await
    }

    pub async fn find<R,S:AsRef<str>,F:FnOnce(Option<&Archive>) -> R>(&self,id:S,f:F) -> R {
        let mut lock = self.lock.read().await;
        let id = id.as_ref();
        if let Some(a) = ArchiveTree::find_i(lock.archives.as_slice(), id) {
            f(Some(a))
        } else {f(None)}
    }
}

impl ArchiveTree {
    pub fn archives(&self) -> &[Archive] { &self.archives }
    pub fn groups(&self) -> &[ArchiveGroup] { &self.groups }
    pub fn find_archive(&self,id:&ArchiveId) -> Option<&Archive> {
        Self::find_i(self.archives.as_slice(), id.as_str())
    }
    pub fn find_group_or_archive(&self,prefix:impl AsRef<str>) -> Option<&ArchiveGroup> {
        let sr = prefix.as_ref();
        let mut ls = self.groups.as_slice();
        let mut ret = None;
        for step in sr.split('/') {
            let i = ls.binary_search_by_key(&step,|v| v.id().last_name())
                .ok()?;
            let elem = &ls[i];
            ret = Some(elem);
            ls = match elem {
                ArchiveGroup::Group { children, .. } => children.as_slice(),
                ArchiveGroup::Archive(..) => &[]
            }
        }
        ret
    }
    fn find_i<'a>(archives:&'a [Archive], id:&str) -> Option<&'a Archive> {
        archives.binary_search_by_key(&id,|a| a.uri().id().as_str()).ok().map(|i| &archives[i])
    }
    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    fn load(&mut self, path:&Path, formats:&[SourceFormat], fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>) -> Vec<Quad> {
        tracing::info!(target:"archives","Searching for archives");
        let (changed,new,deleted,quads) = self.do_specs(ArchiveIterator::new(path,formats),asender);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");
        for a in self.archives.iter_mut().filter_map(|a| match a {
            Archive::Physical(a) => Some(a),
            _ => None
        }) {
            a.update_sources(formats, fsender);
        }
        for g in &mut self.groups {
            g.update(&|id| Self::find_i(self.archives.as_slice(), id.as_str()).and_then(|a|
                if let Archive::Physical(ma) = a {
                    Some(ma.state())
                } else {None }
            ));
        }
        quads
    }

    #[cfg(feature = "rayon")]
    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    fn load_par(&mut self, path:&Path, formats:&[SourceFormat],fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>) -> Vec<Quad> {
        use spliter::ParallelSpliterator;
        use rayon::iter::*;
        tracing::info!(target:"archives","Searching for archives");
        let span = tracing::Span::current();
        
        let news = ArchiveIterator::new_in_span(path,formats,Some(&span)).par_split().into_par_iter()
            .map(|a| {
                let mut a = MathArchive::new_from(a);
                a.update_sources(formats,&fsender);
                a
            }).collect::<Vec<_>>();
        let (changed,new,deleted,quads) = self.do_specs_ii(news,asender);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");

        for g in &mut self.groups {
            g.update(&|id| Self::find_i(self.archives.as_slice(), id.as_str()).and_then(|a|
                if let Archive::Physical(ma) = a {
                    Some(ma.state())
                } else {None }
            ));
        }
        quads
    }

    #[cfg(feature = "tokio")]
    async fn async_dir(path:PathBuf, currp:String) -> Result<(PathBuf,String),Vec<(PathBuf, String)>> {
        let mut curr = match tokio::fs::read_dir(&path).await {
            Ok(rd) => rd,
            _ => {
                tracing::warn!(target:"archives","Could not read directory {}", path.display());
                return Err(Vec::new())
            }
        };
        let mut stack = Vec::new();
        while let Ok(Some(d)) = curr.next_entry().await {
            let md = match d.metadata().await {
                Ok(md) => md,
                _ => continue
            };
            if md.is_dir() {
                if d.file_name().to_str().map_or(true, |s| s.starts_with('.')) { continue }
                let path = d.path();
                if d.file_name().eq_ignore_ascii_case("meta-inf") {
                    match find_manifest_async(&path,&currp).await {
                        Some(path) => {
                            let currp = currp.clone();
                            return Ok((path,currp))
                        },
                        _ => ()
                    }
                }
                let name = d.file_name();
                let name = name.to_str().unwrap();
                stack.push((path, if currp.is_empty() {name.to_string()} else {format!("{}/{}", currp, name)}));
            }
        }
        Err(stack)
    }

    #[cfg(feature = "tokio")]
    #[instrument(level = "info",
        target = "archives",
        name = "Loading archives",
        fields(path = %path.display()),
        skip_all
    )]
    async fn load_async(&mut self, path:PathBuf, formats:Box<[SourceFormat]>,fsender:ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>) -> Vec<Quad> {
        tracing::info!(target:"archives","Searching for archives");

        //let ret = Self::do_dir_i(path, String::new(),formats,fsender).await;
        let mut js = tokio::task::JoinSet::new();
        let mut ret
            = Vec::new();

        fn do_dir(path:PathBuf,currp:String) -> impl std::future::Future<Output=Result<Result<(PathBuf,String),Vec<(PathBuf,String)>>,Option<MathArchive>>> {
            async move {
                Ok(ArchiveTree::async_dir(path, currp).in_current_span().await)
            }.in_current_span()
        }
        fn do_manifest(formats:&Box<[SourceFormat]>,fsender:&ChangeSender<FileChange>,p:PathBuf,s:String) -> impl std::future::Future<Output=Result<Result<(PathBuf,String),Vec<(PathBuf,String)>>,Option<MathArchive>>> {
            let formats = formats.clone();
            let fsender = fsender.clone();
            async move {
                if let Ok(m) = do_manifest_async(&p,&s).in_current_span().await {
                    let mut m = MathArchive::new_from(m);
                    m.update_sources_async(&formats, &fsender).await;
                    Err(Some(m))
                } else {Err(None)}
            }.in_current_span()
        }
        let f = do_dir(path, String::new());
        js.spawn(f);
        while let Some(Ok(r)) = js.join_next().await {
            match r {
                Ok(Ok((p,s))) => {
                    let f = do_manifest(&formats,&fsender,p,s);
                    js.spawn(f);
                }
                Ok(Err(v)) => for (path,currp) in v {
                        let f = do_dir(path, currp);
                        js.spawn(f);
                }
                Err(Some(m)) => ret.push(m),
                Err(_) => ()
            }
        }
        assert!(js.is_empty());
        drop(js);

        let (changed,new,deleted,quads) = self.do_specs_ii(ret,asender);

        for g in &mut self.groups {
            g.update(&|id| Self::find_i(self.archives.as_slice(), id.as_str()).and_then(|a|
                if let Archive::Physical(ma) = a {
                    Some(ma.state())
                } else {None }
            ));
        }
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");
        quads
    }

    fn do_specs_ii(&mut self,iter:Vec<MathArchive>,sender:&ChangeSender<ArchiveChange>) -> (usize,usize,usize,Vec<Quad>) {
        let mut old : Vec<_> = self.archives.iter().filter_map(|a| match a {
            Archive::Physical(a) => Some(a.uri().to_owned()),
            _ => None
        }).collect();
        let mut changed = 0; let mut new = 0;
        let mut ret = vec!();
        for spec in iter {
            match self.archives.binary_search_by(|a| a.uri().id().cmp(spec.id())) {
                Ok(i) => match &self.archives[i] {
                    Archive::Physical(orig) => {
                        if orig.archive_spec() == spec.archive_spec() {
                            old.retain(|a| a.id() != orig.id());
                            continue
                        }
                        let uri = spec.uri().to_owned();
                        self.archives[i] = Archive::Physical(spec);
                        changed += 1;
                        sender.send(ArchiveChange::Update(uri));
                    }
                    _ => unreachable!()
                }
                Err(i) => {
                    let uri = spec.uri().to_owned();
                    self.add_archive(uri.as_ref(),sender,&mut ret);
                    self.archives.insert(i,Archive::Physical(spec));
                    new += 1;
                    sender.send(ArchiveChange::New(uri));
                }
            }
        }
        let deleted = old.len();
        for o in old {
            self.delete_archive(o.id());
            sender.send(ArchiveChange::Deleted(o));
        }
        (changed,new,deleted,ret)
    }

    fn do_specs<F:Iterator<Item=MathArchiveSpec>>(&mut self,iter:F,sender:&ChangeSender<ArchiveChange>) -> (usize,usize,usize,Vec<Quad>) {
        let mut old : Vec<_> = self.archives.iter().map(|a| a.uri().to_owned()).collect();
        let mut changed = 0; let mut new = 0;
        let mut ret = vec!();
        for spec in iter {
            match self.archives.binary_search_by(|a| a.uri().id().cmp(spec.storage.uri.id())) {
                Ok(i) => match &self.archives[i] {
                    Archive::Physical(orig) => {
                        if orig.archive_spec() == spec.as_ref() {
                            old.retain(|a| a.id() != orig.id());
                            continue
                        }
                        let uri = spec.storage.uri.clone();
                        let ma = Archive::Physical(MathArchive::new_from(spec));
                        self.archives[i] = ma;
                        changed += 1;
                        sender.send(ArchiveChange::Update(uri));
                    }
                    _ => unreachable!()
                }
                Err(i) => {
                    let uri = spec.storage.uri.clone();
                    let ma = Archive::Physical(MathArchive::new_from(spec));
                    todo!("state");
                    self.add_archive(uri.as_ref(),sender,&mut ret);
                    self.archives.insert(i,ma);
                    new += 1;
                    sender.send(ArchiveChange::New(uri));
                }
            }
        }
        let deleted = old.len();
        for o in old {
            self.delete_archive(o.id());
            sender.send(ArchiveChange::Deleted(o));
        }
        (changed,new,deleted,ret)
    }

    fn delete_archive(&mut self,id:&ArchiveId) {
        Self::delete_archive_i(&mut self.groups,id);
        // TODO delete from relational
    }
    fn delete_archive_i(groups: &mut Vec<ArchiveGroup>, id:&ArchiveId) {
        if id.is_empty() { return }
        let is_meta = id.is_meta();
        let mut steps: Vec<_> = id.steps().rev().collect();
        let mut curr = groups;
        loop {
            let step = steps.pop().unwrap();
            match curr.binary_search_by(|t| t.id().steps().last().unwrap().cmp(step)) {
                Ok(i) => {
                    if let ArchiveGroup::Archive(ref a) = curr[i] {
                        if steps.is_empty() && a == id {
                            curr.remove(i);
                            return
                        }
                    }
                    match &mut curr[i] {
                        ArchiveGroup::Group{has_meta,..} if steps.len() == 1 && is_meta => { 
                            *has_meta = false;
                            return
                        }
                        ArchiveGroup::Group{children,..} => curr = children,
                        _ => unreachable!()
                    }
                }
                Err(_) => return
            }
        }
    }
    fn add_archive(&mut self,uri:ArchiveURIRef<'_>,sender:&ChangeSender<ArchiveChange>,quads:&mut Vec<Quad>) {
        quads.push(ulo!((uri.to_iri()) : LIBRARY Q));
        let id = uri.id();
        if id.is_empty() { return }
        let is_meta = id.is_meta();
        let mut currsteps = Vec::new();
        let mut steps: Vec<_> = id.steps().rev().collect();
        let mut curr = &mut self.groups;
        let mut currgroup = None;
        loop {
            let step = steps.pop().unwrap();
            currsteps.push(step);
            match curr.binary_search_by(|t| t.id().steps().last().unwrap().cmp(step)) {
                Ok(i) => {
                    match &mut curr[i] {
                        ArchiveGroup::Group{has_meta,id,state,..} if steps.len() == 1 && is_meta => {
                            *has_meta = true;
                            quads.push(ulo!(>(format!("immt://archive-groups#{}",id)) CONTAINS (uri.to_iri()) Q));
                            return
                        }
                        ArchiveGroup::Group{children,id,..} => {
                            currgroup = Some(format!("immt://archive-groups#{}", id));
                            curr = children
                        },
                        _ => {
                            unreachable!("{:?}, {step}",id)
                        }
                    }
                }
                Err(i) if steps.is_empty() => {
                    curr.insert(i, ArchiveGroup::Archive(id.clone())); // TODO: add quad
                    if let Some(gr) = &currgroup {
                        quads.push(ulo!(>(gr) CONTAINS (uri.to_iri()) Q));
                    }
                    return
                }
                Err(i) => {
                    let has_meta = steps.len() == 1 && is_meta;
                    let joined:Arc<str> = currsteps.join("/").into();
                    quads.push(ulo!(>(format!("immt://archive-groups#{}",&joined)) : LIBRARY_GROUP Q));
                    if let Some(gr) = &currgroup {
                        quads.push(ulo!(>(gr) CONTAINS >(format!("immt://archive-groups#{}",&joined)) Q));
                    }
                    let group = ArchiveGroup::Group {
                        id: ArchiveId::new(joined.clone()),
                        has_meta, children:Vec::new(),
                        state: AllStates::default()
                    };
                    sender.send(ArchiveChange::NewGroup(group.id().clone()));
                    curr.insert(i, group);
                    if has_meta {
                        quads.push(ulo!(>(format!("immt://archive-groups#{}",&joined)) CONTAINS (uri.to_iri()) Q));
                        return
                    }
                    currgroup = Some(format!("immt://archive-groups#{}", joined));
                    curr = match &mut curr[i] {
                        ArchiveGroup::Group{children,..} => children,
                        _ => unreachable!()
                    };
                }
            }
        }
    }
}

struct ArchiveIterator<'a> {
    path:&'a Path,
    formats:&'a [SourceFormat],
    stack: Vec<Vec<(PathBuf,String)>>,
    curr:Option<std::fs::ReadDir>,
    currp:String,
    in_span:Option<&'a tracing::Span>
}

immt_core::asyncs!{fn next_dir
    (@s stack: &mut Vec<Vec<(PathBuf,String)>>,curr:&mut Option<std::fs::ReadDir>,currp:&mut String)
    (@a stack: &mut Vec<Vec<(PathBuf,String)>>,curr:&mut Option<tokio::fs::ReadDir>,currp:&mut String) -> bool {
    loop {
        match stack.last_mut() {
            None => return false,
            Some(s) => {
                match s.pop() {
                    Some((e,s)) => {
                        *curr = match read_dir!(&e) {
                            Ok(rd) => Some(rd),
                            _ => {
                                tracing::warn!(target:"archives","Could not read directory {}", e.display());
                                return false
                            }
                        };
                        *currp = s;
                        stack.push( Vec::new() );
                        return true
                    }
                    None => { stack.pop();}
                }
            }
        }
    }
}}

immt_core::asyncs!{fn find_manifest(metainf: &Path,id:&str) -> Option<PathBuf> {
    tracing::trace!("Checking for manifest");
    match read_dir!(metainf) {
        Ok(mut rd) => {
            while let Some(d) = next_file!(rd) {
                let d = match d {
                    Err(_) => {
                        tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
                        continue;
                    }
                    Ok(d) => d,
                };
                if !d.file_name().eq_ignore_ascii_case("manifest.mf") {
                    continue;
                }
                let path = d.path();
                if !path.is_file() {
                    continue;
                }
                return Some(path);
            }
            tracing::trace!("not found");
            None
        }
        _ => {
            tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
            None
        }
    }
}}

immt_core::asyncs!{fn do_manifest(path: &Path,id:&str) -> Result<MathArchiveSpec, ()> {
    let top_dir = path.parent().unwrap().parent().unwrap();
    read_lines!(lines <- path);

    let mut formats = arrayvec::ArrayVec::new();
    let mut dom_uri: String = String::new();
    let mut dependencies = Vec::new();
    let mut ignores = IgnoreSource::default();
    let mut attrs: VecMap<Box<str>,Box<str>> = VecMap::default();
    loop {
        let line = match next_line!(lines) {
            Some(Err(_)) => continue,
            Some(Ok(l)) => l,
            _ => break
        };
        let (k, v) = match line.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            _ => continue
        };
        match k {
            "id" => if v != id {
                tracing::warn!(target:"archives","Archive {v}'s id does not match its location ({id})");
                return Err(())
            }
            "format" => { formats = v.split(',').flat_map(|s| ShortId::try_from(s).map(|s| SourceFormatId::new(s))).collect() }
            "url-base" => { dom_uri = v.into() }
            "dependencies" => {
                for d in v.split(',') {
                    dependencies.push(ArchiveId::new(d))
                }
            }
            "ignore" => {
                ignores = IgnoreSource::new(v, &top_dir.join("source"));//Some(v.into());
            }
            _ => { attrs.insert(k.into(), v.into()); }
        }
    }
    let id = ArchiveId::new(id);
    let is_meta =  id.is_meta();
    if formats.is_empty() && !is_meta {
        tracing::warn!(target:"archives","No formats found for archive {}",id);
        return Err(())
    }
    if dom_uri.is_empty() {
        tracing::warn!(target:"archives","Archive {} has no URL base", id);
        return Err(())
    }
    let dom_uri = match BaseURI::new(dom_uri) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(target:"archives","Archive {} has an invalid URL base: {}", id, e);
            return Err(())
        }
    };
    dependencies.retain(|d: &ArchiveId| !d.is_empty() && d.as_str() != id.as_str());
    let spec = MathArchiveSpec {
        storage: StorageSpec {
            uri: dom_uri / id,
            is_meta,
            attributes: attrs,
            formats,
        },
        ignore_source: ignores,
        path: top_dir.into(),
    };
    Ok(spec)
}}

immt_core::asyncs!{fn next
    (@s curr:&mut Option<std::fs::ReadDir>,stack: &mut Vec<Vec<(PathBuf,String)>>,currp:&mut String)
    (@a curr:&mut Option<tokio::fs::ReadDir>,stack: &mut Vec<Vec<(PathBuf,String)>>,currp:&mut String)
    -> Option<MathArchiveSpec> {
    loop {
        let d = match match curr.as_mut() {
                None => None,
                Some(d) => next_file!(d)
            } {
            None => if switch!((next_dir(stack,curr,currp))(next_dir_async(stack,curr,currp))) { continue } else { return None },
            Some(Ok(d)) => d,
            _ => continue
        };
        let md = match wait!(d.metadata()) {
            Ok(md) => md,
            _ => continue
        };
        let path = d.path();

        //let _span = tracing::debug_span!(target:"archives","checking","{}",path.display()).entered();
        if md.is_dir() {
            if d.file_name().to_str().map_or(true, |s| s.starts_with('.')) { continue }
            else if d.file_name().eq_ignore_ascii_case("meta-inf") {
                match switch!((find_manifest(&path,currp))(find_manifest_async(&path,currp))) {
                    Some(path) => match switch!((do_manifest(&path,currp))(do_manifest_async(&path,currp).in_current_span())) {
                        Ok(m) => {
                            stack.pop();
                            if !switch!((next_dir(stack,curr,currp))(next_dir_async(stack,curr,currp))) {
                                *curr = None;
                            }
                            return Some(m)
                        }
                        _ => {
                            stack.pop();
                            if switch!((next_dir(stack,curr,currp))(next_dir_async(stack,curr,currp))) { continue } else { return None }
                        }
                    }
                    _ => ()
                }
            }
            let mut ins = currp.clone();
            if !ins.is_empty() { ins.push('/') }
            ins.push_str(d.file_name().to_str().unwrap());
            stack.last_mut().unwrap().push((path, ins))
        }
    }
}}

impl<'a> ArchiveIterator<'a> {
    
    fn new_in_span(path:&'a Path, formats:&'a [SourceFormat], span:Option<&'a tracing::Span>) -> Self {
        Self {
            formats,
            stack:vec!(vec!()),
            curr: match std::fs::read_dir(path) {
                Ok(rd) => Some(rd),
                _ => {
                    tracing::warn!(target:"archives","Could not read directory {}", path.display());
                    None
                }
            },
            path,
            currp: String::new(),
            in_span: span
        }
    }

    fn new(path:&'a Path, formats:&'a [SourceFormat]) -> Self {
        Self::new_in_span(path,formats,None)
    }
}

impl Iterator for ArchiveIterator<'_> {
    type Item = MathArchiveSpec;
    fn next(&mut self) -> Option<Self::Item> {
        let _span = self.in_span.map(|s| s.enter());
        next(&mut self.curr,&mut self.stack,&mut self.currp)
    }
}


#[cfg(feature = "rayon")]
impl spliter::Spliterator for ArchiveIterator<'_> {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 || self.stack[0].len() < 2 { return None; }
        let stacksplit = self.stack[0].len() / 2;
        let mut rightstack = self.stack[0].split_off(stacksplit);
        std::mem::swap(&mut self.stack[0], &mut rightstack);
        loop {
            match rightstack.pop() {
                None => return None,
                Some((e,s)) => if let Ok(rd) = std::fs::read_dir(&e) {
                    return Some(Self {
                        path:self.path,
                        formats:self.formats,
                        curr: Some(rd),
                        stack: vec!(rightstack,Vec::new()),
                        currp: s,
                        in_span: self.in_span
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use immt_core::building::formats::{ShortId, SourceFormatId};
    use crate::backend::manager::ArchiveIterator;
    use crate::backend::relational::RelationalManager;
    use crate::building::targets::SourceFormat;
    use crate::tests::*;
    use super::ArchiveManager;

/*
    #[rstest]
    fn manager(setup:()) {
        let stex = SourceFormat{
            id:ShortId::new("stex"),
            file_extensions:&["tex","ltx"],
            description:"",
            targets:&[],
            extension:None
        };
        let mut mgr = ArchiveManager::default();
        let relman = RelationalManager::default();
        crate::utils::time(||  relman.add_quads(|add| mgr.load(Path::new("/home/jazzpirate/work/MathHub"), &[stex],add)),
            "Loading archives synchronously"
        );
    }
*/
/*
    #[cfg(feature = "rayon")]
    #[rstest]
    fn manager_par(setup:()) {
        let stex = SourceFormat{
            id:ShortId::new("stex"),
            file_extensions:&["tex","ltx"],
            description:"",
            targets:&[],
            extension:None
        };
        let mut mgr = ArchiveManager::default();
        let relman = RelationalManager::default();
        crate::utils::time(|| relman.add_quads(mgr.load_par(Path::new("/home/jazzpirate/work/MathHub"), &[stex]).into_iter()),
            "Loading archives in parallel"
        );
    }
*/

    static STEX : [SourceFormat;1] = [SourceFormat{
        id:SourceFormatId::new(ShortId::new_unchecked("stex")),
        file_extensions:&["tex","ltx"],
        description:"",
        targets:&[],
        extension:None
    }];

    #[cfg(feature = "tokio")]
    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    async fn load_archives_async(setup:()) {
        let mut mgr = super::ArchiveManagerAsync::default();
        let relman = RelationalManager::default();
        crate::utils::time_async(
            async {relman.add_quads(mgr.load(Path::new("/home/jazzpirate/work/MathHub").to_owned(), STEX.into()).await.into_iter())},
            "Loading archives in parallel"
        ).await;
    }
}