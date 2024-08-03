use std::collections::BTreeSet;
use std::fmt::Debug;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use oxigraph::io::GraphFormat;
use oxigraph::sparql::{Query, QueryResults};
use oxrdfio::RdfFormat;
use tracing::instrument;
use immt_core::narration::Language;
use immt_core::ontology::rdf::terms::{NamedNode, Quad, Term, Triple};
use immt_core::uris::{DocumentURI, PathURI};
use crate::backend::archives::{Archive, Storage};
use crate::backend::manager::ArchiveManager;


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
    pub fn export(&self,iter:impl Iterator<Item=Triple>,p:&Path,uri:DocumentURI) {
        if let Ok(file) = std::fs::File::create(p) {
            let writer = BufWriter::new(file);
            let ns = PathURI::new(uri.archive(),uri.path(),None).to_string().replace(' ',"%20");
            let mut writer = oxigraph::io::RdfSerializer::from_format(RdfFormat::Turtle)
                .with_prefix("ns", ns).unwrap()
                .with_prefix("ulo","http://mathhub.info/ulo").unwrap()
                .with_prefix("dc","http://purl.org/dc/elements/1.1").unwrap()
                .serialize_to_write(writer);
            for t in iter {
                writer.write_triple(&t).unwrap_or_else(|f| panic!("{f}"));
            }
            let _ = writer.finish();
        }
    }

    pub fn query(&self,s:impl AsRef<str>) -> Option<BTreeSet<NamedNode>> {
        let mut query:Query = s.as_ref().try_into().ok()?;
        query.dataset_mut().set_default_graph_as_union();
        let res = self.store.query(query).ok()?;
        if let QueryResults::Solutions(sol) = res {
            Some(sol.into_iter().filter_map(|r|
                r.ok().map(|r| match r.get(0) {
                    Some(Term::NamedNode(n)) => Some(n.clone()),
                    _ => None
                }).flatten()
            ).collect::<BTreeSet<_>>())
        } else {
            None
        }
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
                let out = a.out_dir();
                if out.exists() && out.is_dir() {
                    let loader = self.store.bulk_loader();
                    for e in walkdir::WalkDir::new(&out)
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|entry| entry.file_name() == "index.ttl") {
                        let parent = e.path().parent().unwrap();
                        let parentname = parent.file_name().unwrap().to_str().unwrap();
                        let parentname =parentname.rsplit_once('.').map(|(s,_)| s ).unwrap_or(parentname);
                        let language = Language::from_rel_path(parentname);
                        let parentname = parentname.strip_suffix(&format!(".{}",language.to_string())).unwrap_or(parentname);
                        let pathstr = parent.parent().unwrap().to_str().unwrap().strip_prefix(out.to_str().unwrap()).unwrap();
                        let pathstr = if pathstr.is_empty() {None} else {Some(pathstr)};
                        let doc = DocumentURI::new(a.uri(),pathstr,parentname,language);
                        let graph = doc.to_iri();
                        let reader = oxigraph::io::RdfParser::from_format(RdfFormat::Turtle).with_default_graph(graph);
                        let file = std::fs::File::open(e.path()).expect("Failed to open file");
                        let buf = BufReader::new(file);
                        let _ = loader.load_quads(reader.parse_read(buf).filter_map(|q| q.ok()));
                    }
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