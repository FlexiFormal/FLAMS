use smallvec::SmallVec;

use crate::{content::terms::ArgMode, Resourcable};

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notation {
    pub is_text: bool,
    pub precedence: isize,
    pub attribute_index: u8,
    pub id: Box<str>,
    pub argprecs: SmallVec<isize, 9>,
    pub components: Box<[NotationComponent]>,
    pub op: Option<OpNotation>,
}
impl Resourcable for Notation {}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNotation {
    pub attribute_index: u8,
    pub is_text:bool,
    pub text:Box<str>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NotationComponent {
    S(Box<str>),
    Arg(u8, ArgMode),
    ArgSep {
        index: u8,
        tp: ArgMode,
        sep: Box<[NotationComponent]>,
    },
    ArgMap {
        index: u8,
        segments: Box<[NotationComponent]>,
    },
    MainComp(Box<str>),
    Comp(Box<str>),
}
