use smallvec::SmallVec;
use crate::{content::terms::ArgMode, shtml::SHTMLKey, Resourcable};

#[derive(Debug,Clone,PartialEq,Eq)]
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

#[derive(Debug,Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpNotation {
    pub attribute_index: u8,
    pub is_text:bool,
    pub inner_index:u16,
    pub text:Box<str>
}
impl OpNotation {

    pub fn display_shtml<'a>(&'a self,as_variable:bool,uri:impl std::fmt::Display + 'a) -> impl std::fmt::Display + 'a {
        struct OpDisplayer<'n,U:std::fmt::Display+'n>{
            op:&'n OpNotation,
            as_variable:bool,
            uri:U,
        }
        impl<U:std::fmt::Display> std::fmt::Display for OpDisplayer<'_,U> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                const TERM :&str = SHTMLKey::Term.attr_name();
                const HEAD:&str = SHTMLKey::Head.attr_name();
                const COMP:&str = SHTMLKey::Comp.attr_name();
                let tp = if self.as_variable { "OMV" } else { "OMID" };
                let uri = &self.uri;
                let text = &self.op.text;
                write!(f,"<mrow {TERM}=\"{tp}\" {HEAD}=\"{uri}\" {COMP}>{text}</mrow>")
            }
        }
        OpDisplayer {
            op:self,
            as_variable,
            uri
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
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
    use std::{fmt::Display, marker::PhantomData};
    use immt_utils::prelude::HMap;

    use crate::{content::terms::{Arg, ArgMode, Informal, Term, Var}, oma, omsp, shtml::SHTMLKey, uris::{ContentURI, DocumentElementURI, SymbolURI, URITrait}};

    pub type Result = std::result::Result<(),PresentationError>;

    #[derive(Debug)]
    pub enum PresentationError {
        Formatting,
        MalformedNotation(String),
        ArgumentMismatch
    }
    impl From<std::fmt::Error> for PresentationError {
        #[inline]
        fn from(value: std::fmt::Error) -> Self {
            Self::Formatting
        }
    }
    impl std::fmt::Display for PresentationError {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,"Presentation error: {self:?}")
        }
    }

    pub trait Presenter: std::fmt::Write+Sized {
        type N : AsRef<Notation>;
        fn get_notation(&mut self,uri:&SymbolURI) -> Option<Self::N>;
        fn get_op_notation(&mut self,uri:&SymbolURI) -> Option<Self::N>;
        fn get_variable_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N>;
        fn get_variable_op_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N>;
        fn in_text(&self) -> bool;
        /// #### Errors
        #[inline]
        fn cont(&mut self,tm:&Term) -> Result {
            tm.present(self)
        }
    }

    impl AsRef<Self> for Notation {
        #[inline]
        fn as_ref(&self) -> &Self {self}
    }

    struct FromStore<'s>{out:String,store:&'s NotationStore}
    impl Presenter for FromStore<'_> {
        type N = Notation;
        #[inline]
        fn get_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {self.store.notations.get(uri).and_then(|v| v.first().cloned())}
        #[inline]
        fn get_op_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {self.store.notations.get(uri).and_then(|v| v.first().cloned())}
        #[inline]
        fn get_variable_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {self.store.var_notations.get(uri).and_then(|v| v.first().cloned())}
        #[inline]
        fn get_variable_op_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {self.store.var_notations.get(uri).and_then(|v| v.first().cloned())}
        #[inline]
        fn in_text(&self) -> bool {false}
        #[inline]
        fn cont(&mut self,tm:&Term) -> Result {
            tm.present(self)
        }
    }
    impl std::fmt::Write for FromStore<'_> {
        #[inline]
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.out.push_str(s);
            Ok(())
        }
    }

    #[derive(Debug,Clone,Default)]
    #[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
    pub struct NotationStore {
        notations: HMap<SymbolURI,Vec<Notation>>,
        var_notations: HMap<DocumentElementURI,Vec<Notation>>,
    }
    impl NotationStore {
        #[inline]
        pub fn push(&mut self, uri:SymbolURI,notation:Notation) {
            self.notations.entry(uri).or_default().push(notation);
        }
        #[inline]
        pub fn push_var(&mut self, uri:DocumentElementURI,notation:Notation) {
            self.var_notations.entry(uri).or_default().push(notation);
        }
    }

    trait Args<W:std::fmt::Write> {
        fn single(&self,idx:u8,mode:ArgMode,out:&mut W) -> Result;
        fn sequence(&self,idx:u8,mode:ArgMode)
        -> std::result::Result<impl Iterator<Item = impl FnOnce(&mut W) -> Result>,PresentationError>;
    }


    struct Displayer {
        args:[char;9],
    }

    const ARGS: &str = "abcdefghijk";
    const VARS: &str = "xyzvwrstu";

    impl Displayer {
        fn new(n:&Notation) -> (Self,u8,&'static str) {
            let mut args = ['_';9];
            let mut vars = 0u8;
            let mut arity = 0;
            let mut vs = n.components.iter().filter_map(|c| match c {
                NotationComponent::Arg(i,m) => {arity = arity.max(*i); Some((*i,*m))},
                NotationComponent::ArgSep { index, mode, .. } => {arity = arity.max(*index); Some((*index,*mode))},
                NotationComponent::ArgMap { index, .. } => {arity = arity.max(*index); Some((*index,ArgMode::Normal))},
                _ => None
            }).collect::<Vec<_>>();
            vs.sort_by_key(|(i,_)| *i);
            for (i,m) in vs {
                if matches!(m,ArgMode::Binding|ArgMode::BindingSequence) {
                    let var = vars as usize;
                    vars += 1;
                    args[(i - 1) as usize] = VARS.chars().nth(var).unwrap_or_else(|| unreachable!());
                } else {
                    let arg = ((i - 1)- vars) as usize;
                    args[arg] = ARGS.chars().nth(arg).unwrap_or_else(|| unreachable!());
                }
            }
            (Self { args },arity, if vars > 0 { "OMBIND" } else { "OMA" })
        }
    }

    struct NotationDisplay<'n,D:Display> {
        d:Displayer,
        notation:&'n Notation,
        arity:u8,
        termstr:&'static str,
        uri:D,
        op:bool,
        omv:bool
    }

    impl<D:Display> std::fmt::Display for NotationDisplay<'_,D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let termstr = if self.arity == 0 || self.op {
                if self.omv { "OMV" } else { "OMID" }
            } else { self.termstr };
            if self.op {
                self.notation.apply_op(f, termstr, &self.uri, &self.d)
            } else {
                self.notation.apply_i(f, None,termstr, &self.uri, &self.d)
            }.map_err(|_| std::fmt::Error)
        }
    }

    impl<W:std::fmt::Write> Args<W> for Displayer {
        fn single(&self,idx:u8,_mode:ArgMode,out:&mut W) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const COMP:&str = SHTMLKey::Comp.attr_name();

            let c = self.args[(idx - 1) as usize];
            write!(out,"<mi {TERM}=\"OMV\" {HEAD}=\"{c}\" {COMP}>{c}</mi>").map_err(Into::into)
        }
        fn sequence(&self,idx:u8,_mode:ArgMode)
                -> std::result::Result<impl Iterator<Item = impl FnOnce(&mut W) -> Result>,PresentationError> {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const COMP:&str = SHTMLKey::Comp.attr_name();
            
            let c = self.args[(idx - 1) as usize];
            Ok([
                Some((c,'1')),
                None,
                Some((c,'n')),
            ].map(|opt|
                move |out:&mut W| if let Some((c,sub)) = opt {
                    write!(out,"<msub {TERM}=\"OMV\" {HEAD}=\"{c}\" {COMP}><mi>{c}</mi><mi>{sub}</mi></mrow>")
                } else {
                    write!(out,"<mo>...</mo>")
                }.map_err(Into::into)
            ).into_iter())
        }
    }

    impl<P:Presenter> Args<P> for &'_ [Arg] {
        fn single(&self,idx:u8,_mode:ArgMode,out:&mut P) -> Result {
            let Some(arg) = self.get((idx - 1) as usize) else {
                return Err(PresentationError::ArgumentMismatch)
            };
            out.cont(&arg.term)
        }
        fn sequence(&self,idx:u8,_mode:ArgMode)
         -> std::result::Result<impl Iterator<Item = impl FnOnce(&mut P) -> Result>,PresentationError> {
            let Some(arg) = self.get((idx - 1) as usize) else {
                return Err(PresentationError::ArgumentMismatch)
            };
            let args = arg.term.as_list().unwrap_or_else(|| std::array::from_ref(arg));
            Ok(args.iter().map(|arg| move |p: &mut P| p.cont(&arg.term)))
        }
    }

    use super::{Notation,NotationComponent};
    impl Notation {
        pub fn display_shtml<'a>(&'a self,op:bool,as_variable:bool,uri:&'a impl URITrait) -> impl Display + 'a {
            let (d,arity,termstr) = Displayer::new(self);
            NotationDisplay {
                d,
                notation: self,
                termstr,
                op,
                arity,
                uri,
                omv:as_variable
            }
        }

        #[inline]
        fn default_omid(out:&mut impl std::fmt::Write,tp:&str,uri:impl Display,txt:&str,in_text:bool) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const COMP:&str = SHTMLKey::Comp.attr_name();
            if in_text {
                write!(out,"<span {TERM}=\"{tp}\" {HEAD}=\"{uri}\" {COMP}>{txt}</span>")
            } else {
                write!(out,"<mtext {TERM}=\"{tp}\" {HEAD}=\"{uri}\" {COMP}>{txt}</mtext>")
            }.map_err(Into::into)
        }

        fn default_oma(presenter:&mut impl Presenter,tp:&str,uri:impl Display,txt:&str,args:&[Arg]) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const COMP:&str = SHTMLKey::Comp.attr_name();
            const ARG:&str = SHTMLKey::Arg.attr_name();
            const MODE:&str = SHTMLKey::ArgMode.attr_name();
            if presenter.in_text() {
                write!(presenter,"<span {TERM}=\"{tp}\" {HEAD}=\"{uri}\" {COMP}>{txt}</span>").map_err(Into::into)
            } else {
                write!(presenter,"<mrow><mtext {TERM}=\"{tp}\" {HEAD}=\"{uri}\" {COMP}>{txt}</mtext><mo>(</mo>")?;
                let mut args = args.iter();
                let Some(Arg{term,mode}) = args.next() else {
                    return write!(presenter,"<mo>)</mo></mrow>").map_err(Into::into);
                };
                let mut idx = 1;
                write!(presenter,"<mrow {ARG}=\"{idx}\" {MODE}=\"{mode}\">")?;
                presenter.cont(term)?;
                write!(presenter,"</mrow>")?;

                for Arg{term,mode} in args {
                    idx += 1;
                    write!(presenter,"<mo>,</mo>")?;
                    write!(presenter,"<mrow {ARG}=\"{idx}\" {MODE}=\"{mode}\">")?;
                    presenter.cont(term)?;
                    write!(presenter,"</mrow>")?;
                }
                write!(presenter,"<mo>)</mo></mrow>").map_err(Into::into)
            }
        }

        /// #### Errors
        pub(crate) fn present_term(term:&Term,presenter:&mut impl Presenter) -> Result {
            match term {
                omsp!(uri) =>
                    if let Some(n) = presenter.get_op_notation(uri) {
                        let args : &[Arg] = &[];
                        n.as_ref().apply_op(presenter,"OMID",uri,&args)
                    } else {
                        Self::default_omid(presenter,"OMID",uri,uri.name().last_name().as_ref(),presenter.in_text())
                    },
                Term::OMV(Var::Ref{declaration:uri,is_sequence:_}) =>
                    if let Some(n) = presenter.get_variable_notation(uri) {
                        let args : &[Arg] = &[];
                        n.as_ref().apply_op(presenter, "OMV",uri,&args)
                    } else {
                        Self::default_omid(presenter,"OMV",uri,uri.name().last_name().as_ref(),presenter.in_text())
                    },
                Term::OMV(Var::Name(name)) => Self::default_omid(presenter,"OMV",name,name.last_name().as_ref(),presenter.in_text()),
                Term::Field { record, owner:Some(box Term::OMID(ContentURI::Symbol(uri))),.. } => 
                    if let Some(n) = presenter.get_op_notation(uri) {
                        n.as_ref().apply_op_this(presenter,record,"COMPLEX",uri)
                    } else {
                        Self::default_omid(presenter,"OMID",uri,uri.name().last_name().as_ref(),presenter.in_text())
                    },
                oma!(omsp!(uri),args)  => 
                    if let Some(n) = presenter.get_notation(uri) {
                        n.as_ref().apply(presenter,None,None,uri,args)
                    } else {
                        Self::default_oma(presenter, "OMA", uri, uri.name().last_name().as_ref(), args)
                    },
                Term::OMA{head:box Term::OMV(Var::Ref{declaration:uri,is_sequence:_}),args,..} => 
                    if let Some(n) = presenter.get_variable_notation(uri) {
                        n.as_ref().apply(presenter,None,None,uri,args)
                    } else {
                        Self::default_oma(presenter, "OMA", uri, uri.name().last_name().as_ref(), args)
                    }
                Term::OMA{head:box Term::OMV(Var::Name(name)),args,..} =>
                    Self::default_oma(presenter, "OMA", name, name.last_name().as_ref(), args),


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
                Informal::Term(i) =>
                    if let Some(t) =terms.get(*i as usize) {
                        presenter.cont(t)?;
                    } else {
                        return Err(PresentationError::MalformedNotation(format!(
                            "Term {i} not found in list {terms:?}"
                        )))
                    },
                Informal::Node { tag, attributes, children } =>
                    Self::informal(presenter,tag,attributes,children,terms)?
            }}
            write!(presenter,"</{tag}>").map_err(Into::into)
        }

        /// #### Errors
        fn apply_op<W:std::fmt::Write>(&self,
            out:&mut W,
            termstr:&str,
            head:impl Display,
            args:&impl Args<W>
        ) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const NID:&str = SHTMLKey::NotationId.attr_name();
            if let Some(opn) = &self.op {
                let index = opn.attribute_index as usize;
                let start = &opn.text[0..index];
                let end = &opn.text[index..];
                write!(out,"{start} {TERM}=\"{termstr}\" {HEAD}=\"{head}\" {NID}=\"{}\"{end}",self.id).map_err(Into::into)
            } else {
                self.apply_i(out,None,termstr,head,args)
            }
        }
        fn apply_op_this(&self,
            presenter:&mut impl Presenter,
            this:&Term,
            termstr:&str,
            head:impl Display
        ) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const NID:&str = SHTMLKey::NotationId.attr_name();
            const HEADTERM:&str = SHTMLKey::HeadTerm.attr_name();
            if let Some(opn) = &self.op {
                write!(presenter,"<msub {TERM}=\"{termstr}\" {HEAD}=\"{head}\" {NID}=\"{}\">{}",self.id,opn.text)?;
                write!(presenter,"<mrow {HEADTERM}>")?;
                presenter.cont(this)?;
                write!(presenter,"</mrow></msub>").map_err(Into::into)
            } else {
                self.apply(presenter,Some(this),Some(termstr),head,&[])
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
            // println!("Here: {:?} \n - {args:?}",self.components);
            let termstr = termstr.unwrap_or_else(|| if args.iter().any(|a| matches!(a.mode,ArgMode::Binding|ArgMode::BindingSequence)) { "OMBIND"} else { "OMA" });
            self.apply_i(presenter,this,termstr,head,&args)
        }

        fn apply_i<W:std::fmt::Write>(
            &self,
            out:&mut W,
            this:Option<&Term>,
            termstr:&str,
            head:impl Display,
            args:&impl Args<W>
        ) -> Result {
            const TERM :&str = SHTMLKey::Term.attr_name();
            const HEAD:&str = SHTMLKey::Head.attr_name();
            const NID:&str = SHTMLKey::NotationId.attr_name();
            //println!("Components: {:?}",self.components);
            let mut comps = self.components.iter();
            match comps.next() {
                Some(NotationComponent::S(start_node)) => {
                    let index = self.attribute_index as usize;
                    let start = &start_node[0..index];
                    let end = &start_node[index..];
                    write!(out,"{start} {TERM}=\"{termstr}\" {HEAD}=\"{head}\" {NID}=\"{}\"{end}",self.id)?;
                    for comp in comps {
                        comp.apply(out,this,args,false)?;
                    }
                    Ok(())
                }
                Some(o) => {
                    write!(out,"<mrow {TERM}=\"{termstr}\" {HEAD}=\"{head}\" {NID}=\"{}\">",self.id)?;
                    o.apply(out,this,args,false)?;
                    for comp in comps {
                        comp.apply(out,this,args,false)?;
                    }
                    write!(out,"</mrow>").map_err(Into::into)
                }
                _ => {
                    Ok(())
                }
            }
        }
    }

    impl NotationComponent {
        fn apply<W:std::fmt::Write>(&self,out:&mut W,this:Option<&Term>,args:&impl Args<W>,in_text:bool) -> Result {
            match self {
                Self::S(s) |
                Self::Comp(s) => out.write_str(s).map_err(Into::into),
                Self::MainComp(s) =>
                    if let Some(this) = this {
                        Self::do_this(out,this,s)
                    } else {
                        out.write_str(s).map_err(Into::into)
                    },
                Self::Arg(idx,mode) =>
                    Self::do_arg(out,*idx,args, *mode,in_text),
                Self::ArgSep { index, mode, sep } => 
                    Self::do_term_ls(out,*mode,*index,args,
                        |p| {
                            //println!("Separator: {sep:?}");
                            for c in sep.iter().skip(1) {
                                c.apply(p, this, args,in_text)?;
                            }
                            Ok(())
                        }
                    ),
                t => write!(out,"<mtext>TODO: {t:?}</mtext>").map_err(Into::into)
            }
        }

        fn do_arg<W:std::fmt::Write>(out:&mut W,idx:u8,args:&impl Args<W>,mode:ArgMode,in_text:bool) -> Result {
            const ARG:&str = SHTMLKey::Arg.attr_name();
            const MODE:&str = SHTMLKey::ArgMode.attr_name();
            match mode {
                ArgMode::Normal|ArgMode::Binding if !in_text => {
                    write!(out,"<mrow {ARG}=\"{idx}\" {MODE}=\"{mode}\">")?;
                    args.single(idx,mode,out)?;
                    write!(out,"</mrow>").map_err(Into::into)
                }
                ArgMode::Sequence|ArgMode::BindingSequence if !in_text => 
                    Self::do_term_ls(out,mode,idx,args,
                        |p| write!(p,"<mo>,</mo>").map_err(Into::into)
                    ),
                _ => write!(out,"<mtext>TODO: argument mode {mode:?}</mtext>").map_err(Into::into)
            }
        }

        fn do_term_ls<W:std::fmt::Write>(out:&mut W,mode:ArgMode,idx:u8,args:&impl Args<W>,sep:impl Fn(&mut W) -> Result) -> Result {
            const ARG:&str = SHTMLKey::Arg.attr_name();
            const MODE:&str = SHTMLKey::ArgMode.attr_name();
            let mut ls = args.sequence(idx,mode)?;
            let mode = match mode {
                ArgMode::Sequence => ArgMode::Normal,
                ArgMode::BindingSequence => ArgMode::Binding,
                _ => unreachable!()
            };
            let Some(first) = ls.next() else { return Ok(())};
            write!(out,"<mrow {ARG}=\"{idx}1\" {MODE}=\"{mode}\">")?;
            //println!("First {idx}{mode}: {first}");
            first(out)?;
            write!(out,"</mrow>")?;
            let mut i = 2;
            for term in ls {
                sep(out)?;
                write!(out,"<mrow {ARG}=\"{idx}{i}\" {MODE}=\"{mode}\">")?;
                //println!("term {i} of {idx}{mode}: {term}");
                term(out)?;
                write!(out,"</mrow>")?;
                i += 1;
            }
            Ok(())//write!(presenter,"</mrow>").map_err(Into::into)
        }

        fn do_this<W:std::fmt::Write>(out:&mut W,this:&Term,main_comp:&str) -> Result {
            write!(out,"<mtext>TODO: this</mtext>").map_err(Into::into)
        }
    }

}