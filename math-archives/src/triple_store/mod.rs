pub mod sparql;

use ftml_uris::{ArchiveUri, DocumentUri, FtmlUri, Language, SymbolUri, UriPath, UriWithPath};
use std::{path::Path, str::FromStr};
use ulo::rdf_types::{Quad, Triple};

use crate::{
    Archive, LocallyBuilt, MathArchive,
    utils::{AsyncEngine, path_ext::PathExt},
};

pub struct RDFStore {
    store: oxigraph::store::Store,
}

#[derive(thiserror::Error, Debug)]
pub enum SparqlError {
    #[error("sparql syntax error: {0}")]
    Syntax(#[from] sparql::SparqlSyntaxError),
    #[error("sparql query error: {0}")]
    Query(#[from] sparql::QueryError),
}

impl RDFStore {
    #[cfg(feature = "rocksdb")]
    #[must_use]
    pub fn new(path: &Path) -> Self {
        let _ = std::fs::remove_dir_all(path);
        let store = oxigraph::store::Store::open(path).expect("failed to open rdf database");
        let _ = store.clear();
        let mut loader = store.bulk_loader();
        loader
            .load_quads(ulo::ulo::QUADS.iter().copied())
            .expect("error loading ulo base ontology; this is a bug.");
        loader
            .commit()
            .expect("error loading ulo base ontology; this is a bug.");
        Self { store }
    }

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
        let mut loader = self.store.bulk_loader();
        let _ = loader.load_quads(iter);
        let _ = loader.commit();
    }

    #[must_use]
    pub fn los<E: AsyncEngine>(&self, s: &SymbolUri, problems: bool) -> Option<sparql::LOIter<'_>> {
        let q = sparql::lo_query(s, problems);
        self.query::<E>(q).ok().and_then(|s| {
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
                    .with_prefix("dc", "http://purl.org/dc/terms")
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
    /// ### Panics
    pub fn query_str<E: AsyncEngine>(
        &self,
        s: impl AsRef<str>,
    ) -> Result<sparql::QueryResult<'_>, SparqlError> {
        /*let mut query_str = String::from(
            r"PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
          PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
          PREFIX dc: <http://purl.org/dc/terms#>
          PREFIX ulo: <http://mathhub.info/ulo#>
      ",
        );
        query_str.push_str(s.as_ref());*/
        let query = spargebra::SparqlParser::new()
            .with_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .expect("bug")
            .with_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#")
            .expect("bug")
            .with_prefix("dc", "http://purl.org/dc/terms#")
            .expect("bug")
            .with_prefix("ulo", "http://mathhub.info/ulo#")
            .expect("bug")
            .parse_query(s.as_ref())?;
        //let query: oxigraph::sparql::Query = query_str.as_str().try_into()?;
        self.query::<E>(query).map_err(Into::into)
    }

    /// ### Errors
    pub fn query<E: AsyncEngine>(
        &self,
        q: spargebra::Query,
    ) -> Result<sparql::QueryResult<'_>, sparql::QueryError> {
        //normalize(&mut q);

        let token = oxigraph::sparql::CancellationToken::new();
        let tk = token.clone();
        E::exec_after(std::time::Duration::from_secs(5), move || tk.cancel());
        /*
        let token = oxigraph::sparql::CancellationToken::new();
        let tk = token.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(5));
            tk.cancel();
        });
         */
        let mut q = oxigraph::sparql::SparqlEvaluator::new()
            .with_cancellation_token(token)
            .for_query(q);
        q.dataset_mut().set_default_graph_as_union();
        Ok(q.on_store(&self.store).execute().map(sparql::QueryResult)?)
    }

    pub(crate) fn load(&self, path: &Path, graph: ulo::rdf_types::NamedNode) {
        let Ok(file) = std::fs::File::open(path) else {
            tracing::error!("Failed to open file {}", path.display());
            return;
        };
        let buf = std::io::BufReader::new(file);
        let mut loader = self.store.bulk_loader();
        let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle)
            .with_default_graph(graph)
            .for_reader(buf);
        let _ = loader.load_quads(reader.filter_map(Result::ok));
        let _ = loader.commit();
    }

    pub fn load_archives(&self, archives: &[Archive]) {
        use rayon::prelude::*;
        tracing::info_span!(target:"relational","loading relational","Loading relational for {} archives...",archives.len()).in_scope(move || {
            let old = self.store.len().unwrap_or_default();
            let all_files = archives
                .par_iter()
                .filter_map(|a| match a {
                    Archive::Local(a) => Some(a),
                    Archive::Ext(..) => None,
                })
                .filter_map(|a| {
                    let out = a.out_dir();
                    if out.exists() && out.is_dir() {
                        Some(
                            walkdir::WalkDir::new(out)
                                .into_iter()
                                .filter_map(Result::ok)
                                .filter(|entry| entry.file_name() == "index.ttl")
                                .filter_map(|e| {
                                    let graph = Self::get_iri(a.uri(), out, &e)?;
                                    Some((e.into_path(), graph))
                                })
                                .collect::<Vec<_>>(),
                        )
                        /*for e in walkdir::WalkDir::new(out)
                            .into_iter()
                            .filter_map(Result::ok)
                            .filter(|entry| entry.file_name() == "index.ttl")
                        {
                            let Some(graph) = Self::get_iri(a.uri(), out, &e) else {
                                return None;
                            };
                            Some((e.into_path(),graph))
                            //self.load(e.path(), graph);
                        }*/
                    } else {
                        None
                    }
                })
                .collect_vec_list();
            for (i,path_graph) in all_files.into_iter().flatten().enumerate() {//.flatten().flatten().enumerate() {
                tracing::info!("Loading {}",i+1);
                let mut loader = self.store.bulk_loader();
                for (path,graph) in path_graph {
                    let Ok(file) = std::fs::File::open(&path) else {
                        tracing::error!("Failed to open file {}", path.display());
                        continue;
                    };
                    let buf = std::io::BufReader::new(file);
                    let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle)
                        .with_default_graph(graph)
                        .for_reader(buf);
                    let _ = loader.load_quads(reader.filter_map(Result::ok));
                }
                let _ = loader.commit();
            }

            tracing::info!(target:"relational","Loaded {} relations", self.store.len().unwrap_or_default() - old);
        });
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
        let mut loader = store.bulk_loader();
        loader
            .load_quads(ulo::ulo::QUADS.iter().copied())
            .expect("error loading ulo base ontology; this is a bug.");
        loader
            .commit()
            .expect("error loading ulo base ontology; this is a bug.");
        Self { store }
    }
}

fn normalize(query: &mut spargebra::Query) {
    fn norm(n: &mut ulo::rdf_types::NamedNode) {
        if n.as_str().starts_with("https:") {
            *n = ulo::rdf_types::NamedNode::new_unchecked(format!("http:{}", &n.as_str()[6..]));
        }
    }
    fn norm_i(n: &mut oxiri::Iri<String>) {
        if n.starts_with("https:") {
            *n = oxiri::Iri::parse_unchecked(format!("http:{}", &n.as_str()[6..]));
        }
    }
    fn dset(d: &mut ::spargebra::algebra::QueryDataset) {
        for n in d.default.iter_mut() {
            norm(n);
        }
        if let Some(n) = &mut d.named {
            for n in n.iter_mut() {
                norm(n);
            }
        }
    }
    fn termpat(p: &mut spargebra::term::TermPattern) {
        use spargebra::term::TermPattern as TP;
        match p {
            TP::BlankNode(_) | TP::Literal(_) | TP::Variable(_) => (),
            TP::NamedNode(n) => norm(n),
            TP::Triple(t) => trip(t),
        }
    }
    fn nnpat(p: &mut spargebra::term::NamedNodePattern) {
        if let spargebra::term::NamedNodePattern::NamedNode(n) = p {
            norm(n);
        }
    }
    fn trip(p: &mut spargebra::term::TriplePattern) {
        termpat(&mut p.subject);
        termpat(&mut p.object);
        nnpat(&mut p.predicate);
    }
    fn expr(e: &mut spargebra::algebra::Expression) {
        use spargebra::algebra::Expression as Exp;
        match e {
            Exp::NamedNode(n) => norm(n),
            Exp::Or(a, b)
            | Exp::Add(a, b)
            | Exp::And(a, b)
            | Exp::Divide(a, b)
            | Exp::Equal(a, b)
            | Exp::SameTerm(a, b)
            | Exp::Greater(a, b)
            | Exp::GreaterOrEqual(a, b)
            | Exp::Less(a, b)
            | Exp::Subtract(a, b)
            | Exp::Multiply(a, b)
            | Exp::LessOrEqual(a, b) => {
                expr(a);
                expr(b);
            }
            Exp::In(a, v) => {
                expr(a);
                for e in v {
                    expr(e);
                }
            }
            Exp::UnaryPlus(e) | Exp::UnaryMinus(e) | Exp::Not(e) => expr(e),
            Exp::Exists(p) => pat(p),
            Exp::If(a, b, c) => {
                expr(a);
                expr(b);
                expr(c);
            }
            Exp::Coalesce(e) | Exp::FunctionCall(_, e) => {
                for e in e {
                    expr(e)
                }
            }
            Exp::Bound(_) | Exp::Literal(_) | Exp::Variable(_) => (),
        }
    }
    fn ppexpr(e: &mut spargebra::algebra::PropertyPathExpression) {
        use spargebra::algebra::PropertyPathExpression as Exp;
        match e {
            Exp::NamedNode(n) => norm(n),
            Exp::Reverse(n) | Exp::ZeroOrMore(n) | Exp::OneOrMore(n) | Exp::ZeroOrOne(n) => {
                ppexpr(n)
            }
            Exp::Sequence(a, b) | Exp::Alternative(a, b) => {
                ppexpr(a);
                ppexpr(b);
            }
            Exp::NegatedPropertySet(v) => {
                for e in v {
                    norm(e);
                }
            }
        }
    }
    fn pat(p: &mut spargebra::algebra::GraphPattern) {
        use spargebra::algebra::GraphPattern as GP;
        match p {
            GP::Bgp { patterns } => {
                for p in patterns {
                    trip(p)
                }
            }
            GP::Distinct { inner } => pat(inner),
            GP::Extend {
                inner, expression, ..
            } => {
                pat(inner);
                expr(expression);
            }
            GP::Filter { expr: exp, inner } => {
                expr(exp);
                pat(inner);
            }
            GP::Graph { name, inner } | GP::Service { name, inner, .. } => {
                nnpat(name);
                pat(inner);
            }
            GP::Group {
                inner, aggregates, ..
            } => {
                pat(inner);
                for (_, e) in aggregates {
                    if let spargebra::algebra::AggregateExpression::FunctionCall {
                        expr: e, ..
                    } = e
                    {
                        expr(e);
                    }
                }
            }
            GP::Join { left, right }
            | GP::Lateral { left, right }
            | GP::Minus { left, right }
            | GP::Union { left, right } => {
                pat(left);
                pat(right);
            }
            GP::LeftJoin {
                left,
                right,
                expression,
            } => {
                pat(left);
                pat(right);
                if let Some(e) = expression {
                    expr(e);
                }
            }
            GP::OrderBy { inner, expression } => {
                pat(inner);
                for e in expression {
                    match e {
                        spargebra::algebra::OrderExpression::Asc(e)
                        | spargebra::algebra::OrderExpression::Desc(e) => expr(e),
                    }
                }
            }
            GP::Path {
                subject,
                path,
                object,
            } => {
                termpat(subject);
                termpat(object);
                ppexpr(path);
            }
            GP::Project { inner, .. } | GP::Reduced { inner } | GP::Slice { inner, .. } => {
                pat(inner);
            }
            GP::Values { .. } => (),
        }
    }
    use spargebra::Query as Q;
    match query {
        Q::Ask {
            dataset,
            pattern,
            base_iri,
        }
        | Q::Describe {
            dataset,
            pattern,
            base_iri,
        }
        | Q::Select {
            dataset,
            pattern,
            base_iri,
        } => {
            if let Some(d) = dataset {
                dset(d);
            }
            if let Some(iri) = base_iri {
                norm_i(iri);
            }
            pat(pattern);
        }
        Q::Construct {
            template,
            dataset,
            pattern,
            base_iri,
        } => {
            if let Some(d) = dataset {
                dset(d);
            }
            if let Some(iri) = base_iri {
                norm_i(iri);
            }
            for t in template {
                trip(t);
            }
            pat(pattern);
        }
    }
}
