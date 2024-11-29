use smallvec::SmallVec;
use crate::{content::terms::ArgMode, Resourcable};

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notation {
    pub is_text: bool,
    pub precedence: isize,
    pub attribute_index: u8,
    pub inner_index:u16,
    pub id: Box<str>,
    pub argprecs: SmallVec<isize, 9>,
    pub components: Box<[NotationComponent]>,
    pub op: Option<OpNotation>,
}
impl Resourcable for Notation {}
impl Notation {
    #[must_use]
    pub fn is_op(&self) -> bool {
        self.op.is_some() || !self.components.iter().any(|c|
            matches!(c,NotationComponent::Arg(..)|NotationComponent::ArgMap{..}|NotationComponent::ArgSep { .. })
        )
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNotation {
    pub attribute_index: u8,
    pub is_text:bool,
    pub inner_index:u16,
    pub text:Box<str>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NotationComponent {
    S(Box<str>),
    Arg(u8, ArgMode),
    ArgSep {
        index: u8,
        mode: ArgMode,
        sep: Box<[NotationComponent]>,
    },
    ArgMap {
        index: u8,
        segments: Box<[NotationComponent]>,
    },
    MainComp(Box<str>),
    Comp(Box<str>),
}

pub use presentation::{Presenter,PresentationError};

mod presentation {
    use std::fmt::Display;
    use crate::{content::terms::{Arg, ArgMode, Informal, Term, Var}, oma, omsp, uris::{ContentURI, DocumentElementURI, SymbolURI}};

    pub type Result = std::result::Result<(),PresentationError>;

    #[derive(Debug)]
    pub enum PresentationError {
        Formatting,
        MalformedNotation,
        ArgumentMismatch
    }
    impl From<std::fmt::Error> for PresentationError {
        #[inline]
        fn from(value: std::fmt::Error) -> Self {
            Self::Formatting
        }
    }

    pub trait Presenter: std::fmt::Write {
        type N : std::ops::Deref<Target=Notation>;
        fn get_notation(&mut self,uri:&SymbolURI) -> Option<Self::N>;
        fn get_op_notation(&mut self,uri:&SymbolURI) -> Option<Self::N>;
        fn get_variable_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N>;
        fn get_variable_op_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N>;
        fn in_text(&self) -> bool;
        /// #### Errors
        fn cont(&mut self,tm:&Term) -> Result;
    }

    use super::{Notation,NotationComponent};
    impl Notation {

        #[inline]
        fn def(presenter:&mut impl Presenter,tp:&str,uri:impl Display,txt:&str) -> Result {
            if presenter.in_text() {
                write!(presenter,"<span data-shtml-term=\"{tp}\" data-shtml-head=\"{uri}\" data-shtml-comp>{txt}</span>")
            } else {
                write!(presenter,"<mtext data-shtml-term=\"{tp}\" data-shtml-head=\"{uri}\" data-shtml-comp>{txt}</mtext>")
            }.map_err(Into::into)
        }

        /// #### Errors
        pub(crate) fn present_term(term:&Term,presenter:&mut impl Presenter) -> Result {
            match term {
                omsp!(uri) =>
                    if let Some(n) = presenter.get_op_notation(uri) {
                        n.apply_op(presenter,None,"OMID",uri)
                    } else {
                        Self::def(presenter,"OMID",uri,uri.name().last_name().as_ref())
                    },
                Term::OMV(Var::Ref{declaration:uri,is_sequence:_}) =>
                    if let Some(n) = presenter.get_variable_notation(uri) {
                        n.apply_op(presenter, None, "OMV",uri)
                    } else {
                        Self::def(presenter,"OMV",uri,uri.name().last_name().as_ref())
                    },
                Term::OMV(Var::Name(name)) => Self::def(presenter,"OMV",name,name.last_name().as_ref()),
                Term::Field { record, owner:Some(box Term::OMID(ContentURI::Symbol(uri))),.. } => 
                    if let Some(n) = presenter.get_op_notation(uri) {
                        n.apply_op(presenter,Some(record),"COMPLEX",uri)
                    } else {
                        Self::def(presenter,"OMID",uri,uri.name().last_name().as_ref())
                    },
                oma!(omsp!(uri),args)  => 
                    if let Some(n) = presenter.get_notation(uri) {
                        n.apply(presenter,None,None,uri,args)
                    } else {
                        write!(presenter,"<mtext>TODO: Default notation for OMA({uri},_)</mtext>").map_err(Into::into)
                    },
                Term::OMA{head:box Term::OMV(Var::Ref{declaration:uri,is_sequence:_}),args,..} => 
                    if let Some(n) = presenter.get_variable_notation(uri) {
                        n.apply(presenter,None,None,uri,args)
                    } else {
                        write!(presenter,"<mtext>TODO: Default notation for OMA({uri},_)</mtext>").map_err(Into::into)
                    }
                Term::OMA{head:box Term::OMV(Var::Name(name)),args,..} =>
                    write!(presenter,"<mtext>TODO: Default notation for OMA({name},_)</mtext>").map_err(Into::into),


                Term::Informal { tag, attributes, children, terms,.. } =>
                    Self::informal(presenter,tag,attributes,children,terms),
                t => write!(presenter,"<mtext>TODO: {t:?}</mtext>").map_err(Into::into)
            }
        }

        fn informal(presenter:&mut impl Presenter,tag:&str, attributes:&[(Box<str>,Box<str>)],children:&[Informal],terms:&[Term]) -> Result {
            fn has_terms(cs:&[Informal]) -> bool {
                cs.iter().any(|c| match c {
                    Informal::Term(_) => true,
                    Informal::Text(_) => false,
                    Informal::Node { children,.. } => has_terms(children)
                })
            }
            write!(presenter,"<{tag}")?;
            for (k,v) in attributes {
                write!(presenter," {k}=\"{v}\"")?;
            }
            if !has_terms(children) {
                write!(presenter," style=\"color:red\"")?;
            }
            write!(presenter,">")?;
            for c in children { match c {
                Informal::Text(t) => write!(presenter,"{t}")?,
                Informal::Term(t) =>
                    if let Some(t) =terms.get(*t as usize) {
                        presenter.cont(t)?;
                    } else {
                        return Err(PresentationError::MalformedNotation)
                    },
                Informal::Node { tag, attributes, children } =>
                    Self::informal(presenter,tag,attributes,children,terms)?
            }}
            write!(presenter,"</{tag}>").map_err(Into::into)
        }

        /// #### Errors
        pub fn apply_op(&self,
            presenter:&mut impl Presenter,
            this:Option<&Term>,
            termstr:&str,
            head:impl Display
        ) -> Result {
            if let Some(opn) = &self.op {
                if let Some(this) = this {
                    write!(presenter,"<munder data-shtml-term=\"{termstr}\" data-shtml-head=\"{head}\" data-shtml-notationid=\"{}\">{}",self.id,opn.text)?;
                    write!(presenter,"<mrow data-shtml-headterm>")?;
                    presenter.cont(this)?;
                    write!(presenter,"</mrow></munder>").map_err(Into::into)
                } else {
                    let index = opn.attribute_index as usize;
                    let start = &opn.text[0..index];
                    let end = &opn.text[index..];
                    write!(presenter,"{start} data-shtml-term=\"{termstr}\" data-shtml-head=\"{head}\" data-shtml-notationid=\"{}\"{end}",self.id).map_err(Into::into)
                }
            } else {
                self.apply(presenter,this,Some(termstr),head,&[])
            }
        }

        /// #### Errors
        pub fn apply(
            &self,
            presenter:&mut impl Presenter,
            this:Option<&Term>,
            termstr:Option<&str>,
            head:impl Display,
            args:&[Arg]
        ) -> Result {
            //println!("Components: {:?}",self.components);
            let termstr = termstr.unwrap_or_else(|| if args.iter().any(|a| matches!(a.mode,ArgMode::Binding|ArgMode::BindingSequence)) { "OMBIND"} else { "OMA" });
            let mut comps = self.components.iter();
            let Some(NotationComponent::S(start_node)) = comps.next() else {
                return Err(PresentationError::MalformedNotation)
            };
            {
                let index = self.attribute_index as usize;
                let start = &start_node[0..index];
                let end = &start_node[index..];
                write!(presenter,"{start} data-shtml-term=\"{termstr}\" data-shtml-head=\"{head}\" data-shtml-notationid=\"{}\"{end}",self.id)?;
            }
            for comp in comps {
                comp.apply(presenter,this,args)?;
            }
            Ok(())
        }
    }

    impl NotationComponent {
        fn apply(&self,presenter:&mut impl Presenter,this:Option<&Term>,args:&[Arg]) -> Result {
            match self {
                Self::S(s) |
                Self::Comp(s) => presenter.write_str(s).map_err(Into::into),
                Self::MainComp(s) =>
                    if let Some(this) = this {
                        Self::do_this(presenter,this,s)
                    } else {
                        presenter.write_str(s).map_err(Into::into)
                    },
                Self::Arg(idx,mode) => {
                    let Some(arg) = args.get((*idx - 1) as usize) else {
                        return Err(PresentationError::ArgumentMismatch)
                    };
                    Self::do_arg(presenter,*idx,arg,*mode)
                },
                Self::ArgSep { index, mode, sep } => {
                    let Some(arg) = args.get((*index - 1) as usize) else {
                        return Err(PresentationError::ArgumentMismatch)
                    };
                    if let Some(ls) = arg.term.as_list() {
                        //println!("Here: {index}{mode}: {ls:?}");
                        Self::do_term_ls(presenter,*mode,*index,ls.iter().map(|a| &a.term),
                            |p| {
                                //println!("Separator: {sep:?}");
                                for c in sep.iter().skip(1) {
                                    c.apply(p, this, args)?;
                                }
                                Ok(())
                            }
                        )
                    } else {
                        write!(presenter,"<mtext>TODO: argument mode {mode:?} im argsep</mtext>").map_err(Into::into)
                    }
                    
                }
                t => write!(presenter,"<mtext>TODO: {t:?}</mtext>").map_err(Into::into)
            }
        }

        fn do_arg(presenter:&mut impl Presenter,idx:u8,arg:&Arg,mode:ArgMode) -> Result {
            match (mode,arg) {
                (ArgMode::Normal|ArgMode::Binding,Arg{term,..}) if !presenter.in_text() => {
                    write!(presenter,"<mrow data-shtml-arg=\"{idx}\" data-shtml-argmode=\"{mode}\">")?;
                    presenter.cont(term)?;
                    write!(presenter,"</mrow>").map_err(Into::into)
                }
                (ArgMode::Sequence|ArgMode::BindingSequence,Arg{term,..}) if !presenter.in_text() => {
                    if let Some(ls) = term.as_list() {
                        /*println!("HERE!");
                        for t in ls {
                            println!(" - {t:?}");
                        }*/
                        //println!("Here: {idx}{mode}: {ls:?}");
                        Self::do_term_ls(presenter,mode,idx,ls.iter().map(|a| &a.term),
                            |p| write!(p,"<mo>,</mo>").map_err(Into::into)
                        )
                    } else {
                        write!(presenter,"<mtext>TODO: argument mode {mode:?}</mtext>").map_err(Into::into)
                    }
                }
                _ => write!(presenter,"<mtext>TODO: argument mode {mode:?}</mtext>").map_err(Into::into)
            }
        }

        fn do_term_ls<'t,P:Presenter>(presenter:&mut P,mode:ArgMode,idx:u8,mut ls:impl Iterator<Item=&'t Term>,sep:impl Fn(&mut P) -> Result) -> Result {
            let Some(first) = ls.next() else { return Ok(())};
            write!(presenter,/*"<mrow data-shtml-arg=\"{idx}\" data-shtml-argmode=\"{mode}\">*/"<mrow data-shtml-arg=\"{idx}1\">")?;
            //println!("First {idx}{mode}: {first}");
            presenter.cont(first)?;
            write!(presenter,"</mrow>")?;
            let mut i = 2;
            for term in ls {
                sep(presenter)?;
                write!(presenter,"<mrow data-shtml-arg=\"{idx}{i}\">")?;
                //println!("term {i} of {idx}{mode}: {term}");
                presenter.cont(term)?;
                write!(presenter,"</mrow>")?;
                i += 1;
            }
            Ok(())//write!(presenter,"</mrow>").map_err(Into::into)
        }

        fn do_this(presenter:&mut impl Presenter,this:&Term,main_comp:&str) -> Result {
            write!(presenter,"<mtext>TODO: this</mtext>").map_err(Into::into)
        }
    }
}