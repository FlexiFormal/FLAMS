pub(crate) mod inputref;
pub(crate) mod sections;
pub(crate) mod terms;
pub(crate) mod exercise;
pub mod documents;
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
  if in_math {
    leptos::either::Either::Left(do_components::<true>(0, elements, orig))
  } else {
    leptos::either::Either::Right(do_components::<false>(0, elements, orig))
  }
}

fn do_components<const MATH:bool>(skip:usize,elements:SHTMLElements,orig:OriginalNode) -> impl IntoView {
  if let Some(next) = elements.iter().rev().nth(skip) {
    //tracing::debug!("Doing {next:?} ({:?})",std::thread::current().id());
    match next {
      OpenSHTMLElement::Section { uri,.. } => sections::section(uri.clone(),move || do_components::<MATH>(skip+1,elements,orig)).into_any(),
      OpenSHTMLElement::Inputref { uri, id } => inputref::inputref(uri.clone(), id).into_any(),
      OpenSHTMLElement::IfInputref(b) => inputref::if_inputref(*b,orig).into_any(),
      OpenSHTMLElement::OpenTerm { term, .. } => {
        #[cfg(feature="omdoc")]
        if MATH {
          let term = term.clone();
          terms::math_term(skip,elements,orig,term).into_any()
        } else {
          terms::do_term::<_,MATH>(term.clone(),move || 
            do_components::<MATH>(skip+1,elements,orig)
          ).into_any()
        }

        #[cfg(not(feature="omdoc"))]
        terms::do_term::<_,MATH>(term.clone(),move || 
          do_components::<MATH>(skip+1,elements,orig)
        ).into_any()
      }
      OpenSHTMLElement::Comp | OpenSHTMLElement::MainComp =>
        terms::do_comp::<_,MATH>(move|| view!(<DomCont skip_head=true orig cont=crate::iterate/>)).into_any(),
      OpenSHTMLElement::Arg(arg) =>
        terms::do_arg(orig,*arg, move |orig| 
          do_components::<MATH>(skip+1,elements,orig)
        ).into_any(),
      OpenSHTMLElement::Exercise { uri, autogradable, sub_exercise,.. } =>
        {
          exercise::exercise(&uri.clone(), *autogradable, *sub_exercise,
            move || do_components::<MATH>(skip+1,elements,orig)
          ).into_any()
        },
      OpenSHTMLElement::ProblemHint => {
        exercise::hint(
          move || do_components::<MATH>(skip+1,elements,orig)
        ).into_any()
      }
      OpenSHTMLElement::ExerciseSolution(id) => {
        let id = id.clone();
        exercise::solution(skip+1,elements,orig,id).into_any()
      }
      OpenSHTMLElement::ExerciseGradingNote => {
        exercise::gnote(skip+1,elements,orig).into_any()
      }
      OpenSHTMLElement::ChoiceBlock { multiple, inline } => {
        exercise::choice_block(*multiple,*inline,
          move || do_components::<MATH>(skip+1,elements,orig)
        ).into_any()
      }
      OpenSHTMLElement::ProblemChoice => {
        exercise::problem_choice(
          move || do_components::<MATH>(skip+1,elements.clone(),orig.clone())
        ).into_any()
      }
      OpenSHTMLElement::Fillinsol(wd) => {
        exercise::fillinsol(*wd).into_any()
      }
      _ => todo!()
    }
  } else {
    view!(<DomCont skip_head=true orig cont=crate::iterate/>).into_any()
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