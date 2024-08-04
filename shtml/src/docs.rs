use std::str::FromStr;
use immt_api::core::content::{ArgType, ArrayVec, Term, TermOrList, VarOrSym};
use immt_api::core::uris::ContentURI;
use crate::parsing::parser::HTMLParser;

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
    pub fn close(self,parser:&HTMLParser) -> Term {
        //println!("  - Closing term {self:?}");
        match self {
            Self::Symref {uri,..} => Term::OMS(uri),
            Self::OMV {name,..} => Term::OMV(name),
            Self::OMA { head: uri,args,..} => Term::OMA {
                head:uri,args:args.into_iter().map(|e| {
                    if let Some(e) = e {e} else {
                        println!("Waaah! {}",parser.uri);
                        (TermOrList::Term(Term::OMV("MISSING".to_string())),ArgType::Normal)
                    }
                }).collect()
            },
            Self::OMBIND { head: uri,args,..} => Term::OMBIND {
                head:uri,args:args.into_iter().map(|e| {
                    if let Some(e) = e {e} else {
                        println!("Waaah! {}",parser.uri);
                        (TermOrList::Term(Term::OMV("MISSING".to_string())),ArgType::Normal)
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