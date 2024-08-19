use std::str::FromStr;
use immt_api::core::content::{ArgType, ArrayVec, FIELD_PROJECTION, OF_TYPE, Term, TermOrList, VarNameOrURI, VarOrSym};
use immt_api::core::{OMA, OMS,ulo};
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
    Complex(VarOrSym,Option<Term>),
}
impl OpenTerm {
    pub fn close(self,parser:&mut HTMLParser) -> Term {
        //println!("  - Closing term {self:?}");
        match self {
            Self::Symref {uri,..} => {
                parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                Term::OMID(uri)
            },
            Self::OMV {name:VarNameOrURI::Name(name),..} => {
                let name = parser.resolve_variable(name);
                Term::OMV(name)
            }
            Self::OMV {name:name@VarNameOrURI::URI(_),..} => Term::OMV(name),
            Self::OMA { head: mut uri,mut args,head_term,..} => {
                if let VarOrSym::S(uri) = uri {
                    parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                }
                match uri {
                    VarOrSym::V(VarNameOrURI::Name(n)) => {
                        uri = VarOrSym::V(parser.resolve_variable(n));
                    }
                    VarOrSym::S(ContentURI::Symbol(s)) if s == *FIELD_PROJECTION && args.len() == 2 => {
                        match (args.get(0),args.get(1)) {
                            (Some(Some((TermOrList::Term(record),_))),Some(Some((TermOrList::Term(Term::OML{name,..}),_)))) => {
                                //println!("Record: {record:?}");
                                return match record {
                                    /*
                                    Term::OMA { head,args,head_term} => {
                                        if let VarOrSym::S(ContentURI::Symbol(ot)) = head {
                                            if ot == &*OF_TYPE {
                                                if matches!(args[..],[_,_]) {
                                                    if let [(TermOrList::Term(record),_),(TermOrList::Term(OMS!(record_type)),_)] = &args[..] {
                                                        println!("matches!");
                                                        let [(TermOrList::Term(record), _), (TermOrList::Term(OMS!(record_type)), _)] = &args[..] else { unreachable!() };
                                                        Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: Some(*record_type) }
                                                    } else {
                                                        println!("args don't match");
                                                        Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: None }
                                                    }
                                                } else {
                                                    println!("args aren't 2");
                                                    Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: None }
                                                }
                                            } else {
                                                println!("head isn't 'of type'");
                                                Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: None }
                                            }
                                        } else {
                                            println!("head doesn't match");
                                            Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: None }
                                        }
                                    }
                                     */
                                    Term::OMA { head: VarOrSym::S(ContentURI::Symbol(ot)), args, head_term: None }
                                    if ot == &*OF_TYPE && matches!(args[..],[(TermOrList::Term(_),_),(TermOrList::Term(OMS!(_)),_)]) => {
                                        let [(TermOrList::Term(record), _), (TermOrList::Term(OMS!(record_type)), _)] = &args[..] else { unreachable!() };
                                        // TODO this clone should be unnecessary
                                        Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: Some(*record_type) }
                                    }
                                    _ => {
                                        // TODO this clone should be unnecessary
                                        let t = Term::Field { record: record.clone().into(), key: VarOrSym::V(VarNameOrURI::Name(*name)), record_type: None };
                                        println!("Doesn't match: {t:?}");
                                        t
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                    _ => ()
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
            Self::OMBIND { head: mut uri,args,head_term,..} => {
                if let VarOrSym::S(uri) = uri {
                    parser.add_triple(ulo!((parser.iri()) CROSSREFS (uri.to_iri())));
                }
                if let VarOrSym::V(VarNameOrURI::Name(n)) = uri {
                    uri = VarOrSym::V(parser.resolve_variable(n));
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
            Self::Complex(head,Some(t)) => {
                if let Term::Field {record,key,record_type} = t {
                    if let VarOrSym::S(ContentURI::Symbol(s)) = head {
                        Term::Field {record,key:VarOrSym::S(s.into()),record_type}
                    } else {
                        Term::Field {record,key,record_type}
                    }
                } else { t }

            }
            _ => {
                todo!()
            }
        }
    }
}