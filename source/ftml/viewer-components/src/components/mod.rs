pub(crate) mod inputref;
pub(crate) mod sections;
pub(crate) mod terms;
pub(crate) mod exercise;
pub(crate) mod paragraphs;
pub mod documents;
mod toc;
pub(crate) mod navigation;
#[cfg(feature="omdoc")]
pub mod omdoc;

use flams_ontology::{narration::{exercises::CognitiveDimension, LOKind}, uris::DocumentElementURI};
use inputref::InInputRef;
pub use inputref::{InputRef,IfInputref};
pub use toc::*;
pub use sections::{OnSectionBegin,OnSectionEnd};

use leptos::prelude::*;
use leptos_dyn_dom::{DomChildrenCont, DomCont, OriginalNode};
use ftml_extraction::{open::OpenFTMLElement, prelude::FTMLElements};
use leptos::tachys::view::any_view::AnyView;

use crate::config::{LogicalLevel, SectionCounters};

#[component]
pub fn FTMLComponents(#[prop(optional)] in_math:bool, elements:FTMLElements,orig:OriginalNode) -> impl IntoView {
  if in_math {
    leptos::either::Either::Left(do_components::<true>(0, elements, orig))
  } else {
    leptos::either::Either::Right(do_components::<false>(0, elements, orig))
  }
}

fn do_components<const MATH:bool>(skip:usize,elements:FTMLElements,orig:OriginalNode) -> impl IntoView {
  if let Some(next) = elements.iter().rev().nth(skip) {
    //tracing::debug!("Doing {next:?} ({:?})",std::thread::current().id());
    match next {
      OpenFTMLElement::Section { uri,.. } => sections::section(uri.clone(),move || do_components::<MATH>(skip+1,elements,orig)).into_any(),
      OpenFTMLElement::Inputref { uri, id } => inputref::inputref(uri.clone(), id).into_any(),
      OpenFTMLElement::IfInputref(b) => inputref::if_inputref(*b,orig).into_any(),
      OpenFTMLElement::OpenTerm { term, .. } => {
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
      OpenFTMLElement::DefComp =>
        terms::do_comp::<_,MATH>(true,move|| view!(<DomCont skip_head=true orig=orig.clone() cont=crate::iterate/>)).into_any(),
      OpenFTMLElement::Comp | OpenFTMLElement::MainComp =>
        terms::do_comp::<_,MATH>(false,move|| view!(<DomCont skip_head=true orig=orig.clone() cont=crate::iterate/>)).into_any(),
      OpenFTMLElement::Definiendum(uri) =>
        terms::do_definiendum::<_,MATH>(move || do_components::<MATH>(skip+1,elements,orig)).into_any(),
      OpenFTMLElement::Arg(arg) =>
        terms::do_arg(orig,*arg, move |orig| 
          do_components::<MATH>(skip+1,elements,orig)
        ).into_any(),
      OpenFTMLElement::Exercise { uri, autogradable, sub_exercise,styles,.. } =>
        {
          let styles = styles.clone();
          exercise::exercise(&uri.clone(), *autogradable, *sub_exercise,styles,
            move || do_components::<MATH>(skip+1,elements,orig)
          ).into_any()
        },
      OpenFTMLElement::ProblemHint => {
        exercise::hint(
          move || do_components::<MATH>(skip+1,elements,orig)
        ).into_any()
      }
      OpenFTMLElement::ExerciseSolution(id) => {
        let id = id.clone();
        exercise::solution(skip+1,elements,orig,id).into_any()
      }
      OpenFTMLElement::ExerciseGradingNote => {
        exercise::gnote(skip+1,elements,orig).into_any()
      }
      OpenFTMLElement::ChoiceBlock { multiple, inline } => {
        exercise::choice_block(*multiple,*inline,
          move || do_components::<MATH>(skip+1,elements,orig)
        ).into_any()
      }
      OpenFTMLElement::ProblemChoice => {
        exercise::problem_choice(
          move || do_components::<MATH>(skip+1,elements.clone(),orig.clone())
        ).into_any()
      }
      OpenFTMLElement::Fillinsol(wd) => {
        exercise::fillinsol(*wd).into_any()
      }
      OpenFTMLElement::SetSectionLevel(level) => {
        use leptos::context::Provider;
        let in_inputref = use_context::<InInputRef>().map(|i| i.0).unwrap_or(false);
        update_context::<SectionCounters,_>(|current| {
          if matches!(current.current,LogicalLevel::None) { 
            current.max = *level;
          } else if !in_inputref {
            tracing::error!("ftml:set-section-level: Section already started");
          }
        });
        ().into_any()
      }
      OpenFTMLElement::Paragraph{kind,inline:false,uri,styles,..} => {
        paragraphs::paragraph(*kind,uri.clone(),styles.clone(),move || do_components::<MATH>(skip+1,elements,orig)).into_any()
      }
      OpenFTMLElement::Paragraph { .. } => do_components::<MATH>(skip+1,elements,orig).into_any(),
      OpenFTMLElement::Title => sections::title(move || view!(<DomChildrenCont orig cont=crate::iterate />)).into_any(),
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