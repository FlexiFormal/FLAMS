use std::str::FromStr;
use immt_api::core::content::{ArgType, ArrayVec, Term, TermOrList, VarNameOrURI, VarOrSym};
use immt_api::core::ulo;
use immt_api::core::uris::{ContentURI, Name};
use crate::parsing::parser::HTMLParser;

#[derive(Debug)]
pub(crate) enum OpenTerm {
    Symref {
        uri:ContentURI,
        notation:Option<Name>,
    },
    OMV{
        name:VarNameOrURI,
        notation:Option<Name>,
    },
    OML{name:Name,df:Option<Term>},
    OMA {
        head:VarOrSym,
        head_term:Option<Term>,
        notation:Option<Name>,
        args:ArrayVec<Option<(TermOrList,ArgType)>,9>
    },
    OMBIND {
        head:VarOrSym,
        head_term:Option<Term>,
        notation:Option<Name>,
        args:ArrayVec<Option<(TermOrList,ArgType)>,9>
    },
    Complex(Option<Term>),
}
impl OpenTerm {
    pub fn close(self,parser:&mut HTMLParser) -> Term {
        //println!("  - Closing term {self:?}");
        match self {
            Self::Symref {uri,..} => {
                parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                Term::OMS(uri)
            },
            Self::OMV {name:VarNameOrURI::Name(name),..} => {
                let name = parser.resolve_variable(name);
                Term::OMV(name)
            }
            Self::OMV {name:name@VarNameOrURI::URI(_),..} => Term::OMV(name),
            Self::OMA { head: uri,args,head_term,..} => {
                if let VarOrSym::S(uri) = uri {
                    parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                }
                Term::OMA {
                    head:uri,args:args.into_iter().map(|e| {
                        if let Some(e) = e {e} else {
                            println!("Waaah! {}",parser.uri());
                            (TermOrList::Term(Term::OMV(VarNameOrURI::Name(Name::new("MISSING")))),ArgType::Normal)
                        }
                    }).collect(),
                    head_term:head_term.map(Box::new)
                }
            },
            Self::OMBIND { head: uri,args,head_term,..} => {
                if let VarOrSym::S(uri) = uri {
                    parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                }
                Term::OMBIND {
                    head:uri,args:args.into_iter().map(|e| {
                        if let Some(e) = e {e} else {
                            println!("Waaah! {}",parser.uri());
                            (TermOrList::Term(Term::OMV(VarNameOrURI::Name(Name::new("MISSING")))),ArgType::Normal)
                        }
                    }).collect(),
                    head_term:head_term.map(Box::new)
                }
            },
            Self::OML {name,df} => Term::OML{name,df:df.map(Box::new)},
            Self::Complex(Some(t)) => t,
            _ => {
                todo!()
            }
        }
    }
}