#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(let_chains)]

//mod popover;

mod extractor;
pub mod components;
pub mod config;

use components::{inputref::InInputRef, SHTMLComponents, TOCSource};
use immt_utils::prelude::HMap;
use immt_web_utils::{components::wait, do_css, inject_css};
use leptos::prelude::*;
use leptos_dyn_dom::{DomStringCont, DomStringContMath};
use shtml_extraction::{open::terms::VarOrSym, prelude::*};
use leptos::tachys::view::any_view::AnyView;
use leptos::web_sys::Element;
use extractor::DOMExtractor;
use crate::extractor::NodeAttrs;
use immt_ontology::{narration::exercises::CognitiveDimension, uris::{DocumentElementURI, DocumentURI, NarrativeURI, URI}};

#[derive(Debug,Clone)]
struct IdPrefix(pub String);
impl IdPrefix {
    pub fn new_id(self,s:&str) -> String {
        if self.0.is_empty() {
            s.to_string()
        } else {
            format!("{}/{s}",self.0)
        }
    }
}

#[cfg(not(feature="omdoc"))]
#[component]
pub fn DocumentFromURI(
    uri:DocumentURI,
    #[prop(optional,into)] toc:TOCSource
) -> impl IntoView {
    wait(
        move || {
            tracing::info!("fetching {uri}");
            let fut = config::server_config.full_doc(uri.clone());
            async move {fut.await.ok()}
        }, 
        move |(uri,css,html)| {
            for c in css { do_css(c); }
            view!(<DocumentString html uri toc=toc.clone()/>)
        }, 
        "Error loading document reference".to_string(),
    )
}

#[cfg(feature="omdoc")]
#[component]
pub fn DocumentFromURI(
    uri:DocumentURI,
    #[prop(optional,into)] toc:TOCSource,
    #[prop(optional)] omdoc:components::omdoc::OMDocSource
) -> impl IntoView {
    wait(
        move || {
            tracing::info!("fetching {uri}");
            let fut = config::server_config.full_doc(uri.clone());
            async move {fut.await.ok()}
        }, 
        move |(uri,css,html)| {
            for c in css { do_css(c); }
            view!(<DocumentString html uri toc=toc.clone() omdoc=omdoc.clone()/>)
        }, 
        "Error loading document reference".to_string(),
    )
}

#[cfg(not(feature="omdoc"))]
#[component]
pub fn DocumentString(
    html:String,
    #[prop(optional)] uri:Option<DocumentURI>,
    #[prop(optional,into)] toc:TOCSource
) -> impl IntoView {
    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!(toc,TOCSource::None);
    view! {<SHTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </SHTMLDocumentSetup>}
}

#[cfg(feature="omdoc")]
#[component]
pub fn DocumentString(
    html:String,
    #[prop(optional)] uri:Option<DocumentURI>,
    #[prop(optional,into)] toc:TOCSource,
    #[prop(optional)] omdoc:components::omdoc::OMDocSource
) -> impl IntoView {
    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!((&toc,&omdoc),(TOCSource::None,components::omdoc::OMDocSource::None));
    view! {<SHTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc,omdoc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </SHTMLDocumentSetup>}
}


#[cfg(not(feature="omdoc"))]
fn do_burger(
    toc:components::TOCSource
) -> impl IntoView {
    use immt_web_utils::components::Burger;
    components::do_toc(toc,move |v| view!{
        <Burger>{v}</Burger>
    })
}

#[cfg(feature="omdoc")]
fn do_burger(
    toc:TOCSource,
    omdoc:components::omdoc::OMDocSource
) -> impl IntoView {
    use immt_web_utils::components::Burger;
    components::do_toc(toc,move |v| view!{
        <Burger>{components::omdoc::do_omdoc(omdoc)}{v}</Burger>
    })
}

#[cfg(feature="omdoc")]
#[derive(Clone)]
pub(crate) struct NotationForces {
    owner: Owner,
    map: StoredValue<immt_utils::prelude::HMap<URI,RwSignal<Option<DocumentElementURI>>>>
}
#[cfg(feature="omdoc")]
impl NotationForces {
    pub fn get(&self,uri:&URI) -> RwSignal<Option<DocumentElementURI>> {
        self.owner.with(||
            self.map
                .with_value(|map| map.get(uri).copied())
                .unwrap_or_else(|| {
                    #[cfg(any(feature="csr",feature="hydrate"))]
                    let sig = {
                        use gloo_storage::Storage;
                        let s = gloo_storage::LocalStorage::get(format!("notation_{uri}"))
                            .map_or_else(
                            |_| RwSignal::new(None),
                            |v:DocumentElementURI| {
                                let uri = uri.clone();
                                let sig = RwSignal::new(None);
                                let _ = Resource::new(|| (),move |()| {let uri = uri.clone(); let v = v.clone();async move {
                                    let _ = crate::config::server_config.notations(uri).await;
                                    sig.set(Some(v));
                                }}
                                );
                                sig
                            }
                        );
                        let uri = uri.clone();
                        Effect::new(move || {
                            s.with(|s|
                                if let Some(s) = s.as_ref() {
                                    let _ = gloo_storage::LocalStorage::set(format!("notation_{uri}"),&s);
                                } else {
                                    let _ = gloo_storage::LocalStorage::delete(format!("notation_{uri}"));
                                }
                            );
                        });
                        s
                    };
                    #[cfg(not(any(feature="csr",feature="hydrate")))]
                    let sig = RwSignal::new(None);
                    self.map.update_value(|map| {map.insert(uri.clone(),sig);});
                    sig
                }
            )
        )
    }
    pub fn new() -> Self {
        let owner = Owner::new();//current().expect("Something went horribly wrong");
        Self {
            owner,
            map:StoredValue::new(immt_utils::prelude::HMap::default())
        }
    }

    pub fn do_in<R>(&self,f:impl FnOnce() -> R) -> R {
        self.owner.clone().with(f)
    }
}

#[derive(Clone)]
pub(crate) struct OnClickProvider {
    owner: Owner,
    map: StoredValue<immt_utils::prelude::HMap<VarOrSym,RwSignal<bool>>>
}
impl OnClickProvider {
    pub fn new() -> Self {
        let owner = Owner::new();//.expect("Something went horribly wrong");
        Self {
            owner,
            map:StoredValue::new(immt_utils::prelude::HMap::default())
        }
    }
    pub fn get(&self,uri:&VarOrSym) -> RwSignal<bool> {
        use thaw::{Dialog,DialogSurface};
        use thaw::{Combobox,ComboboxOption,ComboboxOptionGroup,Divider};
        use crate::components::terms::do_onclick;
        if let Some(s) = self.map.with_value(|map| map.get(uri).copied()) {
            return s
        }
        self.owner.with(move || {
            let signal = RwSignal::new(false);
            let uri = uri.clone();
            self.map.update_value(|map| {map.insert(uri.clone(),signal);});
            let _ = view!{<Dialog open=signal><DialogSurface>{
                    do_onclick(uri)
            }</DialogSurface></Dialog>};
            signal
        })
    }
}


#[component(transparent)]
pub fn SHTMLGlobalSetup<Ch:IntoView+'static>(
    children: TypedChildren<Ch>
) -> impl IntoView {
    let children = children.into_inner();
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    #[cfg(feature="omdoc")]
    provide_context(NotationForces::new());
    provide_context(OnClickProvider::new());
    children()
}

#[component]
pub fn SHTMLDocumentSetup<Ch:IntoView+'static>(
    uri:DocumentURI, 
    children: TypedChildren<Ch>, 
    #[prop(optional)] on_load:Option<RwSignal<bool>>
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
        <Nav on_load/>
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

pub static RULES:[SHTMLExtractionRule<DOMExtractor>;33] = [
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

#[cfg_attr(feature="csr",wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(not(feature="csr"),wasm_bindgen::prelude::wasm_bindgen(module="/shtml-top.js"))]
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