#![recursion_limit = "256"]
//#![feature(let_chains)]
/*! Foo Bar
 *
 * See [endpoints] for public API endpoints
*/
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

#[cfg(feature = "ssr")]
pub mod server;

pub mod router;
pub mod utils;

pub(crate) mod fns {
    use std::{future::Future, pin::Pin};

    use flams_ontology::{
        languages::Language,
        narration::{notations::Notation, LOKind},
        uris::{ArchiveId, DocumentElementURI, DocumentURI, SymbolURI, URI},
    };
    use flams_utils::CSS;
    use ftml_viewer_components::components::{omdoc::AnySpec, TOCElem};
    use leptos::prelude::ServerFnError;

    fn fragment(
        uri: Option<URI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
        e: Option<String>,
        m: Option<String>,
        s: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(URI, Vec<CSS>, String), ServerFnError<String>>> + Send>>
    {
        Box::pin(flams_router_content::server_fns::fragment(
            uri, rp, a, p, l, d, e, m, s,
        ))
    }
    fn full_doc(
        uri: Option<DocumentURI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<(DocumentURI, Vec<CSS>, String), ServerFnError<String>>>
                + Send,
        >,
    > {
        Box::pin(flams_router_content::server_fns::document(
            uri, rp, a, p, l, d,
        ))
    }
    fn toc(
        uri: Option<DocumentURI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<CSS>, Vec<TOCElem>), ServerFnError<String>>> + Send>>
    {
        Box::pin(flams_router_content::server_fns::toc(uri, rp, a, p, l, d))
    }
    fn los(
        uri: Option<SymbolURI>,
        a: Option<ArchiveId>,
        p: Option<String>,
        m: Option<String>,
        s: Option<String>,
        exercises: bool,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<(DocumentElementURI, LOKind)>, ServerFnError<String>>>
                + Send,
        >,
    > {
        Box::pin(flams_router_content::server_fns::los(
            uri, a, p, m, s, exercises,
        ))
    }
    fn omdoc(
        uri: Option<URI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
        e: Option<String>,
        m: Option<String>,
        s: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<CSS>, AnySpec), ServerFnError<String>>> + Send>>
    {
        Box::pin(flams_router_content::server_fns::omdoc(
            uri, rp, a, p, l, d, e, m, s,
        ))
    }
    fn notations(
        uri: Option<URI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
        e: Option<String>,
        m: Option<String>,
        s: Option<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<(DocumentElementURI, Notation)>, ServerFnError<String>>>
                + Send,
        >,
    > {
        Box::pin(flams_router_content::server_fns::notations(
            uri, rp, a, p, l, d, e, m, s,
        ))
    }
    fn solutions(
        uri: Option<URI>,
        rp: Option<String>,
        a: Option<ArchiveId>,
        p: Option<String>,
        l: Option<Language>,
        d: Option<String>,
        e: Option<String>,
        _m: Option<String>,
        _s: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<String, ServerFnError<String>>> + Send>> {
        Box::pin(flams_router_content::server_fns::solution(
            uri, rp, a, p, l, d, e,
        ))
    }
    pub(super) fn init() {
        ftml_viewer_components::remote::ServerConfig::initialize(
            fragment, full_doc, toc, omdoc, los, notations, solutions,
        );
    }
}

#[cfg(feature = "hydrate")]
#[leptos::wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    //use router::*;
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    fns::init();
    leptos::mount::hydrate_body(router::Main);
}

#[cfg(any(doc, feature = "docs"))]
pub mod endpoints;
