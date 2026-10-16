#![allow(clippy::must_use_candidate)]

use flams_router_base::maybe_lazy;
use flams_web_utils::{client_only, components::wait_and_then_fn};
use ftml_components::{
    SidebarPosition,
    components::{content::FtmlViewable, terms::inject_comp_css},
};
use ftml_dom::{DocumentState, FtmlViews, toc::TocSource, utils::css::CssExt};
use ftml_uris::{
    DocumentUri, Uri, UriKind,
    components::{
        DocumentUriComponentTuple, DocumentUriComponents, UriComponentTuple, UriComponents,
        UriComponentsTrait,
    },
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

maybe_lazy!(
    TopDocRouter = {
        let params = use_query_map().get_untracked();
        if let Some(p) = params.get_str("uri") {
            let Ok(uri) = <ftml_uris::Uri as std::str::FromStr>::from_str(p) else {
                return view! { <leptos_router::components::Redirect path="/dashboard"/> }
                    .into_any();
            };
            DocumentOfTop(DocumentOfTopProps { uri }).into_any()
        } else {
            view! { <leptos_router::components::Redirect path="/dashboard"/> }.into_any()
        }
    }
);

maybe_lazy!(
    UriTopRouter = {
        let works = use_query_map()
            .with_untracked(|p| p.get_str("a").is_some() || p.get_str("uri").is_some());
        if works {
            URITop()
        } else {
            view! { <leptos_router::components::Redirect path="/dashboard"/> }.into_any()
        }
    }
);

#[component(transparent)]
pub fn URITop() -> AnyView {
    // TODO: this can be optimized!
    ftml_dom::global_setup(move || {
        crate::Views::top(move || {
            use_query_map().with_untracked(|m| {
                if let Ok(doc) = m.as_document() {
                    return view!(<Document doc=doc.into()/>).into_any();
                }
                let kind = match m.kind() {
                    Ok(k) => k,
                    Err(e) => {
                        return flams_web_utils::components::display_error(
                            format!("Invalid URI: {e}").into(),
                        )
                        .into_any();
                    }
                };
                let comps = match m.as_comps() {
                    Ok(k) => k,
                    Err(e) => {
                        return flams_web_utils::components::display_error(
                            format!("Invalid URI: {e}").into(),
                        )
                        .into_any();
                    }
                };
                match kind {
                    UriKind::Base => {
                        view! { <leptos_router::components::Redirect path="/dashboard"/> }
                            .into_any()
                    }
                    UriKind::Document =>
                    // unreachable
                    {
                        flams_web_utils::components::display_error("Invalid URI".into()).into_any()
                    }
                    UriKind::DocumentElement | UriKind::Symbol => {
                        view!(<Fragment uri=comps.into() position=SidebarPosition::Next/>)
                            .into_any()
                    }
                    UriKind::Module => {
                        let comps: UriComponents = comps.into();
                        client_only!(view!(<DoModule comps=comps.clone()/>)).into_any()
                    }
                    UriKind::Archive | UriKind::Path => {
                        let comps: UriComponents = comps.into();
                        client_only!(
                            view!(<super::archive_views::ArchiveView comps = comps.clone() />)
                        )
                        .into_any()
                    }
                }
            })
        })
    })
    .into_any()
}

#[component]
fn DoModule(comps: UriComponents) -> impl IntoView {
    let comps = UriComponentTuple::from(comps);
    let uri = if let Some(Uri::Module(uri)) = comps.uri {
        Some(uri)
    } else {
        None
    };
    let a = comps.a;
    let p = comps.p;
    let m = comps.m;
    inject_comp_css();
    wait_and_then_fn(
        move || crate::server_fns::get_module(uri.clone(), a.clone(), p.clone(), m.clone()),
        |r| DocumentState::no_document(move || r.as_view()),
    )
}

#[component]
pub fn DocumentOfTop(uri: Uri) -> AnyView {
    use leptos_router::components::Redirect;
    client_only!({
        let uri = uri.clone();
        wait_and_then_fn(
            move || super::server_fns::document_of(uri.clone()),
            |u| {
                view!(<Redirect path=format!("/?uri={}",urlencoding::encode(&u.to_string()))/>)
                    .into_any()
            },
        )
    })
    .into_any()
}

#[component]
pub fn Fragment(uri: UriComponents, position: SidebarPosition) -> AnyView {
    use ftml_dom::utils::css::CssExt;
    let f = move || UriComponentTuple::from(uri).apply1(super::server_fns::fragment, None);
    client_only!({
        ftml_components::utils::wait_and_then(
            f.clone(),
            move |(uri, css, html)| {
                for css in css {
                    css.inject();
                }
                let (uri, src) = match uri {
                    Uri::Document(d) => {
                        //FtmlConfig::set_toc_source(TocSource::Get);
                        (Some(d.into()), TocSource::Get)
                    }
                    Uri::DocumentElement(d) => {
                        //FtmlConfig::set_toc_source(TocSource::None);
                        (Some(d.into()), TocSource::None)
                    }
                    _ => {
                        //FtmlConfig::set_toc_source(TocSource::None);
                        (None, TocSource::None)
                    }
                };
                crate::Views::render_fragment(uri, position, true, src, move || {
                    crate::Views::render_ftml(html.into_string(), None).into_any()
                })
            },
            |e| view!(<span style="color:red">{e.to_string()}</span>).into_any(),
        )
    })
    .into_any()
    //})
}

#[component]
pub fn Document(doc: DocumentUriComponents) -> AnyView {
    client_only!({
        let doc = doc.clone();
        ftml_components::utils::wait_and_then(
            move || DocumentUriComponentTuple::from(doc).apply(super::server_fns::document),
            move |(uri, css, html)| {
                for c in css {
                    c.inject();
                }
                {
                    //FtmlConfig::set_toc_source(TocSource::Get);
                    crate::Views::setup_document(
                        uri,
                        SidebarPosition::Next,
                        true,
                        TocSource::Get,
                        move || crate::Views::render_ftml(html.into_string(), None).into_any(),
                    )
                }
                .into_any()
            },
            |e| view!(<span style="color:red">{e.to_string()}</span>).into_any(),
        )
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
                crate::Views::setup_document(
                    DocumentUri::no_doc().clone(),
                    SidebarPosition::None,
                    true,
                    TocSource::None,
                    move || crate::Views::render_ftml(html.into_string(),None).into_any()
                )
            }</div>}
            .into_any()
        },
    )
    .into_any()
}
