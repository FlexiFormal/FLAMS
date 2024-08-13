use std::fmt::Write;
use std::str::FromStr;
pub use arrayvec::ArrayVec;
use crate::content::{Term, TermOrList};
use crate::narration::NarrativeRef;
use crate::uris::{Name, NarrDeclURI};
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
impl Notation {
    fn first_str(&self,tmtp:&'static str,idx:u8,sym:SymbolURI,s:&str,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        let start = &s[0..idx as usize];
        let end = &s[idx as usize..];
        write!(f,"{start} shtml:term=\"{tmtp}\" shtml:head=\"{sym}\" shtml:notationid=\"{}\"{end}",self.id.as_ref())
    }

    pub fn apply_op(&self,sym:SymbolURI,f:&mut std::fmt::Formatter<'_>) -> Option<std::fmt::Result> {
        if let Some((op,idx,is_text)) = &self.op {
            if *is_text {
                let _ = f.write_str("<mtext>");
            }
            let r = self.first_str("OMID",*idx,sym,op,f);
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
                    let r = self.first_str("OMID",self.attribute_index,sym,s,f);
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
    pub fn apply_op_this<'f>(&self,this:&Term,sym:SymbolURI,f:&mut std::fmt::Formatter<'f>,cont:impl (Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) + Copy) -> Option<std::fmt::Result> {
        //println!("Trying {self:?}");
        let _ = write!(f,"<msub><mrow>");
        self.apply_op(sym,f).map(|r| {
            f.write_str("</mrow>")?;
            cont(this,f,0)?;
            f.write_str("</msub>")?;
            r
        })
    }
    fn do_comp<'f>(&self,this:Option<&Term>,e:&NotationComponent,f:&mut std::fmt::Formatter<'f>,args:&[(TermOrList,ArgType)],cont:impl (Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) + Copy) -> std::fmt::Result {
        match e {
            NotationComponent::S(s) => {
                f.write_str(s)
            },
            NotationComponent::MainComp(s) => match this {
                None => f.write_str(s),
                Some(t) => {
                    write!(f,"<msub><mrow>{s}</mrow>")?;
                    cont(t,f,0)?;
                    f.write_str("</msub>")
                }
            }
            NotationComponent::Arg(a,ArgType::Normal|ArgType::Binding) => {
                if let Some((TermOrList::Term(t),ArgType::Normal)) = args.get(a.index() as usize - 1) {
                    let np = self.argprecs.get(a.index() as usize).copied().unwrap_or(0);
                    cont(t,f,np)
                } else {
                    println!("waah: {:?}",args.get(a.index() as usize - 1));
                    return Err(std::fmt::Error::default())
                }
            },
            NotationComponent::Arg(a,ArgType::Sequence|ArgType::BindingSequence) => {
                if let Some((TermOrList::List(ts),ArgType::Sequence|ArgType::BindingSequence)) = args.get(a.index() as usize - 1) {
                    let np = self.argprecs.get(a.index() as usize - 1).copied().unwrap_or(0);
                    let mut tms = ts.iter();
                    if let Some(h) = tms.next() {
                        cont(h,f,np)?;
                    }
                    for t in tms {
                        f.write_str("<mo>,</mo>")?;
                        cont(t,f,np)?
                    }
                    Ok(())
                } else {
                    println!("waah: {:?}",args.get(a.index() as usize - 1));
                    return Err(std::fmt::Error::default())
                }
            }
            NotationComponent::ArgSep {index,tp:ArgType::Sequence|ArgType::BindingSequence,sep} => {
                if let Some((TermOrList::List(ts),ArgType::Sequence|ArgType::BindingSequence)) = args.get(*index as usize - 1) {
                    let np = self.argprecs.get(*index as usize - 1).copied().unwrap_or(0);
                    let mut tms = ts.iter();
                    if let Some(h) = tms.next() {
                        cont(h,f,np)?;
                    }
                    for t in tms {
                        for e in sep {
                            if let NotationComponent::Arg(i,_) = e {
                                if i.index() == *index {continue}
                            }
                            self.do_comp(this,e, f, args, cont)?;
                        }
                        cont(t,f,np)?
                    }
                    Ok(())
                } else {
                    println!("waah sep: {:?}",args.get(*index as usize - 1));
                    return Err(std::fmt::Error::default())
                }
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
    pub fn apply<'f>(&self,this:Option<&Term>,tpstr:&'static str,sym:SymbolURI,f:&mut std::fmt::Formatter<'f>,args:&[(TermOrList,ArgType)],prec:isize,cont:impl Fn(&Term,&mut std::fmt::Formatter<'f>,isize) -> std::fmt::Result) -> Option<std::fmt::Result> {
        //println!("Trying {sym}({args:?})\n  = {self:?}");
        let mut comps = self.nt.iter();
        if let Some(NotationComponent::S(s)) = comps.next() {
            self.first_str(tpstr,self.attribute_index,sym,s,f).ok()?;
        } else {
            println!("wut");
            return None
        }
        for e in comps {
            self.do_comp(this,e,f,args,&cont).ok()?;
        }
        //println!("Success!");
        Some(Ok(()))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NotationComponent {
    S(String),
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
    MainComp(String)
}
