pub mod archives;
mod cache;
mod docfile;
pub mod rdf;

use archives::{manager::ArchiveManager, source_files::FileState, Archive, ArchiveOrGroup, ArchiveTree, LocalArchive};
use cache::BackendCache;
use docfile::PreDocFile;
use immt_ontology::{
    content::{
        checking::ModuleChecker, declarations::{Declaration, DeclarationTrait, OpenDeclaration}, modules::Module, terms::Term, ContentReference, ModuleLike
    }, languages::Language, narration::{
        checking::DocumentChecker, documents::Document, notations::{Notation, PresentationError, Presenter}, DocumentElement, LazyDocRef, NarrationTrait, NarrativeReference
    }, uris::{
        ArchiveId, ArchiveURI, ArchiveURITrait, ContentURITrait, DocumentElementURI, DocumentURI, ModuleURI, NameStep, PathURIRef, PathURITrait, SymbolURI, URIOrRefTrait, URIWithLanguage
    }, Checked, DocumentRange, LocalBackend, Unchecked
};
use immt_utils::{prelude::HMap, triomphe, vecmap::VecMap, CSS};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use rdf::RDFStore;
use std::{ops::Deref, path::{Path, PathBuf}, rc::Rc};
use crate::formats::{HTMLData, SourceFormatId};

#[derive(Clone, Debug)]
pub enum BackendChange {
    NewArchive(ArchiveURI),
    ArchiveUpdate(ArchiveURI),
    ArchiveDeleted(ArchiveURI),
    FileChange {
        archive: ArchiveURI,
        relative_path: String,
        format: SourceFormatId,
        old: Option<FileState>,
        new: FileState,
    },
}

#[derive(Clone,Debug)]
pub enum AnyBackend{
    Global(&'static GlobalBackend),
    Temp(TemporaryBackend)
}

pub trait Backend {
    type ArchiveIter<'a> : Iterator<Item=&'a Archive> where Self:Sized;

    #[inline]
    fn presenter(&self) -> StringPresenter<'_,Self> where Self:Sized {
        StringPresenter::new(self,false)
    }

    fn to_any(&self) -> AnyBackend;
    fn get_document(&self, uri: &DocumentURI) -> Option<Document>;
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike>;
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf>;
    fn get_declaration<T: DeclarationTrait>(&self, uri: &SymbolURI) -> Option<ContentReference<T>>
    where Self: Sized {
            let m = self.get_module(uri.module())?;
        // TODO this unnecessarily clones
        ContentReference::new(&m, uri.name())
    }    
    fn get_document_element<T: NarrationTrait>(&self, uri: &DocumentElementURI) -> Option<NarrativeReference<T>>
    where Self: Sized {
            let d = self.get_document(uri.document())?;
        // TODO this unnecessarily clones
        NarrativeReference::new(&d, uri.name())
    }
    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R
    where
        Self: Sized;

    #[allow(irrefutable_let_patterns)]
    fn archive_of<R>(&self,p:&Path,mut f:impl FnMut(&LocalArchive,&str) -> R) -> Option<R> where Self:Sized {
        let base = p.as_os_str().to_str()?;
        self.with_archives(|mut a| a.find_map(|a| {
            let Archive::Local(a) = a else {return None};
            let ap = a.path().as_os_str().to_str()?;
            base.strip_prefix(ap).map(|rp| f(a,rp))
        }))
    }

    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized;

    //fn with_archive_tree<R>(&self,f:impl FnOnce(&ArchiveTree) -> R) -> R where Self:Sized;

    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=immt_ontology::rdf::Triple>)
        where Self:Sized;
    
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    where Self:Sized;

    fn get_html_body(&self,
        d:&DocumentURI,full:bool
    ) -> Option<(Vec<CSS>,String)>;

    fn get_html_fragment(&self,
        d:&DocumentURI,range:DocumentRange
    ) -> Option<(Vec<CSS>,String)>;

    fn get_reference<T:immt_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T>
    where Self:Sized;
    
    #[allow(unreachable_patterns)]
    fn with_local_archive<R>(
        &self,
        id: &ArchiveId,
        f: impl FnOnce(Option<&LocalArchive>) -> R,
    ) -> R  where Self:Sized {
        self.with_archive(id, |a| {
            f(a.and_then(|a| match a {
                Archive::Local(a) => Some(a),
                _ => None,
            }))
        })
    }

    /*fn get_archive_for_path(p:&Path) -> Option<(ArchiveURI,String)> {

    }*/

    #[inline]
    fn as_checker(&self) -> AsChecker<Self> where Self:Sized {
        AsChecker(self)
    }
}

impl Backend for AnyBackend {
    type ArchiveIter<'a> = std::slice::Iter<'a,Archive>;
    #[inline]
    fn to_any(&self) -> AnyBackend {
        self.clone()
    }

    #[inline]
    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        match self {
            Self::Global(b) => b.with_archives(f),
            Self::Temp(b) => b.with_archives(f),
        }
    }

    #[inline]
    fn get_reference<T:immt_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        match self {
            Self::Global(b) => b.get_reference(rf),
            Self::Temp(b) => b.get_reference(rf),
        }
    }

    #[inline]
    fn get_html_body(&self,
            d:&DocumentURI,full:bool
        ) -> Option<(Vec<CSS>,String)> {
        match self {
            Self::Global(b) => b.get_html_body(d,full),
            Self::Temp(b) => b.get_html_body(d,full),
        }
    }

    #[inline]
    fn get_html_fragment(&self,
            d:&DocumentURI,range:DocumentRange
        ) -> Option<(Vec<CSS>,String)> {
        match self {
            Self::Global(b) => b.get_html_fragment(d,range),
            Self::Temp(b) => b.get_html_fragment(d,range),
        }
    }

    #[inline]
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=immt_ontology::rdf::Triple>) {
        match self {
            Self::Global(b) => b.submit_triples(in_doc,rel_path,iter),
            Self::Temp(b) => b.submit_triples(in_doc,rel_path,iter),
        }
    }

    #[inline]
    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        match self {
            Self::Global(b) => b.get_document(uri),
            Self::Temp(b) => b.get_document(uri),
        }
    }

    #[inline]
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        match self {
            Self::Global(b) => b.get_module(uri),
            Self::Temp(b) => b.get_module(uri),
        }
    }

    #[inline]
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        match self {
            Self::Global(b) => b.get_base_path(id),
            Self::Temp(b) => b.get_base_path(id),
        }
    }

    #[inline]
    fn get_declaration<T: DeclarationTrait>(&self, uri: &SymbolURI) -> Option<ContentReference<T>>
    where Self: Sized {
        match self {
            Self::Global(b) => b.get_declaration(uri),
            Self::Temp(b) => b.get_declaration(uri),
        }
    }

    #[inline]
    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R
    where Self: Sized {
        match self {
            Self::Global(b) => b.with_archive_or_group(id,f),
            Self::Temp(b) => b.with_archive_or_group(id,f),
        }
    }
    
    #[inline]
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    where Self:Sized {
        match self {
            Self::Global(b) => b.with_archive(id, f),
            Self::Temp(b) => b.with_archive(id, f),
        }
    }
    
    #[inline]
    fn with_local_archive<R>(
        &self,
        id: &ArchiveId,
        f: impl FnOnce(Option<&LocalArchive>) -> R,
    ) -> R where Self:Sized {
        match self {
            Self::Global(b) => b.with_local_archive(id, f),
            Self::Temp(b) => b.with_local_archive(id, f),
        }
    }
}

#[derive(Debug)]
pub struct GlobalBackend {
    archives: ArchiveManager,
    cache: RwLock<cache::BackendCache>,
    triple_store: RDFStore,
}

lazy_static! {
    static ref GLOBAL: GlobalBackend = GlobalBackend {
        archives: ArchiveManager::default(),
        cache: RwLock::new(cache::BackendCache::default()),
        triple_store: RDFStore::default()
    };
}

impl GlobalBackend {
    #[inline]
    #[must_use]
    pub fn get() -> &'static Self
    where
        Self: Sized,
    {
        &GLOBAL
    }

    #[inline]
    pub fn with_archive_tree<R>(&self,f:impl FnOnce(&ArchiveTree) -> R) -> R {
        self.archives.with_tree(f)
    }

    #[cfg(feature="tokio")]
    pub async fn get_html_body_async(&self,
        d:&DocumentURI,full:bool
    ) -> Option<(Vec<CSS>,String)> {
        let f = self.manager().with_archive(d.archive_id(), move |a|
            a.map(move |a| a.load_html_body_async(d.path(), d.name().first_name(), d.language(),full))
        )??;
        f.await
    }

    #[cfg(feature="tokio")]
    pub async fn get_html_fragment_async(&self,
        d:&DocumentURI,range:DocumentRange
    ) -> Option<(Vec<CSS>,String)> {
        let f = self.manager().with_archive(d.archive_id(), move |a|
            a.map(move |a| a.load_html_fragment_async(d.path(), d.name().first_name(), d.language(),range))
        )??;
        f.await
    }

    #[inline]
    pub const fn manager(&self) -> &ArchiveManager {&self.archives}

    #[inline]
    pub const fn triple_store(&self) -> &RDFStore { &self.triple_store }

    #[inline]
    pub fn all_archives(&self) -> impl Deref<Target = [Archive]> + '_ {
        self.archives.all_archives()
    }

    #[cfg(feature = "tokio")]
    #[allow(clippy::similar_names)]
    #[allow(clippy::significant_drop_tightening)]
    pub async fn get_document_async(&self, uri: &DocumentURI) -> Option<Document> {
        {
            let lock = self.cache.read();
            if let Some(doc) = lock.has_document(uri) {
                return Some(doc.clone());
            }
        }
        let uri = uri.clone();
        tokio::task::spawn_blocking(move || {
                let slf = Self::get();
                let mut cache = slf.cache.write();
                let mut flattener = GlobalFlattener(&mut cache, &slf.archives);
                flattener.load_document(uri.as_path(), uri.language(), uri.name().first_name())
            })
            .await
            .ok()
            .flatten()
    }

    #[cfg(feature = "tokio")]
    #[allow(clippy::similar_names)]
    #[allow(clippy::significant_drop_tightening)]
    pub async fn get_module_async(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        {
            let lock = self.cache.read();
            if uri.name().is_simple() {
                if let Some(m) = lock.has_module(uri) {
                    return Some(ModuleLike::Module(m.clone()));
                }
            } else {
                let top_uri = !uri.clone();
                if let Some(m) = lock.has_module(&top_uri) {
                    return ModuleLike::in_module(m, uri.name());
                }
            }
        }

        let top = !uri.clone();
        let m = tokio::task::spawn_blocking(move || {
                let slf = Self::get();
                let mut cache = slf.cache.write();
                let mut flattener = GlobalFlattener(&mut cache, &slf.archives);
                flattener.load_module(top.as_path(), top.language(), top.name().first_name())
            })
            .await
            .ok()??;
        ModuleLike::in_module(&m, uri.name())
    }

    #[cfg(feature = "tokio")]
    pub async fn get_declaration_async<T: DeclarationTrait>(&self, uri: &SymbolURI) -> Option<ContentReference<T>> {
        let m = self.get_module_async(uri.module()).await?;
        // TODO this unnecessarily clones
        ContentReference::new(&m, uri.name())
    }    
    
    #[cfg(feature = "tokio")]
    pub async fn get_document_element_async<T: NarrationTrait>(&self, uri: &DocumentElementURI) -> Option<NarrativeReference<T>> {
        let d = self.get_document_async(uri.document()).await?;
        // TODO this unnecessarily clones
        NarrativeReference::new(&d, uri.name())
    }
}

impl Backend for &'static GlobalBackend {
    type ArchiveIter<'a> = std::slice::Iter<'a,Archive>;

    #[inline]
    fn to_any(&self) -> AnyBackend {
        AnyBackend::Global(self)
    }

    #[inline]
    fn get_html_body(&self,
            d:&DocumentURI,full:bool
        ) -> Option<(Vec<CSS>,String)> {
        GlobalBackend::get_html_body(self, d, full)
    }

    #[inline]
    fn get_html_fragment(&self,
            d:&DocumentURI,range:DocumentRange
        ) -> Option<(Vec<CSS>,String)> {
        GlobalBackend::get_html_fragment(self, d, range)
    }

    #[inline]
    fn get_reference<T:immt_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        GlobalBackend::get_reference(self,rf)
    }

    #[inline]
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=immt_ontology::rdf::Triple>) {
        GlobalBackend::submit_triples(self,in_doc,rel_path,iter);
    }

    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        GlobalBackend::with_archives(self,f)
    }

    #[inline]
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    {
        GlobalBackend::with_archive(self, id,f)
    }

    #[inline]
    fn with_local_archive<R>(
        &self,
        id: &ArchiveId,
        f: impl FnOnce(Option<&LocalArchive>) -> R,
    ) -> R {
        GlobalBackend::with_local_archive(self, id,f)
    }
    #[inline]
    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R {
        GlobalBackend::with_archive_or_group(self, id,f)
    }
    #[inline]
    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        GlobalBackend::get_document(self, uri)
    }
    #[inline]
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        GlobalBackend::get_module(self, uri)
    }
    #[inline]
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        GlobalBackend::get_base_path(self, id)
    }
    #[inline]
    fn get_declaration<T: DeclarationTrait>(&self, uri: &SymbolURI) -> Option<ContentReference<T>> {
        GlobalBackend::get_declaration(self, uri)
    }
}

impl Backend for GlobalBackend {
    type ArchiveIter<'a> = std::slice::Iter<'a,Archive>;

    #[inline]
    fn to_any(&self) -> AnyBackend {
        AnyBackend::Global(Self::get())
    }

    fn get_html_fragment(&self,
        d:&DocumentURI,range:DocumentRange
    ) -> Option<(Vec<CSS>,String)> {
        self.manager().with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_fragment(d.path(), d.name().first_name(), d.language(),range))
        )
    }

    fn get_reference<T:immt_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        self.manager().with_archive(rf.in_doc.archive_id(),|a|
            a.and_then(|a| a.load_reference(rf.in_doc.path(), rf.in_doc.name().first_name(), rf.in_doc.language(),DocumentRange {start:rf.start, end:rf.end}))
        )
    }

    #[inline]
    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        self.archives.with_tree(|t| f(t.archives.iter()))
    }

    fn get_html_body(&self,
        d:&DocumentURI,full:bool
    ) -> Option<(Vec<CSS>,String)> {
        self.manager().with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_body(d.path(), d.name().first_name(), d.language(),full))
        )
    }

    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=immt_ontology::rdf::Triple>) {
        self.manager().with_archive(in_doc.archive_id(), |a| {
            if let Some(a) = a {
                a.submit_triples(in_doc,rel_path,self.triple_store(),iter);
            }
        });
    }

    #[inline]
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    {
        let archives = &*self.all_archives();
        f(archives.iter().find(|a| a.uri().archive_id() == id))
    }

    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R {
        self.with_archive_tree(|t| f(t.find(id)))
    }
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        self.with_local_archive(id, |a| a.map(|a| a.path().to_path_buf()))
    }

    #[allow(clippy::significant_drop_tightening)]
    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        {
            let lock = self.cache.read();
            if let Some(doc) = lock.has_document(uri) {
                return Some(doc.clone());
            }
        }
        let mut cache = self.cache.write();
        let mut flattener = GlobalFlattener(&mut cache, &self.archives);
        flattener.load_document(uri.as_path(), uri.language(), uri.name().first_name())
    }

    #[allow(clippy::significant_drop_tightening)]
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        {
            let lock = self.cache.read();
            if uri.name().is_simple() {
                if let Some(m) = lock.has_module(uri) {
                    return Some(ModuleLike::Module(m.clone()));
                }
            } else {
                let top_uri = !uri.clone();
                if let Some(m) = lock.has_module(&top_uri) {
                    return ModuleLike::in_module(m, uri.name());
                }
            }
        }
        let m = {
            let mut cache = self.cache.write();
            let mut flattener = GlobalFlattener(&mut cache, &self.archives);
            flattener.load_module(uri.as_path(), uri.language(), uri.name().first_name())?
        };
        // TODO: this unnecessarily clones
        ModuleLike::in_module(&m, uri.name())
    }

}

#[derive(Debug)]
struct TemporaryBackendI {
    modules: parking_lot::Mutex<HMap<ModuleURI, Module>>,
    documents: parking_lot::Mutex<HMap<DocumentURI, Document>>,
    html:parking_lot::Mutex<HMap<DocumentURI,HTMLData>>,
    parent:AnyBackend
}

#[derive(Clone,Debug)]
pub struct TemporaryBackend {
    inner: triomphe::Arc<TemporaryBackendI>
}
impl Default for TemporaryBackend {
    #[inline]
    fn default() -> Self {
        Self::new(GlobalBackend::get().to_any())
    }
}

impl TemporaryBackend {
    #[must_use]
    pub fn new(parent:AnyBackend) -> Self {
        Self { inner: triomphe::Arc::new(TemporaryBackendI { 
            modules: parking_lot::Mutex::new(HMap::default()), 
            documents: parking_lot::Mutex::new(HMap::default()),
            html:parking_lot::Mutex::new(HMap::default()),
            parent 
        }) }
    }
    pub fn add_module(&self,m:Module) {
        self.inner.modules.lock().insert(m.uri().clone(), m);
    }
    pub fn add_document(&self,d:Document) {
        self.inner.documents.lock().insert(d.uri().clone(), d);
    }
    pub fn add_html(&self,uri:DocumentURI,d:HTMLData) {
        self.inner.html.lock().insert(uri, d);
    }
}

impl Backend for TemporaryBackend {
    type ArchiveIter<'a> = std::slice::Iter<'a,Archive>;

    #[inline]
    fn to_any(&self) -> AnyBackend {
        AnyBackend::Temp(self.clone())
    }
    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        self.inner.documents.lock().get(uri).cloned().or_else(|| 
            self.inner.parent.get_document(uri)
        )
    }

    #[inline]
    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        self.inner.parent.with_archives(f)
    }

    fn get_html_body(&self,
            d:&DocumentURI,full:bool
        ) -> Option<(Vec<CSS>,String)> {
        self.inner.html.lock().get(d).map_or_else(
            || self.inner.parent.get_html_body(d,full),
            |html| Some((
                html.css.clone(),
                if full { html.html[html.body.start..html.body.end].to_string() } else {
                    html.html[html.body.start + html.inner_offset..html.body.end].to_string()
                }
            ))
        )
    }

    fn get_html_fragment(&self,
            d:&DocumentURI,range:DocumentRange
        ) -> Option<(Vec<CSS>,String)> {
        self.inner.html.lock().get(d).map_or_else(
            || self.inner.parent.get_html_fragment(d,range),
            |html| Some((
                html.css.clone(),
                html.html[range.start..range.end].to_string()
            ))
        )
    }

    fn get_reference<T:immt_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        self.inner.html.lock().get(&rf.in_doc).map_or_else(
            || self.inner.parent.get_reference(rf),
            |html| {
                let bytes = html.refs.as_slice().get(rf.start..rf.end)?;
                bincode::serde::decode_from_slice(bytes, bincode::config::standard()).ok().map(|(a,_)| a)
            }
        )
    }

    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        if uri.name().is_simple() {
            return self.inner.modules.lock().get(uri).cloned().map(ModuleLike::Module).or_else(
                || self.inner.parent.get_module(uri)
            )
        }
        let top_uri = !uri.clone();
        let top = self.inner.modules.lock().get(&top_uri).cloned().or_else(
            || match self.inner.parent.get_module(&top_uri) {
                Some(ModuleLike::Module(m)) => Some(m),
                _ => None
            }
        )?;
        ModuleLike::in_module(&top, uri.name())
    }
    #[inline]
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        self.inner.parent.get_base_path(id)
    }

    #[inline]
    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R
        where
            Self: Sized {
        self.inner.parent.with_archive_or_group(id, f)
    }

    #[inline]
    fn with_archive<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&Archive>) -> R) -> R {
        self.inner.parent.with_archive(id,f)
    }

    #[inline]
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=immt_ontology::rdf::Triple>)
            where Self:Sized {
        self.inner.parent.submit_triples(in_doc,rel_path,iter);
    }

    
}

pub struct AsChecker<'a,B:Backend>(&'a B);

impl<B:Backend> LocalBackend for AsChecker<'_,B> {
    #[inline]
    fn get_document(&mut self, uri: &DocumentURI) -> Option<Document> {
        self.0.get_document(uri)
    }
    #[inline]
    fn get_declaration<T: DeclarationTrait>(
            &mut self,
            uri: &SymbolURI,
        ) -> Option<ContentReference<T>> {
        self.0.get_declaration(uri)
    }
    #[inline]
    fn get_module(&mut self, uri: &ModuleURI) -> Option<ModuleLike> {
        self.0.get_module(uri)
    }
}


impl<B:Backend> DocumentChecker for AsChecker<'_,B> {
    #[inline]
    fn open(&mut self, _elem: &mut DocumentElement<Unchecked>) {}
    #[inline]
    fn close(&mut self, _elem: &mut DocumentElement<Checked>) {}
}

impl<B:Backend> ModuleChecker for AsChecker<'_,B> {
    #[inline]
    fn open(&mut self, _elem: &mut OpenDeclaration<Unchecked>) {}
    #[inline]
    fn close(&mut self, _elem: &mut Declaration) {}
}



struct GlobalFlattener<'a>(&'a mut BackendCache, &'a ArchiveManager);
impl GlobalFlattener<'_> {
    fn load_document(
        &mut self,
        path: PathURIRef,
        language: Language,
        name: &NameStep,
    ) -> Option<Document> {
        //println!("Document {path}&d={name}&l={language}");
        let pre = self.1.load_document(path, language, name)?;
        let doc_file = PreDocFile::resolve(pre,self);
        let doc = doc_file.clone();
        self.0.insert_document(doc_file);
        Some(doc)
    }
    fn load_module(
        &mut self,
        path: PathURIRef,
        language: Language,
        name: &NameStep,
    ) -> Option<Module> {
        //println!("Module {path}&m={name}&l={language}");
        let pre = self.1.load_module(path, language, name)?;
        let module = pre.check(self);
        self.0.insert_module(module.clone());
        Some(module)
    }
}

impl LocalBackend for GlobalFlattener<'_> {
    #[allow(clippy::option_if_let_else)]
    fn get_document(&mut self, uri: &DocumentURI) -> Option<Document> {
        if let Some(doc) = self.0.has_document(uri) {
            Some(doc.clone())
        } else {
            self.load_document(uri.as_path(), uri.language(), uri.name().first_name())
        }
    }

    fn get_module(&mut self, uri: &ModuleURI) -> Option<ModuleLike> {
        if uri.name().is_simple() {
            if let Some(m) = self.0.has_module(uri) {
                return Some(ModuleLike::Module(m.clone()));
            }
        } else {
            let top_uri = !uri.clone();
            if let Some(m) = self.0.has_module(&top_uri) {
                return ModuleLike::in_module(m, uri.name());
            }
        }
        let m = self.load_module(uri.as_path(), uri.language(), uri.name().first_name())?;
        // TODO this unnecessarily clones
        ModuleLike::in_module(&m, uri.name())
    }

    fn get_declaration<T: DeclarationTrait>(
        &mut self,
        uri: &SymbolURI,
    ) -> Option<immt_ontology::content::ContentReference<T>> {
        let m = self.get_module(uri.module())?;
        // TODO this unnecessarily clones
        ContentReference::new(&m, uri.name())
    }
}

impl DocumentChecker for GlobalFlattener<'_> {
    #[inline]
    fn open(&mut self, _elem: &mut DocumentElement<Unchecked>) {}
    #[inline]
    fn close(&mut self, _elem: &mut DocumentElement<Checked>) {}
}

impl ModuleChecker for GlobalFlattener<'_> {
    #[inline]
    fn open(&mut self, _elem: &mut OpenDeclaration<Unchecked>) {}
    #[inline]
    fn close(&mut self, _elem: &mut Declaration) {}
}

pub struct TermPresenter<'a,W:std::fmt::Write,B:Backend>{
    out:W,
    backend:&'a B,
    in_text:bool,
    cache:VecMap<SymbolURI,Option<Rc<Notation>>>,
    op_cache:VecMap<SymbolURI,Option<Rc<Notation>>>,
    var_cache:VecMap<DocumentElementURI,Option<Rc<Notation>>>,
    var_op_cache:VecMap<DocumentElementURI,Option<Rc<Notation>>>,
}
impl<'a,W:std::fmt::Write,B:Backend> TermPresenter<'a,W,B> {
    #[inline]
    pub fn new_with_writer(out:W,backend:&'a B,in_text:bool) -> Self {
        Self { out, backend, in_text, 
            cache:VecMap::default(), 
            op_cache:VecMap::default(),
            var_cache:VecMap::default(),
            var_op_cache:VecMap::default(),
        }
    }
    #[inline]
    pub fn close(self) -> W { self.out }

    #[inline]
    pub fn backend(&self) -> &'a B {
        &self.backend
    }

    fn load_notation(backend:&B,uri:&SymbolURI,needs_op:bool) -> Option<Notation> {
        use rdf::sparql::{Select,Var};
        use immt_ontology::rdf::ontologies::ulo2;
        let iri = uri.to_iri();
        let q = Select {
            subject: Var('n'),
            pred: ulo2::NOTATION_FOR.into_owned(),
            object: iri
        };
        let iter = GlobalBackend::get().triple_store().query(q.into()).ok()?;
        iter.into_uris().find_map(|uri| {
            let elem = backend.get_document_element::<DocumentElement<Checked>>(&uri)?;
            let DocumentElement::Notation{notation,..} = elem.as_ref() else {
                return None
            };
            //println!("Found notation {notation:?}");
            let r = backend.get_reference(&notation)?;
            if r.is_op() || !needs_op { Some(r) } else { None }
        })
    }

    fn load_var_notation(backend:&B,uri:&DocumentElementURI,needs_op:bool) -> Option<Notation> {
        use immt_ontology::narration::{sections::Section,paragraphs::LogicalParagraph,exercises::Exercise};
        let parent = uri.parent();
        //println!("Looking for {uri} in {parent}");
        let parent = backend.get_document_element::<DocumentElement<Checked>>(&parent)?;
        let mut ch = parent.as_ref().children().iter();
        let mut stack = Vec::new();
        loop {
            let Some(next) = ch.next() else {
                if let Some(n) = stack.pop() {
                    ch = n;
                    continue
                }
                return None
            };
            let not = match next {
                DocumentElement::Module { children,.. } |
                DocumentElement::Section(Section{children,..}) |
                DocumentElement::Morphism{children,..} |
                DocumentElement::MathStructure{children,..} |
                DocumentElement::Extension{children,..} |
                DocumentElement::Paragraph(LogicalParagraph{children,..}) |
                DocumentElement::Exercise(Exercise{children,..}) => {
                    let old = std::mem::replace(&mut ch,children.iter());
                    stack.push(old);
                    continue
                }
                DocumentElement::VariableNotation { variable, id, notation } if variable == uri => notation,
                _ => continue
            };
            let Some(r) = backend.get_reference(&not) else { continue };
            if r.is_op() || !needs_op { return Some(r) }
        }
    }
}

impl<'a,W:std::fmt::Write,B:Backend> std::fmt::Write for TermPresenter<'a,W,B> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.out.write_str(s)
    }
}
impl<'a,W:std::fmt::Write,B:Backend> Presenter for TermPresenter<'a,W,B> {
    type N = Rc<Notation>;
    #[inline]
    fn cont(&mut self,tm:&immt_ontology::content::terms::Term) -> Result<(),PresentationError> {
        tm.present(self)
    }

    #[inline]
    fn in_text(&self) -> bool { self.in_text }

    fn get_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {
        //println!("Getting notation for {uri:?}");
        if let Some(n) = self.cache.get(uri) {
            //println!("Returning from cache {n:?}");
            return n.clone()
        };
        let r = Self::load_notation(self.backend,uri,false).map(Rc::new);
        self.cache.insert(uri.clone(),r.clone());
        //println!("Returning {r:?}");
        if let Some(r) = &r {
            if r.is_op() { self.op_cache.insert(uri.clone(),Some(r.clone())); }
        }
        r
    }

    fn get_op_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {
        //println!("Getting op notation for {uri:?}");
        if let Some(n) = self.op_cache.get(uri) {
            //println!("Returning from cache {n:?}");
            return n.clone()
        };
        let r = Self::load_notation(self.backend,uri,true).map(Rc::new);
        self.op_cache.insert(uri.clone(),r.clone());
        //println!("Returning {r:?}");
        if self.cache.get(uri).is_none() {
            self.cache.insert(uri.clone(),r.clone());
        }
        r
    }

    #[inline]
    fn get_variable_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {
        if let Some(n) = self.var_cache.get(uri) {return n.clone()};
        let r = Self::load_var_notation(self.backend,uri,false).map(Rc::new);
        self.var_cache.insert(uri.clone(),r.clone());
        if let Some(r) = &r {
            if r.is_op() { self.var_op_cache.insert(uri.clone(),Some(r.clone())); }
        }
        r
    }
    #[inline]
    fn get_variable_op_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {
        if let Some(n) = self.var_op_cache.get(uri) {return n.clone()};
        let r = Self::load_var_notation(self.backend,uri,true).map(Rc::new);
        self.var_op_cache.insert(uri.clone(),r.clone());
        if self.var_cache.get(uri).is_none() {
            self.var_cache.insert(uri.clone(),r.clone());
        }
        r
    }
}

pub type StringPresenter<'a,B:Backend> = TermPresenter<'a,String,B>;

impl<'a,B:Backend> StringPresenter<'a,B> {
    #[inline]
    pub fn new(backend:&'a B,in_text:bool) -> Self {
        Self::new_with_writer(String::new(), backend, in_text)
    }
    #[inline]
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.out)
    }

    pub fn present(&mut self,term:&Term) -> Result<String,PresentationError> {
        self.out.clear();
        //println!("Presenting: {term}");
        let r = term.present(self);
        let s = std::mem::take(&mut self.out);
        //println!("Returning: {s}");
        r.map(|_|s)
    }
}