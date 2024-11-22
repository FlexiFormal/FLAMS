use smallvec::SmallVec;
use std::str::FromStr;

use crate::{
    content::terms::{ArgMode, Term},
    uris::SymbolURI, Resolvable,
};

use super::{Declaration, DeclarationTrait};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Symbol {
    pub uri: SymbolURI,
    pub arity: ArgSpec,
    pub macroname: Option<Box<str>>,
    pub role: Box<[Box<str>]>,
    pub tp: Option<Term>,
    pub df: Option<Term>,
    pub assoctype: Option<AssocType>,
    pub reordering: Option<Box<str>>,
}
impl Resolvable for Symbol {
    type From = SymbolURI;
    fn id(&self) -> std::borrow::Cow<'_,Self::From> {
        std::borrow::Cow::Borrowed(&self.uri)
    }
}
impl super::private::Sealed for Symbol {}
impl DeclarationTrait for Symbol {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::Symbol(m) => Some(m),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum AssocType {
    LeftAssociativeBinary,
    RightAssociativeBinary,
    Conjunctive,
    PairwiseConjunctive,
    Prenex,
}
impl FromStr for AssocType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binl" | "bin" => Ok(Self::LeftAssociativeBinary),
            "binr" => Ok(Self::RightAssociativeBinary),
            "conj" => Ok(Self::Conjunctive),
            "pwconj" => Ok(Self::PairwiseConjunctive),
            "pre" => Ok(Self::Prenex),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgSpec(SmallVec<ArgMode, 9>);
impl IntoIterator for ArgSpec {
    type Item = ArgMode;
    type IntoIter = smallvec::IntoIter<ArgMode, 9>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Default for ArgSpec {
    #[inline]
    fn default() -> Self {
        Self(SmallVec::new())
    }
}

impl FromStr for ArgSpec {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        if let Ok(u) = s.parse::<u8>() {
            return Ok(Self((0..u).map(|_| ArgMode::Normal).collect()));
        }
        let mut ret = SmallVec::new();
        for c in s.bytes() {
            ret.push(match c {
                b'i' => ArgMode::Normal,
                b'a' => ArgMode::Sequence,
                b'b' => ArgMode::Binding,
                b'B' => ArgMode::BindingSequence,
                _ => return Err(()),
            });
        }
        Ok(Self(ret))
    }
}
