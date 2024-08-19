use std::fmt::{Display, Formatter, Write};
use std::marker::PhantomData;
use arrayvec::ArrayVec;
use crate::content::{Arg, ArgType, InformalChild, Term, TermOrList, VarNameOrURI, VarOrSym};
use crate::{OMA, OMB, OMS};
use crate::uris::{ContentURI, Name, SymbolURI, URI};

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notation {
    pub is_text:bool,
    pub precedence:isize,
    pub attribute_index:u8,
    pub id:Name,
    pub argprecs:ArrayVec<isize,9>,
    pub nt:Vec<NotationComponent>,
    pub op:Option<(String,u8,bool)>
}


pub struct TermDisplay<'a,I,J,F,G> where
    F:(Fn(SymbolURI) -> I)+Copy,
    I:Iterator<Item=Notation>,
    J:Iterator<Item=Notation>,
    G:(Fn(VarNameOrURI) -> J)+Copy
{
    pub(crate) term:&'a Term,
    pub(crate) notations:F,
    pub(crate) vars:G
}
impl<'a,I,J,F,G> TermDisplay<'a,I,J,F,G> where
    F:(Fn(SymbolURI) -> I)+Copy,
    I:Iterator<Item=Notation>,
    J:Iterator<Item=Notation>,
    G:(Fn(VarNameOrURI) -> J)+Copy {
    fn with_prec(term:&Term,notations:&F,vars:&G,f:&mut Formatter<'_>,prec:isize) -> std::fmt::Result {
        match term {
            OMS!(s) => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply_op("OMID",*s,f) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                let name = s.name();
                let name = name.as_ref();
                write!(f,"<mi shtml:term=\"OMID\" shtml:head=\"{}\" shtml:maincomp>{}</mi>",name,name.split('/').last().unwrap_or_else(|| name))
            },
            Term::Field{record,key:VarOrSym::S(ContentURI::Symbol(s)),..} => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply_op_this(&*record,"OMID",*s,f,|t,f,p| Self::with_prec(t,notations,vars,f,p)) {
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
            Term::OMV(s) => {
                for n in (vars)(*s) {
                    if let Some(r) = n.apply_op("OMV",*s,f) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                let name = s.name();
                let name = name.as_ref();
                write!(f,"<mi shtml:term=\"OMV\" shtml:head=\"{}\" shtml:varcomp>{}</mi>",name,name.split('/').last().unwrap_or_else(|| name))
            },
            Term::OML{name,..} => {
                f.write_str("<mtext>")?;
                f.write_str(name.as_ref())?;
                f.write_str("</mtext>")
            }
            OMA!(S s,args)|OMB!(S s,args) => {
                for n in (notations)(*s) {
                    if let Some(r) = n.apply(None,"OMA",*s,f,args,prec,|t,f,p| Self::with_prec(t,notations,vars,f,p)) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                write!(f,"<mrow><mtext>TODO: OMA with no applicable notation ({s})</mtext></mrow>")
            }
            Term::OMA{head:VarOrSym::V(vn),head_term:None,args}|Term::OMBIND{head:VarOrSym::V(vn),head_term:None,args}
            => {
                for n in (vars)(*vn) {
                    if let Some(r) = n.apply(None,"OMA",*vn,f,args,prec,|t,f,p| Self::with_prec(t,notations,vars,f,p)) {
                        return r
                    }
                }
                //println!("Here 1: {s}");
                write!(f,"<mrow><mtext>TODO: OMA with no applicable notation ({vn})</mtext></mrow>")
            },
            Term::OMA{head,head_term,args}|Term::OMBIND{head,head_term,args}
            => {
                //println!("Here 1: {head}");
                write!(f,"<mrow><mtext>TODO: OMA with head {head}</mtext></mrow>")
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
                Self::do_children(children,terms,notations,vars,f)?;
                f.write_str("</")?;
                f.write_str(tag)?;
                f.write_char('>')
            }
        }
    }

    fn do_children(children:&[InformalChild],terms:&[Term],notations:&F,vars:&G,f:&mut Formatter<'_>) -> std::fmt::Result {
        for c in children {match c {
            InformalChild::Text(s) => f.write_str(s)?,
            InformalChild::Term(n) => {
                f.write_str("<mrow style=\"color:initial\">")?;
                TermDisplay::with_prec(&terms[*n as usize], notations, vars,f, 0)?;
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
                Self::do_children(children, terms, notations, vars, f)?;
                f.write_str("</")?;
                f.write_str(tag)?;
                f.write_char('>')?;
            }
        }}
        Ok(())
    }
    fn do_fmt(&self,f:&mut Formatter<'_>) -> std::fmt::Result {
        Self::with_prec(self.term,&self.notations,&self.vars,f,0)
    }
}
impl<I,J,F,G> Display for TermDisplay<'_,I,J,F,G> where
    F:(Fn(SymbolURI) -> I)+Copy,
    I:Iterator<Item=Notation>,
    J:Iterator<Item=Notation>,
    G:(Fn(VarNameOrURI) -> J)+Copy {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.do_fmt(f)
    }
}

impl Notation {
    const ARGS : &'static str = "abcdefghijk";
    const VARS : &'static str = "xyzvwrstu";

    pub fn display(&self,sym:SymbolURI,f:&mut impl std::fmt::Write) -> std::fmt::Result {
        let mut comps = self.nt.iter();
        let tpstr = if self.argprecs.is_empty() {"OMID"} else {"OMA"};
        if let Some(NotationComponent::S(s)) = comps.next() {
            self.first_str(tpstr,self.attribute_index,sym,s,f)?;
        } else {
            println!("wut");
            return Err(std::fmt::Error::default())
        }
        for e in comps {
            self.do_comp(e,f,
                         |s,f| f.write_str(s),
                         |a,var,f| {
                             let i = a.index() as usize - 1;
                             let c = if var {
                                 Self::VARS.chars().nth(i).unwrap()
                             } else {
                                 Self::ARGS.chars().nth(i).unwrap()
                             };
                             f.write_str("<mi>")?;
                             f.write_char(c)?;
                             f.write_str("</mi>")
                         },
                         |i,var| {
                             let i =i as usize - 1;
                             let c = if var {
                                 Self::VARS.chars().nth(i).unwrap()
                             } else {
                                 Self::ARGS.chars().nth(i).unwrap()
                             };
                             Ok([DummyCont(Some((c,'1'))),DummyCont(None),DummyCont(Some((c,'n')))].into_iter())
                         }
            )?;
        }

        //println!("Success!");
        Ok(())
    }
    fn first_str<D:std::fmt::Display>(&self,tmtp:&'static str,idx:u8,sym:D,s:&str,f:&mut impl std::fmt::Write) -> std::fmt::Result {
        let start = &s[0..idx as usize];
        let end = &s[idx as usize..];
        write!(f,"{start} shtml:term=\"{tmtp}\" shtml:head=\"{sym}\" shtml:notationid=\"{}\"{end}",self.id.as_ref())
    }

    pub fn apply_op<D:std::fmt::Display>(&self,tmtp:&'static str,sym:D,f:&mut impl std::fmt::Write) -> Option<std::fmt::Result> {
        if let Some((op,idx,is_text)) = &self.op {
            if *is_text {
                let _ = f.write_str("<mtext>");
            }
            let r = self.first_str(tmtp,*idx,sym,op,f);
            if *is_text {
                let _ = f.write_str("</mtext>");
            }
            return Some(r)
        }
        if self.argprecs.is_empty() && self.nt.len() == 1 {
            return self.nt.iter().next().and_then(|s| match s {
                NotationComponent::S(s) => {
                    if self.is_text {
                        let _ = f.write_str("<mtext>");
                    }
                    let r = self.first_str(tmtp,self.attribute_index,sym,s,f);
                    if self.is_text {
                        let _ = f.write_str("</mtext>");
                    }
                    Some(r)
                },
                // should never happen:
                _ => None
            })
        }
        None
    }
    pub fn apply_op_this<'f>(&self,this:&Term,tmtp:&'static str,sym:SymbolURI,f:&mut std::fmt::Formatter<'f>,cont:impl (Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) + Copy) -> Option<std::fmt::Result> {
        //println!("Trying {self:?}");
        let _ = write!(f,"<msub><mrow>");
        self.apply_op(tmtp,sym,f).map(|r| {
            f.write_str("</mrow>")?;
            cont(this,f,0)?;
            f.write_str("</msub>")?;
            r
        })
    }
    fn do_comp<'a,W:std::fmt::Write,C:ContTrait<W>,I:Iterator<Item=C>>(&self,e:&NotationComponent,f:&mut W,
                                                                       do_this:impl (Fn(&str,&mut W) -> std::fmt::Result)+Copy,
                                                                       do_arg:impl (Fn(&Arg,bool,&mut W) -> std::fmt::Result) + Copy,
                                                                       do_arg_ls: impl (Fn(u8,bool) -> Result<I,std::fmt::Error>) + Copy
    ) -> std::fmt::Result {
        match e {
            NotationComponent::S(s) => {
                f.write_str(s)
            },
            NotationComponent::Comp(s) => f.write_str(s), // TODO <- varcomp/comp
            NotationComponent::MainComp(s) => do_this(s,f),
            NotationComponent::Arg(a,tp@(ArgType::Normal|ArgType::Binding)) => do_arg(a,*tp == ArgType::Binding,f),
            NotationComponent::Arg(a,tp@ (ArgType::Sequence|ArgType::BindingSequence)) => {
                let mut iter = do_arg_ls(a.index(),*tp == ArgType::BindingSequence)?;
                if let Some(cont) = iter.next() {
                    cont.apply(f)?;
                }
                for cont in iter {
                    f.write_str("<mo>,</mo>")?;
                    cont.apply(f)?;
                }
                Ok(())
            }
            NotationComponent::ArgSep {index,tp:tp@(ArgType::Sequence|ArgType::BindingSequence),sep} => {
                let mut iter = do_arg_ls(*index,*tp == ArgType::BindingSequence)?;
                if let Some(cont) = iter.next() {
                    cont.apply(f)?;
                }
                for cont in iter {
                    for e in sep {
                        if let NotationComponent::Arg(i,_) = e {
                            if i.index() == *index {continue}
                        }
                        self.do_comp(e, f, do_this,do_arg,do_arg_ls)?;
                    }
                    cont.apply(f)?;
                }
                Ok(())
            }
            NotationComponent::ArgSep{index,tp,..} => {
                println!("wut sep: {index}, {tp:?}");
                return Err(std::fmt::Error::default())
            }
            NotationComponent::ArgMap {..} => {
                println!("ArgMap");
                return Err(std::fmt::Error::default())
            }
        }
    }
    #[inline]
    pub fn apply<'f,D:std::fmt::Display>(&self,this:Option<&Term>,tpstr:&'static str,sym:D,f:&mut std::fmt::Formatter<'f>,args:&[(TermOrList,ArgType)],prec:isize,cont:impl (Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) + Copy) -> Option<std::fmt::Result> {
        //println!("Trying {sym}({args:?})\n  = {self:?}");
        let mut comps = self.nt.iter();
        if let Some(NotationComponent::S(s)) = comps.next() {
            self.first_str(tpstr,self.attribute_index,sym,s,f).ok()?;
        } else {
            println!("wut");
            return None
        }
        for e in comps {
            self.do_comp(e,f,
                         |s,f| Self::do_this(this,s,f,cont),
                         |a,_,f| self.do_arg(args,a,f,cont),
                         |i,_| self.do_arg_ls(args,i,cont)
            ).ok()?;//this,e,f,args,&cont).ok()?;
        }
        //println!("Success!");
        Some(Ok(()))
    }
    fn do_this<'f>(this:Option<&Term>,s:&str,f:&mut std::fmt::Formatter<'f>,cont:impl (Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) + Copy) -> std::fmt::Result {
        match this {
            None => f.write_str(s),
            Some(t) => {
                write!(f,"<msub><mrow>{s}</mrow>")?;
                cont(t,f,0)?;
                f.write_str("</msub>")
            }
        }
    }
    fn do_arg<'f>(&self,
                  args:&[(TermOrList,ArgType)],
                  arg:&Arg,
                  f:&mut std::fmt::Formatter<'f>,
                  cont:impl Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result
    ) -> std::fmt::Result {
        if let Some((TermOrList::Term(t),ArgType::Normal|ArgType::Binding)) = args.get(arg.index() as usize - 1) {
            let np = self.argprecs.get(arg.index() as usize).copied().unwrap_or(0);
            cont(t,f,np)
        } else {
            println!("waah: {:?}",args.get(arg.index() as usize - 1));
            return Err(std::fmt::Error::default())
        }
    }
    fn do_arg_ls<'a,'f,F:(Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result)+Copy> (
        &'a self,
        args:&'a [(TermOrList,ArgType)],
        index:u8,
        cont:F,
    ) -> Result<impl Iterator<Item=Cont<'a,'f,F>>,std::fmt::Error> {
        if let Some((TermOrList::List(ts),ArgType::Sequence|ArgType::BindingSequence)) = args.get(index as usize - 1) {
            let np = self.argprecs.get(index as usize).copied().unwrap_or(0);
            Ok(ts.iter().map(move |t| Cont {f:cont,t,np,phantom:PhantomData}))
            //cont(t,f,np)
        } else {
            println!("waah: {:?}",args.get(index as usize - 1));
            return Err(std::fmt::Error::default())
        }
    }
}

trait ContTrait<W:std::fmt::Write> {
    fn apply(&self,f:&mut W) -> std::fmt::Result;
}

struct Cont<'a,'f,F:Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result> {
    f:F,
    t:&'a Term,
    phantom:PhantomData<&'f ()>,
    np:isize
}
impl<'a,'f,F:Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result>
ContTrait<std::fmt::Formatter<'f>> for Cont<'a,'f,F> {
    fn apply(&self,f:&mut std::fmt::Formatter<'f>) -> std::fmt::Result {
        (self.f)(self.t, f, self.np)
    }
}

#[derive(Copy,Clone)]
struct DummyCont(Option<(char,char)>);
impl<W:std::fmt::Write> ContTrait<W> for DummyCont {
    fn apply(&self,f:&mut W) -> std::fmt::Result {
        if let Some((a,b)) = self.0 {
            f.write_str("<msub><mi>")?;
            f.write_char(a)?;
            f.write_str("</mi><mi>")?;
            f.write_char(b)?;
            f.write_str("</mi></msub>")
        } else {
            f.write_str("<mi>...</mi>")
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NotationComponent {
    S(Box<str>),
    Arg(Arg,ArgType),
    ArgSep{
        index:u8,
        tp:ArgType,
        sep:Vec<NotationComponent>
    },
    ArgMap {
        index:u8,
        segments:Vec<NotationComponent>,
    },
    MainComp(Box<str>),
    Comp(Box<str>)
}
