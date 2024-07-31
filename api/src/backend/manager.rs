use std::io::BufRead;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Output;
use futures::TryFutureExt;
use tracing::{instrument, Instrument};
use immt_core::building::formats::{ShortId, SourceFormatId};
use immt_core::ontology::archives::{ArchiveGroup, MathArchiveSpec, StorageSpec};
use immt_core::uris::archives::{ArchiveId, ArchiveURI};
use immt_core::uris::base::BaseURI;
use immt_core::utils::ignore_regex::IgnoreSource;
use immt_core::utils::{arrayvec, VecMap};
use crate::backend::archives::{Archive, MathArchive, Storage};
use crate::building::targets::SourceFormat;
use crate::utils::asyncs::{ChangeSender, lock};
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::narration::{CSS, Document, HTMLDocSpec};
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
    pub fn with_tree<R>(&self, f:impl FnOnce(&ArchiveTree) -> R) -> R {
        self.lock.read(|s| f(&*s))
    }

    pub fn get_tree(&self) -> impl Deref<Target=ArchiveTree> + '_ {
        self.lock.returnable()
    }

    pub fn find<R>(&self,id:ArchiveId,f:impl FnOnce(Option<&Archive>) -> R) -> R {
        self.lock.read(|s| {
            if let Some(a) = ArchiveTree::find_i(s.archives.as_slice(), id) {
                f(Some(a))
            } else {f(None)}
        })
    }

    pub fn get_document(&self,id:ArchiveId,rel_path:impl AsRef<str>) -> Option<Document> {
        self.find(id, |a| match a {
            Some(Archive::Physical(ma)) => {
                let p = ma.out_dir().join(rel_path.as_ref()).join("index.nomd");
                if p.exists() {
                    HTMLDocSpec::get_doc(&p)
                } else { None }
            }
            _ => None
        })
    }

    pub async fn get_document_async(&self,id:ArchiveId,rel_path:impl AsRef<str>) -> Option<Document> {
        let p = self.find(id, |a| match a {
            Some(Archive::Physical(ma)) => {
                Some(ma.out_dir().join(rel_path.as_ref()).join("index.nomd"))
            }
            _ => None
        })?;
        if p.exists() {
            HTMLDocSpec::get_doc_async(&p).await
        } else { None }
    }

    pub async fn get_html_async(&self,id:ArchiveId,rel_path:impl AsRef<str>) -> Option<(Vec<CSS>,String)> {
        let p = self.find(id, |a| match a {
            Some(Archive::Physical(ma)) => {
                Some(ma.out_dir().join(rel_path.as_ref()).join("index.nomd"))
            }
            _ => None
        })?;
        if p.exists() {
            HTMLDocSpec::get_css_and_body_async(&p).await
        } else { None }
    }
}

impl ArchiveTree {
    pub fn archives(&self) -> &[Archive] { &self.archives }
    pub fn groups(&self) -> &[ArchiveGroup] { &self.groups }
    pub fn find_archive(&self,id:ArchiveId) -> Option<&Archive> {
        Self::find_i(self.archives.as_slice(), id)
    }
    pub fn find_group_or_archive(&self,prefix:ArchiveId) -> Option<&ArchiveGroup> {
        let mut ls = self.groups.as_slice();
        let mut ret = None;
        for step in prefix.steps() {
            let i = ls.binary_search_by_key(&step,|v| v.id().last_name())
                .ok()?;
            let elem = &ls[i];
            //println!("Here: {elem:?}");
            ret = Some(elem);
            ls = match elem {
                ArchiveGroup::Group { children, .. } => children.as_slice(),
                ArchiveGroup::Archive(..) => &[]
            }
        }
        ret
    }
    fn find_i<'a>(archives:&'a [Archive], id:ArchiveId) -> Option<&'a Archive> {
        archives.iter().find(|a| a.id() == id)
        //archives.binary_search_by_key(&id.as_str(),|a| a.uri().id().as_str()).ok().map(|i| &archives[i])
    }


    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    fn load(&mut self, path:&Path, formats:&[SourceFormat],fsender:&ChangeSender<FileChange>, asender:&ChangeSender<ArchiveChange>) -> Vec<Quad> {
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
        let (changed,new,deleted,quads) = self.do_specs(news,asender);
        tracing::info!(target:"archives","Done; {new} new, {changed} changed, {deleted} deleted");

        for g in &mut self.groups {
            g.update(&|id| Self::find_i(self.archives.as_slice(), id).and_then(|a|
                if let Archive::Physical(ma) = a {
                    Some(ma.state())
                } else {None }
            ));
        }
        quads
    }

    fn do_specs(&mut self,iter:Vec<MathArchive>,sender:&ChangeSender<ArchiveChange>) -> (usize,usize,usize,Vec<Quad>) {
        let mut old : Vec<_> = self.archives.iter().filter_map(|a| match a {
            Archive::Physical(a) => Some(a.uri().to_owned()),
            _ => None
        }).collect();
        let mut changed = 0; let mut new = 0;
        let mut ret = vec!();
        for spec in iter {
            match self.archives.binary_search_by(|a| a.uri().id().cmp(&spec.id())) {
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
                    self.add_archive(uri,sender,&mut ret);
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


    fn delete_archive(&mut self,id:ArchiveId) {
        Self::delete_archive_i(&mut self.groups,id);
        // TODO delete from relational
    }
    fn delete_archive_i(groups: &mut Vec<ArchiveGroup>, id:ArchiveId) {
        if id.is_empty() { return }
        let is_meta = id.is_meta();
        let mut steps: Vec<_> = id.steps().rev().collect();
        let mut curr = groups;
        loop {
            let step = steps.pop().unwrap();
            match curr.binary_search_by(|t| t.id().steps().last().unwrap().cmp(step)) {
                Ok(i) => {
                    if let ArchiveGroup::Archive(a) = curr[i] {
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
    fn add_archive(&mut self,uri:ArchiveURI,sender:&ChangeSender<ArchiveChange>,quads:&mut Vec<Quad>) {
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

fn next_dir(stack: &mut Vec<Vec<(PathBuf,String)>>,curr:&mut Option<std::fs::ReadDir>,currp:&mut String) -> bool {
    loop {
        match stack.last_mut() {
            None => return false,
            Some(s) => {
                match s.pop() {
                    Some((e,s)) => {
                        *curr = match e.read_dir() {
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
}

fn find_manifest(metainf: &Path,id:&str) -> Option<PathBuf> {
    tracing::trace!("Checking for manifest");
    match metainf.read_dir() {
        Ok(mut rd) => {
            while let Some(d) = rd.next() {
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
}

fn do_manifest(path: &Path,id:&str) -> Result<MathArchiveSpec, ()> {
    let top_dir = path.parent().unwrap().parent().unwrap();

    let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let mut lines = reader.lines();

    let mut formats = arrayvec::ArrayVec::new();
    let mut dom_uri: String = String::new();
    let mut dependencies = Vec::new();
    let mut ignores = IgnoreSource::default();
    let mut attrs: VecMap<Box<str>,Box<str>> = VecMap::default();
    loop {
        let line = match lines.next() {
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
}

fn next(curr:&mut Option<std::fs::ReadDir>,stack: &mut Vec<Vec<(PathBuf,String)>>,currp:&mut String) -> Option<MathArchiveSpec> {
    loop {
        let d = match match curr.as_mut() {
                None => None,
                Some(d) => d.next()
            } {
            None => if next_dir(stack,curr,currp) { continue } else { return None },
            Some(Ok(d)) => d,
            _ => continue
        };
        let md = match d.metadata() {
            Ok(md) => md,
            _ => continue
        };
        let path = d.path();

        //let _span = tracing::debug_span!(target:"archives","checking","{}",path.display()).entered();
        if md.is_dir() {
            if d.file_name().to_str().map_or(true, |s| s.starts_with('.')) { continue }
            else if d.file_name().eq_ignore_ascii_case("meta-inf") {
                match find_manifest(&path,currp) {
                    Some(path) => match do_manifest(&path,currp) {
                        Ok(m) => {
                            stack.pop();
                            if !next_dir(stack,curr,currp) {
                                *curr = None;
                            }
                            return Some(m)
                        }
                        _ => {
                            stack.pop();
                            if next_dir(stack,curr,currp) { continue } else { return None }
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
}

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

    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    async fn manager_par(setup:()) {
        let mut mgr = ArchiveManager::default();
        let relman = RelationalManager::default();
        crate::utils::time(|| relman.add_quads(mgr.load(Path::new("/home/jazzpirate/work/MathHub"), &STEX).into_iter()),
            "Loading archives in parallel"
        );
    }


    static STEX : [SourceFormat;1] = [SourceFormat{
        id:SourceFormatId::new(ShortId::new_unchecked("stex")),
        file_extensions:&["tex","ltx"],
        description:"",
        targets:&[],
        extension:None
    }];
/*
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

 */
}