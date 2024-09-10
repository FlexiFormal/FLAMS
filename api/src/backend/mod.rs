use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use oxigraph::model::Variable;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use tokio::io::AsyncReadExt;
use immt_core::content::{ContentElement, ContentElemRef, MathStructure, Module, ModuleLike, Notation, OF_TYPE, Term, TermDisplay, VarNameOrURI};
use immt_core::narration::{CSS, DocData, DocElemRef, DocumentElement, FullDocument, LogicalParagraph, NarrativeRef};
use immt_core::ontology::rdf::ontologies::{rdf, ulo2};
use immt_core::uris::{ArchiveId, DocumentURI, ModuleURI, Name, SymbolURI};
use crate::backend::archives::Archive;
use crate::backend::manager::ArchiveManager;
use crate::backend::relational::RelationalManager;
use immt_utils::prelude::{*,triomphe::Arc};

pub mod archives;
pub mod manager;
#[cfg(feature="oxigraph")]
pub mod relational;

#[derive(Default,Debug)]
struct BackendCache {
    modules: HMap<ModuleURI,Arc<Module>>,
    documents: HMap<DocumentURI,DocData>,
}
impl BackendCache {
    const EVERY_MODS:usize = 50;
    const EVERY_DOCS:usize = 50;

    #[inline]
    fn gc(&mut self) {
        if self.modules.len() >= Self::EVERY_MODS {
            self.modules.retain(|_, v| Arc::strong_count(v) > 1);
        }
        if self.documents.len() >= Self::EVERY_DOCS {
            self.documents.retain(|_, v| v.strong_count() > 1);
        }
    }

    fn get_document(&mut self,am:&ArchiveManager,uri:DocumentURI) -> Option<DocData> {
        match self.documents.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let r = e.get().clone();
                self.gc();
                return Some(r)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let p = Backend::find_file(am,uri)?;
                if p.exists() {
                    let d = DocData::get(p)?;
                    e.insert(d.clone());
                    self.gc();
                    Some(d)
                } else {None}
            }
        }
    }
    async fn get_document_async(&mut self,am:&ArchiveManager,uri:DocumentURI) -> Option<DocData> {
        match self.documents.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let r = e.get().clone();
                self.gc();
                return Some(r)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let p = Backend::find_file_async(am,uri).await?;
                if p.exists() {
                    let d = DocData::get_async(p).await?;
                    e.insert(d.clone());
                    self.gc();
                    Some(d)
                } else {None}
            }
        }
    }

    fn load_module(uri:ModuleURI,am:&ArchiveManager) -> Option<Module> {
        let p = am.find(uri.archive().id(),|a| match a {
            Some(Archive::Physical(ma)) => Some(match uri.path() {
                None => ma.out_dir(),
                Some(p) => ma.out_dir().join(p.as_ref())
            }),
            _ => None
        })?.join(".modules").join(uri.name().as_ref()).join::<&'static str>(uri.language().into()).with_extension("comd");
        if p.exists() {
            bincode::serde::decode_from_std_read(&mut BufReader::new(std::fs::File::open(p).ok()?),
                                                 bincode::config::standard()
            ).ok()
        } else {None}
    }
    async fn load_module_async(uri:ModuleURI,am:&ArchiveManager) -> Option<Module> {
        let p = am.find(uri.archive().id(),|a| match a {
            Some(Archive::Physical(ma)) => Some(match uri.path() {
                None => ma.out_dir(),
                Some(p) => ma.out_dir().join(p.as_ref())
            }),
            _ => None
        })?.join(".modules").join(uri.name().as_ref()).join::<&'static str>(uri.language().into()).with_extension("comd");
        if p.exists() {
            let mut f = tokio::fs::File::open(p).await.ok()?;
            let mut v = Vec::new();
            f.read_to_end(&mut v).await.ok()?;
            bincode::serde::decode_from_slice(v.as_slice(),
                                                            bincode::config::standard()
            ).ok().map(|(m,_)| m)
        } else {None}
    }

    fn get_module(&mut self,am:&ArchiveManager,uri:ModuleURI) -> Option<Arc<Module>> {
        match self.modules.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let r = e.get().clone();
                self.gc();
                return Some(r)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let top = Self::load_module(uri,am)?;
                let ret = Arc::new(top);
                e.insert(ret.clone());
                self.gc();
                Some(ret)
            }
        }
    }
    /*
    async fn get_module_async(&mut self,am:&ArchiveManager,uri:ModuleURI) -> Option<Arc<Module>> {
        match self.modules.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => {
                return Some(e.get().clone())
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let top = Self::load_module_async(uri,am).await?;
                let ret = Arc::new(top);
                e.insert(ret.clone());
                Some(ret)
            }
        }
    }

     */
}

#[derive(Default,Debug)]
pub struct Backend {
    relational_manager: RelationalManager,
    archive_manager: ArchiveManager,
    cache: parking_lot::Mutex<BackendCache>
}
impl Backend  {
    #[inline]
    pub fn get_archive<R>(&self,a: ArchiveId, f:impl FnOnce(Option<&Archive>) -> R) -> R {
        self.archive_manager.find(a,f)
    }
    #[inline]
    pub fn all_archives(&self) -> impl Deref<Target=[Archive]> + '_ {
        self.archive_manager.get_archives()
    }
    #[inline]
    pub fn relations(&self) -> &RelationalManager {
        &self.relational_manager
    }
    #[inline]
    pub fn archive_manager(&self) -> &ArchiveManager { &self.archive_manager }

    // -----------------------------------------------------------------------------

    #[inline]
    pub fn display_term<'a>(&'a self,t:&'a Term) -> impl std::fmt::Display + 'a {
        t.displayable(
            |s| self.get_notations(s).map(|(_,n)| n),
            |s| self.get_var_notations(s).into_iter().flatten(),
        )
    }
    pub fn get_var_notations(&self,sym:VarNameOrURI) -> Option<impl Iterator<Item=Notation> + '_> {
        match sym {
            VarNameOrURI::URI(nd) => self.get_document(nd.doc()).map(move |dd| {
                dd.iter().filter_map(|c| match c {
                    DocumentElement::VarNotation{name,notation,..} if *name == sym => {
                        dd.read_resource(*notation)
                    },
                    _ => None
                }).collect::<Vec<_>>().into_iter()
            }),
            _ => None
        }
    }

    pub fn get_definitions(&self,sym:SymbolURI) -> impl Iterator<Item=DocElemRef<LogicalParagraph>> + '_ {
        let s = sym.to_iri();
        let x = TermPattern::Variable(Variable::new_unchecked("x"));
        let query = spargebra::Query::Select {
            dataset:None,
            base_iri:None,
            pattern:spargebra::algebra::GraphPattern::Bgp {
                patterns:vec![
                    TriplePattern{subject:x,
                        predicate:NamedNodePattern::NamedNode(ulo2::DEFINES.into_owned()),
                        object:TermPattern::NamedNode(s)
                    }
                ]
            }
        };

        self.relations().query(query.into()).into_iter().filter_map(move |res|
            res.doc_elem_iter().map(|i| i.filter_map(move |s| {
                //println!(" - {s}");
                self.get_document(s.doc()).and_then(|d| {
                    d.into_elem(s.name(),|d| match d {
                        DocumentElement::Paragraph(p) => Some(p),
                        _ => None
                    }).ok()
                })
            }))
        ).flatten()
    }
    pub fn get_notations(&self,sym:SymbolURI) -> impl Iterator<Item=(ModuleURI,Notation)> + '_ {
        use immt_core::ontology::rdf::ontologies::*;
        use spargebra::term::*;
        //println!("Notations for {sym}");
        let s = sym.to_iri();
        let x = TermPattern::Variable(Variable::new_unchecked("x"));
        let query = spargebra::Query::Select {
            dataset:None,
            base_iri:None,
            pattern:spargebra::algebra::GraphPattern::Bgp {
                patterns:vec![
                    TriplePattern{subject:x.clone(),
                        predicate:NamedNodePattern::NamedNode(rdf::TYPE.into_owned()),
                        object:TermPattern::NamedNode(ulo2::NOTATION.into_owned())
                    },
                    TriplePattern{subject:x,
                        predicate:NamedNodePattern::NamedNode(ulo2::NOTATION_FOR.into_owned()),
                        object:TermPattern::NamedNode(s)
                    }
                ]
            }
        };

        self.relations().query(query.into()).into_iter().filter_map(move |res|
            res.symbol_iter().map(|i| i.filter_map(move |s| {
                //println!(" - {s}");
                self.get_module(s.module()).and_then(|m| {
                    //println!(" - in {}",m.uri());
                    m.get(s.name()).and_then(|c| {
                        //println!(" - is {c:?}");
                        match c {
                            ContentElement::Notation(n) => {
                                let rf = self.get_ref(n.range);
                                //println!(" - returns {rf:?}");
                                if let Some(n) = rf {
                                    Some((m.uri().clone(),n))
                                } else {None}
                            }
                            _ => None
                        }
                    })
                })
            }))
        ).flatten()
    }

    #[inline]
    fn get_ref<T:std::fmt::Debug+for <'a> serde::Deserialize<'a>>(&self,rf:NarrativeRef<T>) -> Option<T> {
        self.get_document(rf.in_doc).and_then(|d| d.read_resource(rf))
    }

    fn find_file(am:&ArchiveManager,uri:DocumentURI) -> Option<PathBuf> {
        let p = am.find(uri.archive().id(), |a| match a {
            Some(Archive::Physical(ma)) => Some(match uri.path() {
                None => ma.out_dir(),
                Some(p) => ma.out_dir().join(p.as_ref())
            }),
            _ => None
        })?;
        let _name = uri.name();
        let name = _name.as_ref();
        for d in std::fs::read_dir(p).ok()? {
            let dir = if let Ok(d) = d { d } else {continue};
            let m = if let Ok(d) = dir.metadata() { d } else {continue};
            if !m.is_dir() {continue}
            let _name = dir.file_name();
            let d = if let Some(d) = _name.to_str() { d } else {continue};
            if !d.starts_with(name) {continue}
            let rest = &d[name.len()..];
            if !rest.is_empty() && !rest.starts_with('.') { continue }
            let rest = rest.strip_prefix('.').unwrap_or(rest);
            if rest.contains('.') {
                let lang : &'static str = uri.language().into();
                if !rest.starts_with(lang) { continue }
            }
            let p = dir.path().join("index.nomd");
            if p.exists() {
                return Some(p)
            }
        }
        None
    }

    async fn find_file_async(am:&ArchiveManager,uri:DocumentURI) -> Option<PathBuf> {
        let p = am.find(uri.archive().id(), |a| match a {
            Some(Archive::Physical(ma)) => Some(match uri.path() {
                None => ma.out_dir(),
                Some(p) => ma.out_dir().join(p.as_ref())
            }),
            _ => None
        })?;
        let mut dir = tokio::fs::read_dir(p).await.ok()?;
        let _name = uri.name();
        let name = _name.as_ref();
        while let Ok(Some(dir)) = dir.next_entry().await  {
            let m = if let Ok(d) = dir.metadata().await { d } else {continue};
            if !m.is_dir() {continue}
            let _name = dir.file_name();
            let d = if let Some(d) = _name.to_str() { d } else {continue};
            if !d.starts_with(name) {continue}
            let rest = &d[name.len()..];
            if !rest.is_empty() && !rest.starts_with('.') { continue }
            let rest = rest.strip_prefix('.').unwrap_or(rest);
            if rest.contains('.') {
                let lang : &'static str = uri.language().into();
                if !rest.starts_with(lang) { continue }
            }
            let p = dir.path().join("index.nomd");
            if p.exists() {
                return Some(p)
            }
        }
        None
    }

    #[inline]
    pub async fn get_html_async(&self,uri:DocumentURI) -> Option<(Box<[CSS]>,Box<str>)> {
        let d = self.cache.lock().documents.get(&uri).cloned();
        if let Some(d) = d {
            return d.read_css_and_body_async().await
        }
        let p = Self::find_file_async(&self.archive_manager,uri).await?;
        if p.exists() {
            FullDocument::get_css_and_body_async(&p).await
        } else { None }
    }

    #[inline]
    pub async fn get_document_async(&self,uri:DocumentURI) -> Option<DocData> {
        {
            if let Some(d) = self.cache.lock().documents.get(&uri) {
                return Some(d.clone())
            }
        }
        let p = Self::find_file_async(&self.archive_manager,uri).await?;
        if p.exists() {
            let d = DocData::get_async(p).await?;
            self.cache.lock().documents.insert(uri,d.clone());
            Some(d)
        } else { None }
    }

    #[inline]
    pub fn get_document(&self,uri:DocumentURI) -> Option<DocData> {
        self.cache.lock().get_document(&self.archive_manager,uri)
    }

    #[inline]
    fn find_in_module(m:Arc<Module>,name:Name) -> Option<ModuleLike> {
        let mut names = name.as_ref().split('/');
        let _ = names.next();
        let mut r: Option<Result<&Module,&MathStructure>> = None;
        'top: for n in names {
            let ch = match r {
                None => m.elements.as_slice(),
                Some(Ok(n)) => n.elements.as_slice(),
                Some(Err(s)) => s.elements.as_slice(),
            };
            for c in ch {match c {
                ContentElement::NestedModule(m) if m.uri.name().as_ref().split('/').last() == Some(n) =>
                    {r = Some(Ok(m)); continue 'top},
                ContentElement::MathStructure(s) if s.uri.name().as_ref().split('/').last() == Some(n) =>
                    {r = Some(Err(s)); continue 'top},
                _ => ()
            }}
            return None
        }
        match r {
            None => Some(ModuleLike::Module(m)),
            Some(Ok(n)) => {let n = n as _;Some(ModuleLike::NestedModule(m,n))}
            Some(Err(s)) => {let s = s as _; Some(ModuleLike::Structure(m,s)) }
        }
    }

    #[inline]
    pub async fn get_module_async(&self,uri:ModuleURI) -> Option<ModuleLike> {
        let top_uri = !uri;
        {
            if let Some(m) = self.cache.lock().modules.get(&top_uri) {
                return Self::find_in_module(m.clone(),uri.name())
            }
        }
        let m = BackendCache::load_module_async(top_uri,&self.archive_manager).await?;
        let m = Arc::new(m);
        self.cache.lock().modules.insert(top_uri,m.clone());
        Self::find_in_module(m,uri.name())
    }

    #[inline]
    pub fn get_module(&self,uri:ModuleURI) -> Option<ModuleLike> {
        let nuri = !uri;
        self.cache.lock().get_module(&self.archive_manager,nuri).and_then(|m| Self::find_in_module(m,uri.name()))
    }

    pub async fn get_constant_async(&self,uri:SymbolURI) -> Option<ContentElemRef<immt_core::content::Constant>> {
        let m = self.get_module_async(uri.module()).await?;
        m.into_elem(uri.name(),|ce| {
            if let ContentElement::Constant(c) = ce {Some(c)} else {None}
        }).ok()
    }
    pub fn get_constant(&self,uri:SymbolURI) -> Option<ContentElemRef<immt_core::content::Constant>> {
        let m = self.get_module(uri.module())?;
        m.into_elem(uri.name(),|ce| {
            if let ContentElement::Constant(c) = ce {Some(c)} else {None}
        }).ok()
    }
}