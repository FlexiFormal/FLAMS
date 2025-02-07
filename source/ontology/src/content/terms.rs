use crate::uris::{ContentURI, DocumentElementURI, Name, URIOrRefTrait, URIRef};
use crate::{oma, oms, omsp};
use immt_utils::prelude::{DFSContinuation, Indentor, TreeChild, TreeChildIter, TreeLike};
use std::fmt::{Debug, Display, Formatter, Write};
use std::str::FromStr;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Term {
    OMID(ContentURI),
    OMV(Var),
    OMA {
        head: Box<Term>,
        args: Box<[Arg]>,
    },
    Field {
        record: Box<Term>,
        key: Name,
        owner: Option<Box<Term>>,
    },
    OML {
        name: Name,
        df: Option<Box<Term>>,
        tp: Option<Box<Term>>,
    },
    Informal {
        tag: String,
        attributes: Box<[(Box<str>, Box<str>)]>,
        children: Box<[Informal]>,
        terms: Box<[Term]>,
    },
}

impl Term {
    /// #### Errors
    #[inline]
    pub fn present(&self,presenter:&mut impl crate::narration::notations::Presenter) -> Result<(),crate::narration::notations::PresentationError> {
        //println!("presenting {self}");
        crate::narration::notations::Notation::present_term(self, presenter)
    }

    pub fn term_list(i: impl Iterator<Item = Self>) -> Self {
        oma!(oms!(shtml:SEQUENCE_EXPRESSION),I@N:i)
    }

    #[must_use]
    pub fn as_list(&self) -> Option<&[Arg]> {
        match self {
            Self::OMA{
                head:box Self::OMID(ContentURI::Symbol(s)),
                args
            } if *s == *crate::metatheory::SEQUENCE_EXPRESSION => Some(&**args),
            _ => None
        }
    }

    #[must_use]
    pub fn is_record_field(&self) -> bool {
        matches!(self,oma!(omsp!(fp),[N:_,N:_]) if *fp == *crate::metatheory::FIELD_PROJECTION)
    }
    /*
    #[must_use]
    pub fn as_ref(&self) -> TermRef {
        match self {
            Self::OMID(uri) => TermRef::OMID(uri.as_content()),
            Self::OMV(v) => TermRef::OMV(v),
            Self::OMA{head,args} => TermRef::OMA{head,args},
            Self::Field{record,key} => TermRef::Field{record,key},
            Self::OML{name,df,tp} => TermRef::OML{name,df:df.as_deref(),tp:tp.as_deref()},
            Self::Informal{tag,attributes,children,terms} =>
                TermRef::Informal{tag,attributes,children,terms}
        }
    }

     */
    fn display_top<const SHORT: bool>(
        &self,
        f: &mut Formatter<'_>,
        indent: Option<Indentor>,
    ) -> std::fmt::Result {
        if let Some(i) = &indent {
            i.skip_next();
        };
        Self::display_children(
            TermChildrenIter::One(self),
            f,
            |t, i, f| t.display_start::<true>(i, f),
            |t, f| t.display_short_end(f),
            indent,
        )
    }
    fn display_start<const SHORT: bool>(
        &self,
        ind: &mut Indentor,
        f: &mut Formatter<'_>,
    ) -> Result<DFSContinuation<()>, std::fmt::Error> {
        macro_rules! cont {
            ($t:expr) => {
                $t.display_top::<SHORT>(f, Some(ind.clone()))
            };
        }
        match self {
            Self::OMID(uri) => if SHORT {
                Display::fmt(uri.name(), f)
            } else {
                Display::fmt(uri, f)
            }
            .map(|()| DFSContinuation::Continue),
            Self::OMV(v) => Display::fmt(v, f).map(|()| DFSContinuation::Continue),
            Self::OMA { head, .. } => {
                cont!(head)?;
                f.write_char('(')?;
                Ok(DFSContinuation::SkipNextAndClose(()))
            }
            Self::Field { record, key, owner } => {
                cont!(record)?;
                f.write_char('.')?;
                Display::fmt(key, f)?;
                if let Some(owner) = owner {
                    f.write_char('(')?;
                    cont!(owner)?;
                    f.write_char(')')?;
                }
                Ok(DFSContinuation::SkipChildren)
            }
            Self::OML { name, df, tp } => {
                write!(f, "\"{name}\"")?;
                if let Some(tp) = tp {
                    write!(f, "{ind}  :  ")?;
                    cont!(tp)?;
                }
                if let Some(df) = df {
                    write!(f, "{ind}  := ")?;
                    cont!(df)?;
                }
                Ok(DFSContinuation::SkipChildren)
            }
            Self::Informal { .. } => {
                write!(f,"TODO: Display for Term::Informal")?;
                Ok(DFSContinuation::SkipChildren)
            }
        }
    }
    fn display_short_end(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OMID(_) | Self::OMV(_) | Self::Field { .. } => unreachable!(),
            Self::OMA { .. } => f.write_char(')'),
            Self::OML { .. } => f.write_char('>'),
            Self::Informal { .. } => todo!(),
        }
    }

    #[inline]
    pub fn subterm_iter(&self) -> impl Iterator<Item=&'_ Self> {
        <TermChildrenIter as TreeChildIter<Self>>::dfs(
            TermChildrenIter::One(self)
        )
    }

    #[inline]
    pub fn uri_iter(&self) -> impl Iterator<Item=URIRef<'_>> {
        self.subterm_iter().filter_map(|t| match t {
            Self::OMID(uri) => Some(uri.as_uri()),
            Self::OMV(Var::Ref { declaration,.. }) => Some(declaration.as_uri()),
            _ => None
        })
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display_top::<true>(f, None)
    }
}

pub enum TermChildrenIter<'a> {
    Slice(std::slice::Iter<'a, Term>),
    Args(std::slice::Iter<'a, Arg>),
    One(&'a Term),
    Two(&'a Term, &'a Term),
    WithHead(&'a Term, &'a [Arg]),
    Empty,
}
impl<'a> Iterator for TermChildrenIter<'a> {
    type Item = &'a Term;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Slice(i) => i.next(),
            Self::Args(i) => i.next().map(|a| &a.term),
            Self::One(t) => {
                let t = *t;
                *self = Self::Empty;
                Some(t)
            }
            Self::Two(t1, t2) => {
                let t1 = *t1;
                *self = Self::One(t2);
                Some(t1)
            }
            Self::WithHead(head, args) => {
                let head = *head;
                *self = Self::Args(args.iter());
                Some(head)
            }
            Self::Empty => None,
        }
    }
}

impl TreeLike for Term {
    type Child<'a> = &'a Self;
    type RefIter<'a> = TermChildrenIter<'a>;
    #[allow(clippy::enum_glob_use)]
    fn children(&self) -> Option<Self::RefIter<'_>> {
        use Term::*;
        match self {
            OMID(_)
            | OMV(_)
            | OML {
                tp: None, df: None, ..
            } => None,
            Field { record, owner, .. } => Some(
                owner
                    .as_ref()
                    .map_or(TermChildrenIter::One(record), |owner| {
                        TermChildrenIter::Two(record, owner)
                    }),
            ),
            OMA { head, args } => Some(TermChildrenIter::WithHead(head, args)),
            //Field { record, .. } => Some(TermChildrenIter::One(record)),
            OML {
                df: Some(df),
                tp: Some(tp),
                ..
            } => Some(TermChildrenIter::Two(tp, df)),
            OML {
                df: None,
                tp: Some(tp),
                ..
            } => Some(TermChildrenIter::One(tp)),
            OML {
                df: Some(df),
                tp: None,
                ..
            } => Some(TermChildrenIter::One(df)),
            Informal { terms, .. } => Some(TermChildrenIter::Slice(terms.iter())),
        }
    }
}
impl TreeChild<Term> for &Term {
    fn children<'b>(&self) -> Option<<Term as TreeLike>::RefIter<'b>>
    where
        Self: 'b,
    {
        <Term as TreeLike>::children(self)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arg {
    pub term: Term,
    pub mode: ArgMode,
}
impl From<(Term, ArgMode)> for Arg {
    fn from((term, mode): (Term, ArgMode)) -> Self {
        Self { term, mode }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArgMode {
    #[default]
    Normal,
    Sequence,
    Binding,
    BindingSequence,
}
impl ArgMode {
    #[inline]#[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Normal => 'i',
            Self::Sequence => 'a',
            Self::Binding => 'b',
            Self::BindingSequence => 'B',
        }
    }
}
impl std::fmt::Display for ArgMode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.as_char())
    }
}
impl TryFrom<u8> for ArgMode {
    type Error = ();
    fn try_from(c: u8) -> Result<Self, Self::Error> {
        match c {
            b'i' => Ok(Self::Normal),
            b'a' => Ok(Self::Sequence),
            b'b' => Ok(Self::Binding),
            b'B' => Ok(Self::BindingSequence),
            _ => Err(()),
        }
    }
}
impl FromStr for ArgMode {
    type Err = ();
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(());
        }
        s.as_bytes()[0].try_into()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Var {
    Name(Name),
    Ref {
        declaration: DocumentElementURI,
        is_sequence: Option<bool>,
    },
}
impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => Display::fmt(n, f),
            Self::Ref { declaration, .. } => Display::fmt(declaration.name().last_name(), f),
        }
    }
}
impl Debug for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => Debug::fmt(n, f),
            Self::Ref { declaration, .. } => Debug::fmt(declaration, f),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Informal {
    Term(u8),
    Node {
        tag: String,
        attributes: Box<[(Box<str>, Box<str>)]>,
        children: Box<[Informal]>,
    },
    Text(Box<str>),
}

impl Informal {
    #[must_use]
    pub fn iter_opt(&self) -> Option<impl Iterator<Item=&Self>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node{children,..} => Some(
                InformalIter {
                    curr: children.iter(),
                    stack: Vec::new()
                })
        }
    }
    #[must_use]
    pub fn iter_mut_opt(&mut self) -> Option<impl Iterator<Item=&mut Self>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node{children,..} => Some(
                InformalIterMut {
                    curr: children.iter_mut(),
                    stack: Vec::new()
                })
        }
    }
}

struct InformalIter<'a> {
    curr:std::slice::Iter<'a,Informal>,
    stack:Vec<std::slice::Iter<'a,Informal>>
}
impl<'a> Iterator for InformalIter<'a> {
    type Item = &'a Informal;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.curr.next().or_else(|| {
            self.curr = self.stack.pop()?;
            self.curr.next()
        });
        if let Some(Informal::Node{children,..}) = r {
            self.stack.push(std::mem::replace(&mut self.curr,children.iter()));
        }
        r
    }
}
struct InformalIterMut<'a> {
    curr:std::slice::IterMut<'a,Informal>,
    stack:Vec<std::slice::IterMut<'a,Informal>>
}
impl<'a> Iterator for InformalIterMut<'a> {
    type Item = &'a mut Informal;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let r = self.curr.next().or_else(|| {
                self.curr = self.stack.pop()?;
                self.curr.next()
            });
            if let Some(Informal::Node { children, .. }) = r {
                self.stack.push(std::mem::replace(&mut self.curr, children.iter_mut()));
            } else {
                return r
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::uris::{ArchiveURI, BaseURI, ModuleURI, SymbolURI};
    use crate::{content::terms::Term, oma, oml, oms, omv};
    use lazy_static::lazy_static;
    lazy_static! {
        static ref NAMESPACE: BaseURI = BaseURI::new_unchecked("http://example.com/");
        static ref ARCHIVE1: ArchiveURI = NAMESPACE.clone() & "some/archive";
        static ref ARCHIVE2: ArchiveURI = NAMESPACE.clone() & "some/other/archive";
        static ref MODULE1: ModuleURI = (ARCHIVE1.clone() | "some/module").unwrap();
        static ref MODULE2: ModuleURI = (ARCHIVE2.clone() | "some/module").unwrap();
        static ref SYM1: SymbolURI = (MODULE1.clone() | "some symbol").unwrap();
        static ref SYM2: SymbolURI = (MODULE2.clone() | "other symbol").unwrap();
        static ref FUNC1: SymbolURI = (MODULE1.clone() | "some function").unwrap();
        static ref FUNC2: SymbolURI = (MODULE2.clone() | "other function").unwrap();
        static ref TERM: Term = oma!(oms!(FUNC1.clone()),[
            {N:oma!(oms!(FUNC2.clone()),[
                {N:oms!(SYM1.clone())},
                {N:oms!(SYM2.clone())}
            ])},
            {N:oma!(oms!(FUNC1.clone()),[
                {N:oml!("some name".parse().unwrap(); := oms!(SYM2.clone()))},
                {N:oms!(SYM1.clone())},
                {N:omv!("some var".parse().unwrap();)}
            ])}
        ]);
    }

    #[test]
    fn test_term_display() {
        let term = &*TERM;
        let s = format!("{term}");
        let refs = r#"some function(
  other function(
    some symbol
    other symbol
  )
  some function(
    "some name"
      := other symbol
    some symbol
    some var
  )
)"#;
        assert_eq!(s, refs);
    }
}
