pub(crate) mod inputref;
pub(crate) mod sections;
mod terms;
mod toc;
pub(crate) mod navigation;
#[cfg(feature="omdoc")]
pub mod omdoc;

pub use inputref::{InputRef,IfInputref};
pub use terms::*;
pub use toc::*;
pub use sections::{OnSectionBegin,OnSectionEnd};

use leptos::prelude::*;
use leptos_dyn_dom::{DomCont, OriginalNode};
use shtml_extraction::{open::OpenSHTMLElement, prelude::SHTMLElements};
use leptos::tachys::view::any_view::AnyView;

#[component]
pub fn SHTMLComponents(#[prop(optional)] in_math:bool, elements:SHTMLElements,orig:OriginalNode) -> AnyView {
  if in_math { 
    do_components::<true>(0, elements, orig) 
  } else {
    do_components::<false>(0, elements, orig)
  }
}

fn do_components<const MATH:bool>(skip:usize,elements:SHTMLElements,orig:OriginalNode) -> AnyView {
  if let Some(next) = elements.iter().nth(skip) {
    tracing::debug!("Doing {next:?} ({:?})",std::thread::current().id());
    match next {
      OpenSHTMLElement::Section { uri,.. } => sections::section(uri.clone(),move || do_components::<MATH>(skip+1,elements,orig)).into_any(),
      OpenSHTMLElement::Inputref { uri, id } => inputref::inputref(uri.clone(), id).into_any(),
      OpenSHTMLElement::IfInputref(b) => inputref::if_inputref(*b,orig).into_any(),
      OpenSHTMLElement::OpenTerm { term, .. } =>
        terms::do_term::<_,MATH>(term.clone(),move || 
          do_components::<MATH>(skip+1,elements,orig)
        ).into_any(),
      OpenSHTMLElement::Comp | OpenSHTMLElement::MainComp =>
        terms::do_comp::<_,MATH>(|| view!(<DomCont orig cont=crate::iterate/>)).into_any(),
      OpenSHTMLElement::Arg { .. } =>
        terms::do_arg(|| view!(<DomCont orig cont=crate::iterate/>)).into_any(),
      _ => todo!()
    }
  } else {
    view!(<DomCont orig cont=crate::iterate/>).into_any()
  }
}