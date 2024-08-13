use std::fmt::{Debug, Display, Formatter, Write};
use std::str::FromStr;
use lazy_static::lazy_static;
use crate::content::{ArgType, Notation};
use crate::uris::{ContentURI, Name, NarrDeclURI};
use crate::uris::symbols::SymbolURI;
use crate::utils::VecMap;

#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TermOrList {
    Term(Term),
    List(Vec<Term>)
}

impl Debug for TermOrList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TermOrList::Term(t) => Debug::fmt(t,f),
            TermOrList::List(l) => {
                f.write_char('[')?;
                for (i,t) in l.iter().enumerate() {
                    if i > 0 { f.write_char(',')? }
                    Debug::fmt(t,f)?
                }
                f.write_char(']')
            }
        }
    }
}

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VarNameOrURI {
    Name(Name),
    URI(NarrDeclURI)
}
impl Display for VarNameOrURI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => Display::fmt(n,f),
            Self::URI(u) => Display::fmt(u,f)
        }
    }
}
impl VarNameOrURI {
    pub fn name(&self) -> Name {
        match self {
            Self::Name(n) => *n,
            Self::URI(u) => u.name()
        }
    }
}

lazy_static! {
    pub static ref FIELD_PROJECTION : SymbolURI = SymbolURI::from_str("http://mathhub.info/:sTeX?a=sTeX/meta-inf&m=Metatheory&l=en&c=record field").unwrap();
    pub static ref OF_TYPE : SymbolURI = SymbolURI::from_str("http://mathhub.info/:sTeX?a=sTeX/meta-inf&m=Metatheory&l=en&c=of type").unwrap();
}

#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Term {
    OMID(ContentURI),
    OMA {
        head:VarOrSym,
        head_term:Option<Box<Term>>,
        args:Vec<(TermOrList,ArgType)>
    },
    OMBIND {
        head:VarOrSym,
        head_term:Option<Box<Term>>,
        args:Vec<(TermOrList,ArgType)>
    },
    Field {
        record:Box<Term>,
        record_type:Option<SymbolURI>,
        key:VarOrSym
    },
    OMV(VarNameOrURI),
    OML{name:Name,df:Option<Box<Term>>},
    Informal {
        tag:String,
        attributes:VecMap<String,String>,
        children:Vec<InformalChild>,
        terms:Vec<Term>
    }
}
impl Debug for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OMID(p) => Debug::fmt(p, f),
            Self::OMA{head,args,head_term} => {
                Debug::fmt(head,f)?;
                f.write_char('(')?;
                for (t,a) in args {
                    f.write_char(' ')?;
                    Debug::fmt(t,f)?;
                    f.write_char(' ')?;
                }
                f.write_char(')')
            },
            Self::OMBIND {head,args,head_term} => {
                f.write_char('{')?;
                Debug::fmt(head,f)?;
                for (t,a) in args {
                    f.write_char(' ')?;
                    Debug::fmt(t,f)?;
                    f.write_char(' ')?;
                    Debug::fmt(a,f)?;
                }
                f.write_char('}')
            },
            Self::OMV(name) => {
                f.write_char('V')?;
                f.write_char('(')?;
                f.write_str(name.name().as_ref())?;
                f.write_char(')')
            }
            Self::OML{name,..} => {
                f.write_char('L')?;
                f.write_char('(')?;
                f.write_str(name.as_ref())?;
                f.write_char(')')
            }
            Self::Field {record,key,..} => {
                f.write_char('(')?;
                Debug::fmt(record, f)?;
                f.write_char('.')?;
                Debug::fmt(key, f)?;
                f.write_char(')')
            }
            Self::Informal {tag,attributes,children,terms} => {
                f.write_char('<')?;
                f.write_str(tag)?;
                for (k,v) in attributes.iter() {
                    f.write_char(' ')?;
                    f.write_str(k)?;
                    f.write_char('=')?;
                    f.write_char('"')?;
                    f.write_str(v)?;
                    f.write_char('"')?;
                }
                f.write_char('>')?;
                for c in children {
                    Debug::fmt(c,f)?;
                }
                f.write_char('<')?;
                f.write_char('/')?;
                f.write_str(tag)?;
                f.write_char('>')?;
                f.write_char('[')?;
                for t in terms {
                    Debug::fmt(t,f)?;
                }
                f.write_char(']')
            }
        }
    }
}
impl Term {
    pub fn display<I:Iterator<Item=Notation>,F:(Fn(SymbolURI) -> I)+Copy>(&self,notations:F) -> TermDisplay<'_,I,F> {
        TermDisplay { term: self, notations }
    }
}

#[macro_export]
macro_rules! OMS {
    ($s:pat) => { $crate::content::Term::OMID(ContentURI::Symbol($s)) };
}
#[macro_export]
macro_rules! OMMOD {
    ($s:pat) => { $crate::content::Term::OMID($crate::uris::ContentURI::Module($s)) };
}


#[macro_export]
macro_rules! OMA {
    (S $s:pat,$i:ident) => {
        $crate::content::Term::OMA{head:$crate::content::VarOrSym::S($crate::uris::ContentURI::Symbol($s)),args:$i,..}
    };
    ($s:pat,$i:ident) => {
        $crate::content::Term::OMA{head:$s,args:$i,..}|
        $crate::content::Term::OMA{head_term:$s,args:$i,..}
    };
}

#[macro_export]
macro_rules! OMB {
    (S $s:pat,$i:ident) => {
        $crate::content::Term::OMBIND{head:VarOrSym::S(ContentURI::Symbol($s)),args:$i,..}
    };
    ($s:pat,$i:ident) => {
        $crate::content::Term::OMBIND{head:$s,args:$i,..}|
        $crate::content::Term::OMBIND{head_term:$s,args:$i,..}
    };
}

pub struct TermDisplay<'a,I,F> where F:(Fn(SymbolURI) -> I)+Copy,I:Iterator<Item=Notation> {
    term:&'a Term,
    notations:F
}
impl<'a,I,F> TermDisplay<'a,I,F> where F:(Fn(SymbolURI) -> I)+Copy,I:Iterator<Item=Notation> {
    fn with_prec(term:&Term,notations:&F,f:&mut Formatter<'_>,prec:isize) -> std::fmt::Result {
        match term {
            OMS!(s) => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply_op(*s,f) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                write!(f,"<mi shtml:term=\"OMID\" shtml:head=\"{}\" shtml:maincomp>{}</mi>",s,s.name().as_ref())
            },
            Term::Field{record,key:VarOrSym::S(ContentURI::Symbol(s)),..} => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply_op_this(&*record,*s,f,|t,f,p| Self::with_prec(t,notations,f,p)) {
                        return r
                    }
                }
                println!("Here: {record:?}\n  @ {s}");
                f.write_str("<mrow><mtext>TODO: Field</mtext></mrow>")
            },
            Term::Field{record,key,..} => {
                println!("Here: {record:?}\n  = {key}");
                f.write_str("<mrow><mtext>TODO: Field</mtext></mrow>")
            },
            Term::OMID(_) =>
                f.write_str("<mrow><mtext>TODO: OMMOD</mtext></mrow>"),
            Term::OMV(name) => {
                f.write_str("<mi>")?;
                f.write_str(name.name().as_ref())?;
                f.write_str("</mi>")
            },
            Term::OML{name,..} => {
                f.write_str("<mtext>")?;
                f.write_str(name.as_ref())?;
                f.write_str("</mtext>")
            }
            OMA!(S s,args)|OMB!(S s,args) => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply(None,"OMA",*s,f,args,prec,|t,f,p| Self::with_prec(t,notations,f,p)) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                f.write_str("<mrow><mtext>TODO: OMA</mtext></mrow>")
            }
            Term::OMA{head,head_term,args}|Term::OMBIND{head,head_term,args}
            => {
                //println!("Here 1: {head}");
                f.write_str("<mrow><mtext>TODO: OMA</mtext></mrow>")
            },
            Term::Informal {tag,attributes,children,terms} => {
                f.write_char('<')?;
                f.write_str(tag)?;
                f.write_str(" style=\"color:red;\"")?;
                for (k,v) in attributes.iter() {
                    f.write_char(' ')?;
                    f.write_str(k)?;
                    f.write_char('=')?;
                    f.write_char('"')?;
                    f.write_str(v)?;
                    f.write_char('"')?;
                }
                f.write_char('>')?;
                fn do_children<I:Iterator<Item=Notation>,F:(Fn(SymbolURI) -> I)+Copy>(children:&[InformalChild],terms:&[Term],notations:&F,f:&mut Formatter<'_>) -> std::fmt::Result {
                    for c in children {match c {
                        InformalChild::Text(s) => f.write_str(s)?,
                        InformalChild::Term(n) => {
                            f.write_str("<mrow style=\"color:initial\">")?;
                            TermDisplay::with_prec(&terms[*n as usize], notations, f, 0)?;
                            f.write_str("</mrow>")?;
                        }
                        InformalChild::Node {tag,attributes,children} => {
                            f.write_char('<')?;
                            f.write_str(tag)?;
                            for (k,v) in attributes.iter() {
                                f.write_char(' ')?;
                                f.write_str(k)?;
                                f.write_char('=')?;
                                f.write_char('"')?;
                                f.write_str(v)?;
                                f.write_char('"')?;
                            }
                            f.write_char('>')?;
                            do_children(children,terms,notations,f)?;
                            f.write_str("</")?;
                            f.write_str(tag)?;
                            f.write_char('>')?;
                        }
                    }}
                    Ok(())
                }
                do_children(children,terms,notations,f)?;
                f.write_str("</")?;
                f.write_str(tag)?;
                f.write_char('>')
            }
        }
    }
    fn do_fmt(&self,f:&mut Formatter<'_>) -> std::fmt::Result {
        Self::with_prec(self.term,&self.notations,f,0)
    }
}
impl<I:Iterator<Item=Notation>,F:(Fn(SymbolURI) -> I)+Copy> Display for TermDisplay<'_,I,F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.do_fmt(f)
    }
}

#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InformalChild {
    Term(u8),
    Node {
        tag:String,
        attributes:VecMap<String,String>,
        children:Vec<InformalChild>
    },
    Text(String)
}
impl Debug for InformalChild {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Term(n) => {
                f.write_char('T')?;
                f.write_char('(')?;
                f.write_fmt(format_args!("{}",n))?;
                f.write_char(')')
            },
            Self::Node {tag,attributes,children} => {
                f.write_char('<')?;
                f.write_str(tag)?;
                for (k,v) in attributes.iter() {
                    f.write_char(' ')?;
                    f.write_str(k)?;
                    f.write_char('=')?;
                    f.write_char('"')?;
                    f.write_str(v)?;
                    f.write_char('"')?;
                }
                f.write_char('>')?;
                for c in children {
                    Debug::fmt(c,f)?;
                }
                f.write_char('<')?;
                f.write_char('/')?;
                f.write_str(tag)?;
                f.write_char('>')?;
                Ok(())
            },
            Self::Text(s) => {
                f.write_char('"')?;
                f.write_str(s)?;
                f.write_char('"')
            }
        }
    }
}
impl InformalChild {
    pub fn iter(&self) -> Option<impl Iterator<Item=&InformalChild>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node{children,..} => Some(
                InformalIter {
                    curr: children.iter(),
                    stack: Vec::new()
                })
        }
    }
    pub fn iter_mut(&mut self) -> Option<impl Iterator<Item=&mut InformalChild>> {
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
    curr:std::slice::Iter<'a,InformalChild>,
    stack:Vec<std::slice::Iter<'a,InformalChild>>
}
impl<'a> Iterator for InformalIter<'a> {
    type Item = &'a InformalChild;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.curr.next().or_else(|| {
            self.curr = self.stack.pop()?;
            self.curr.next()
        });
        if let Some(InformalChild::Node{children,..}) = r {
            self.stack.push(std::mem::replace(&mut self.curr,children.iter()))
        }
        r
    }
}
struct InformalIterMut<'a> {
    curr:std::slice::IterMut<'a,InformalChild>,
    stack:Vec<std::slice::IterMut<'a,InformalChild>>
}
impl<'a> Iterator for InformalIterMut<'a> {
    type Item = &'a mut InformalChild;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let r = self.curr.next().or_else(|| {
                self.curr = self.stack.pop()?;
                self.curr.next()
            });
            if let Some(InformalChild::Node { children, .. }) = r {
                self.stack.push(std::mem::replace(&mut self.curr, children.iter_mut()));
            } else {
                return r
            }
        }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VarOrSym {
    V(VarNameOrURI),
    S(ContentURI)
}
impl std::fmt::Display for VarOrSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarOrSym::V(s) => std::fmt::Display::fmt(s,f),
            VarOrSym::S(s) => std::fmt::Display::fmt(s,f)
        }
    }
}