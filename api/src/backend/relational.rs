use std::collections::BTreeSet;
use std::fmt::Debug;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::str::FromStr;
use oxigraph::sparql::{Query, QueryResults};
use oxrdfio::RdfFormat;
use tracing::instrument;
use immt_core::narration::Language;
use immt_core::ontology::rdf::terms::{NamedNode, Quad, Subject, Term, Triple};
use immt_core::uris::{DocumentURI, NarrDeclURI, PathURI, SymbolURI};
use crate::backend::archives::{Archive, Storage};
use crate::backend::manager::ArchiveManager;

pub struct QueryResult(QueryResults);
impl QueryResult {
    pub fn symbol_iter(self) -> Option<impl Iterator<Item=SymbolURI>> {
        match self.0 {
            QueryResults::Solutions(sols) => Some(sols.flat_map(|s|
                s.ok().map(|s| {
                    if let Some(Some(Term::NamedNode(nn))) = s.values().first() {
                        SymbolURI::from_str(nn.as_str()).ok()
                    } else { None }
                }).flatten()
            )),
            _ => None
        }
    }
    pub fn doc_elem_iter(self) -> Option<impl Iterator<Item=NarrDeclURI>> {
        match self.0 {
            QueryResults::Solutions(sols) => Some(sols.flat_map(|s|
                s.ok().map(|s| {
                    if let Some(Some(Term::NamedNode(nn))) = s.values().first() {
                        NarrDeclURI::from_str(nn.as_str()).ok()
                    } else { None }
                }).flatten()
            )),
            _ => None
        }
    }
    pub fn resolve(self) -> ResolvedQueryResult {
        match self.0 {
            QueryResults::Boolean(b) => ResolvedQueryResult::Bool(b),
            QueryResults::Solutions(solutions) => {
                let solutions: Vec<Solution> = solutions
                    .filter_map(|s| s.ok().map(|solution| {
                        Solution {
                            bindings: solution
                                .into_iter()
                                .map(|(var, term)| (var.to_string(), SerializableTerm::from(term)))
                                .collect(),
                        }
                    }))
                    .collect();
                ResolvedQueryResult::Solutions(solutions)
            }
            QueryResults::Graph(triples) => {
                let triples: Vec<SerializableTriple> = triples
                    .filter_map(|t| t.ok().map(|triple| SerializableTriple {
                        subject: SerializableTerm::from(triple.subject),
                        predicate: SerializableTerm::from(triple.predicate),
                        object: SerializableTerm::from(triple.object),
                    }))
                    .collect();
                ResolvedQueryResult::Graph(triples)
            }
        }
    }
}
impl AsRef<QueryResults> for QueryResult {
    fn as_ref(&self) -> &QueryResults {
        &self.0
    }
}
impl AsMut<QueryResults> for QueryResult {
    fn as_mut(&mut self) -> &mut QueryResults { &mut self.0 }
}
impl From<QueryResults> for QueryResult {
    fn from(q: QueryResults) -> Self { Self(q) }
}

#[derive(Debug, serde::Serialize, serde::Deserialize,Clone)]
pub enum ResolvedQueryResult {
    Bool(bool),
    Solutions(Vec<Solution>),
    Graph(Vec<SerializableTriple>)
}
impl ResolvedQueryResult {
    pub fn results(&self) -> usize {
        match self {
            ResolvedQueryResult::Bool(_) => 0,
            ResolvedQueryResult::Solutions(solutions) => solutions.len(),
            ResolvedQueryResult::Graph(triples) => triples.len(),
        }
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize,Clone)]
pub struct SerializableTriple {
    subject: SerializableTerm,
    predicate: SerializableTerm,
    object: SerializableTerm,
}

#[derive(Debug, serde::Serialize, serde::Deserialize,Clone)]
pub enum SerializableTerm {
    NamedNode(String),
    BlankNode(String),
    Literal(String, Option<String>, String),
    Triple(String,String,String)
}
impl From<Subject> for SerializableTerm {
    fn from(subject: Subject) -> Self {
        match subject {
            Subject::NamedNode(nn) => SerializableTerm::NamedNode(nn.into_string()),
            Subject::BlankNode(bn) => SerializableTerm::BlankNode(bn.into_string()),
            Subject::Triple(t) => SerializableTerm::Triple(
                t.subject.to_string(),
                t.predicate.to_string(),
                t.object.to_string()
            )
        }
    }
}

impl From<NamedNode> for SerializableTerm {
    fn from(nn: NamedNode) -> Self {
        SerializableTerm::NamedNode(nn.into_string())
    }
}
impl From<&NamedNode> for SerializableTerm {
    fn from(nn: &NamedNode) -> Self {
        SerializableTerm::NamedNode(nn.to_string())
    }
}

impl From<Term> for SerializableTerm {
    fn from(term: Term) -> Self {
        match term {
            Term::NamedNode(nn) => nn.into(),
            Term::BlankNode(bn) => SerializableTerm::BlankNode(bn.into_string()),
            Term::Literal(l) => SerializableTerm::Literal(
                l.value().to_string(),
                l.language().map(|lang| lang.to_string()),
                l.datatype().to_string(),
            ),
            Term::Triple(t) => SerializableTerm::Triple(
                t.subject.to_string(),
                t.predicate.to_string(),
                t.object.to_string(),
            ),
        }
    }
}

impl From<&Term> for SerializableTerm {
    fn from(term: &Term) -> Self {
        match term {
            Term::NamedNode(nn) => nn.into(),
            Term::BlankNode(bn) => SerializableTerm::BlankNode(bn.to_string()),
            Term::Literal(l) => SerializableTerm::Literal(
                l.value().to_string(),
                l.language().map(|lang| lang.to_string()),
                l.datatype().to_string(),
            ),
            Term::Triple(t) => SerializableTerm::Triple(
                t.subject.to_string(),
                t.predicate.to_string(),
                t.object.to_string(),
            ),
        }
    }
}


impl std::fmt::Display for SerializableTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializableTerm::NamedNode(iri) => write!(f, "<{}>", iri),
            SerializableTerm::BlankNode(id) => write!(f, "_:{}", id),
            SerializableTerm::Literal(value, lang, datatype) => {
                write!(f, "\"{}\"", value)?;
                if let Some(lang) = lang {
                    write!(f, "@{}", lang)?;
                }
                write!(f, "^^<{}>", datatype)
            }
            SerializableTerm::Triple(s,p,o) => write!(f, "{} {} {}", s,p,o)
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize,Clone)]
pub struct Solution {
    bindings: Vec<(String, SerializableTerm)>,
}

impl std::fmt::Display for ResolvedQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedQueryResult::Bool(b) => write!(f, "Boolean result: {}", b),
            ResolvedQueryResult::Solutions(solutions) => {
                writeln!(f, "Solutions:")?;
                for (i, solution) in solutions.iter().enumerate() {
                    writeln!(f, "  Solution {}:", i + 1)?;
                    for (var, term) in &solution.bindings {
                        writeln!(f, "    {}: {}", var, term)?;
                    }
                }
                Ok(())
            }
            ResolvedQueryResult::Graph(triples) => {
                writeln!(f, "Graph:")?;
                for triple in triples {
                    writeln!(f, "  {} {} {}", triple.subject, triple.predicate, triple.object)?;
                }
                Ok(())
            }
        }
    }
}



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
    #[inline]
    pub fn num_relations(&self) -> usize {
        self.store.len().unwrap_or_default()
    }
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
                .with_prefix("rdf","http://www.w3.org/1999/02/22-rdf-syntax-ns").unwrap()
                .with_prefix("ulo","http://mathhub.info/ulo").unwrap()
                .with_prefix("dc","http://purl.org/dc/elements/1.1").unwrap()
                .serialize_to_write(writer);
            for t in iter {
                writer.write_triple(&t).unwrap_or_else(|f| panic!("{f}"));
            }
            let _ = writer.finish();
        }
    }

    pub fn query_str(&self, s:impl AsRef<str>) -> Option<QueryResult> {
        let mut query_str = String::from(r#"PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            PREFIX dc: <http://purl.org/dc/elements/1.1#>
            PREFIX ulo: <http://mathhub.info/ulo#>
        "#);
        query_str.push_str(s.as_ref());
        let mut query:Query = query_str.as_str().try_into().ok()?;
        query.dataset_mut().set_default_graph_as_union();
        self.store.query(query).ok().map(|i| i.into())
    }
    pub fn query(&self, mut q:Query) -> Option<QueryResult> {
        q.dataset_mut().set_default_graph_as_union();
        self.store.query(q).ok().map(|i| i.into())
    }


    #[instrument(level = "info", name = "relational", target="relational", skip_all)]
    pub fn load_archives(&self,archives:&[Archive]) {
        use rayon::prelude::*;
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