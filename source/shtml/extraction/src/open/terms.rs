use std::fmt::Display;
use std::str::FromStr;
use either::Either;
use immt_ontology::content::terms::{Arg, ArgMode, Term, Var};
use immt_ontology::uris::{ContentURI, DocumentElementURI, Name};
use crate::extractor::SHTMLExtractor;

#[cfg(feature="rdf")]
use immt_ontology::{triple,uris::URIOrRefTrait};
use immt_ontology::{oma, omfp, omsp};
use crate::errors::SHTMLError;

#[derive(Debug,Clone)]
pub enum OpenTerm {
    Symref {
        uri:ContentURI,
        notation:Option<Name>,
    },
    Varref{
        name:PreVar,
        notation:Option<Name>,
    },
    OML{name:Name},//,tp:Option<Term>,df:Option<Term>},
    OMA {
        head:VarOrSym,
        notation:Option<Name>,
        //args:SmallVec<Option<(TermOrList,ArgMode)>,9>
    },
    Complex(VarOrSym),
}
impl OpenTerm {
    #[must_use]
    pub fn take_head(self) -> VarOrSym {
        match self {
            Self::Symref {uri,..} => VarOrSym::S(uri),
            Self::Varref {name,..} => VarOrSym::V(name),
            Self::OML {name,..} => VarOrSym::V(PreVar::Unresolved(name)),
            Self::OMA {head,..}|Self::Complex(head,..) => head
        }
    }
    pub fn close<E:SHTMLExtractor>(self, extractor:&mut E) -> Term {
        match self {
            Self::Symref {uri,notation:_todo} => {
                #[cfg(feature="rdf")]
                if E::RDF { 
                    let iri = extractor.get_document_iri();
                    extractor.add_triples([
                        triple!(<(iri)> ulo:CROSSREFS <(uri.to_iri())>)
                    ]);
                }
                Term::OMID(uri)
            }
            Self::Varref {name,notation:_todo} => {
                name.resolve(extractor)
            }
            Self::OML {name} => {
                let (tp,df) = if let Some((tp,df)) = extractor.close_decl() {(tp,df)} else {
                    extractor.add_error(SHTMLError::NotInContent);
                    (None,None)
                };
                Term::OML{name,df:df.map(Box::new),tp:tp.map(Box::new)}
            }
            Self::Complex(varorsym) => {
                let term = extractor.close_complex_term();
                if let Some(oma!(omsp!(ref fp),[N:ref p,N:Term::OML {ref name,tp:Option::None,df:Option::None}])) = term {
                    if *fp == *immt_ontology::metatheory::FIELD_PROJECTION {
                        omfp!((p.clone()).(name.clone()) = (varorsym.resolve(extractor))) // TODO avoid clone here
                    } else {
                        term.unwrap_or_else(|| unreachable!())
                    }
                }
                else if let Some(mut t) = term {
                    if let Term::Field { owner,.. } = &mut t {
                        *owner = Some(Box::new(varorsym.resolve(extractor)));
                    }
                    t
                }
                else {
                    extractor.add_error(SHTMLError::MissingTermForComplex(varorsym.clone()));
                    varorsym.resolve(extractor)
                }
            }
            Self::OMA{head,notation:_todo} => {
                let (args,head_term) = extractor.close_args();
                let args = args.into_boxed_slice();
                let mut head = head.resolve(extractor);
                if let Some(oma!(omsp!(ref fp),[N:ref p,N:Term::OML {ref name,tp:Option::None,df:Option::None}])) = head_term {
                    if *fp == *immt_ontology::metatheory::FIELD_PROJECTION {
                        //println!("Is Field!");
                        return omfp!((p.clone()).(name.clone()) = (head)) // TODO avoid clone here
                    }
                    head = head_term.unwrap_or_else(|| unreachable!());
                }
                match (head,args) {
                    (omsp!(fp),box [Arg{ref term,mode:ArgMode::Normal},Arg{term:Term::OML{ref name,tp:Option::None,df:Option::None},mode:ArgMode::Normal}]) if fp == *immt_ontology::metatheory::FIELD_PROJECTION => {
                        //println!("Is Field!");
                        Term::Field {
                            record:Box::new(term.clone()), // TODO avoid clone here
                            key:name.clone(), // TODO avoid clone here
                            owner:None
                        }
                    }
                    (head,args) => Term::OMA {head:Box::new(head),args}
                }
            }
        }
    }
}

#[derive(Debug,Clone)]
pub enum TermOrList {
    Term(Term),
    List(Vec<Option<Term>>)
}
impl TermOrList {
    /// #### Errors
    pub fn close(self,mode:ArgMode) -> Result<Arg,Arg> {
        match self {
            Self::Term(t) => Ok(Arg {term:t,mode}),
            Self::List(ls) => {
                let mut ret = Vec::new();
                let mut ls = ls.into_iter();
                while let Some(Some(t)) = ls.next() {
                    ret.push(t);
                }
                let ret = Arg{term: Term::term_list(ret.into_iter()),mode};
                for a in ls {
                    if a.is_some() {
                        return Err(ret)
                    }
                }
                Ok(ret)
            }
        }
    }
}

#[derive(Clone,Debug)]
pub enum PreVar {
    Resolved(DocumentElementURI),
    Unresolved(Name)
}
impl Display for PreVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolved(declaration) => Display::fmt(declaration,f),
            Self::Unresolved(name) => Display::fmt(name,f)
        }
    }
}
impl PreVar {
    fn resolve<State:SHTMLExtractor>(self,state:&State) -> Term { Term::OMV(match self {
        Self::Resolved(declaration) => Var::Ref {declaration,is_sequence:None},
            // TODO can we know is_sequence yet?
        Self::Unresolved(name) => {
            match state.resolve_variable_name(name) {
                Var::Name(name) => {
                    //state.add_error(SHTMLError::UnresolvedVariable(name.clone()));
                    Var::Name(name)
                }
                v@Var::Ref{..} => v
            }
        }
    }) }
    #[inline]#[must_use]
    pub const fn name(&self) -> &Name {
        match self {
            Self::Resolved(declaration) => declaration.name(),
            Self::Unresolved(name) => name
        }
    }
}

#[derive(Clone,Debug)]
pub enum VarOrSym {
    S(ContentURI),
    V(PreVar)
}
impl Display for VarOrSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S(uri) => Display::fmt(uri,f),
            Self::V(v) => Display::fmt(v,f)
        }
    }
}
impl VarOrSym {
    fn resolve<E:SHTMLExtractor>(self,extractor:&mut E) -> Term {
        match self {
            Self::S(uri) => {
                #[cfg(feature="rdf")]
                if E::RDF {
                    let iri = extractor.get_document_iri();
                    extractor.add_triples([
                        triple!(<(iri)> ulo:CROSSREFS <(uri.to_iri())>)
                    ]);
                }
                Term::OMID(uri)
            },
            Self::V(pv) => pv.resolve(extractor)
        }
    }
}

#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
pub enum OpenTermKind {
    OMID,
    OMV,
    OMA,
    OML,
    Complex
}
impl FromStr for OpenTermKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "OMID"|"OMMOD" => Self::OMID,
            "OMV" => Self::OMV,
            "OMA"|"OMBIND" => Self::OMA,
            "OML" => Self::OML,
            "complex" => Self::Complex,
            _ => return Err(())
        })
    }
}

#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
pub struct OpenArg {
    pub index:Either<u8,(u8,u8)>,
    pub mode:ArgMode
}
impl OpenArg {
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_strs<Idx:AsRef<str>,M:AsRef<str>>(idx: Idx,mode:Option<M>) -> Option<Self> {
        let mode = mode.and_then(|s| s.as_ref().parse().ok()).unwrap_or_default();
        let idx = idx.as_ref();
        let index = if idx.chars().count() > 1 {
            let a = idx.chars().next().unwrap_or_else(|| unreachable!()).to_digit(10);
            let b = u32::from_str(&idx[1..]).ok();
            match (a,b) {
                (Some(a),Some(b)) if a < 256 && b < 256 => Either::Right((a as u8,b as u8)),
                _ => return None
            }
        } else if idx.len() == 1 {
            let a = idx.chars().next().unwrap_or_else(|| unreachable!()).to_digit(10)?;
            if a < 256 {
                Either::Left(a as u8)
            } else {
                return None
            }
        } else {
            return None
        };
        Some(Self { index,mode })
    }
}