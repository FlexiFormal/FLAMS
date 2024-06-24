use std::fmt::Debug;
use tracing::instrument;
use immt_core::ontology::rdf::terms::Quad;
use crate::backend::archives::{Archive, Storage};
use crate::backend::manager::{ArchiveManager, ArchiveManagerAsync};


pub struct RelationalManager {
    store: oxigraph::store::Store
}
impl Debug for RelationalManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalManager")
            .finish()
    }
}
impl RelationalManager {
    pub fn add_quads(&self,iter:impl Iterator<Item=Quad>) {
        let loader = self.store.bulk_loader();
        let _ = loader.load_quads(iter);
    }


    #[cfg(feature = "tokio")]
    #[instrument(level = "info", name = "relational", target="relational", skip_all)]
    pub async fn load_archives_async(&self,archives:&ArchiveManagerAsync) {
        let mut js = tokio::task::JoinSet::new();
        let old = self.store.len().unwrap();
        let archives = archives.with_archives(|archs| archs.iter().filter_map(|a| match a {
            Archive::Physical(a) => Some(a.path().join(".immt").join("rel.ttl")),
            _ => None
        }).collect::<Vec<_>>()).await;
        tracing::info!(target:"relational","Loading relational for {} archives...",archives.len());
        for file in archives {
            if file.exists() {
                let s = self.store.clone();
                js.spawn(async move {
                    let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle);
                    let file = tokio::fs::File::open(file).await.unwrap();
                    let mut reader = reader.parse_tokio_async_read(file);
                    while let Some(q) = reader.next().await {
                        if let Ok(q) = q {
                            let _ = s.insert(&q);//loader.load_quads(std::iter::once(q));
                        }
                    };
                });
            }
        }
        while let Some(_) = js.join_next().await {}
        tracing::info!(target:"relational","Loaded {} relations", self.store.len().unwrap() - old);
    }


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