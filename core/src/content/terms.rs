use std::fmt::{Debug, Display, Formatter, Write};
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

#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Term {
    OMS(ContentURI),
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
            Self::OMS(p) => Debug::fmt(p,f),
            Self::OMA{head,args,head_term} => {
                f.write_char('(')?;
                Debug::fmt(head,f)?;
                for (t,a) in args {
                    f.write_char(' ')?;
                    Debug::fmt(t,f)?;
                    f.write_char(' ')?;
                    Debug::fmt(a,f)?;
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

pub struct TermDisplay<'a,I,F> where F:(Fn(SymbolURI) -> I)+Copy,I:Iterator<Item=Notation> {
    term:&'a Term,
    notations:F
}
impl<'a,I,F> TermDisplay<'a,I,F> where F:(Fn(SymbolURI) -> I)+Copy,I:Iterator<Item=Notation> {
    fn with_prec(term:&Term,notations:&F,f:&mut Formatter<'_>,prec:isize) -> std::fmt::Result {
        match term {
            Term::OMS(ContentURI::Symbol(s)) => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply_op(*s,f) {
                        return r
                    }
                }
                f.write_str("<mrow><mtext>TODO: OMS</mtext></mrow>")
            },
            Term::OMS(_) =>
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
            Term::OMA{head:VarOrSym::S(ContentURI::Symbol(s)),head_term:None,args}|Term::OMBIND{head:VarOrSym::S(ContentURI::Symbol(s)),head_term:None,args}
            => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply(*s,f,args,prec,|t,f,p| Self::with_prec(t,notations,f,p)) {
                        return r
                    }
                }
                println!("Here 1: {s}");
                f.write_str("<mrow><mtext>TODO: OMA</mtext></mrow>")
            }
            Term::OMA{head,head_term,args}|Term::OMBIND{head,head_term,args}
            => {
                println!("Here 1: {head}");
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