use std::str::FromStr;
use immt_api::core::content::{ArgSpec, ArgType, ArrayVec, Constant, ContentElement, Module, Notation, Term, TermOrList, VarOrSym};
use immt_api::core::narration::{DocumentElement, DocumentModule, Language, LogicalParagraph, Section, SectionLevel, StatementKind};
use immt_api::core::uris::ContentURI;
use immt_api::core::uris::documents::DocumentURI;
use immt_api::core::uris::modules::ModuleURI;
use immt_api::core::uris::symbols::SymbolURI;
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::parse::ParseSource;
use immt_api::core::utils::VecMap;
//use crate::parsing::OpenNode;
use crate::parsing::parser::HTMLParser;

#[derive(Debug)]
pub(crate) enum Arg {
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
        if s.len() == 1 {s.parse().map(Self::Ib).map_err(|_| ())}
        else if s.len() == 2 {
            let f = if s.as_bytes()[0] > 47 {s.as_bytes()[0] - 48} else { return Err(())};
            let s = if s.as_bytes()[1] > 47 {s.as_bytes()[1] - 48} else { return Err(())};
            Ok(Self::AB(f,s))
        } else {Err(())}
    }
}

#[derive(Debug)]
pub(crate) enum OpenTerm {
    Symref {
        uri:ContentURI,
        notation:Option<String>,
    },
    OMV{
        name:String,
        notation:Option<String>,
    },
    OML{name:String},
    OMA {
        head:VarOrSym,
        notation:Option<String>,
        args:ArrayVec<Option<(TermOrList,ArgType)>,9>
    },
    OMBIND {
        head:VarOrSym,
        notation:Option<String>,
        args:ArrayVec<Option<(TermOrList,ArgType)>,9>
    },
    Complex(Option<Term>),
}
impl OpenTerm {
    pub fn close(self) -> Term {
        match self {
            Self::Symref {uri,..} => Term::OMS(uri),
            Self::OMV {name,..} => Term::OMV(name),
            Self::OMA { head: uri,args,..} => Term::OMA {
                head:uri,args:args.into_iter().map(|e| {
                    if let Some(e) = e {e} else {
                        todo!()
                    }
                }).collect()
            },
            Self::OMBIND { head: uri,args,..} => Term::OMBIND {
                head:uri,args:args.into_iter().map(|e| {
                    if let Some(e) = e {e} else {
                        todo!()
                    }
                }).collect()
            },
            Self::OML {name} => Term::OML(name),
            Self::Complex(Some(t)) => t,
            _ => {
                todo!()
            }
        }
    }
}