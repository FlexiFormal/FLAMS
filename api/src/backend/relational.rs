use tracing::instrument;
use immt_core::ontology::rdf::terms::Quad;
use crate::backend::archives::{Archive, Storage};
use crate::backend::manager::ArchiveManager;

pub struct RelationalManager {
    store: oxigraph::store::Store
}
impl RelationalManager {
    pub fn add_quads<R,F:FnOnce(&mut dyn FnMut(Quad)) -> R>(&self,f:F) -> R {
        let loader = self.store.bulk_loader();
        f(&mut |q| { let _ = loader.load_quads(std::iter::once(q)); })
    }
    /*
    #[cfg(feature="async")]
    pub async fn add_quads_async<R,Fu:std::future::Future<Output=R>,F:FnOnce(&'static mut dyn FnMut(Quad)) -> Fu>(&self,f:F) -> R {
        let mut loader = self.store.bulk_loader();
        f(move |q| { let _ = loader.load_quads(std::iter::once(q)); }).await
    }

     */

    #[instrument(level = "info", name = "relational", target="relational", skip_all)]
    pub fn load_archives(&self,archives:&ArchiveManager) {
        use rayon::prelude::*;
        archives.with_archives(|archives| {
            tracing::info!(target:"relational","Loading relational for {} archives...",archives.len());
            let old = self.store.len().unwrap();
            archives.par_iter().filter_map(|a| match a {
                Archive::Physical(a) => Some(a),
                _ => None
            }).for_each(|a| {
                let dir = a.path().join(".immt").join("rel.ttl");
                if dir.exists() {
                    let loader = self.store.bulk_loader();
                    let iri = a.spec().uri.to_iri();
                    let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle).with_default_graph(iri);
                    let mut file = std::fs::File::open(dir).unwrap();
                    let mut buf = std::io::BufReader::new(&mut file);
                    //self.store.load_from_read(reader, &mut buf).unwrap();
                    let _ = loader.load_quads(reader.parse_read(&mut buf).filter_map(|q| q.ok()/*{
                        q.ok().map(|q| Quad {
                            subject: q.subject,
                            predicate: q.predicate,
                            object: q.object,
                            graph_name: oxigraph::model::GraphName::NamedNode(iri.clone()),
                        }*/)
                    //}
                    );//);
                }
            });
            tracing::info!(target:"relational","Loaded {} relations", self.store.len().unwrap() - old);
        });
    }
}
impl Default for RelationalManager {
    fn default() -> Self {
        let store = oxigraph::store::Store::new().unwrap();
        store.bulk_loader().load_quads(immt_core::ontology::rdf::ontologies::ulo2::QUADS.iter().copied())
            .unwrap();
        Self { store }
    }
}