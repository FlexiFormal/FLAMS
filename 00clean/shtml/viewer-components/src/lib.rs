#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]

//mod popover;

mod extractor;
pub mod components;

use components::{inputref::InInputRef, SHTMLComponents};
use leptos::prelude::*;
use shtml_extraction::prelude::*;
use leptos::tachys::view::any_view::AnyView;
use leptos::web_sys::Element;
use extractor::DOMExtractor;
use crate::extractor::NodeAttrs;

pub mod config;

#[component]
pub fn SHTMLDocument(#[prop(optional)] server:Option<String>, children: Children) -> impl IntoView {
    if let Some(server) = server {
        config::set_server_url(server);
    };
    //let config = config::ServerConfig::clone_static();
    //provide_context(config);
    #[cfg(any(feature="csr",feature="hydrate"))]
    provide_context(RwSignal::new(DOMExtractor::default()));
    provide_context(InInputRef(false));
    children()
}

pub static RULES:[SHTMLExtractionRule<DOMExtractor>;22] = [
    SHTMLTag::Term.rule(),
    SHTMLTag::Arg.rule(),

    SHTMLTag::InputRef.rule(),


    SHTMLTag::Comp.rule(),
    SHTMLTag::VarComp.rule(),
    SHTMLTag::MainComp.rule(),

    SHTMLTag::IfInputref.rule(),

    // ---- no-ops --------
    SHTMLTag::ArgMode.rule(),
    SHTMLTag::NotationId.rule(),
    SHTMLTag::Head.rule(),
    SHTMLTag::Language.rule(),
    SHTMLTag::Metatheory.rule(),
    SHTMLTag::Signature.rule(),
    SHTMLTag::Args.rule(),
    SHTMLTag::Macroname.rule(),
    SHTMLTag::Inline.rule(),
    SHTMLTag::Fors.rule(),
    SHTMLTag::Id.rule(),
    SHTMLTag::NotationFragment.rule(),
    SHTMLTag::Precedence.rule(),
    SHTMLTag::Role.rule(),
    SHTMLTag::Argprecs.rule()
];

#[allow(clippy::missing_const_for_fn)]
#[allow(unreachable_code)]
#[allow(clippy::needless_return)]
pub fn iterate(e:&Element) -> Option<AnyView<Dom>> {
    tracing::trace!("iterating {} ({:?})",e.outer_html(),std::thread::current().id());
    #[cfg(any(feature="csr",feature="hydrate"))]
    {
        let sig = expect_context::<RwSignal<DOMExtractor>>();
        let r = sig.update_untracked(|extractor| {
            let mut attrs = NodeAttrs::new(e);
            RULES.applicable_rules(extractor,&mut attrs)
        });
        return r.map(|elements| {
            tracing::trace!("got elements: {elements:?}");
            let in_math = immt_web_utils::mathml::is(&e.tag_name()).is_some();
            let orig = e.clone().into();
            view!(<SHTMLComponents orig elements in_math/>).into_any()
        })
    }
    #[cfg(feature="ssr")]
    {None}
}