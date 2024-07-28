use std::str::FromStr;
pub use arrayvec::ArrayVec;
use crate::content::Term;
use crate::uris::symbols::SymbolURI;
use crate::utils::sourcerefs::{ByteOffset, SourceRange};

#[derive(Debug, Copy,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum AssocType {
    LeftAssociativeBinary,RightAssociativeBinary,Conjunctive,Prenex
}
impl FromStr for AssocType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binl"|"bin" => Ok(AssocType::LeftAssociativeBinary),
            "binr" => Ok(AssocType::RightAssociativeBinary),
            "conj" => Ok(AssocType::Conjunctive),
            "pre" => Ok(AssocType::Prenex),
            _ => Err(())
        }
    }

}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Constant {
    pub uri:SymbolURI,
    pub arity:ArgSpec,
    pub macroname:Option<String>,
    pub role:Option<Vec<String>>,
    pub tp:Option<Term>,
    pub df:Option<Term>,
    pub assoctype : Option<AssocType>,
    pub reordering:Option<String>
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgSpec(ArrayVec<ArgType,9>);

impl Default for ArgSpec {
    fn default() -> Self {
        ArgSpec(ArrayVec::new())
    }
}

impl FromStr for ArgSpec {
    type Err = ();
    fn from_str(s:&str) -> Result<Self,()> {
        let mut ret = ArrayVec::new();
        for c in s.bytes() {
            ret.push(match c {
                b'0' => return Ok(ArgSpec(ArrayVec::new())),
                b'i' => ArgType::Normal,
                b'a' => ArgType::Sequence,
                b'b' => ArgType::Binding,
                b'B' => ArgType::BindingSequence,
                _ => return Err(())
            })
        }
        Ok(ArgSpec(ret))
    }
}

#[derive(Debug, Clone,Copy)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArgType {
    Normal,Sequence,Binding,BindingSequence
}
impl FromStr for ArgType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(ArgType::Normal),
            "a" => Ok(ArgType::Sequence),
            "b" => Ok(ArgType::Binding),
            "B" => Ok(ArgType::BindingSequence),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notation {
    pub uri:SymbolURI,
    pub id:String,
    pub precedence:isize,
    pub argprecs:ArrayVec<isize,9>,
    pub range:SourceRange<ByteOffset>
}