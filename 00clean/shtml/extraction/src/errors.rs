use std::fmt::Display;
use immt_ontology::uris::Name;
use crate::open::terms::{OpenTermKind, VarOrSym};

#[derive(Clone,Debug)]
pub enum SHTMLError {
    MissingArguments,
    MissingElementsInList,
    MissingTermForComplex(VarOrSym),
    UnresolvedVariable(Name),
    InvalidSymbolURI(String),
    InvalidModuleURI(String),
    InvalidDocumentURI(String),
    MissingHeadForTerm,
    InvalidTermKind(String),
    InvalidHeadForTermKind(OpenTermKind,VarOrSym),
    MissingInputrefURI,
    InvalidArgSpec
}

impl std::error::Error for SHTMLError {}
impl Display for SHTMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingArguments => f.write_str("missing arguments in OMA"),
            Self::MissingElementsInList => f.write_str("missing elements in term list"),
            Self::MissingTermForComplex(head) => write!(f,"missing actual term for complex term {head:?}"),
            Self::UnresolvedVariable(name) => write!(f,"unresolved variable {name}"),
            Self::InvalidSymbolURI(s) => write!(f, "invalid symbol {s}"),
            Self::InvalidModuleURI(s) => write!(f, "invalid module {s}"),
            Self::InvalidDocumentURI(s) => write!(f, "invalid document {s}"),
            Self::MissingHeadForTerm => f.write_str("missing shtml:head attribute for term"),
            Self::InvalidTermKind(s) => write!(f, "invalid term kind {s}"),
            Self::InvalidHeadForTermKind(kind,head) => write!(f, "invalid head {head:?} for term kind {kind:?}"),
            Self::MissingInputrefURI => f.write_str("missing or invalid document URI in shtml:inputref attribute"),
            Self::InvalidArgSpec => write!(f, "invalid or missing argument marker"),
        }
    }
}