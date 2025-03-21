#![allow(clippy::must_use_candidate)]
use crate::uris::{DocURIComponents, URIComponents, URIComponentsTrait};
use flams_ontology::uris::{NarrativeURI, URI};
use flams_web_utils::{components::wait_and_then, do_css};
use ftml_viewer_components::components::{
    TOCSource,
    documents::{DocumentString, FragmentString, FragmentStringProps},
    omdoc::OMDocSource,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component(transparent)]
pub fn URITop() -> impl IntoView {
    use flams_web_utils::components::Themer;
    use ftml_viewer_components::FTMLGlobalSetup;
    use leptos::either::EitherOf3 as Either;
    use leptos_meta::Stylesheet;
    #[cfg(not(feature = "ssr"))]
    let qm = leptos_router::hooks::use_location();
    #[cfg(not(feature = "ssr"))]
    let _ = Effect::new(move |_| {
        let url = format!(
            "{}{}{}{}",
            window()
                .location()
                .origin()
                .expect("Getting URL origin failed"),
            qm.pathname.get(),
            qm.query.get().to_query_string(),
            qm.hash.get()
        );
        let js_url = window().location().href().expect("Getting URL failed");
        if url != js_url {
            window()
                .location()
                .set_href(&url)
                .expect("Updating url failed");
        }
    });
    view! {
      <Stylesheet id="leptos" href="/pkg/flams.css"/>
      <Themer><FTMLGlobalSetup>//<Login>
        <div style="min-height:100vh;color:black;width:min-content">{
          use_query_map().with_untracked(|m| m.as_doc().map_or_else(
            || {
              let Some(uri) = m.as_comps() else {
                return Either::C(flams_web_utils::components::display_error("Invalid URI".into()));
              };
              Either::B(view!(<Fragment uri/>))
            },
            |doc| Either::A(view!(<Document doc/>))
          ))
        }</div>//</Login>
      </FTMLGlobalSetup></Themer>
    }
}

#[component]
pub fn Fragment(uri: URIComponents) -> impl IntoView {
    wait_and_then(
        move || uri.clone().into_args(super::server_fns::fragment),
        move |(uri, css, html)| {
            let uri = if let URI::Narrative(NarrativeURI::Element(uri)) = uri {
                Some(uri)
            } else {
                None
            };
            view! {<div>{
              for css in css { do_css(css); }
              FragmentString(FragmentStringProps{html,uri})
            }</div>}
        },
    )
}

#[component]
pub fn Document(doc: DocURIComponents) -> impl IntoView {
    wait_and_then(
        move || doc.clone().into_args(super::server_fns::document),
        |(uri, css, html)| {
            view! {<div>{
                for css in css { do_css(css); }
                view!(<DocumentString html uri toc=TOCSource::Get omdoc=OMDocSource::Get/>)
            }</div>}
        },
    )
}

#[component]
pub fn DocumentInner(doc: DocURIComponents) -> impl IntoView {
    let doc: URIComponents = doc.into();
    wait_and_then(
        move || doc.clone().into_args(super::server_fns::fragment),
        |(_, css, html)| {
            view! {<div>{
                for css in css { do_css(css); }
                FragmentString(FragmentStringProps{html,uri:None})
            }</div>}
        },
    )
}
