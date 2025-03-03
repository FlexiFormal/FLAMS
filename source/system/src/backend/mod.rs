pub mod archives;
mod cache;
mod docfile;
pub mod rdf;

use archives::{manager::ArchiveManager, source_files::FileState, Archive, ArchiveGroup, ArchiveOrGroup, ArchiveTree, LocalArchive};
use cache::BackendCache;
use docfile::PreDocFile;
use flams_ontology::{
    content::{
        checking::ModuleChecker, declarations::{Declaration, DeclarationTrait, OpenDeclaration}, modules::Module, terms::Term, ContentReference, ModuleLike
    }, languages::Language, narration::{
        checking::DocumentChecker, documents::Document, exercises::Exercise, notations::{Notation, PresentationError, Presenter}, paragraphs::LogicalParagraph, sections::Section, DocumentElement, LazyDocRef, NarrationTrait, NarrativeReference
    }, uris::{
        ArchiveId, ArchiveURI, ArchiveURITrait, ContentURITrait, DocumentElementURI, DocumentURI, ModuleURI, NameStep, PathURIRef, PathURITrait, SymbolURI, URIOrRefTrait, URIWithLanguage
    }, Checked, DocumentRange, LocalBackend, Unchecked
};
use flams_utils::{prelude::{HMap, TreeLike}, triomphe, vecmap::{VecMap, VecSet}, CSS};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use rdf::RDFStore;
use std::{ops::Deref, path::{Path, PathBuf}, rc::Rc};
use crate::{formats::{HTMLData, SourceFormatId}, settings::Settings};

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
            if let Some(r) = base.strip_prefix(ap) {
                if r.starts_with('/') || r.is_empty() { return Some(f(a,r))}
            }
            None
        }))
    }

    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized;

    //fn with_archive_tree<R>(&self,f:impl FnOnce(&ArchiveTree) -> R) -> R where Self:Sized;

    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>)
        where Self:Sized;
    
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    where Self:Sized;

    fn get_html_body(&self,
        d:&DocumentURI,full:bool
    ) -> Option<(Vec<CSS>,String)>;

    fn get_html_fragment(&self,
        d:&DocumentURI,range:DocumentRange
    ) -> Option<(Vec<CSS>,String)>;

    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T>
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

    fn get_notations(&self,uri:&SymbolURI) -> Option<VecSet<(DocumentElementURI,Notation)>> where Self:Sized {
        use rdf::sparql::{Select,Var};
        use flams_ontology::rdf::ontologies::ulo2;
        let iri = uri.to_iri();
        let q = Select {
            subject: Var('n'),
            pred: ulo2::NOTATION_FOR.into_owned(),
            object: iri
        };
        let ret:VecSet<_> = GlobalBackend::get().triple_store().query(q.into()).ok()?
            .into_uris().filter_map(|uri| {
                let elem = self.get_document_element::<DocumentElement<Checked>>(&uri)?;
                let DocumentElement::Notation{notation,..} = elem.as_ref() else {
                    return None
                };
                self.get_reference(&notation).map(|n| (uri,n))
            })
            .collect();
        if ret.is_empty() { None } else {Some(ret)}
    }

    fn get_var_notations(&self,uri:&DocumentElementURI) -> Option<VecSet<(DocumentElementURI,Notation)>> where Self:Sized {
        let parent = uri.parent();
        let parent = self.get_document_element::<DocumentElement<Checked>>(&parent)?;
        let mut ch = parent.as_ref().children().iter();
        let mut stack = Vec::new();
        let mut ret = VecSet::new();
        loop {
            let Some(next) = ch.next() else {
                if let Some(n) = stack.pop() {
                    ch = n;
                    continue
                }
                break
            };
            let (uri,not) = match next {
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
                DocumentElement::VariableNotation { variable, id, notation } if variable == uri => (id,notation),
                _ => continue
            };
            let Some(r) = self.get_reference(&not) else { continue };
            ret.insert((uri.clone(),r));
        }
        if ret.is_empty() { None } else {Some(ret)}
    }

    /*fn get_archive_for_path(p:&Path) -> Option<(ArchiveURI,String)> {

    }*/

    #[inline]
    fn as_checker(&self) -> AsChecker<Self> where Self:Sized {
        AsChecker(self)
    }
}


#[derive(Clone,Debug)]
pub enum AnyBackend{
    Global(&'static GlobalBackend),
    Temp(TemporaryBackend),
    Sandbox(SandboxedBackend)
}
impl AnyBackend {
    pub fn mathhubs(&self) -> Vec<PathBuf> {
        let mut global: Vec<PathBuf> = Settings::get().mathhubs.iter().map(|p| p.to_path_buf()).collect();
        match self {
            AnyBackend::Global(g) => global,
            AnyBackend::Temp(t) => global,
            AnyBackend::Sandbox(s) => {
                global.insert(0,s.0.path.to_path_buf());
                global
            },
        }
    }
}

pub enum EitherArchiveIter<'a> {
    Global(std::slice::Iter<'a,Archive>),
    Sandbox(std::iter::Chain<std::slice::Iter<'a,Archive>,std::slice::Iter<'a,Archive>>)
}
impl<'a> From<std::slice::Iter<'a,Archive>> for EitherArchiveIter<'a> {
    #[inline]
    fn from(value: std::slice::Iter<'a, Archive>) -> Self {
        Self::Global(value)
    }
}
impl<'a> From<std::iter::Chain<std::slice::Iter<'a,Archive>,std::slice::Iter<'a,Archive>>> for EitherArchiveIter<'a> {
    #[inline]
    fn from(value: std::iter::Chain<std::slice::Iter<'a,Archive>,std::slice::Iter<'a,Archive>>) -> Self {
        Self::Sandbox(value)
    }
}
impl<'a> Iterator for EitherArchiveIter<'a> {
    type Item = &'a Archive;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Global(i) => i.next(),
            Self::Sandbox(i) => i.next(),
        }
    }
}

impl Backend for AnyBackend {
    type ArchiveIter<'a> = EitherArchiveIter<'a>;
    #[inline]
    fn to_any(&self) -> AnyBackend {
        self.clone()
    }

    #[inline]
    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        match self {
            Self::Global(b) => b.with_archives(|i| f(i.into())),
            Self::Temp(b) => b.with_archives(f),
            Self::Sandbox(b) => b.with_archives(|i| f(i.into())),
        }
    }

    #[inline]
    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        match self {
            Self::Global(b) => b.get_reference(rf),
            Self::Temp(b) => b.get_reference(rf),
            Self::Sandbox(b) => b.get_reference(rf),
        }
    }

    #[inline]
    fn get_html_body(&self,
            d:&DocumentURI,full:bool
        ) -> Option<(Vec<CSS>,String)> {
        match self {
            Self::Global(b) => b.get_html_body(d,full),
            Self::Temp(b) => b.get_html_body(d,full),
            Self::Sandbox(b) => b.get_html_body(d,full),
        }
    }

    #[inline]
    fn get_html_fragment(&self,
            d:&DocumentURI,range:DocumentRange
        ) -> Option<(Vec<CSS>,String)> {
        match self {
            Self::Global(b) => b.get_html_fragment(d,range),
            Self::Temp(b) => b.get_html_fragment(d,range),
            Self::Sandbox(b) => b.get_html_fragment(d,range),
        }
    }

    #[inline]
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>) {
        match self {
            Self::Global(b) => b.submit_triples(in_doc,rel_path,iter),
            Self::Temp(b) => b.submit_triples(in_doc,rel_path,iter),
            Self::Sandbox(b) => b.submit_triples(in_doc,rel_path,iter),
        }
    }

    #[inline]
    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        match self {
            Self::Global(b) => b.get_document(uri),
            Self::Temp(b) => b.get_document(uri),
            Self::Sandbox(b) => b.get_document(uri),
        }
    }

    #[inline]
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        match self {
            Self::Global(b) => b.get_module(uri),
            Self::Temp(b) => b.get_module(uri),
            Self::Sandbox(b) => b.get_module(uri),
        }
    }

    #[inline]
    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        match self {
            Self::Global(b) => b.get_base_path(id),
            Self::Temp(b) => b.get_base_path(id),
            Self::Sandbox(b) => b.get_base_path(id),
        }
    }

    #[inline]
    fn get_declaration<T: DeclarationTrait>(&self, uri: &SymbolURI) -> Option<ContentReference<T>>
    where Self: Sized {
        match self {
            Self::Global(b) => b.get_declaration(uri),
            Self::Temp(b) => b.get_declaration(uri),
            Self::Sandbox(b) => b.get_declaration(uri),
        }
    }

    #[inline]
    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R
    where Self: Sized {
        match self {
            Self::Global(b) => b.with_archive_or_group(id,f),
            Self::Temp(b) => b.with_archive_or_group(id,f),
            Self::Sandbox(b) => b.with_archive_or_group(id,f),
        }
    }
    
    #[inline]
    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R
    where Self:Sized {
        match self {
            Self::Global(b) => b.with_archive(id, f),
            Self::Temp(b) => b.with_archive(id, f),
            Self::Sandbox(b) => b.with_archive(id, f),
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
            Self::Sandbox(b) => b.with_local_archive(id, f),
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

    pub fn initialize() {
        let settings = crate::settings::Settings::get();
        let archives = Self::get().manager();
        for p in settings.mathhubs.iter().rev() {
            archives.load(p);
        }
        let f = || {
            let backend = Self::get();
            backend.triple_store().load_archives(&backend.all_archives());
        };
        #[cfg(feature="tokio")]
        flams_utils::background(f);
        #[cfg(not(feature="tokio"))]
        f();
    }

    #[inline]
    pub fn with_archive_tree<R>(&self,f:impl FnOnce(&ArchiveTree) -> R) -> R {
        self.archives.with_tree(f)
    }

    pub fn reset(&self) {
        self.cache.write().clear();
        self.archives.reinit(|_| (), crate::settings::Settings::get().mathhubs.iter().map(|b| &**b));
        self.triple_store.clear();
        flams_utils::background(|| {
            let global = GlobalBackend::get();
            global.triple_store.load_archives(&global.all_archives());
        });
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
                flattener.load_module(top.as_path(), top.name().first_name())
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
    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        GlobalBackend::get_reference(self,rf)
    }

    #[inline]
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>) {
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
        self.archives.with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_fragment(d.path(), d.name().first_name(), d.language(),range))
        )
    }

    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        self.archives.with_archive(rf.in_doc.archive_id(),|a|
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
        self.archives.with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_body(d.path(), d.name().first_name(), d.language(),full))
        )
    }

    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>) {
        self.archives.with_archive(in_doc.archive_id(), |a| {
            if let Some(a) = a {
                a.submit_triples(in_doc,rel_path,self.triple_store(),true,iter);
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
            flattener.load_module(uri.as_path(), uri.name().first_name())?
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

    pub fn reset(&self) {
        self.inner.modules.lock().clear();
        self.inner.documents.lock().clear();
        let global = GlobalBackend::get();
        global.reset();
    }

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
    type ArchiveIter<'a> = EitherArchiveIter<'a>;

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

    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
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
    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>)
            where Self:Sized {
        self.inner.parent.submit_triples(in_doc,rel_path,iter);
    }
}

#[derive(Debug,Clone)]
pub enum SandboxedRepository {
    Copy(ArchiveId),
    Git{
        id: ArchiveId,
        branch:Box<str>,
        commit:flams_git::Commit,
        remote:Box<str>
    }
}
impl SandboxedRepository {
    #[inline]
    pub fn id(&self) -> &ArchiveId {
        match self {
            SandboxedRepository::Copy(id) => id,
            SandboxedRepository::Git{id,..} => id
        }
    }
}

#[derive(Debug)]
struct SandboxedBackendI {
    path: Box<Path>,
    span:tracing::Span,
    repos: parking_lot::RwLock<Vec<SandboxedRepository>>,
    manager: ArchiveManager,
    cache: RwLock<cache::BackendCache>,
}
#[derive(Debug,Clone)]
pub struct SandboxedBackend(triomphe::Arc<SandboxedBackendI>);
impl Drop for SandboxedBackendI {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path);
    }
}
impl SandboxedBackend {
    #[inline]
    pub fn get_repos(&self) -> Vec<SandboxedRepository> {
        self.0.repos.read().clone()
    }

    #[inline]
    pub fn with_repos<R>(&self,f:impl FnOnce(&[SandboxedRepository]) -> R) -> R {
        let inner = self.0.repos.read();
        f(inner.as_slice())
    }

    #[inline]
    pub fn path_for(&self,id:&ArchiveId) -> PathBuf {
        self.0.path.join(id.as_ref())
    }

    pub fn new(name:&str) -> Self {
        let p = crate::settings::Settings::get().temp_dir().join(name);
        let i = SandboxedBackendI {
            span:tracing::info_span!(target:"sandbox","sandbox",path=%p.display()),
            path: p.into(),
            repos: parking_lot::RwLock::new(Vec::new()),
            manager: ArchiveManager::default(),
            cache: RwLock::new(cache::BackendCache::default()),
        };
        SandboxedBackend(triomphe::Arc::new(i))
    }

    #[tracing::instrument(level = "info",
        parent = &self.0.span,
        target = "sandbox",
        name = "migrating",
        fields(path = %self.0.path.display()),
        skip_all
    )]
    pub fn migrate(&self) -> usize {
        #[cfg(target_os="windows")]
        fn same_fs(p1:&Path,p2:&Path) -> bool {
            let Some(p1) = p1.components().next()
                .and_then(|c| c.as_os_str().to_str()) else {return false};
            let Some(p2) = p2.components().next()
                .and_then(|c| c.as_os_str().to_str()) else {return false};
            p1 == p2
        }
        #[cfg(not(target_os="windows"))]
        fn same_fs(p1:&Path,p2:&Path) -> bool {
            use std::os::unix::fs::MetadataExt; 
            fn existent_parent(p:&Path) -> &Path {
                if p.exists() {return p}
                existent_parent(p.parent().unwrap_or_else(|| unreachable!()))
            }
            let p1 = existent_parent(p1);
            let p2 = existent_parent(p2);
            let md1 = p1.metadata().unwrap_or_else(|_| unreachable!());
            let md2 = p2.metadata().unwrap_or_else(|_| unreachable!());
            md1.dev() == md2.dev()
        }
        let mut count = 0;
        let mut cnt = &mut count;
        let global = GlobalBackend::get();
        let mut global_cache = global.cache.write();
        let mut sandbox_cache = self.0.cache.write();
        self.0.manager.reinit(move |sandbox| 
            global.archives.reinit(|global| {
                sandbox.groups.clear();
                let Some(main) = Settings::get().mathhubs.first() else {unreachable!()};
                for a in std::mem::take(&mut sandbox.archives) {
                    *cnt += 1;
                    let Archive::Local(a) = a else { unreachable!()};
                    let source = a.path();
                    let target = main.join(a.id().as_ref());
                    if target.exists() {
                        let _ = std::fs::remove_dir_all(&target);
                    }
                    if let Some(p) = target.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    if same_fs(source,&target) {
                        let _ = std::fs::rename(source, target);
                    } else {
                        let _ = flams_utils::fs::copy_dir_all(source,&target);
                    }
                }
            }, Settings::get().mathhubs.iter().map(|p| &**p)), 
            [&*self.0.path]
        );
        global.triple_store.clear();
        global_cache.clear();
        sandbox_cache.clear();
        drop(global_cache);
        drop(sandbox_cache);
        flams_utils::background(|| {
            let global = GlobalBackend::get();
            global.triple_store.load_archives(&global.all_archives());
        });
        count
    }

    #[tracing::instrument(level = "info",
        parent = &self.0.span,
        target = "sandbox",
        name = "adding",
        fields(repository = ?sb),
        skip_all
    )]
    pub fn add(&self,sb:SandboxedRepository,then:impl FnOnce()) {
        let mut repos = self.0.repos.write();
        let id = sb.id();
        if let Some(i) = repos.iter().position(|r| r.id() == id) {
            repos.remove(i);
        }
        self.require_meta_infs(id,&mut *repos,
            |_,_| {},
            |_,_,_| {
                tracing::error!(target:"sandbox","A group with id {id} already exists!");
            },
            || {}
        );
        repos.push(sb);
        drop(repos);
        then();
        self.0.manager.load(&self.0.path);
    }

    fn require_meta_infs(&self,id:&ArchiveId,repos: &mut Vec<SandboxedRepository>,
        then:impl FnOnce(&LocalArchive,&mut Vec<SandboxedRepository>),
        group:impl FnOnce(&ArchiveGroup,&ArchiveTree,&mut Vec<SandboxedRepository>),
        else_:impl FnOnce()
    ) {
        if repos.iter().any(|r| r.id() == id) { return }
        let backend = GlobalBackend::get();
        backend.manager().with_tree(move |t| {
            let mut steps = id.steps();
            let Some(mut current) = steps.next() else {
                tracing::error!("empty archive ID");
                return
            };
            let mut ls = &t.groups;
            loop {
                let Some(a) = ls.iter().find(|a| a.id().last_name() == current) else {
                    else_(); return
                };
                match a {
                    ArchiveOrGroup::Archive(a) => {
                        if steps.next().is_some() {
                            else_(); return
                        }
                        let Some(Archive::Local(a)) = t.get(id) else {
                            else_(); return
                        };
                        then(a,repos); return
                    }
                    ArchiveOrGroup::Group(g) => {
                        let Some(next) = steps.next() else {
                            group(g,t,repos);
                            return
                        };
                        if let Some(ArchiveOrGroup::Archive(a)) = g.children.iter().find(|a| a.id().is_meta()) {
                            if !repos.iter().any(|r| r.id() == a) {
                                let Some(Archive::Local(a)) = t.get(a) else {
                                    else_(); return
                                };
                                repos.push(SandboxedRepository::Copy(a.id().clone()));
                                self.copy_archive(a);
                            }
                        }
                        current = next;
                        ls = &g.children;
                    }
                }
            }
        });
    }

    #[tracing::instrument(level = "info",
        parent = &self.0.span,
        target = "sandbox",
        name = "require",
        skip(self)
    )]
    pub fn require(&self,id:&ArchiveId) {
        // TODO this can be massively optimized
        let mut repos = self.0.repos.write();
        self.require_meta_infs(id, &mut *repos, 
            |a,repos| {
                repos.push(SandboxedRepository::Copy(id.clone()));
                self.copy_archive(a);
            }, 
            |g,t,repos| for a in g.dfs().unwrap_or_else(|| unreachable!()) {
                if let ArchiveOrGroup::Archive(id) = a {
                    if let Some(Archive::Local(a)) = t.get(id) {
                        if !repos.iter().any(|r| r.id() == id) {
                            repos.push(SandboxedRepository::Copy(id.clone()));
                            self.copy_archive(a);
                        }
                    }
                }
            },
            || tracing::error!("could not find archive {id}")
        );
        drop(repos);
        self.0.manager.load(&self.0.path);
    }

    fn copy_archive(&self,a:&LocalArchive) {
        let path = a.path();
        let target = self.0.path.join(a.id().as_ref());
        if target.exists() { return }
        tracing::info!("copying archive {} to {}",a.id(),target.display());
        if let Err(e) = flams_utils::fs::copy_dir_all(path,&target) {
            tracing::error!("could not copy archive {}: {e}",a.id());
        }
    }
}

impl Backend for SandboxedBackend {
    type ArchiveIter<'a> = std::iter::Chain<std::slice::Iter<'a,Archive>,std::slice::Iter<'a,Archive>>;

    #[inline]
    fn to_any(&self) -> AnyBackend {
        AnyBackend::Sandbox(self.clone())
    }

    fn get_html_fragment(&self,
        d:&DocumentURI,range:DocumentRange
    ) -> Option<(Vec<CSS>,String)> {
        self.with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_fragment(d.path(), d.name().first_name(), d.language(),range))
        )
    }
    fn get_reference<T:flams_ontology::Resourcable>(&self,rf:&LazyDocRef<T>) -> Option<T> {
        self.with_archive(rf.in_doc.archive_id(),|a|
            a.and_then(|a| a.load_reference(rf.in_doc.path(), rf.in_doc.name().first_name(), rf.in_doc.language(),DocumentRange {start:rf.start, end:rf.end}))
        )
    }

    #[inline]
    fn with_archives<R>(&self,f:impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R where Self:Sized {
        self.0.manager.with_tree(|t1|
            GlobalBackend::get().with_archive_tree(|t2| {
                f(t1.archives.iter().chain(t2.archives.iter()))
            })
        )
    }

    fn get_html_body(&self,
        d:&DocumentURI,full:bool
    ) -> Option<(Vec<CSS>,String)> {
        self.with_archive(d.archive_id(), |a|
            a.and_then(|a| a.load_html_body(d.path(), d.name().first_name(), d.language(),full))
        )
    }

    fn submit_triples(&self,in_doc:&DocumentURI,rel_path:&str,iter:impl Iterator<Item=flams_ontology::rdf::Triple>) {
        self.0.manager.with_archive(in_doc.archive_id(), |a| {
            if let Some(a) = a {
                a.submit_triples(in_doc,rel_path,GlobalBackend::get().triple_store(),false,iter);
            }
        });
    }

    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R {
        if let Some(r) = self.0.manager.all_archives().iter().find(|a| a.uri().archive_id() == id) {
            return f(Some(r))
        };
        GlobalBackend::get().with_archive(id,f)
    }

    fn with_archive_or_group<R>(&self,id:&ArchiveId,f:impl FnOnce(Option<&ArchiveOrGroup>) -> R) -> R {
        let cell = std::cell::Cell::new(Some(f));
        if let Some(r) = self.0.manager.with_tree(|t|
            t.find(id).map(|a| (cell.take().unwrap_or_else(|| unreachable!()))(Some(a)))
        ) { return r };
        let f = cell.take().unwrap_or_else(|| unreachable!());
        GlobalBackend::get().with_archive_or_group(id,f)
    }

    fn get_base_path(&self,id:&ArchiveId) -> Option<PathBuf> {
        self.with_local_archive(id, |a| a.map(|a| a.path().to_path_buf()))
    }

    fn get_document(&self, uri: &DocumentURI) -> Option<Document> {
        let id = uri.archive_id();
        if self.0.manager.with_archive(id, |a| a.is_none()) {
            return GlobalBackend::get().get_document(uri);
        }
        {
            let lock = self.0.cache.read();
            if let Some(doc) = lock.has_document(uri) {
                return Some(doc.clone());
            }
        }
        let mut cache = self.0.cache.write();
        let mut flattener = SandboxFlattener(&mut cache, &self.0.manager,&GlobalBackend::get().archives);
        flattener.load_document(uri.as_path(), uri.language(), uri.name().first_name())
    }

    #[allow(clippy::significant_drop_tightening)]
    fn get_module(&self, uri: &ModuleURI) -> Option<ModuleLike> {
        let id = uri.archive_id();
        if self.0.manager.with_archive(id, |a| a.is_none()) {
            return GlobalBackend::get().get_module(uri);
        }
        {
            let lock = self.0.cache.read();
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
            let mut cache = self.0.cache.write();
            let mut flattener = SandboxFlattener(&mut cache, &self.0.manager,&GlobalBackend::get().archives);
            flattener.load_module(uri.as_path(), uri.name().first_name())?
        };
        // TODO: this unnecessarily clones
        ModuleLike::in_module(&m, uri.name())
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
        let doc_file = pre.check(self);
        let doc = doc_file.clone();
        self.0.insert_document(doc_file);
        Some(doc)
    }
    fn load_module(
        &mut self,
        path: PathURIRef,
        name: &NameStep,
    ) -> Option<Module> {
        //println!("Module {path}&m={name}&l={language}");
        let pre = self.1.load_module(path, name)?;
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
        let m = self.load_module(uri.as_path(), uri.name().first_name())?;
        // TODO this unnecessarily clones
        ModuleLike::in_module(&m, uri.name())
    }

    fn get_declaration<T: DeclarationTrait>(
        &mut self,
        uri: &SymbolURI,
    ) -> Option<flams_ontology::content::ContentReference<T>> {
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


struct SandboxFlattener<'a>(&'a mut BackendCache, &'a ArchiveManager,&'a ArchiveManager);
impl SandboxFlattener<'_> {
    fn load_document(
        &mut self,
        path: PathURIRef,
        language: Language,
        name: &NameStep,
    ) -> Option<Document> {
        let be = if self.1.with_archive(path.archive_id(), |a| a.is_some()) {
            self.1
        } else {self.2};
        //println!("Document {path}&d={name}&l={language}");
        let pre = be.load_document(path, language, name)?;
        let doc_file = pre.check(self);
        let doc = doc_file.clone();
        self.0.insert_document(doc_file);
        Some(doc)
    }
    fn load_module(
        &mut self,
        path: PathURIRef,
        name: &NameStep,
    ) -> Option<Module> {
        let be = if self.1.with_archive(path.archive_id(), |a| a.is_some()) {
            self.1
        } else {self.2};
        //println!("Module {path}&m={name}&l={language}");
        let pre = be.load_module(path, name)?;
        let module = pre.check(self);
        self.0.insert_module(module.clone());
        Some(module)
    }
}

impl LocalBackend for SandboxFlattener<'_> {
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
        let m = self.load_module(uri.as_path(), uri.name().first_name())?;
        // TODO this unnecessarily clones
        ModuleLike::in_module(&m, uri.name())
    }

    fn get_declaration<T: DeclarationTrait>(
        &mut self,
        uri: &SymbolURI,
    ) -> Option<flams_ontology::content::ContentReference<T>> {
        let m = self.get_module(uri.module())?;
        // TODO this unnecessarily clones
        ContentReference::new(&m, uri.name())
    }
}

impl DocumentChecker for SandboxFlattener<'_> {
    #[inline]
    fn open(&mut self, _elem: &mut DocumentElement<Unchecked>) {}
    #[inline]
    fn close(&mut self, _elem: &mut DocumentElement<Checked>) {}
}

impl ModuleChecker for SandboxFlattener<'_> {
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
        use flams_ontology::rdf::ontologies::ulo2;
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
    fn cont(&mut self,tm:&flams_ontology::content::terms::Term) -> Result<(),PresentationError> {
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