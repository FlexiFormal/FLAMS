use std::fmt::Display;
use flams_ontology::{ftml::FTMLKey, uris::{InvalidURICharacter, Name}};
use crate::open::terms::{OpenTermKind, VarOrSym};

#[derive(Clone,Debug)]
pub enum FTMLError {
    MissingArguments,
    MissingElementsInList,
    MissingTermForComplex(VarOrSym),
    UnresolvedVariable(Name),
    MissingHeadForTerm,
    InvalidTermKind(String),
    InvalidHeadForTermKind(OpenTermKind,VarOrSym),
    InvalidArgSpec,
    InvalidKeyFor(&'static str,Option<String>),
    NotInContent,
    NotInNarrative,
    NotInParagraph,
    NotInExercise(&'static str),
    InvalidKey,
    InvalidURI(String),
    IncompleteArgs(u8)
}

impl std::error::Error for FTMLError {}
const HEAD:&str = FTMLKey::Head.attr_name();
impl Display for FTMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingArguments => f.write_str("missing arguments in OMA"),
            Self::MissingElementsInList => f.write_str("missing elements in term list"),
            Self::MissingTermForComplex(head) => write!(f,"missing actual term for complex term {head:?}"),
            Self::UnresolvedVariable(name) => write!(f,"unresolved variable {name}"),
            Self::MissingHeadForTerm => write!(f,"missing {HEAD} attribute for term"),
            Self::InvalidTermKind(s) => write!(f, "invalid term kind {s}"),
            Self::InvalidHeadForTermKind(kind,head) => write!(f, "invalid head {head:?} for term kind {kind:?}"),
            Self::InvalidArgSpec => write!(f, "invalid or missing argument marker"),
            Self::InvalidKeyFor(tag, Some(value)) => write!(f,"invalid key {value} for ftml tag {tag}"),
            Self::InvalidKeyFor(tag, None) => write!(f,"missing key for ftml tag {tag}"),
            Self::NotInContent => f.write_str("content element outside of a module"),
            Self::NotInNarrative => f.write_str("unbalanced narrative element"),
            Self::NotInParagraph => f.write_str("unbalanced logical paragraph"),
            Self::NotInExercise(s) => write!(f,"unbalanced exercise element: {s}"),
            Self::InvalidKey => f.write_str("invalid key in ftml element"),
            Self::IncompleteArgs(i) => write!(f,"incomplete argument list: {i}"),
            Self::InvalidURI(s) => write!(f,"invalid URI: {s}"),
        }
    }
}