#![cfg_attr(docsrs, feature(doc_cfg))]

use leptos::prelude::*;
use leptos_dyn_dom::{DomChildrenCont, OriginalNode};
use shtml_viewer_components::SHTMLDocument;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
/// sets the server url used to the provided one; by default `https://immt.mathhub.info`.
pub fn set_server_url(server_url: String) {
    tracing::debug!("setting server url: {server_url}");
    shtml_viewer_components::config::set_server_url(server_url);
}


#[cfg(any(doc,not(feature="ts")))]
#[cfg_attr(docsrs, doc(cfg(not(feature = "ts"))))]
#[wasm_bindgen(start)]
pub fn run() {
    //console_error_panic_hook::set_once();

    use immt_ontology::uris::DocumentURI;
    #[allow(unused_mut)]
    let mut config = tracing_wasm::WASMLayerConfigBuilder::new();
    //#[cfg(not(debug_assertions))]
    //config.set_max_level(tracing::Level::INFO);
    tracing_wasm::set_as_global_default_with_config(config.build());

    leptos_dyn_dom::hydrate_body(|orig| {
        leptos_meta::provide_meta_context();
        let on_load = RwSignal::new(false);
        view!(<SHTMLDocument on_load uri=DocumentURI::no_doc()>
            <DomChildrenCont orig on_load cont=shtml_viewer_components::iterate/>
            </SHTMLDocument>
        )

    });
}

#[cfg(feature="ts")]
mod ts;
#[cfg(feature="ts")]
#[cfg_attr(docsrs, doc(cfg(feature = "ts")))]
pub use ts::*;

#[cfg(all(feature="ts",doc))]
pub use shtml_viewer_components::components::{TOC,TOCElem};