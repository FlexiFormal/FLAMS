use crate::{
    content::{
        declarations::symbols::{ArgSpec, AssocType},
        terms::Term,
    },
    uris::DocumentElementURI, Checked,
};

use super::{DocumentElement, NarrationTrait};

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variable {
    pub uri: DocumentElementURI,
    pub arity: ArgSpec,
    pub macroname: Option<Box<str>>,
    pub role: Box<[Box<str>]>,
    pub tp: Option<Term>,
    pub df: Option<Term>,
    pub bind:bool,
    pub assoctype: Option<AssocType>,
    pub reordering: Option<Box<str>>,
    pub is_seq:bool
}

impl NarrationTrait for Variable {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &[]
    }

    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        if let DocumentElement::Variable(e) = e {Some(e)} else {None}
    }
}