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

use components::{inputref::InInputRef, FTMLComponents, TOCSource};
use config::{IdPrefix, FTMLConfig, SectionCounters};
use flams_utils::{prelude::HMap, vecmap::VecMap};
use flams_web_utils::{components::wait, do_css, inject_css};
use leptos::prelude::*;
use leptos_dyn_dom::{DomStringCont, DomStringContMath};
use ftml_extraction::{open::terms::VarOrSym, prelude::*};
use leptos::tachys::view::any_view::AnyView;
use leptos::web_sys::Element;
use extractor::DOMExtractor;
use crate::extractor::NodeAttrs;
use flams_ontology::{narration::exercises::{CognitiveDimension, ExerciseResponse, Solutions}, uris::{DocumentElementURI, DocumentURI, NarrativeURI, URI}};
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
pub fn is_in_ftml() -> bool {
    with_context::<FTMLConfig,_>(|_| ()).is_some()
}

#[component(transparent)]
pub fn FTMLGlobalSetup<Ch:IntoView+'static>(
    //#[prop(optional)] exercises:Option<ExerciseOptions>,
    children: TypedChildren<Ch>
) -> impl IntoView {
    let children = children.into_inner();
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(SectionCounters::default());
    provide_context(NarrativeURI::Document(DocumentURI::no_doc()));
    provide_context(FTMLConfig::new());
    //provide_context(exercises.unwrap_or_default());
    children()
}

#[component]
pub fn FTMLDocumentSetup<Ch:IntoView+'static>(
    uri:DocumentURI, 
    children: TypedChildren<Ch>
) -> impl IntoView {
    use crate::components::navigation::{Nav,NavElems,URLFragment};
    let children = children.into_inner();
    inject_css("ftml-comp", include_str!("components/comp.css"));
    //let config = config::ServerConfig::clone_static();
    //provide_context(config);
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(InInputRef(false));
    provide_context(RwSignal::new(NavElems{ids:HMap::default(),titles:HMap::default()}));
    provide_context(IdPrefix(String::new()));
    provide_context(SectionCounters::default());
    provide_context(URLFragment::new());
    provide_context(NarrativeURI::Document(uri));
    let r = children();
    view! {
        <Nav/>
        {r}
    }
}

#[component]
pub fn FTMLString(html:String) -> impl IntoView {
    view!(<DomStringCont html cont=iterate/>)
}
#[component]
pub fn FTMLStringMath(html:String) -> impl IntoView {
    view!(<math><DomStringContMath html cont=iterate/></math>)
}

pub static RULES:[FTMLExtractionRule<DOMExtractor>;42] = [
    rule(FTMLTag::Section),
    rule(FTMLTag::Term),
    rule(FTMLTag::Arg),

    rule(FTMLTag::InputRef),


    rule(FTMLTag::Comp),
    rule(FTMLTag::VarComp),
    rule(FTMLTag::MainComp),
    rule(FTMLTag::DefComp),

    rule(FTMLTag::Definiendum),
    rule(FTMLTag::IfInputref),

    rule(FTMLTag::Problem),
    rule(FTMLTag::SubProblem),
    rule(FTMLTag::ExerciseHint),
    rule(FTMLTag::ExerciseSolution),
    rule(FTMLTag::ExerciseGradingNote),
    rule(FTMLTag::ProblemMultipleChoiceBlock),
    rule(FTMLTag::ProblemSingleChoiceBlock),
    rule(FTMLTag::ProblemChoice),
    rule(FTMLTag::ProblemFillinsol),
    rule(FTMLTag::SetSectionLevel),
    rule(FTMLTag::Title),
    rule(FTMLTag::Definition),
    rule(FTMLTag::Paragraph),
    rule(FTMLTag::Assertion),
    rule(FTMLTag::Example),

    // ---- no-ops --------
    rule(FTMLTag::ArgMode),
    rule(FTMLTag::NotationId),
    rule(FTMLTag::Head),
    rule(FTMLTag::Language),
    rule(FTMLTag::Metatheory),
    rule(FTMLTag::Signature),
    rule(FTMLTag::Args),
    rule(FTMLTag::Macroname),
    rule(FTMLTag::Inline),
    rule(FTMLTag::Fors),
    rule(FTMLTag::Id),
    rule(FTMLTag::NotationFragment),
    rule(FTMLTag::Precedence),
    rule(FTMLTag::Role),
    rule(FTMLTag::Argprecs),
    rule(FTMLTag::Autogradable),
    rule(FTMLTag::AnswerClassPts),
];

#[cfg_attr(all(feature="csr",not(feature="ts")),wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(all(not(feature="csr"),not(feature="ts")),wasm_bindgen::prelude::wasm_bindgen(module="/ftml-top.js"))]
#[cfg_attr(feature="ts",wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function hasFtmlAttribute(node) {
  if (node.tagName.toLowerCase() === "img") {
    // replace "srv:" by server url
    const attributes = node.attributes;
    for (let i = 0; i < attributes.length; i++) {
        if (attributes[i].name === 'src') {
            const src = attributes[i].value;
            if (src.startsWith('srv:')) {
                attributes[i].value = src.replace('srv:', window.FLAMS_SERVER_URL);
            }
        }
    }
  }
  //if (node.tagName.toLowerCase() === "section") {return true}
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
      if (attributes[i].name.startsWith('data-ftml-')) {
          return true;
      }
  }
  return false;
}

window.FLAMS_SERVER_URL = "https://flams.mathhub.info";

export function setServerUrl(url) {
  window.FLAMS_SERVER_URL = url;
  set_server_url(url);
}
"#))]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = "hasFtmlAttribute")]
    fn has_ftml_attribute(node: &leptos::web_sys::Node) -> bool;
}

#[cfg(feature="hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js="export function init_flams_url() { window.FLAMS_SERVER_URL=\"\";}\ninit_flams_url();")]
extern "C" {
    pub fn init_flams_url();
}

#[allow(clippy::missing_const_for_fn)]
#[allow(unreachable_code)]
#[allow(clippy::needless_return)]
pub fn iterate(e:&Element) -> Option<impl FnOnce() -> AnyView> {
    //tracing::trace!("iterating {}",e.outer_html());
    #[cfg(any(feature="csr",feature="hydrate"))]
    {
        if !has_ftml_attribute(e) {
            //tracing::trace!("No attributes");
            return None
        }
        //tracing::trace!("Has ftml attributes");
        let sig = expect_context::<RwSignal<DOMExtractor>>();
        let r = sig.update_untracked(|extractor| {
            let mut attrs = NodeAttrs::new(e);
            RULES.applicable_rules(extractor,&mut attrs)
        });
        return r.map(|elements| {
            //tracing::trace!("got elements: {elements:?}");
            let in_math = flams_web_utils::mathml::is(&e.tag_name()).is_some();
            let orig = e.clone().into();
            move || view!(<FTMLComponents orig elements in_math/>).into_any()
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