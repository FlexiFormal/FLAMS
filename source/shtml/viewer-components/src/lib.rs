#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(generic_const_exprs)]
#![feature(let_chains)]
#![recursion_limit = "256"]

//mod popover;

mod extractor;
pub mod components;
pub mod remote;
pub mod config;
pub mod ts;

use components::{inputref::InInputRef, SHTMLComponents, TOCSource};
use config::{IdPrefix, SHTMLConfig};
use immt_utils::{prelude::HMap, vecmap::VecMap};
use immt_web_utils::{components::wait, do_css, inject_css};
use leptos::prelude::*;
use leptos_dyn_dom::{DomStringCont, DomStringContMath};
use shtml_extraction::{open::terms::VarOrSym, prelude::*};
use leptos::tachys::view::any_view::AnyView;
use leptos::web_sys::Element;
use extractor::DOMExtractor;
use crate::extractor::NodeAttrs;
use immt_ontology::{narration::exercises::{CognitiveDimension, ExerciseResponse, Solutions}, uris::{DocumentElementURI, DocumentURI, NarrativeURI, URI}};
pub use components::exercise::ExerciseOptions;

/*
#[derive(Debug,Clone,Default,serde::Serialize,serde::Deserialize)]
#[cfg_attr(feature="ts",derive(tsify_next::Tsify))]
#[cfg_attr(feature="ts",tsify(into_wasm_abi, from_wasm_abi))]
pub struct ExerciseOptions {
  pub responses:VecMap<DocumentElementURI,(Solutions,ExerciseResponse)>
}
*/

#[inline]
pub fn is_in_shtml() -> bool {
    with_context::<SHTMLConfig,_>(|_| ()).is_some()
}

#[component(transparent)]
pub fn SHTMLGlobalSetup<Ch:IntoView+'static>(
    //#[prop(optional)] exercises:Option<ExerciseOptions>,
    children: TypedChildren<Ch>
) -> impl IntoView {
    let children = children.into_inner();
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(SHTMLConfig::new());
    //provide_context(exercises.unwrap_or_default());
    children()
}

#[component]
pub fn SHTMLDocumentSetup<Ch:IntoView+'static>(
    uri:DocumentURI, 
    children: TypedChildren<Ch>
) -> impl IntoView {
    use crate::components::navigation::{Nav,NavElems,URLFragment};
    let children = children.into_inner();
    inject_css("shtml-comp", include_str!("components/comp.css"));
    //let config = config::ServerConfig::clone_static();
    //provide_context(config);
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(InInputRef(false));
    provide_context(RwSignal::new(NavElems{ids:HMap::default(),titles:HMap::default()}));
    provide_context(IdPrefix(String::new()));
    provide_context(URLFragment::new());
    provide_context(NarrativeURI::Document(uri));
    let r = children();
    view! {
        <Nav/>
        {r}
    }
}

#[component]
pub fn SHTMLString(html:String) -> impl IntoView {
    view!(<DomStringCont html cont=iterate/>)
}
#[component]
pub fn SHTMLStringMath(html:String) -> impl IntoView {
    view!(<math><DomStringContMath html cont=iterate/></math>)
}

pub static RULES:[SHTMLExtractionRule<DOMExtractor>;34] = [
    rule(SHTMLTag::Section),
    rule(SHTMLTag::Term),
    rule(SHTMLTag::Arg),

    rule(SHTMLTag::InputRef),


    rule(SHTMLTag::Comp),
    rule(SHTMLTag::VarComp),
    rule(SHTMLTag::MainComp),

    rule(SHTMLTag::IfInputref),

    rule(SHTMLTag::Problem),
    rule(SHTMLTag::SubProblem),
    rule(SHTMLTag::ExerciseHint),
    rule(SHTMLTag::ExerciseSolution),
    rule(SHTMLTag::ExerciseGradingNote),
    rule(SHTMLTag::ProblemMultipleChoiceBlock),
    rule(SHTMLTag::ProblemSingleChoiceBlock),
    rule(SHTMLTag::ProblemChoice),
    rule(SHTMLTag::ProblemFillinsol),

    // ---- no-ops --------
    rule(SHTMLTag::ArgMode),
    rule(SHTMLTag::NotationId),
    rule(SHTMLTag::Head),
    rule(SHTMLTag::Language),
    rule(SHTMLTag::Metatheory),
    rule(SHTMLTag::Signature),
    rule(SHTMLTag::Args),
    rule(SHTMLTag::Macroname),
    rule(SHTMLTag::Inline),
    rule(SHTMLTag::Fors),
    rule(SHTMLTag::Id),
    rule(SHTMLTag::NotationFragment),
    rule(SHTMLTag::Precedence),
    rule(SHTMLTag::Role),
    rule(SHTMLTag::Argprecs),
    rule(SHTMLTag::Autogradable),
    rule(SHTMLTag::AnswerClassPts),
];

#[cfg_attr(all(feature="csr",not(feature="ts")),wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(all(not(feature="csr"),not(feature="ts")),wasm_bindgen::prelude::wasm_bindgen(module="/shtml-top.js"))]
#[cfg_attr(feature="ts",wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function hasShtmlAttribute(node) {
  //if (node.tagName.toLowerCase() === "section") {return true}
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
      if (attributes[i].name.startsWith('data-shtml-')) {
          return true;
      }
  }
  return false;
}
"#))]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = "hasShtmlAttribute")]
    fn has_shtml_attribute(node: &leptos::web_sys::Node) -> bool;
}

#[allow(clippy::missing_const_for_fn)]
#[allow(unreachable_code)]
#[allow(clippy::needless_return)]
pub fn iterate(e:&Element) -> Option<impl FnOnce() -> AnyView> {
    //tracing::trace!("iterating {}",e.outer_html());
    #[cfg(any(feature="csr",feature="hydrate"))]
    {
        if !has_shtml_attribute(e) {
            //tracing::trace!("No attributes");
            return None
        }
        //tracing::trace!("Has shtml attributes");
        let sig = expect_context::<RwSignal<DOMExtractor>>();
        let r = sig.update_untracked(|extractor| {
            let mut attrs = NodeAttrs::new(e);
            RULES.applicable_rules(extractor,&mut attrs)
        });
        return r.map(|elements| {
            //tracing::trace!("got elements: {elements:?}");
            let in_math = immt_web_utils::mathml::is(&e.tag_name()).is_some();
            let orig = e.clone().into();
            move || view!(<SHTMLComponents orig elements in_math/>).into_any()
        })
    }
    #[cfg(not(any(feature="csr",feature="hydrate")))]
    {None::<fn() -> AnyView>}
}

/*
#[cfg(feature="ts")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = web_sys::js_sys::Function)]
    #[wasm_bindgen::prelude::wasm_bindgen(typescript_type = "(uri: string) => (((HTMLDivElement) => void) | undefined)")]
    pub type SectionContinuation;

    //#[wasm_bindgen::prelude::wasm_bindgen(method, structural, js_name = call)]
    //fn call(this: &SectionContinuation, uri: wasm_bindgen::JsValue) -> wasm_bindgen::JsValue;
}

#[cfg(not(feature="ts"))]
pub struct SectionContinuation;
 */