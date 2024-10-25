/*! Foo Bar
 * 
 * See [endpoints] for public API endpoints
*/
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "ssr", feature = "hydrate", not(doc)))]
compile_error!("features \"ssr\" and \"hydrate\" cannot be enabled at the same time");

#[cfg(feature = "ssr")]
pub mod server;

pub mod router;
pub mod users;
pub mod utils;

#[cfg(feature = "hydrate")]
fn fragment(uri:Option<immt_ontology::uris::URI>,rp:Option<String>,a:Option<immt_ontology::uris::ArchiveId>,p:Option<String>,l:Option<immt_ontology::languages::Language>,d:Option<String>,e:Option<String>,m:Option<String>,s:Option<String>)
-> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(Vec<immt_utils::CSS>,String),leptos::prelude::ServerFnError<String>>> + Send>> {
    Box::pin(crate::router::content::fragment(uri,rp,a,p,l,d,e,m,s))
}

#[cfg(feature = "hydrate")]
#[leptos::wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    //use router::*;
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    shtml_viewer_components::config::ServerConfig::initialize(
        fragment
    );
    leptos::mount::hydrate_body(router::Main);
}

#[cfg(doc)]
pub mod endpoints;