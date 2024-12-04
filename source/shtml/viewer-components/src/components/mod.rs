pub(crate) mod inputref;
pub(crate) mod sections;
pub(crate) mod terms;
mod toc;
pub(crate) mod navigation;
#[cfg(feature="omdoc")]
pub mod omdoc;

use immt_ontology::{narration::{exercises::CognitiveDimension, LOKind}, uris::DocumentElementURI};
pub use inputref::{InputRef,IfInputref};
pub use toc::*;
pub use sections::{OnSectionBegin,OnSectionEnd};

use leptos::prelude::*;
use leptos_dyn_dom::{DomCont, OriginalNode};
use shtml_extraction::{open::OpenSHTMLElement, prelude::SHTMLElements};
use leptos::tachys::view::any_view::AnyView;

#[component]
pub fn SHTMLComponents(#[prop(optional)] in_math:bool, elements:SHTMLElements,orig:OriginalNode) -> impl IntoView {
  let signal = RwSignal::new(false);
  if in_math {
    leptos::either::Either::Left(do_components::<true>(0, elements, orig,signal))
  } else {
    leptos::either::Either::Right(do_components::<false>(0, elements, orig,signal))
  }
}

#[cfg(not(feature="omdoc"))]
type EitherOf<A,B,C,D,E,F,G> = leptos::either::EitherOf7<A,B,C,D,E,F,G>;
#[cfg(feature="omdoc")]
type EitherOf<A,B,C,D,E,F,G,H> = leptos::either::EitherOf8<A,B,C,D,E,F,G,H>;


fn do_components<const MATH:bool>(skip:usize,elements:SHTMLElements,orig:OriginalNode,on_load:RwSignal<bool>) -> impl IntoView {
  if let Some(next) = elements.iter().rev().nth(skip) {
    //tracing::debug!("Doing {next:?} ({:?})",std::thread::current().id());
    match next {
      OpenSHTMLElement::Section { uri,.. } => EitherOf::A(sections::section(uri.clone(),move || do_components::<MATH>(skip+1,elements,orig,on_load).into_any())),
      OpenSHTMLElement::Inputref { uri, id } => EitherOf::B(inputref::inputref(uri.clone(), id)),
      OpenSHTMLElement::IfInputref(b) => EitherOf::C(inputref::if_inputref(*b,orig)),
      OpenSHTMLElement::OpenTerm { term, .. } => {
        #[cfg(feature="omdoc")]
        if MATH {
          let term = term.clone();
          EitherOf::H(terms::math_term(skip,elements,orig,on_load,term))
        } else {
          EitherOf::D(terms::do_term::<_,MATH>(term.clone(),move || 
            do_components::<MATH>(skip+1,elements,orig,on_load).into_any()
          ))
        }

        #[cfg(not(feature="omdoc"))]
        EitherOf::D(terms::do_term::<_,MATH>(term.clone(),move || 
          do_components::<MATH>(skip+1,elements,orig,on_load).into_any()
        ))
      }
      OpenSHTMLElement::Comp | OpenSHTMLElement::MainComp =>
        EitherOf::E(terms::do_comp::<_,MATH>(move|| view!(<DomCont skip_head=true orig on_load cont=crate::iterate/>))),
      OpenSHTMLElement::Arg(arg) =>
        EitherOf::F(terms::do_arg(orig,*arg, move |orig| 
          do_components::<MATH>(skip+1,elements,orig,on_load).into_any()
        )),
      _ => todo!()
    }
  } else {
    EitherOf::G(view!(<DomCont skip_head=true orig on_load cont=crate::iterate/>))
  }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct LOs {
  pub definitions:Vec<DocumentElementURI>,
  pub examples:Vec<DocumentElementURI>,
  pub exercises:Vec<(bool,DocumentElementURI,CognitiveDimension)>
}


pub(crate) trait IntoLOs {
  fn lo_sort(self) -> LOs;
}

impl IntoLOs for Vec<(DocumentElementURI,LOKind)> {
  fn lo_sort(self) -> LOs {
      let mut definitions = Vec::new();
      let mut examples = Vec::new();
      let mut exercises = Vec::new();
      for (uri,k) in self { match k {
        LOKind::Definition => definitions.push(uri),
        LOKind::Example => examples.push(uri),
        LOKind::Exercise(cd) => exercises.push((false,uri,cd)),
        LOKind::SubExercise(cd) => exercises.push((true,uri,cd))
      }}
      LOs { definitions, examples, exercises}
  }
}