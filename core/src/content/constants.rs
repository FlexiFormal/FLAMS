use std::fmt::Write;
use std::marker::PhantomData;
use std::str::FromStr;
pub use arrayvec::ArrayVec;
use crate::content::{Notation, Term, TermOrList};
use crate::narration::NarrativeRef;
use crate::uris::{Name, NarrDeclURI};
use crate::uris::symbols::SymbolURI;
use immt_utils::sourcerefs::{ByteOffset, SourceRange};

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
    pub macroname:Option<Name>,
    pub role:Option<Vec<Name>>,
    pub tp:Option<Term>,
    pub df:Option<Term>,
    pub assoctype : Option<AssocType>,
    pub reordering:Option<String>
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgSpec(ArrayVec<ArgType,9>);
impl IntoIterator for ArgSpec {
    type Item = ArgType;
    type IntoIter = arrayvec::IntoIter<ArgType,9>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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

#[derive(Debug, Clone,Copy,Hash,PartialEq,Eq)]
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


#[derive(Debug,Clone,Copy)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Arg {
    Ib(u8),
    AB(u8,u8)
}
impl Arg {
    pub fn index(&self) -> u8 {
        match self {
            Arg::Ib(i) => *i,
            Arg::AB(i, _) => *i
        }
    }
}
impl FromStr for Arg {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //println!("HERE: {s}");
        if s.len() == 1 {s.parse().map(Self::Ib).map_err(|_| ())}
        else if s.len() > 1 {
            let f = if s.as_bytes()[0] > 47 {s.as_bytes()[0] - 48} else { return Err(())};
            let s = if let Ok(s) = (&s[1..]).parse() {s} else { return Err(())};
            let r = Self::AB(f,s);
            //println!(" = {r:?}");
            Ok(r)
        } else {Err(())}
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NotationRef {
    pub symbol:SymbolURI,
    pub uri:SymbolURI,
    pub range:NarrativeRef<Notation>
}
