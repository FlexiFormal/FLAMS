use std::{fmt::Display, str::FromStr};

use flams_utils::vecmap::VecMap;

use crate::{
    content::terms::Term, ftml::FTMLKey, uris::{DocumentElementURI, Name, SymbolURI}, Checked, CheckingState, DocumentRange
};

use super::{DocumentElement, NarrationTrait};

#[derive(Debug)]
pub struct LogicalParagraph<State:CheckingState> {
    pub kind: ParagraphKind,
    pub uri: DocumentElementURI,
    pub inline: bool,
    pub title: Option<DocumentRange>,
    pub range: DocumentRange,
    pub styles: Box<[Name]>,
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
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
    const DEF:&str = FTMLKey::Definition.attr_name();
    const ASS:&str = FTMLKey::Assertion.attr_name();
    const PAR:&str = FTMLKey::Paragraph.attr_name();
    const PRO:&str = FTMLKey::Proof.attr_name();
    const SUB:&str = FTMLKey::SubProof.attr_name();
    #[must_use]
    pub fn from_ftml(s: &str) -> Option<Self> {
        
        Some(match s {
            Self::DEF => Self::Definition,
            Self::ASS => Self::Assertion,
            Self::PAR => Self::Paragraph,
            Self::PRO => Self::Proof,
            Self::SUB => Self::SubProof,
            _ => return None,
        })
    }
    pub fn is_definition_like(&self, styles: &[Name]) -> bool {
        match &self {
            Self::Definition | Self::Assertion => true,
            _ => styles
                .iter()
                .any(|s| s.first_name().as_ref() == "symdoc" || s.first_name().as_ref() == "decl"),
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
impl FromStr for ParagraphKind {
    type Err = ();
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "definition" => Ok(Self::Definition),
            "assertion" => Ok(Self::Assertion),
            "paragraph" => Ok(Self::Paragraph),
            "proof" => Ok(Self::Proof),
            "subproof" => Ok(Self::SubProof),
            "example" => Ok(Self::Example),
            _ => Err(()),
        }
    }
}