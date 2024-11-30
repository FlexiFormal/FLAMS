use std::fmt::Display;

use immt_utils::vecmap::VecMap;

use crate::{
    content::terms::Term, shtml::SHTMLKey, uris::{DocumentElementURI, SymbolURI}, Checked, CheckingState, DocumentRange
};

use super::{DocumentElement, NarrationTrait};

#[derive(Debug)]
pub struct LogicalParagraph<State:CheckingState> {
    pub kind: ParagraphKind,
    pub uri: DocumentElementURI,
    pub inline: bool,
    pub title: Option<DocumentRange>,
    pub range: DocumentRange,
    pub styles: Box<[Box<str>]>,
    pub children: State::Seq<DocumentElement<State>>,
    pub fors: VecMap<SymbolURI, Option<Term>>,
}

crate::serde_impl!{
    struct LogicalParagraph[kind,uri,inline,title,range,styles,children,fors]
}

impl NarrationTrait for LogicalParagraph<Checked> {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.children
    }
    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        if let DocumentElement::Paragraph(e) = e {Some(e)} else {None}
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ParagraphKind {
    Definition,
    Assertion,
    Paragraph,
    Proof,
    SubProof,
    Example,
}

impl ParagraphKind {
    const DEF:&str = SHTMLKey::Definition.attr_name();
    const ASS:&str = SHTMLKey::Assertion.attr_name();
    const PAR:&str = SHTMLKey::Paragraph.attr_name();
    const PRO:&str = SHTMLKey::Proof.attr_name();
    const SUB:&str = SHTMLKey::SubProof.attr_name();
    #[must_use]
    pub fn from_shtml(s: &str) -> Option<Self> {
        
        Some(match s {
            Self::DEF => Self::Definition,
            Self::ASS => Self::Assertion,
            Self::PAR => Self::Paragraph,
            Self::PRO => Self::Proof,
            Self::SUB => Self::SubProof,
            _ => return None,
        })
    }
    pub fn is_definition_like<S: AsRef<str>>(&self, styles: &[S]) -> bool {
        match &self {
            Self::Definition | Self::Assertion => true,
            _ => styles
                .iter()
                .any(|s| s.as_ref() == "symdoc" || s.as_ref() == "decl"),
        }
    }
    #[cfg(feature = "rdf")]
    #[must_use]
    #[allow(clippy::wildcard_imports)]
    pub const fn rdf_type(&self) -> crate::rdf::NamedNodeRef {
        use crate::rdf::ontologies::ulo2::*;
        match self {
            Self::Definition => DEFINITION,
            Self::Assertion => PROPOSITION,
            Self::Paragraph => PARA,
            Self::Proof => PROOF,
            Self::SubProof => SUBPROOF,
            Self::Example => EXAMPLE,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Assertion => "assertion",
            Self::Paragraph => "paragraph",
            Self::Proof => "proof",
            Self::SubProof => "subproof",
            Self::Example => "example",
        }
    }

    #[must_use]
    pub const fn as_display_str(self) -> &'static str {
        match self {
            Self::Definition => "Definition",
            Self::Assertion => "Assertion",
            Self::Paragraph => "Paragraph",
            Self::Proof => "Proof",
            Self::SubProof => "Subproof",
            Self::Example => "Example",
        }
    }
}
impl Display for ParagraphKind {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_display_str())
    }
}
