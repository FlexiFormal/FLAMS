use std::io::BufRead;
use std::path::{Path, PathBuf};
use tracing::instrument;
use immt_core::building::formats::ShortId;
use immt_core::ontology::archives::{ArchiveGroup, MathArchiveSpec, StorageSpec};
use immt_core::uris::archives::{ArchiveId, ArchiveURI, ArchiveURIRef};
use immt_core::uris::base::BaseURI;
use immt_core::utils::ignore_regex::IgnoreSource;
use immt_core::utils::{arrayvec, VecMap};
use crate::backend::archives::{Archive, MathArchive, Storage};
use crate::building::targets::SourceFormat;
use crate::utils::asyncs::{ChangeSender, lock};
use immt_core::building::buildstate::BuildState;
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
struct ArchiveManagerI {
    archives: Vec<Archive>,
    groups: Vec<ArchiveGroup>,
}

pub struct ArchiveManager{
    lock:lock::Lock<ArchiveManagerI>,
    change_sender: ChangeSender<ArchiveChange>,
    filechange_sender: ChangeSender<FileChange>,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self {
            lock:lock::Lock::new(ArchiveManagerI {
                archives: Vec::new(),
                groups: Vec::new()
            }),
            change_sender: ChangeSender::new(256),
            filechange_sender: ChangeSender::new(256),
        }
    }
}

#[cfg(feature = "tokio")]
pub struct ArchiveManagerAsync{
    lock:tokio::sync::RwLock<ArchiveManagerI>,
    change_sender: ChangeSender<ArchiveChange>,
    filechange_sender: ChangeSender<FileChange>,
}
#[cfg(feature = "tokio")]
impl Default for ArchiveManagerAsync {
    fn default() -> Self {
        Self {
            lock:tokio::sync::RwLock::new(ArchiveManagerI {
                archives: Vec::new(),
                groups: Vec::new()
            }),
            change_sender: ChangeSender::new(256),
            filechange_sender: ChangeSender::new(256),
        }
    }
}

impl ArchiveManager {
    pub fn load(&self, path:&Path, formats:&[SourceFormat], add:&mut dyn FnMut(Quad)) {
        self.lock.write(|s| s.load(path,formats,&self.filechange_sender,&self.change_sender,add))
    }
    pub fn with_archives<R,F:FnOnce(&[Archive]) -> R>(&self,f:F) -> R {
        self.lock.read(|s| f(s.archives()) )
    }
    pub fn load_par(&self, path:&Path, formats:&[SourceFormat], add:&mut dyn FnMut(Quad)) {
        self.lock.write(|s| s.load_par(path,formats,&self.filechange_sender,&self.change_sender,add))
    }
    pub fn with_tree<R>(&self, f:impl FnOnce(&[ArchiveGroup]) -> R) -> R {
        self.lock.read(|s| f(s.groups.as_slice()))
    }
    
    pub fn find<R,S:AsRef<str>>(&self,id:S,f:impl FnOnce(Option<&Archive>) -> R) -> R {
        self.lock.read(|s| {
            let id = id.as_ref();
            if let Ok(i) = s.archives.binary_search_by_key(&id,|a| a.uri().id().as_str()) {
                f(Some(&s.archives[i]))
            } else {f(None)}
        })
    }
}

#[cfg(feature = "tokio")]
impl ArchiveManagerAsync {
    pub async fn load(&self, path:&Path, formats:&'static[SourceFormat], add:&mut dyn FnMut(Quad)) {
        let mut lock = self.lock.write().await;
        lock.load_async(path,formats,&self.filechange_sender,&self.change_sender,add).await
    }
    pub async fn with_archives<R,F:std::future::Future<Output=R>>(&self,f:impl FnOnce(&[Archive]) -> F) -> R {
        let mut lock = self.lock.read().await;
        f(lock.archives()).await
    }
    pub async fn with_tree<R,F:std::future::Future<Output=R>>(&self, f:impl FnOnce(&[ArchiveGroup]) -> F) -> R {
        let mut lock = self.lock.read().await;
        f(lock.groups.as_slice()).await
    }

    pub async fn find<R,S:AsRef<str>,F:std::future::Future<Output=R>>(&self,id:S,f:impl FnOnce(Option<&Archive>) -> F) -> R {
        let mut lock = self.lock.read().await;
        let id = id.as_ref();
        if let Ok(i) = lock.archives.binary_search_by_key(&id,|a| a.uri().id().as_str()) {
            f(Some(&lock.archives[i])).await
        } else {f(None).await}
    }
}

impl ArchiveManagerI {
    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    fn load(&mut self, path:&Path, formats:&[SourceFormat], fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>, add:&mut dyn FnMut(Quad)) {
        tracing::info!(target:"archives","Searching for archives");
        let (changed,new,deleted) = self.do_specs(ArchiveIterator::new(path,formats),asender,add);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");
        for a in self.archives.iter_mut().filter_map(|a| match a {
            Archive::Physical(a) => Some(a),
            _ => None
        }) {
            a.update_sources(formats, fsender);
        }
    }
    
    fn archives(&self) -> &[Archive] { &self.archives }

    #[cfg(feature = "rayon")]
    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    fn load_par(&mut self, path:&Path, formats:&[SourceFormat],fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>, add:&mut dyn FnMut(Quad)) {
        use spliter::ParallelSpliterator;
        use rayon::iter::*;
        tracing::info!(target:"archives","Searching for archives");
        let span = tracing::Span::current();
        
        let news = ArchiveIterator::new_in_span(path,formats,Some(&span)).par_split().into_par_iter().collect::<Vec<_>>();
        let (changed,new,deleted) = self.do_specs(news.into_iter(),asender,add);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");
        self.archives.par_iter_mut().filter_map(|a| match a {
            Archive::Physical(a) => Some(a),
            _ => None
        }).for_each(|a| { a.update_sources(formats,fsender); })
    }

    #[cfg(feature = "tokio")]
    #[instrument(level = "info",
        target = "archives",
        name = "Loading archives",
        fields(path = %path.display()),
        skip_all
    )]
    async fn load_async(&mut self, path:&Path, formats:&'static[SourceFormat],fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>, add:&mut dyn FnMut(Quad)) {
        tracing::info!(target:"archives","Searching for archives");
        let mut stack = vec!(vec!());
        let mut curr = match tokio::fs::read_dir(path).await {
            Ok(rd) => Some(rd),
            _ => {
                tracing::warn!(target:"archives","Could not read directory {}", path.display());
                return
            }
        };
        let mut currp = String::new();
        let mut ret = vec!();
        while let Some(m) = next_async(&mut curr,&mut stack,&mut currp).await {
            ret.push(m);
        }
        let (changed,new,deleted) = self.do_specs(ret.into_iter(),asender,add);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");
        let mut js = tokio::task::JoinSet::new();
        for mut a in std::mem::take(&mut self.archives) {
            if let Archive::Physical(mut a) = a {
                let sender = fsender.clone();
                js.spawn(async move { a.update_sources_async(formats,&sender).await; a});
            } else {
                self.archives.push(a);
            }
        }
        while let Some(a) = js.join_next().await {
            if let Ok(a) = a {self.archives.push(Archive::Physical(a))}
        }
    }

    fn do_specs<F:Iterator<Item=MathArchiveSpec>>(&mut self,iter:F,sender:&ChangeSender<ArchiveChange>,add:&mut dyn FnMut(Quad)) -> (usize,usize,usize) {
        let mut old : Vec<_> = self.archives.iter().map(|a| a.uri().to_owned()).collect();
        let mut changed = 0; let mut new = 0;
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
                    self.add_archive(uri.as_ref(),sender,add);
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
        (changed,new,deleted)
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
    fn add_archive(&mut self,uri:ArchiveURIRef<'_>,sender:&ChangeSender<ArchiveChange>,add:&mut dyn FnMut(Quad)) {
        add(ulo!((uri.to_iri()) : LIBRARY Q));
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
                        ArchiveGroup::Group{has_meta,id,..} if steps.len() == 1 && is_meta => {
                            *has_meta = true;
                            add(ulo!(>(format!("immt://archive-groups#{}",id)) CONTAINS (uri.to_iri()) Q));
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
                        add(ulo!(>(gr) CONTAINS (uri.to_iri()) Q));
                    }
                    return
                }
                Err(i) => {
                    let has_meta = steps.len() == 1 && is_meta;
                    let joined:Arc<str> = currsteps.join("/").into();
                    add(ulo!(>(format!("immt://archive-groups#{}",&joined)) : LIBRARY_GROUP Q));
                    if let Some(gr) = &currgroup {
                        add(ulo!(>(gr) CONTAINS >(format!("immt://archive-groups#{}",&joined)) Q));
                    }
                    let group = ArchiveGroup::Group {
                        id: ArchiveId::new(joined.clone()),
                        has_meta, children:Vec::new()
                    };
                    sender.send(ArchiveChange::NewGroup(group.id().clone()));
                    curr.insert(i, group);
                    if has_meta {
                        add(ulo!(>(format!("immt://archive-groups#{}",&joined)) CONTAINS (uri.to_iri()) Q));
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

immt_core::asyncs!{fn get_manifest(metainf: &Path,id:&str) -> Option<Result<MathArchiveSpec, ()>> {
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
                return Some(switch!((do_manifest(&path,id))(do_manifest_async(&path,id))));
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
            "format" => { formats = v.split(',').flat_map(ShortId::try_from).collect() }
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
                match get_manifest(&path,currp) {
                    Some(Ok(m)) => {
                        stack.pop();
                        if !switch!((next_dir(stack,curr,currp))(next_dir_async(stack,curr,currp))) {
                            *curr = None;
                        }
                        return Some(m)
                    }
                    Some(_) => {
                        stack.pop();
                        if switch!((next_dir(stack,curr,currp))(next_dir_async(stack,curr,currp))) { continue } else { return None }
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


#[cfg(feature = "tokio")]
pub struct ArchiveLoaderAsync<'a,Add:FnMut(MathArchiveSpec) -> bool> {
    found:usize,
    mh:&'a Path,
    formats:&'a [SourceFormat],
    add:Add,
    stack: Vec<Vec<(PathBuf,String)>>,
    curr:tokio::fs::ReadDir,
    currp:String
}
#[cfg(feature = "tokio")]
impl<'a,Add:FnMut(MathArchiveSpec) -> bool> ArchiveLoaderAsync<'a,Add> {
    #[instrument(level = "info",
    target = "backend",
    name = "Loading archives",
    skip(formats,add),
    fields(found)
    )]
    pub async fn load(mh:&'a Path, formats:&'a [SourceFormat], add:Add) {
        tracing::info!("Searching for archives");
        let mut loader = Self {
            found:0,
            formats,
            add,
            stack:vec!(vec!()),
            curr: match tokio::fs::read_dir(mh).await {
                Ok(rd) => rd,
                _ => {
                    tracing::warn!(target:"archives","Could not read directory {}", mh.display());
                    return
                }
            },
            mh,
            currp: String::new()
        };
        loader.run().await;
        tracing::Span::current().record("found", loader.found);
        tracing::info!("Done");
    }
    
    async fn next(&mut self) -> bool {
        loop {
            match self.stack.last_mut() {
                None => return false,
                Some(s) => {
                    match s.pop() {
                        Some((e,s)) => {
                            self.curr = match tokio::fs::read_dir(&e).await {
                                Ok(rd) => rd,
                                _ => {
                                    tracing::warn!(target:"archives","Could not read directory {}", e.display());
                                    return false
                                }
                            };
                            self.currp = s;
                            self.stack.push( Vec::new() );
                            return true
                        }
                        None => { self.stack.pop();}
                    }
                }
            }
        }
    }

    async fn run(&mut self) {
        loop {
            let d = match self.curr.next_entry().await {
                Ok(None) => if self.next().await {continue} else {return },
                Ok(Some(d)) => d,
                _ => continue
            };
            let md = match d.metadata().await {
                Ok(md) => md,
                _ => continue
            };
            let path = d.path();
            let _span = tracing::trace_span!("checking","{}",path.display()).entered();
            if md.is_dir() {
                if d.file_name().to_str().map_or(true, |s| s.starts_with('.')) { continue }
                else if d.file_name().eq_ignore_ascii_case("meta-inf") {
                    match self.get_manifest(&path,&self.currp).await {
                        Some(Ok(m)) => {
                            if (self.add)(m) {
                                self.found += 1
                            }
                            self.stack.pop();
                            if self.next().await {continue} else {return }
                        }
                        Some(_) => {
                            self.stack.pop();
                            if self.next().await {continue} else {return }
                        }
                        _ => ()
                    }
                }
                let mut ins = self.currp.clone();
                if !ins.is_empty() { ins.push('/') }
                ins.push_str(d.file_name().to_str().unwrap());
                self.stack.last_mut().unwrap().push((path, ins))
            }
        }
    }

    async fn get_manifest(&self, metainf: &Path,id:&str) -> Option<Result<MathArchiveSpec, ()>> {
        tracing::trace!("Checking for manifest");
        match tokio::fs::read_dir(metainf).await {
            Ok(mut rd) => {
                loop {
                    let d = match rd.next_entry().await {
                        Err(_) => {
                            tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
                            continue;
                        }
                        Ok(Some(d)) => d,
                        Ok(None) => break
                    };
                    if !d.file_name().eq_ignore_ascii_case("manifest.mf") {
                        continue;
                    }
                    let path = d.path();
                    if !path.is_file() {
                        continue;
                    }
                    return Some(self.do_manifest(&path,id).await);
                }
                tracing::trace!("not found");
                None
            }
            _ => {
                tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
                None
            }
        }
    }

    async fn do_manifest(&self, path: &Path,id:&str) -> Result<MathArchiveSpec, ()> {
        use tokio::io::AsyncBufReadExt;
        let top_dir = path.parent().unwrap().parent().unwrap();
        let reader = tokio::io::BufReader::new(tokio::fs::File::open(path).await.unwrap());
        let mut lines = reader.lines();

        let mut formats = arrayvec::ArrayVec::new();
        let mut dom_uri: String = String::new();
        let mut dependencies = Vec::new();
        let mut ignores = IgnoreSource::default();
        let mut attrs: VecMap<Box<str>,Box<str>> = VecMap::default();
        loop {
            let line = match lines.next_line().await {
                Err(_) => continue,
                Ok(Some(l)) => l,
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
                "format" => { formats = v.split(',').flat_map(ShortId::try_from).collect() }
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
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use immt_core::building::formats::ShortId;
    use crate::backend::manager::ArchiveIterator;
    use crate::backend::relational::RelationalManager;
    use crate::building::targets::SourceFormat;
    use crate::tests::*;
    use super::ArchiveManager;
    
    fn time<A,F:FnOnce() -> A>(f:F,s:&str) -> A {
        let start = std::time::Instant::now();
        let ret = f();
        let dur = start.elapsed();
        tracing::info!("{s} took {:?}", dur);
        ret
    }
    #[cfg(feature="tokio")]
    async fn time_async<A>(f:impl std::future::Future<Output=A>,s:&str) -> A {
        let start = std::time::Instant::now();
        let ret = f.await;
        let dur = start.elapsed();
        tracing::info!("{s} took {:?}", dur);
        ret
    }
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
        time(||  relman.add_quads(|add| mgr.load(Path::new("/home/jazzpirate/work/MathHub"), &[stex],add)),
            "Loading archives synchronously"
        );
    }

 */

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
        time(|| relman.add_quads(|add| mgr.load_par(Path::new("/home/jazzpirate/work/MathHub"), &[stex],add)),
            "Loading archives in parallel"
        );
    }
/*
    #[cfg(feature = "tokio")]
    #[rstest]
    #[tokio::test]
    async fn load_archives_async(setup:()) {
        let stex = SourceFormat{
            id:ShortId::new("stex"),
            file_extensions:&["tex","ltx"],
            description:"",
            targets:&[],
            extension:None
        };
        let mut mgr = super::ArchiveManagerAsync::default();
        let relman = RelationalManager::default();
        time_async(
            relman.add_quads_async(|add| mgr.load(Path::new("/home/jazzpirate/work/MathHub"), &[stex], add))
        , "Loading archives in parallel"
        ).await;
    }

 */
}