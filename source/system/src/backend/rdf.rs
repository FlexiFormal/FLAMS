use immt_ontology::languages::Language;
use immt_ontology::rdf::{NamedNode, Quad, Triple};
use immt_ontology::uris::{ArchiveURIRef, DocumentURI, PathURITrait, URIOrRefTrait, URIRefTrait};
use oxigraph::sparql::{EvaluationError, Query, QueryResults};
use oxrdfio::RdfFormat;
use spargebra::SparqlSyntaxError;
use std::fmt::{Debug, Display};
use std::io::{BufReader, BufWriter};
use std::ops::Deref;
use std::path::Path;
use std::string::FromUtf8Error;
use tracing::instrument;

use super::archives::Archive;

pub enum QueryError {
    Syntax(SparqlSyntaxError),
    Evaluation(EvaluationError),
    Utf8(FromUtf8Error)
}
impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(e) => Display::fmt(e,f),
            Self::Evaluation(e) => Display::fmt(e,f),
            Self::Utf8(e) => Display::fmt(e,f)
        }
    }
}
impl From<SparqlSyntaxError> for QueryError {
    fn from(e: SparqlSyntaxError) -> Self { Self::Syntax(e) }
}
impl From<EvaluationError> for QueryError {
    fn from(e: EvaluationError) -> Self { Self::Evaluation(e) }
}
impl From<FromUtf8Error> for QueryError {
    fn from(e: FromUtf8Error) -> Self { Self::Utf8(e) }
}

pub struct QueryResult(QueryResults);
impl QueryResult {
    
    /// ### Errors
    pub fn into_json(self) -> Result<String,QueryError> {
        use sparesults::QueryResultsFormat;
        let mut buf = Vec::new();
        match self.0 {
            QueryResults::Boolean(_) | QueryResults::Solutions(_) => {self.0.write(&mut buf, QueryResultsFormat::Json)?;}
            QueryResults::Graph(_) => {self.0.write_graph(&mut buf, RdfFormat::Turtle)?;}
        }
        Ok(String::from_utf8(buf)?)
    }
}
impl AsRef<QueryResults> for QueryResult {
    #[inline]
    fn as_ref(&self) -> &QueryResults {
        &self.0
    }
}
impl Deref for QueryResult {
    type Target = QueryResults;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RDFStore {
    store: oxigraph::store::Store,
}
impl Debug for RDFStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RDFStore").finish()
    }
}

impl Default for RDFStore {
    fn default() -> Self {
        let store = oxigraph::store::Store::new().unwrap_or_else(|_| unreachable!());
        store
            .bulk_loader()
            .load_quads(immt_ontology::rdf::ontologies::ulo2::QUADS.iter().copied())
            .unwrap_or_else(|_| unreachable!());
        Self { store }
    }
}

impl RDFStore {
    #[inline]
    #[must_use]
    pub fn num_relations(&self) -> usize {
        self.store.len().unwrap_or_default()
    }
    pub fn add_quads(&self, iter: impl Iterator<Item = Quad>) {
        let loader = self.store.bulk_loader();
        let _ = loader.load_quads(iter);
    }
    pub fn export(&self, iter: impl Iterator<Item = Triple>, p: &Path, uri: &DocumentURI) {
        if let Ok(file) = std::fs::File::create(p) {
            let writer = BufWriter::new(file);
            let ns = uri.as_path().to_iri().to_string();
            let ns = ns.strip_prefix("<").unwrap_or(&ns);
            let ns = ns.strip_suffix(">").unwrap_or(ns);
            let mut writer = oxigraph::io::RdfSerializer::from_format(RdfFormat::Turtle)
                .with_prefix("ns", ns)
                .unwrap_or_else(|_| unreachable!())
                .with_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns")
                .unwrap_or_else(|_| unreachable!())
                .with_prefix("ulo", "http://mathhub.info/ulo")
                .unwrap_or_else(|_| unreachable!())
                .with_prefix("dc", "http://purl.org/dc/elements/1.1")
                .unwrap_or_else(|_| unreachable!())
                .for_writer(writer);
            for t in iter {
                if let Err(e) = writer.serialize_triple(&t) {
                    tracing::warn!("Error serializing triple: {e:?}");
                }
            }
            let _ = writer.finish();
        }
    }

    /// ### Errors
    pub fn query_str(&self, s: impl AsRef<str>) -> Result<QueryResult,QueryError> {
        let mut query_str = String::from(
            r"PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
          PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
          PREFIX dc: <http://purl.org/dc/elements/1.1#>
          PREFIX ulo: <http://mathhub.info/ulo#>
      ",
        );
        query_str.push_str(s.as_ref());
        let mut query: Query = query_str.as_str().try_into()?;
        query.dataset_mut().set_default_graph_as_union();
        Ok(self.store.query(query).map(QueryResult)?)
    }

    /// ### Errors
    pub fn query(&self, mut q: Query) -> Result<QueryResult,QueryError> {
        q.dataset_mut().set_default_graph_as_union();
        Ok(self.store.query(q).map(QueryResult)?)
    }

    pub(crate) fn load(&self,path:&Path,graph:NamedNode) {
        let Ok(file) = std::fs::File::open(path) else {
            tracing::error!("Failed to open file {}",path.display());
            return;
        };
        let buf = BufReader::new(file);
        let loader = self.store.bulk_loader();
        let reader = oxigraph::io::RdfParser::from_format(RdfFormat::Turtle)
            .with_default_graph(graph)
            .for_reader(buf);
        let _ = loader.load_quads(reader.filter_map(Result::ok));
    }

    #[allow(unreachable_patterns)]
    #[instrument(level = "info", name = "relational", target = "relational", skip_all)]
    pub fn load_archives(&self, archives: &[Archive]) {
        use rayon::prelude::*;
        tracing::info!(target:"relational","Loading relational for {} archives...",archives.len());
        let old = self.store.len().unwrap_or_default();
        archives
            .par_iter()
            .filter_map(|a| match a {
                Archive::Local(a) => Some(a),
                _ => None,
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
                        self.load(e.path(),graph);
                    }
                }
            });
        tracing::info!(target:"relational","Loaded {} relations", self.store.len().unwrap_or_default() - old);
    }

    fn get_iri(a: ArchiveURIRef, out: &Path, e: &walkdir::DirEntry) -> Option<NamedNode> {
        let parent = e.path().parent()?;
        let parentname = parent.file_name()?.to_str()?;
        let parentname = parentname.rsplit_once('.').map_or(parentname, |(s, _)| s);
        let language = Language::from_rel_path(parentname);
        let parentname = parentname
            .strip_suffix(&format!(".{language}"))
            .unwrap_or(parentname);
        let pathstr = parent.parent()?.to_str()?.strip_prefix(out.to_str()?)?;
        let doc = (a.owned() % pathstr) & (parentname, language);
        Some(doc.to_iri())
    }
}
