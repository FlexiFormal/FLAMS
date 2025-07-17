#![allow(clippy::must_use_candidate)]
use flams_ontology::uris::{NarrativeUri, Uri};
use flams_web_utils::{components::wait_and_then_fn, do_css};
use ftml_uris::components::{
    DocumentUriComponentTuple, DocumentUriComponents, UriComponentTuple, UriComponents,
    UriComponentsTrait,
};
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
    use thaw::Scrollbar;
    #[cfg(not(feature = "ssr"))]
    let qm = leptos_router::hooks::use_location();
    #[cfg(not(feature = "ssr"))]
    let _ = Effect::new(move |_| {
        let Ok(origin) = window().location().origin() else {
            tracing::error!("Getting URL origin failed");
            panic!("Getting URL origin failed");
        };
        let url = format!(
            "{origin}{}{}{}",
            qm.pathname.get(),
            qm.query.get().to_query_string(),
            qm.hash.get()
        );
        let Ok(js_url) = window().location().href() else {
            tracing::error!("Getting URL failed");
            panic!("Getting URL failed");
        };
        if url != js_url {
            if !window().location().set_href(&url).is_ok() {
                tracing::error!("Updating url failed");
                panic!("Updating url failed");
            }
        }
    });
    view! {
      <Stylesheet id="leptos" href="/pkg/flams.css"/>
      <Themer><FTMLGlobalSetup>//<Login>
      <Scrollbar style="width:100vw;max-height:100vh;">
        <div style="min-height:100vh;color:black;width:min-content">{
          use_query_map().with_untracked(|m| m.as_document().map_or_else(
            |_| match m.as_comps() {
                Ok(uri) => Either::B(view!(<Fragment uri=uri.into()/>)),
                Err(e) => Either::C(flams_web_utils::components::display_error(format!("Invalid URI: {e}").into()))
            },
            |doc| Either::A(view!(<Document doc=doc.into()/>))
          ))
        }</div>
      </Scrollbar>//</Login>
      </FTMLGlobalSetup></Themer>
    }
}

#[component]
pub fn DocumentOfTop(uri: Uri) -> impl IntoView {
    use leptos_router::components::Redirect;
    wait_and_then_fn(
        move || super::server_fns::document_of(uri.clone()),
        |u| view!(<Redirect path=format!("/?uri={}",urlencoding::encode(&u.to_string()))/>),
    )
}

#[component]
pub fn Fragment(uri: UriComponents) -> impl IntoView {
    wait_and_then_fn(
        move || UriComponentTuple::from(uri.clone()).apply1(super::server_fns::fragment, None),
        move |(uri, css, html)| {
            for css in css {
                do_css(css);
            }
            //leptos::logging::log!("Here 2: {html}");
            if let Uri::DocumentElement(uri) = uri {
                leptos::either::Either::Left(view! {
                    //<pre>"Here: "{html.clone()}</pre>
                    <div><FragmentString html uri/></div>
                })
            } else {
                leptos::either::Either::Right(view! {
                    //<pre>"Here: "{html.clone()}</pre>
                    <div style="padding: 0 60px;--rustex-this-width:590px;">
                    <FragmentString html/>
                    </div>
                })
            }
        },
    )
}

#[component]
pub fn Document(doc: DocumentUriComponents) -> impl IntoView {
    wait_and_then_fn(
        move || DocumentUriComponentTuple::from(doc.clone()).apply(super::server_fns::document),
        |(uri, css, html)| {
            for css in css {
                do_css(css);
            }
            view! {<div>
                <DocumentString html uri toc=TOCSource::Get omdoc=OMDocSource::Get/>
            </div>}
        },
    )
}

#[component]
pub fn DocumentInner(doc: DocumentUriComponents) -> impl IntoView {
    let doc: UriComponents = doc.into();
    wait_and_then_fn(
        move || UriComponentTuple::from(doc.clone()).apply1(super::server_fns::fragment, None),
        |(_, css, html)| {
            view! {<div>{
                for css in css { do_css(css); }
                FragmentString(FragmentStringProps{html,uri:None})
            }</div>}
        },
    )
}
