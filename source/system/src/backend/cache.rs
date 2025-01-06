use immt_ontology::{
    content::modules::Module, narration::documents::Document, uris::{DocumentURI, ModuleURI}
};
use immt_utils::prelude::HMap;

#[derive(Default, Debug)]
pub(super) struct BackendCache {
    modules: HMap<ModuleURI, Module>,
    documents: HMap<DocumentURI, Document>,
}

impl BackendCache {
    const EVERY_MODS: usize = 500;
    const EVERY_DOCS: usize = 500;

    pub fn clear(&mut self) {
        self.modules.clear();self.documents.clear();
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
    /*
    pub(super) fn get_document(&mut self,am:&ArchiveManager,uri:&DocumentURI) -> Option<&DocFile> {
      if let Some(d) = self.has_document(uri) { return Some(d) };
      let doc = am.load_document(uri)?;
      if !path.exists() { return None }


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

    fn load_module(uri:&ModuleURI,am:&ArchiveManager) -> Option<Module> {
        let p = am.find(uri.archive().id(),|a| match a {
            Some(Archive::Physical(ma)) => Some(match uri.path() {
                None => ma.out_dir(),
                Some(p) => ma.out_dir().join(p.as_ref())
            }),
            _ => None
        })?.join(".modules").join(uri.name().as_ref()).join::<&'static str>(uri.language().into()).with_extension("comd");
        if p.exists() {
            /*bincode::serde::decode_from_std_read(&mut BufReader::new(std::fs::File::open(p).ok()?),
                                                 bincode::config::standard()
            ).ok()*/
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
     */
}
