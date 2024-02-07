pub struct RelationalManager {
    store:oxigraph::store::Store
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