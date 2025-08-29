pub mod sparql;

use ftml_uris::{ArchiveUri, DocumentUri, FtmlUri, Language, SymbolUri, UriPath, UriWithPath};
use std::path::Path;
use ulo::rdf_types::{Quad, Triple};

use crate::{Archive, LocallyBuilt, MathArchive, utils::path_ext::PathExt};

pub struct RDFStore {
    store: oxigraph::store::Store,
}

impl RDFStore {
    #[inline]
    pub fn clear(&self) {
        let _ = self.store.clear();
    }
    #[inline]
    #[must_use]
    pub fn num_relations(&self) -> usize {
        self.store.len().unwrap_or_default()
    }
    pub fn add_quads(&self, iter: impl Iterator<Item = Quad>) {
        let loader = self.store.bulk_loader();
        let _ = loader.load_quads(iter);
    }

    #[must_use]
    pub fn los(&self, s: &SymbolUri, problems: bool) -> Option<sparql::LOIter> {
        let q = sparql::lo_query(s, problems);
        self.query(q).ok().and_then(|s| {
            if let sparql::QueryResults::Solutions(s) = s.0 {
                Some(sparql::LOIter { inner: s })
            } else {
                None
            }
        })
    }

    pub fn export(&self, iter: impl Iterator<Item = Triple>, p: &Path, uri: &DocumentUri) {
        if let Ok(file) = std::fs::File::create(p) {
            let writer = std::io::BufWriter::new(file);
            let iri = uri.path_uri().to_iri();
            let ns = iri.as_str();
            // SAFETY: all prefixes are valid iris
            let mut writer = unsafe {
                oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::Turtle)
                    .with_prefix("ns", ns)
                    .unwrap_unchecked()
                    .with_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns")
                    .unwrap_unchecked()
                    .with_prefix("ulo", "http://mathhub.info/ulo")
                    .unwrap_unchecked()
                    .with_prefix("dc", "http://purl.org/dc/elements/1.1")
                    .unwrap_unchecked()
                    .for_writer(writer)
            };
            for t in iter {
                if let Err(e) = writer.serialize_triple(&t) {
                    tracing::warn!("Error serializing triple: {e:?}");
                }
            }
            let _ = writer.finish();
        }
    }

    /// ### Errors
    pub fn query_str(&self, s: impl AsRef<str>) -> Result<sparql::QueryResult, sparql::QueryError> {
        let mut query_str = String::from(
            r"PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
          PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
          PREFIX dc: <http://purl.org/dc/elements/1.1#>
          PREFIX ulo: <http://mathhub.info/ulo#>
      ",
        );
        query_str.push_str(s.as_ref());
        let query: oxigraph::sparql::Query = query_str.as_str().try_into()?;
        self.query(query)
    }

    /// ### Errors
    pub fn query(
        &self,
        mut q: oxigraph::sparql::Query,
    ) -> Result<sparql::QueryResult, sparql::QueryError> {
        q.dataset_mut().set_default_graph_as_union();

        // TODO THIS NEEDS TO BE TIMEOUTED!!
        Ok(self.store.query(q).map(sparql::QueryResult)?)
    }

    pub(crate) fn load(&self, path: &Path, graph: ulo::rdf_types::NamedNode) {
        let Ok(file) = std::fs::File::open(path) else {
            tracing::error!("Failed to open file {}", path.display());
            return;
        };
        let buf = std::io::BufReader::new(file);
        let loader = self.store.bulk_loader();
        let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle)
            .with_default_graph(graph)
            .for_reader(buf);
        let _ = loader.load_quads(reader.filter_map(Result::ok));
    }

    #[allow(unreachable_patterns)]
    pub fn load_archives(&self, archives: &[Archive]) {
        use rayon::prelude::*;
        tracing::info!(target:"relational","Loading relational for {} archives...",archives.len());
        let old = self.store.len().unwrap_or_default();
        archives
            .par_iter()
            .filter_map(|a| match a {
                Archive::Local(a) => Some(a),
                Archive::Ext(..) => None,
            })
            .for_each(|a| {
                let out = a.out_dir();
                if out.exists() && out.is_dir() {
                    for e in walkdir::WalkDir::new(out)
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|entry| entry.file_name() == "index.ttl")
                    {
                        let Some(graph) = Self::get_iri(a.uri(), out, &e) else {
                            continue;
                        };
                        self.load(e.path(), graph);
                    }
                }
            });
        tracing::info!(target:"relational","Loaded {} relations", self.store.len().unwrap_or_default() - old);
    }

    fn get_iri(
        a: &ArchiveUri,
        out: &Path,
        e: &walkdir::DirEntry,
    ) -> Option<ulo::rdf_types::NamedNode> {
        let parent = e.path().parent()?;
        let parentname = parent.file_name()?.to_str()?;
        let parentname = parentname.rsplit_once('.').map_or(parentname, |(s, _)| s);
        let language = Language::from_rel_path(parentname);
        let parentname = parentname
            .strip_suffix(&format!(".{language}"))
            .unwrap_or(parentname);
        let path: UriPath = parent.parent()?.relative_to(&out)?.parse().ok()?;
        let doc: DocumentUri = (a.clone() / path) & (parentname.parse().ok()?, language);
        Some(doc.to_iri())
    }
}

impl std::fmt::Debug for RDFStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RDFStore").finish()
    }
}
impl Default for RDFStore {
    fn default() -> Self {
        let store = oxigraph::store::Store::new().unwrap_or_else(|_| unreachable!());
        store
            .bulk_loader()
            .load_quads(ulo::ulo::QUADS.iter().copied())
            .unwrap_or_else(|_| unreachable!());
        Self { store }
    }
}
