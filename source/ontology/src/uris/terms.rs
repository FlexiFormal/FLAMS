use crate::uris::{
    debugdisplay, ContentURI, DocumentElementURI, NameStep, NarrativeURI, SymbolURI, URIParseError,
    URI,
};
use smallvec::SmallVec;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TermURI {
    SymbolTp(SymbolURI),
    SymbolDf(SymbolURI),
    VariableTp(DocumentElementURI),
    VariableDf(DocumentElementURI),
    DocumentTerm(DocumentElementURI),
}
impl TermURI {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn as_ref(&self) -> TermURIRef {
        match self {
            Self::SymbolTp(uri) => TermURIRef::SymbolTp(uri),
            Self::SymbolDf(uri) => TermURIRef::SymbolDf(uri),
            Self::VariableTp(uri) => TermURIRef::VariableTp(uri),
            Self::VariableDf(uri) => TermURIRef::VariableDf(uri),
            Self::DocumentTerm(uri) => TermURIRef::DocumentTerm(uri),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermURIRef<'a> {
    SymbolTp(&'a SymbolURI),
    SymbolDf(&'a SymbolURI),
    VariableTp(&'a DocumentElementURI),
    VariableDf(&'a DocumentElementURI),
    DocumentTerm(&'a DocumentElementURI),
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SubTermIndex(SmallVec<u16, 8>);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SubTermURI {
    uri: TermURI,
    subterm: SubTermIndex,
}

impl Display for TermURIRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SymbolTp(uri) => write!(f, "{uri}&tp"),
            Self::SymbolDf(uri) => write!(f, "{uri}&df"),
            Self::VariableTp(uri) => write!(f, "{uri}&tp"),
            Self::VariableDf(uri) => write!(f, "{uri}&df"),
            Self::DocumentTerm(uri) => Display::fmt(uri, f),
        }
    }
}
debugdisplay!(TermURIRef<'_>);
impl Display for TermURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_ref(), f)
    }
}
debugdisplay!(TermURI);
impl FromStr for TermURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! symordoc {
            ($s:ident,$tp:ident) => {
                paste::paste!{
                    match URI::from_str($s)? {
                        URI::Narrative(NarrativeURI::Element(uri)) => Ok(Self::[<Variable $tp>](uri)),
                        URI::Content(ContentURI::Symbol(uri)) => Ok(Self::[<Symbol $tp>](uri)),
                        _ => Err(URIParseError::UnrecognizedPart {
                            original:s.to_string()
                        })
                    }
                }
            }
        }
        s.strip_suffix("&tp").map_or_else(
            || {
                s.strip_suffix("&df").map_or_else(
                    || DocumentElementURI::from_str(s).map(TermURI::DocumentTerm),
                    |s| symordoc!(s, Df),
                )
            },
            |s| symordoc!(s, Tp),
        )
    }
}
