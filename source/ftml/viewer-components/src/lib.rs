#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//#![feature(generic_const_exprs)]
//#![feature(let_chains)]
#![recursion_limit = "256"]

//mod popover;

pub mod components;
pub mod config;
mod extractor;
pub mod remote;
pub mod ts;

use crate::extractor::NodeAttrs;
pub use components::problem::ProblemOptions;
use components::{
    counters::SectionCounters, inputref::InInputRef, FTMLComponents, TOCElem, TOCSource,
};
use config::{FTMLConfig, IdPrefix};
use extractor::DOMExtractor;
use flams_ontology::{
    narration::problems::{CognitiveDimension, ProblemResponse, Solutions},
    uris::{DocumentElementURI, DocumentURI, NarrativeURI, URI},
};
use flams_utils::{prelude::HMap, vecmap::VecMap};
use flams_web_utils::{components::wait_local, do_css, inject_css};
use ftml_extraction::{open::terms::VarOrSym, prelude::*};
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyView;
use leptos::web_sys::Element;
use leptos_dyn_dom::{DomStringCont, DomStringContMath};

use crate::ts::{
    InputRefContinuation, OnSectionTitle, ParagraphContinuation, SectionContinuation,
    SlideContinuation,
};

#[inline]
pub fn is_in_ftml() -> bool {
    with_context::<FTMLConfig, _>(|_| ()).is_some()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AllowHovers(pub bool);
impl AllowHovers {
    pub fn get() -> bool {
        use_context::<Self>().map(|s| s.0).unwrap_or(true)
    }
}

#[component(transparent)]
pub fn FTMLGlobalSetup<Ch: IntoView + 'static>(
    //#[prop(optional)] problems:Option<ProblemOptions>,
    #[prop(default=None)] allow_hovers: Option<bool>,
    #[prop(default=None)] on_section: Option<SectionContinuation>,
    #[prop(default=None)] on_section_title: Option<OnSectionTitle>,
    #[prop(default=None)] on_paragraph: Option<ParagraphContinuation>,
    #[prop(default=None)] on_inpuref: Option<InputRefContinuation>,
    #[prop(default=None)] on_slide: Option<SlideContinuation>,
    #[prop(default=None)] problem_opts: Option<ProblemOptions>,
    children: TypedChildren<Ch>,
) -> impl IntoView {
    let children = children.into_inner();
    if allow_hovers.is_some_and(|e| !e) {
        provide_context(AllowHovers(false))
    }
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(SectionCounters::default());
    provide_context(NarrativeURI::Document(DocumentURI::no_doc()));
    provide_context(FTMLConfig::new());
    provide_context(RwSignal::new(None::<Vec<TOCElem>>));
    provide_context(on_section);
    provide_context(on_section_title);
    provide_context(on_paragraph);
    provide_context(on_inpuref);
    provide_context(on_slide);
    if let Some(problem_opts) = problem_opts {
        provide_context(problem_opts);
    }
    //provide_context(problems.unwrap_or_default());
    children()
}

#[component]
pub fn FTMLDocumentSetup<Ch: IntoView + 'static>(
    uri: DocumentURI,
    #[prop(default=None)] allow_hovers: Option<bool>,
    #[prop(default=None)] on_section: Option<SectionContinuation>,
    #[prop(default=None)] on_section_title: Option<OnSectionTitle>,
    #[prop(default=None)] on_paragraph: Option<ParagraphContinuation>,
    #[prop(default=None)] on_inpuref: Option<InputRefContinuation>,
    #[prop(default=None)] on_slide: Option<SlideContinuation>,
    #[prop(default=None)] problem_opts: Option<ProblemOptions>,
    children: TypedChildren<Ch>,
) -> impl IntoView {
    use crate::components::navigation::{Nav, NavElems, URLFragment};
    let children = children.into_inner();
    inject_css("ftml-comp", include_str!("components/comp.css"));
    //let config = config::ServerConfig::clone_static();
    //provide_context(config);
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    if allow_hovers.is_some_and(|e| !e) {
        provide_context(AllowHovers(false))
    }
    provide_context(InInputRef(false));
    provide_context(RwSignal::new(NavElems {
        ids: HMap::default(),
        titles: HMap::default(),
        initialized: RwSignal::new(false),
    }));
    provide_context(IdPrefix(String::new()));
    provide_context(SectionCounters::default());
    provide_context(RwSignal::new(None::<Vec<TOCElem>>));
    provide_context(URLFragment::new());
    provide_context(NarrativeURI::Document(uri));
    if let Some(on_section) = on_section {
        provide_context(Some(on_section));
    }
    if let Some(on_section_title) = on_section_title {
        provide_context(Some(on_section_title));
    }
    if let Some(on_paragraph) = on_paragraph {
        provide_context(Some(on_paragraph));
    }
    if let Some(on_inpuref) = on_inpuref {
        provide_context(Some(on_inpuref));
    }
    if let Some(on_slide) = on_slide {
        provide_context(Some(on_slide));
    }
    if let Some(problem_opts) = problem_opts {
        provide_context(problem_opts);
    }
    let r = children();
    view! {
        <Nav/>
        {r}
    }
}

#[component]
pub fn FTMLString(html: String) -> impl IntoView {
    view!(<DomStringCont html cont=iterate/>)
}
#[component]
pub fn FTMLStringMath(html: String) -> impl IntoView {
    view!(<math><DomStringContMath html cont=iterate/></math>)
}

pub static RULES: [FTMLExtractionRule<DOMExtractor>; 51] = [
    rule(FTMLTag::Section),
    rule(FTMLTag::SkipSection),
    rule(FTMLTag::Term),
    rule(FTMLTag::Arg),
    rule(FTMLTag::InputRef),
    rule(FTMLTag::Slide),
    rule(FTMLTag::Style),
    rule(FTMLTag::CounterParent),
    rule(FTMLTag::Counter),
    rule(FTMLTag::Comp),
    rule(FTMLTag::VarComp),
    rule(FTMLTag::MainComp),
    rule(FTMLTag::DefComp),
    rule(FTMLTag::Definiendum),
    rule(FTMLTag::IfInputref),
    rule(FTMLTag::Problem),
    rule(FTMLTag::SubProblem),
    rule(FTMLTag::ProblemHint),
    rule(FTMLTag::ProblemSolution),
    rule(FTMLTag::ProblemGradingNote),
    rule(FTMLTag::ProblemMultipleChoiceBlock),
    rule(FTMLTag::ProblemSingleChoiceBlock),
    rule(FTMLTag::ProblemChoice),
    rule(FTMLTag::ProblemFillinsol),
    rule(FTMLTag::SetSectionLevel),
    rule(FTMLTag::Title),
    rule(FTMLTag::ProofTitle),
    rule(FTMLTag::Definition),
    rule(FTMLTag::Paragraph),
    rule(FTMLTag::Assertion),
    rule(FTMLTag::Example),
    rule(FTMLTag::SlideNumber),
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
    rule(FTMLTag::ProofHide),
    rule(FTMLTag::ProofBody),
];

#[cfg_attr(
    all(feature = "csr", not(feature = "ts")),
    wasm_bindgen::prelude::wasm_bindgen
)]
#[cfg_attr(
    any(not(feature = "csr"), feature = "ts"),
    wasm_bindgen::prelude::wasm_bindgen(module = "/ftml-top.js")
)]
/*
#[cfg_attr(all(not(feature="csr"),not(feature="ts")),wasm_bindgen::prelude::wasm_bindgen(module="/ftml-top.js"))]
#[cfg_attr(feature="ts",wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function hasFtmlAttribute(node) {
  if (typeof window === "undefined") { return false; }
  if (node.tagName.toLowerCase() === "img") {
    // replace "srv:" by server url
    const attributes = node.attributes;
    for (let i = 0; i < attributes.length; i++) {
        if (attributes[i].name === 'data-flams-src') {
            const src = attributes[i].value;
            node.setAttribute('src',src.replace('srv:', window.FLAMS_SERVER_URL));
            break;
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

if (typeof window !== "undefined") {
  window.FLAMS_SERVER_URL = "";
}
export function setServerUrl(url) {
  if (typeof window !== "undefined") { window.FLAMS_SERVER_URL = url; }
  set_server_url(url);
}
"#))]
 */
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = "hasFtmlAttribute")]
    fn has_ftml_attribute(node: &leptos::web_sys::Node) -> bool;
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function init_flams_url() { window.FLAMS_SERVER_URL=\"\";}\ninit_flams_url();"
)]
extern "C" {
    pub fn init_flams_url();
}

#[allow(clippy::missing_const_for_fn)]
#[allow(unreachable_code)]
#[allow(clippy::needless_return)]
#[allow(unused_variables)]
pub fn iterate(e: &Element) -> Option<impl FnOnce() -> AnyView> {
    //tracing::info!("iterating {}", e.outer_html());
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        if !has_ftml_attribute(e) {
            //tracing::trace!("No attributes");
            return None;
        }
        //tracing::trace!("Has ftml attributes");
        let sig = expect_context::<RwSignal<DOMExtractor>>();
        let r = sig.update_untracked(|extractor| {
            let mut attrs = NodeAttrs::new(e);
            RULES.applicable_rules(extractor, &mut attrs)
        });
        return r.map(|elements| {
            //tracing::trace!("got elements: {elements:?}");
            let in_math = flams_web_utils::mathml::is(&e.tag_name()).is_some();
            let orig = e.clone().into();
            move || view!(<FTMLComponents orig elements in_math/>).into_any()
        });
    }
    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    {
        None::<fn() -> AnyView>
    }
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
