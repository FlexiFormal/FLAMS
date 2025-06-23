use flams_ontology::{
    content::modules::Module,
    narration::documents::Document,
    uris::{DocumentURI, ModuleURI},
};
use flams_utils::prelude::HMap;

#[derive(Default, Debug)]
pub(super) struct BackendCache {
    modules: HMap<ModuleURI, Module>,
    documents: HMap<DocumentURI, Document>,
}

impl BackendCache {
    const EVERY_MODS: usize = 500;
    const EVERY_DOCS: usize = 500;

    pub fn clear(&mut self) {
        self.modules.clear();
        self.documents.clear();
    }

    #[inline]
    fn gc(&mut self) {
        if self.modules.len() >= Self::EVERY_MODS {
            self.modules.retain(|_, v| v.strong_count() > 1);
        }
        if self.documents.len() >= Self::EVERY_DOCS {
            self.documents.retain(|_, v| v.strong_count() > 1);
        }
    }

    #[inline]
    pub(super) fn has_document(&self, uri: &DocumentURI) -> Option<&Document> {
        self.documents.get(uri)
    }

    #[inline]
    pub(super) fn insert_document(&mut self, doc: Document) {
        self.gc();
        self.documents.insert(doc.uri().clone(), doc);
    }

    #[inline]
    pub(super) fn has_module(&self, uri: &ModuleURI) -> Option<&Module> {
        self.modules.get(uri)
    }

    #[inline]
    pub(super) fn insert_module(&mut self, m: Module) {
        self.gc();
        self.modules.insert(m.uri().clone(), m);
    }
}
