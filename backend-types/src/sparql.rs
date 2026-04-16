#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SparqlResult {
    Boolean {
        head: SparqlResultsHead,
        boolean: bool,
    },
    Bindings {
        head: SparqlResultsHead,
        results: SparqlResultBindings,
    },
}

impl From<bool> for SparqlResult {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean {
            head: SparqlResultsHead::default(),
            boolean: value,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparqlResultsHead {
    #[cfg_attr(feature = "serde", serde(default))]
    pub vars: Vec<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub link: Vec<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub version: Option<String>,
}

#[cfg(feature = "rdf")]
impl From<&[ulo::rdf_types::Variable]> for SparqlResultsHead {
    fn from(value: &[ulo::rdf_types::Variable]) -> Self {
        Self {
            vars: value.iter().map(|s| s.as_ref().to_string()).collect(),
            link: Vec::new(),
            version: Some("1.2".to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparqlResultBindings {
    bindings: Vec<rustc_hash::FxHashMap<String, SparqlResultTerm>>,
}

#[cfg(feature = "rdf")]
impl SparqlResultBindings {
    pub fn from_iter<I, J: IntoIterator<Item = I>>(iter: J, decode_uris: bool) -> Self
    where
        for<'a> &'a I:
            IntoIterator<Item = (&'a ulo::rdf_types::Variable, &'a ulo::rdf_types::RDFTerm)>,
    {
        Self {
            bindings: iter
                .into_iter()
                .map(|v| {
                    v.into_iter()
                        .map(|(v, t)| {
                            (
                                v.as_str().to_string(),
                                SparqlResultTerm::from_term(t, decode_uris),
                            )
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SparqlResultTerm {
    #[cfg_attr(feature = "serde", serde(rename = "uri"))]
    Iri { value: String },
    #[cfg_attr(feature = "serde", serde(rename = "literal"))]
    Literal {
        value: String,
        #[cfg_attr(feature = "serde", serde(rename = "xml:lang", default))]
        lang: Option<String>,
        #[cfg_attr(feature = "serde", serde(rename = "its:dir", default))]
        base_direction: Option<String>,
        #[cfg_attr(feature = "serde", serde(default))]
        datatype: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "bnode"))]
    BlankNode { value: String },
    #[cfg_attr(feature = "serde", serde(rename = "triple"))]
    Triple { value: Box<SparqlResultTriple> },
}

impl SparqlResultTerm {
    #[cfg(feature = "rdf")]
    fn from_term(value: &ulo::rdf_types::RDFTerm, decode_uris: bool) -> Self {
        use ulo::rdf_types::RDFTerm as T;
        match value {
            T::NamedNode(r) => Self::from_node(r, decode_uris),
            T::Literal(lit) => Self::Literal {
                value: lit.value().to_string(),
                lang: lit.language().as_ref().map(|s| (*s).to_string()),
                base_direction: None,
                datatype: None,
            },
            T::BlankNode(bn) => Self::BlankNode {
                value: bn.as_str().to_string(),
            },
            T::Triple(t) => Self::Triple {
                value: Box::new(SparqlResultTriple {
                    subject: Self::from_subject(&t.subject, decode_uris),
                    predicate: Self::from_node(&t.predicate, decode_uris),
                    object: Self::from_term(&t.object, decode_uris),
                }),
            },
        }
    }
    #[cfg(feature = "rdf")]
    fn from_node(r: &ulo::rdf_types::NamedNode, decode_uris: bool) -> Self {
        use ftml_uris::{FtmlUri, Uri};
        Self::Iri {
            value: if decode_uris {
                Uri::from_iri(r.as_ref())
                    .map_or_else(|_| r.as_str().to_string(), |uri| uri.to_string())
            } else {
                r.as_str().to_string()
            },
        }
    }
    #[cfg(feature = "rdf")]
    fn from_subject(value: &ulo::rdf_types::Subject, decode_uris: bool) -> Self {
        match value {
            ulo::rdf_types::Subject::NamedNode(n) => Self::from_node(n, decode_uris),
            ulo::rdf_types::Subject::BlankNode(b) => Self::BlankNode {
                value: b.as_str().to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparqlResultTriple {
    pub subject: SparqlResultTerm,
    pub predicate: SparqlResultTerm,
    pub object: SparqlResultTerm,
}
