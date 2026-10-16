#[cfg(feature="rdf")]
mod spql {
    pub use ulo::sparql::*;
}

#[cfg(not(feature="rdf"))]
mod spql {
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

    #[derive(Debug, Clone, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct SparqlResultBindings {
        bindings: Vec<rustc_hash::FxHashMap<String, SparqlResultTerm>>,
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

    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct SparqlResultTriple {
        pub subject: SparqlResultTerm,
        pub predicate: SparqlResultTerm,
        pub object: SparqlResultTerm,
    }
}
pub use spql::*;
