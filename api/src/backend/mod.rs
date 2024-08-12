use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use immt_core::content::{ContentElement, Module, Notation};
use immt_core::narration::{CSS, Document, FullDocument, NarrativeRef};
use immt_core::ontology::rdf::terms::{NamedNodeRef, Quad};
use immt_core::uris::{ArchiveId, DocumentURI, ModuleURI, SymbolURI};
use crate::backend::archives::Archive;
use crate::backend::manager::ArchiveManager;
use crate::backend::relational::RelationalManager;

pub mod archives;
pub mod manager;
#[cfg(feature="oxigraph")]
pub mod relational;

#[derive(Default,Debug)]
pub struct Backend {
    relational_manager: RelationalManager,
    archive_manager: ArchiveManager
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

    pub fn get_notations(&self,sym:SymbolURI) -> impl Iterator<Item=Notation> + '_ {
        use immt_core::ontology::rdf::ontologies::*;
        use spargebra::term::*;
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

        self.relations().query(query.into()).into_iter().flat_map(move |res|
            res.symbol_iter().find_map(move |s| {
                self.get_module(s.module()).and_then(|m| {
                    m.get(s.name()).and_then(|c| {
                        match c {
                            ContentElement::Notation(n) => {
                                self.get_ref(n.range)
                            }
                            _ => None
                        }
                    })
                })
            })
        )
    }

    fn get_ref<T:std::fmt::Debug+for <'a> serde::Deserialize<'a>>(&self,rf:NarrativeRef<T>) -> Option<T> {
        let p = self.find_file(rf.in_doc)?;
        let r = FullDocument::get_resource(&p, rf);
        r
    }

    fn find_file(&self,uri:DocumentURI) -> Option<PathBuf> {
        let p = self.get_archive(uri.archive().id(), |a| match a {
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

    async fn find_file_async(&self,uri:DocumentURI) -> Option<PathBuf> {
        let p = self.get_archive(uri.archive().id(), |a| match a {
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

    pub async fn get_html_async(&self,uri:DocumentURI) -> Option<(Vec<CSS>,String)> {
        let p = self.find_file_async(uri).await?;
        if p.exists() {
            FullDocument::get_css_and_body_async(&p).await
        } else { None }
    }

    pub async fn get_document_async(&self,uri:DocumentURI) -> Option<Document> {
        if let Some(p) = self.find_file_async(uri).await {
            return FullDocument::get_doc_async(&p).await
        } else {None}
    }

    pub fn get_document(&self,uri:DocumentURI) -> Option<Document> {
        self.find_file(uri).map(|p| FullDocument::get_doc(&p)).flatten()
    }

    pub fn get_module(&self,uri:ModuleURI) -> Option<Module> {
        let p = self.get_archive(uri.archive().id(), |a| match a {
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
        } else { None }

    }

    /*

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


    pub fn get_document_reader(&self,uri:DocumentURI) -> Option<DocumentReader> {
        self.find_file(uri).map(|p| HTMLDocSpec::reader(&p)).flatten()
    }

    pub async fn get_document_data_async(&self, uri:DocumentURI) -> Option<DocumentData> {
        if let Some(p) = self.find_file_async(uri).await {
            return HTMLDocSpec::reader_async(&p).await
        } else {None}
    }
     */
}