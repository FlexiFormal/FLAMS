pub struct RelationalManager {
    store:oxigraph::store::Store
}
impl RelationalManager {
    pub fn loader(&mut self) -> oxigraph::store::BulkLoader {
        self.store.bulk_loader()
    }
    pub fn size(&self) -> usize {
        self.store.len().unwrap()
    }
}
impl Default for RelationalManager {
    fn default() -> Self {
        let store = oxigraph::store::Store::new().unwrap();//oxigraph::store::Store::open("foo").unwrap();
        for q in immt_api::ontology::rdf::ulo2::QUADS.iter() {
            store.insert(*q).unwrap();
        }
        RelationalManager {
            store
        }
    }
}