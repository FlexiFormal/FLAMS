#![allow(clippy::must_use_candidate)]

use flams_web_utils::components::wait_and_then_fn;
use ftml_components::{SidebarPosition, config::FtmlConfig};
use ftml_dom::{FtmlViews, structure::TocSource, utils::css::CssExt};
use ftml_uris::{
    DocumentUri, Uri,
    components::{
        DocumentUriComponentTuple, DocumentUriComponents, UriComponentTuple, UriComponents,
        UriComponentsTrait,
    },
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component(transparent)]
pub fn URITop() -> AnyView {
    ftml_dom::global_setup(move || {
        crate::Views::top(move || {
            use_query_map().with_untracked(|m| {
                m.as_document().map_or_else(
                    |_| match m.as_comps() {
                        Ok(uri) => view!(<Fragment uri=uri.into() position=SidebarPosition::Next/>)
                            .into_any(),
                        Err(e) => flams_web_utils::components::display_error(
                            format!("Invalid URI: {e}").into(),
                        )
                        .into_any(),
                    },
                    |doc| view!(<Document doc=doc.into()/>).into_any(),
                )
            })
        })
    })
    .into_any()
}

#[component]
pub fn DocumentOfTop(uri: Uri) -> AnyView {
    use leptos_router::components::Redirect; // make sure this runs client side rather than server side because of hydration errors
    // I don't understand.
    let sig = RwSignal::new(false);
    Effect::new(move || {
        //sig.track();
        #[cfg(feature = "hydrate")]
        {
            sig.set(true);
        }
    });
    (move || {
        if sig.get() {
            let uri = uri.clone();
            Some(wait_and_then_fn(
                move || super::server_fns::document_of(uri.clone()),
                |u| {
                    view!(<Redirect path=format!("/?uri={}",urlencoding::encode(&u.to_string()))/>)
                        .into_any()
                },
            ))
        } else {
            None
        }
    })
    .into_any()
}

#[component]
pub fn Fragment(uri: UriComponents, position: SidebarPosition) -> AnyView {
    use ftml_dom::utils::css::CssExt;
    // make sure this runs client side rather than server side because of hydration errors
    // I don't understand.
    let sig = RwSignal::new(false);
    Effect::new(move || {
        //sig.track();
        #[cfg(feature = "hydrate")]
        {
            sig.set(true);
        }
    });
    (move || {
        let uri = uri.clone();
        if sig.get() {
            Some(ftml_components::utils::wait_and_then(
                move || UriComponentTuple::from(uri).apply1(super::server_fns::fragment, None),
                move |(uri, css, html)| {
                    for css in css {
                        css.inject();
                    }
                    let uri = match uri {
                        Uri::Document(d) => {
                            FtmlConfig::set_toc_source(TocSource::Get);
                            Some(d.into())
                        }
                        Uri::DocumentElement(d) => {
                            FtmlConfig::set_toc_source(TocSource::None);
                            Some(d.into())
                        }
                        _ => {
                            FtmlConfig::set_toc_source(TocSource::None);
                            None
                        }
                    };
                    crate::Views::render_fragment::<crate::backend::FtmlBackend>(
                        uri,
                        position,
                        true,
                        move || crate::Views::render_ftml(html.into_string(), None).into_any(),
                    )
                    .into_any()
                },
                |e| view!(<span style="color:red">{e.to_string()}</span>).into_any(),
            ))
        } else {
            None
        }
    })
    .into_any()
    //})
}

#[component]
pub fn Document(doc: DocumentUriComponents) -> AnyView {
    // make sure this runs client side rather than server side because of hydration errors
    // I don't understand.
    let sig = RwSignal::new(false);
    let _ = Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            sig.set(true);
        }
    });
    (move || {
        if sig.get() {
            let doc = doc.clone();
            Some(ftml_components::utils::wait_and_then(
                move || DocumentUriComponentTuple::from(doc).apply(super::server_fns::document),
                move |(uri, css, html)| {
                    for c in css {
                        c.inject();
                    }
                    {
                        FtmlConfig::set_toc_source(TocSource::Get);
                        crate::Views::setup_document::<crate::backend::FtmlBackend>(
                            uri,
                            SidebarPosition::Next,
                            true,
                            move || crate::Views::render_ftml(html.into_string(), None).into_any(),
                        )
                    }
                    .into_any()
                },
                |e| view!(<span style="color:red">{e.to_string()}</span>).into_any(),
            ))
        } else {
            None
        }
    })
    .into_any()
}

#[component]
pub fn DocumentInner(doc: DocumentUriComponents) -> AnyView {
    let doc: UriComponents = doc.into();
    wait_and_then_fn(
        move || UriComponentTuple::from(doc.clone()).apply1(super::server_fns::fragment, None),
        move |(uri, css, html)| {
            for css in css {
                css.inject();
            }
            view! {<div>{
                crate::Views::setup_document::<crate::backend::FtmlBackend>(
                    DocumentUri::no_doc().clone(),
                    SidebarPosition::None,
                    true,
                    move || crate::Views::render_ftml(html.into_string(),None).into_any()
                )
            }</div>}
            .into_any()
        },
    )
    .into_any()
}
