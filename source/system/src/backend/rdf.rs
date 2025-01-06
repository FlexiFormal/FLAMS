use immt_ontology::languages::Language;
use immt_ontology::narration::exercises::CognitiveDimension;
use immt_ontology::narration::LOKind;
use immt_ontology::rdf::ontologies::ulo2;
use immt_ontology::rdf::{NamedNode, Quad, Triple};
use immt_ontology::uris::{ArchiveURIRef, DocumentElementURI, DocumentURI, PathURITrait, SymbolURI, URIOrRefTrait, URIRefTrait, URITrait};
use oxigraph::sparql::QuerySolutionIter;
use oxrdfio::RdfFormat;
use std::fmt::{Debug, Display};
use std::io::{BufReader, BufWriter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;
use std::string::FromUtf8Error;
use tracing::instrument;

pub mod sparql {
    use immt_ontology::{rdf::ontologies::{self, ulo2}, uris::{SymbolURI, URIOrRefTrait}};
    pub use oxigraph::sparql::*;
    pub use spargebra::{SparqlSyntaxError,Query as QueryBuilder};
    pub struct Var(pub char);
    impl From<Var> for spargebra::term::TermPattern {
        fn from(v:Var) -> Self {
            Self::Variable(immt_ontology::rdf::Variable::new_unchecked(v.0))
        }
    }
    impl From<Var> for spargebra::term::NamedNodePattern {
        fn from(v:Var) -> Self {
            Self::Variable(immt_ontology::rdf::Variable::new_unchecked(v.0))
        }
    }
    pub trait TermPattern : Into<spargebra::term::TermPattern> {}
    impl TermPattern for Var {}
    pub trait NamedNodePattern: Into<spargebra::term::NamedNodePattern> {}
    impl NamedNodePattern for Var {}
    impl NamedNodePattern for super::NamedNode {}
    impl TermPattern for super::NamedNode {}

    pub struct Select<S:TermPattern,P:NamedNodePattern,O:TermPattern>{
        pub subject:S,
        pub pred:P,
        pub object:O
    }
    impl<S:TermPattern,P:NamedNodePattern,O:TermPattern> From<Select<S,P,O>> for Query {
        fn from(s:Select<S,P,O>) -> Self {
            QueryBuilder::Select {
                dataset:None,
                base_iri:None,
                pattern:spargebra::algebra::GraphPattern::Distinct { inner: Box::new(
                    spargebra::algebra::GraphPattern::Bgp {
                        patterns: vec![
                            spargebra::term::TriplePattern {
                                subject: s.subject.into(),
                                predicate: s.pred.into(),
                                object: s.object.into(),
                            }
                        ]
                    }
                ) }
            }.into()
        }
    }

    pub fn lo_query(s:&SymbolURI,exercises:bool) -> Query {
        /* 
SELECT DISTINCT ?x ?R ?t ?s WHERE {
  { 
    ?x ulo:defines ?s.
    BIND("DEF" as ?R) 
  } UNION { 
    ?x ulo:example-for ?s.
    BIND("EX" as ?R) 
  } UNION {
    ?x ulo:objective ?bn .
    ?bn ulo:po-symbol ?s .
    ?bn ulo:cognitive-dimension ?R .
    ?x rdf:type ?t.
  }
        */
        use spargebra::{algebra::{GraphPattern,Expression},term::TriplePattern};
        #[inline]
        fn var(s:&'static str) -> spargebra::term::Variable {
            spargebra::term::Variable::new_unchecked(s)
        }
        let iri = s.to_iri();

        let defs_and_exs = GraphPattern::Union {
            left:Box::new(GraphPattern::Extend { 
                inner: Box::new(GraphPattern::Bgp { 
                    patterns: vec![
                        TriplePattern { 
                            subject: var("x").into(), 
                            predicate: ulo2::DEFINES.into_owned().into(), 
                            object: iri.clone().into() 
                        }
                    ] 
                }), 
                variable: var("R").into(), 
                expression: Expression::Literal(
                    "DEF".into()
                )
            }),
            right:Box::new(GraphPattern::Extend {
                inner: Box::new(GraphPattern::Bgp { 
                    patterns: vec![
                        TriplePattern { 
                            subject: var("x").into(), 
                            predicate: ulo2::EXAMPLE_FOR.into_owned().into(), 
                            object: iri.clone().into() 
                        }
                    ] 
                }), 
                variable: var("R").into(), 
                expression: Expression::Literal(
                    "EX".into()
                )
            })
        };

        QueryBuilder::Select {
            dataset:None,
            base_iri:None,
            pattern: GraphPattern::Distinct {
                inner: Box::new(GraphPattern::Project {
                    inner: Box::new(if exercises {
                        GraphPattern::Union {
                            left: Box::new(defs_and_exs),
                            right: Box::new(GraphPattern::Bgp { patterns: vec![
                                TriplePattern { 
                                    subject: var("x").into(), 
                                    predicate: ulo2::OBJECTIVE.into_owned().into(), 
                                    object: var("bn").into() 
                                }, 
                                TriplePattern { 
                                    subject: var("bn").into(), 
                                    predicate: ulo2::POSYMBOL.into_owned().into(), 
                                    object: iri.into() 
                                }, 
                                TriplePattern { 
                                    subject: var("bn").into(), 
                                    predicate: ulo2::COGDIM.into_owned().into(), 
                                    object: var("R").into() 
                                }, 
                                TriplePattern { 
                                    subject: var("x").into(), 
                                    predicate: ontologies::rdf::TYPE.into_owned().into(), 
                                    object: var("t").into() 
                                }
                            ] })
                        }
                    } else {defs_and_exs}),
                    variables:if(exercises) {
                        vec![var("x").into(),var("R").into(),var("t").into()]
                    } else {
                        vec![var("x").into(),var("R").into()]
                    }
                })
            }
        }.into()
    }
}
use sparql::{SparqlSyntaxError,EvaluationError, Query, QueryResults};

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

    #[must_use]
    pub fn into_uris<U:URITrait>(self) -> RetIter<U> {
        RetIter(match self.0 {
            QueryResults::Boolean(_) | QueryResults::Graph(_) => RetIterI::None,
            QueryResults::Solutions(sols) => RetIterI::Sols(sols)
        },PhantomData)
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

#[derive(Default)]
enum RetIterI {
    #[default]
    None,
    Sols(QuerySolutionIter)
}

pub struct RetIter<U:URITrait>(RetIterI,PhantomData<U>);
impl<U:URITrait> Default for RetIter<U> {
    #[inline]
    fn default() -> Self {
        Self(RetIterI::default(),PhantomData)
    }
}

impl<U:URITrait> Iterator for RetIter<U> {
    type Item = U;
    fn next(&mut self) -> Option<Self::Item> {
        let RetIterI::Sols(s) = &mut self.0 else {return None};
        loop {
            let s = match s.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(s)) => s
            };
            //println!("Solution: {s:?}");
            let [Some(immt_ontology::rdf::RDFTerm::NamedNode(n))] = s.values() else {continue};
            let s = n.as_str();
            //println!("Iri: {s}");
            let s = immt_utils::escaping::IRI_ESCAPE.unescape(&s).to_string();
            if let Ok(s) = s.parse() {
                //println!("Parsed: {s}");
                return Some(s)
            }
        }
    }
}

pub struct LOIter {
    inner: QuerySolutionIter
}
impl Iterator for LOIter {
    type Item = (DocumentElementURI,LOKind);
    fn next(&mut self) -> Option<Self::Item> {
        use immt_ontology::rdf::RDFTerm;
        loop {
            let s = match self.inner.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(s)) => s
            };
            let Some(RDFTerm::NamedNode(n)) = s.get("x") else {continue};
            let Ok(uri) = immt_utils::escaping::IRI_ESCAPE.unescape(&n.as_str()).to_string().parse() else {continue};
            let n = match s.get("R") {
                Some(RDFTerm::Literal(l)) if l.value() == "DEF" =>
                    return Some((uri,LOKind::Definition)),
                Some(RDFTerm::Literal(l)) if l.value() == "EX" =>
                    return Some((uri,LOKind::Example)),
                Some(RDFTerm::NamedNode(s)) => s,
                _ => continue,
            };
            let cd = match n.as_ref() {
                ulo2::REMEMBER => CognitiveDimension::Remember,
                ulo2::UNDERSTAND => CognitiveDimension::Understand,
                ulo2::APPLY => CognitiveDimension::Apply,
                ulo2::ANALYZE => CognitiveDimension::Analyze,
                ulo2::EVALUATE => CognitiveDimension::Evaluate,
                ulo2::CREATE => CognitiveDimension::Create,
                _ => continue
            };
            let sub = matches!(s.get("t"),Some(RDFTerm::NamedNode(n)) if n.as_ref() == ulo2::SUBPROBLEM);
            return Some((uri,if sub {LOKind::SubExercise(cd)} else {LOKind::Exercise(cd)}))
        }
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
    pub fn clear(&self) {
        self.store.clear();
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

    pub fn los(&self,s:&SymbolURI,exercises:bool) -> Option<LOIter> {
        let q = sparql::lo_query(s,exercises);
        self.query(q).ok().and_then(|s|
            if let QueryResults::Solutions(s) = s.0  {
                Some(LOIter {inner: s})
            } else {None}
        )
    }

    pub fn export(&self, iter: impl Iterator<Item = Triple>, p: &Path, uri: &DocumentURI) {
        if let Ok(file) = std::fs::File::create(p) {
            let writer = BufWriter::new(file);
            let iri = uri.as_path().to_iri();
            let ns = iri.as_str();
            //let ns = ns.strip_prefix("<").unwrap_or(&ns);
            //let ns = ns.strip_suffix(">").unwrap_or(ns);
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
        let doc = ((a.owned() % pathstr).ok()? & (parentname, language)).ok()?;
        Some(doc.to_iri())
    }
}