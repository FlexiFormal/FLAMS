pub mod spargebra {
    pub use ::spargebra::Query as QueryBuilder;
    pub use oxigraph::sparql::*;
    pub use spargebra::{algebra, term};
}
use sparesults::QueryResultsSerializer;
pub use spargebra::*;

use ftml_ontology::narrative::elements::{ParagraphOrProblemKind, problems::CognitiveDimension};
use ftml_uris::{DocumentElementUri, FtmlUri, SymbolUri};
use std::marker::PhantomData;
use ulo::rdf_types::NamedNode;

pub trait TermPattern {
    fn into_term(self) -> spargebra::term::TermPattern;
}
impl<U: FtmlUri> TermPattern for &'_ U {
    #[inline]
    fn into_term(self) -> spargebra::term::TermPattern {
        spargebra::term::TermPattern::NamedNode(self.to_iri())
    }
}
impl TermPattern for NamedNode {
    #[inline]
    fn into_term(self) -> spargebra::term::TermPattern {
        self.into()
    }
}

pub trait NamedNodePattern {
    fn into_named(self) -> spargebra::term::NamedNodePattern;
}
impl<U: FtmlUri> NamedNodePattern for &'_ U {
    #[inline]
    fn into_named(self) -> spargebra::term::NamedNodePattern {
        spargebra::term::NamedNodePattern::NamedNode(self.to_iri())
    }
}
/*
impl<T> NamedNodePattern for T
where
    T: Into<spargebra::term::NamedNodePattern>,
{
    #[inline]
    fn into_named(self) -> spargebra::term::NamedNodePattern {
        self.into()
    }
}
impl<T> TermPattern for T
where
    T: Into<spargebra::term::TermPattern>,
{
    #[inline]
    fn into_term(self) -> spargebra::term::TermPattern {
        self.into()
    }
}
 */

/*
pub struct Var(pub char);
impl From<Var> for spargebra::term::TermPattern {
    fn from(v: Var) -> Self {
        Self::Variable(ulo::rdf_types::Variable::new_unchecked(v.0))
    }
}
impl From<Var> for spargebra::term::NamedNodePattern {
    fn from(v: Var) -> Self {
        Self::Variable(ulo::rdf_types::Variable::new_unchecked(v.0))
    }
}

pub struct Select<S: TermPattern, P: NamedNodePattern, O: TermPattern> {
    pub subject: S,
    pub pred: P,
    pub object: O,
}
impl<S: TermPattern, P: NamedNodePattern, O: TermPattern> From<Select<S, P, O>>
    for spargebra::Query
{
    fn from(s: Select<S, P, O>) -> Self {
        spargebra::QueryBuilder::Select {
            dataset: None,
            base_iri: None,
            pattern: spargebra::algebra::GraphPattern::Distinct {
                inner: Box::new(spargebra::algebra::GraphPattern::Bgp {
                    patterns: vec![spargebra::term::TriplePattern {
                        subject: s.subject.into_term(),
                        predicate: s.pred.into_named(),
                        object: s.object.into_term(),
                    }],
                }),
            },
        }
        .into()
    }
}
 */

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("{0}")]
    Syntax(#[from] spargebra::SparqlSyntaxError),
    #[error("{0}")]
    Evaluation(#[from] QueryEvaluationError),
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub struct QueryResult<'r>(pub(super) QueryResults<'r>);
impl<'r> AsRef<QueryResults<'r>> for QueryResult<'r> {
    #[inline]
    fn as_ref(&self) -> &QueryResults<'r> {
        &self.0
    }
}
impl<'r> std::ops::Deref for QueryResult<'r> {
    type Target = QueryResults<'r>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'r> QueryResult<'r> {
    /// ### Errors
    pub fn into_json(self) -> Result<String, std::io::Error> {
        use sparesults::QueryResultsFormat;
        let mut buf = Vec::new();
        let ser = QueryResultsSerializer::from_format(QueryResultsFormat::Json);
        match self.0 {
            QueryResults::Boolean(b) => {
                ser.serialize_boolean_to_writer(&mut buf, b)?;
            }
            QueryResults::Solutions(sol) => {
                let mut ser =
                    ser.serialize_solutions_to_writer(&mut buf, sol.variables().to_vec())?;
                for s in sol.flatten() {
                    ser.serialize(s.iter())?;
                }
                ser.finish()?;
            }
            QueryResults::Graph(_) => {
                return Ok(String::new());
                //self.0.write_graph(&mut buf, oxigraph::io::RdfFormat::Turtle)?;
            }
        }
        String::from_utf8(buf).map_err(|e| std::io::Error::other(e.to_string()))
    }

    #[must_use]
    pub fn into_uris<U: FtmlUri>(self) -> RetIter<'r, U> {
        RetIter(
            match self.0 {
                QueryResults::Boolean(_) | QueryResults::Graph(_) => RetIterI::None,
                QueryResults::Solutions(sols) => RetIterI::Sols(sols),
            },
            PhantomData,
        )
    }
}

#[derive(Default)]
enum RetIterI<'r> {
    #[default]
    None,
    Sols(QuerySolutionIter<'r>),
}

pub struct RetIter<'r, U: FtmlUri>(RetIterI<'r>, PhantomData<U>);
impl<U: FtmlUri> Default for RetIter<'_, U> {
    #[inline]
    fn default() -> Self {
        Self(RetIterI::default(), PhantomData)
    }
}

impl<U: FtmlUri> Iterator for RetIter<'_, U> {
    type Item = U;
    fn next(&mut self) -> Option<Self::Item> {
        let RetIterI::Sols(s) = &mut self.0 else {
            return None;
        };
        loop {
            let s = match s.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(s)) => s,
            };
            let [Some(spargebra::term::Term::NamedNode(n))] = s.values() else {
                continue;
            };
            if let Ok(u) = U::from_iri(n.as_ref()) {
                return Some(u);
            }
        }
    }
}

pub struct LOIter<'r> {
    pub(super) inner: QuerySolutionIter<'r>,
}
impl Iterator for LOIter<'_> {
    type Item = (DocumentElementUri, ParagraphOrProblemKind);
    fn next(&mut self) -> Option<Self::Item> {
        use spargebra::term::Term;
        loop {
            let s = match self.inner.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(s)) => s,
            };
            let Some(Term::NamedNode(n)) = s.get("x") else {
                continue;
            };
            let Ok(uri) = DocumentElementUri::from_iri(n.as_ref()) else {
                continue;
            };
            let n = match s.get("R") {
                Some(Term::Literal(l)) if l.value() == "DEF" => {
                    return Some((uri, ParagraphOrProblemKind::Definition));
                }
                Some(Term::Literal(l)) if l.value() == "EX" => {
                    return Some((uri, ParagraphOrProblemKind::Example));
                }
                Some(Term::NamedNode(s)) => s,
                _ => continue,
            };
            let Some(cd) = CognitiveDimension::from_iri(n.as_ref()) else {
                continue;
            };
            let sub =
                matches!(s.get("t"),Some(Term::NamedNode(n)) if n.as_ref() == ulo::ulo::subproblem);
            return Some((
                uri,
                if sub {
                    ParagraphOrProblemKind::SubProblem(cd)
                } else {
                    ParagraphOrProblemKind::Problem(cd)
                },
            ));
        }
    }
}

#[must_use]
pub fn lo_query(s: &SymbolUri, problems: bool) -> ::spargebra::Query {
    if problems {
        crate::sparql!(SELECT DISTINCT ?x ?R ?t WHERE {
            {
                ?x ulo:defines s.
                BIND("DEF" as ?R)
            } UNION {
                ?x ulo:example_for s.
                BIND("EX" as ?R)
            } UNION {
                ?x ulo:has_objective ?b.
                ?b ulo:po_has_symbol s.
                ?b ulo:has_cognitive_dimension ?R.
                ?x rdf:TYPE ?t.
            }
        })
    } else {
        crate::sparql!(SELECT DISTINCT ?x ?R WHERE {
            {
                ?x ulo:defines s.
                BIND("DEF" as ?R)
            } UNION {
                ?x ulo:example_for s.
                BIND("EX" as ?R)
            }
        })
    }
}

#[macro_export]
macro_rules! sparql {
   (SELECT DISTINCT $(?$c:ident)+ WHERE {$($rest:tt)*}) => {
       $crate::triple_store::sparql::QueryBuilder::Select {
           dataset: None,
           base_iri: None,
           pattern: $crate::triple_store::sparql::algebra::GraphPattern::Distinct {
               inner: Box::new(
                   $crate::triple_store::sparql::algebra::GraphPattern::Project{
                       inner: Box::new($crate::sparql!(@PAT $($rest)*)),
                       variables: vec![$(
                           $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($c)).into()
                       ),*]
                   }
               )
           }
       }

   };
   (SELECT $(?$c:ident)* WHERE {$($rest:tt)*}) => {
       $crate::triple_store::sparql::QueryBuilder::Select {
           dataset: None,
           base_iri: None,
           pattern:
           $crate::triple_store::sparql::algebra::GraphPattern::Project{
               inner: Box::new($crate::sparql!(@PAT $($rest)*)),
               variables: vec![$(
                   $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($c)).into()
               ),*]
           }
       }
   };
   (@PAT {$($first:tt)*} UNION $($rest:tt)*) => {
       $crate::triple_store::sparql::algebra::GraphPattern::Union {
           left:Box::new($crate::sparql!(@TRIP {} {} $($first)*).into()),
           right:Box::new($crate::sparql!(@PAT $($rest)*)),
       }
   };
   (@PAT {$($rest:tt)*}) => {
       $crate::sparql!(@TRIP {} {} $($rest)*)
   };
   (@PAT $($rest:tt)*) => {
       $crate::sparql!(@TRIP {} {} $($rest)*)
   };
   (@TRIP {$($trips:tt)*} {$($binds:tt)*} ?$v:ident $path:ident:$pred:ident ?$v2:ident . $($rest:tt)*) => {
       $crate::sparql!(@TRIP {
           $($trips)*
           (
               $crate::triple_store::sparql::term::TriplePattern {
                   subject: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v)).into(),
                   predicate: ::ulo::$path::$pred.into_owned().into(),
                   object: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v2)).into(),
               }
           )
       } { $($binds)* } $($rest)* )
   };
   (@TRIP {$($trips:tt)*} {$($binds:tt)*} ?$v:ident $path:ident:$pred:ident $p2:ident:$t:ident . $($rest:tt)*) => {
       $crate::sparql!(@TRIP {
           $($trips)*
           (
               $crate::triple_store::sparql::term::TriplePattern {
                   subject: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v)).into(),
                   predicate: ::ulo::$path::$pred.into_owned().into(),
                   object: ::ulo::$p2::$t.into_owned().into(),
               }
           )
       } { $($binds)* } $($rest)* )
   };
   (@TRIP {$($trips:tt)*} {$($binds:tt)*} $node:ident $path:ident:$pred:ident ?$v2:ident . $($rest:tt)*) => {
       $crate::sparql!(@TRIP {
           $($trips)*
           (
               $crate::triple_store::sparql::term::TriplePattern {
                   subject: $crate::triple_store::sparql::TermPattern::into_term($node).into(),
                   predicate: ::ulo::$path::$pred.into_owned().into(),
                   object: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v2)).into(),
               }
           )
       } { $($binds)* } $($rest)* )
   };
   (@TRIP {$($trips:tt)*} {$($binds:tt)*} ?$v:ident $path:ident:$pred:ident $node:ident . $($rest:tt)*) => {
       $crate::sparql!(@TRIP {
           $($trips)*
           (
               $crate::triple_store::sparql::term::TriplePattern {
                   subject: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v)).into(),
                   predicate: ::ulo::$path::$pred.into_owned().into(),
                   object:$crate::triple_store::sparql::TermPattern::into_term($node),
               }
           )
       } { $($binds)* } $($rest)* )
   };
   (@TRIP {$($trips:tt)*} {$($binds:tt)*} BIND($name:literal as ?$v:ident) $($rest:tt)*) => {
       $crate::sparql!(@TRIP { $($trips)* } { $($binds)* ($name,$v) } $($rest)* )
   };
   (@TRIP {} {}) => {
       compile_error!("pattern has no body")
   };
   (@TRIP { $(($e:expr))+ } {}) => {
       $crate::triple_store::sparql::algebra::GraphPattern::Bgp {
           patterns:vec![ $($e),* ]
       }
   };
   (@TRIP {$( ($e:expr) )+} {($name:literal,$v:ident) $($rest:tt)*}) => {
       $crate::triple_store::sparql::algebra::GraphPattern::Extend {
           inner:Box::new(
               $crate::sparql!(@TRIP { $(($e))+ } { $($rest)* })
           ),
           variable: $crate::triple_store::sparql::term::Variable::new_unchecked(stringify!($v)).into(),
           expression: $crate::triple_store::sparql::algebra::Expression::Literal($name.into())
       }
   };
}
