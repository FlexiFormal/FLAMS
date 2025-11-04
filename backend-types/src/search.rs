#![allow(clippy::wildcard_imports)]

use ftml_ontology::narrative::elements::paragraphs::ParagraphKind;
use ftml_uris::{DocumentElementUri, DocumentUri, SymbolUri};

#[allow(dead_code)]
const fn get_true() -> bool {
    true
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct QueryFilter {
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_documents: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_paragraphs: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_definitions: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_examples: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_assertions: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_problems: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub definition_like_only: bool,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            allow_documents: true,
            allow_paragraphs: true,
            allow_definitions: true,
            allow_examples: true,
            allow_assertions: true,
            allow_problems: true,
            definition_like_only: false,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResult {
    Document(DocumentUri),
    Paragraph {
        uri: DocumentElementUri,
        fors: Vec<SymbolUri>,
        def_like: bool,
        kind: SearchResultKind,
    },
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResultKind {
    Document = 0,
    Paragraph = 1,
    Definition = 2,
    Example = 3,
    Assertion = 4,
    Problem = 5,
}
impl SearchResultKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Paragraph => "Paragraph",
            Self::Definition => "Definition",
            Self::Example => "Example",
            Self::Assertion => "Assertion",
            Self::Problem => "Problem",
        }
    }
}

impl From<SearchResultKind> for u64 {
    fn from(value: SearchResultKind) -> Self {
        match value {
            SearchResultKind::Document => 0,
            SearchResultKind::Paragraph => 1,
            SearchResultKind::Definition => 2,
            SearchResultKind::Example => 3,
            SearchResultKind::Assertion => 4,
            SearchResultKind::Problem => 5,
        }
    }
}

impl TryFrom<u64> for SearchResultKind {
    type Error = ();
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Document,
            1 => Self::Paragraph,
            2 => Self::Definition,
            3 => Self::Example,
            4 => Self::Assertion,
            5 => Self::Problem,
            _ => return Err(()),
        })
    }
}
impl TryFrom<ParagraphKind> for SearchResultKind {
    type Error = ();
    fn try_from(value: ParagraphKind) -> Result<Self, Self::Error> {
        Ok(match value {
            ParagraphKind::Assertion => Self::Assertion,
            ParagraphKind::Definition => Self::Definition,
            ParagraphKind::Example => Self::Example,
            ParagraphKind::Paragraph => Self::Paragraph,
            _ => return Err(()),
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum SearchIndex {
    Document {
        uri: DocumentUri,
        title: Option<String>,
        body: String,
    },
    Paragraph {
        uri: DocumentElementUri,
        kind: SearchResultKind,
        definition_like: bool,
        title: Option<String>,
        fors: Vec<SymbolUri>,
        body: String,
    },
}
