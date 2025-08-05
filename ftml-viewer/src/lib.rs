#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// sets the server url used to the provided one; by default `https://mathhub.info`.
pub fn set_server_url(server_url: String) {
    console_error_panic_hook::set_once();
    tracing::debug!("setting server url: {server_url}");
    ftml_viewer_components::remote::set_server_url(server_url);
}

pub use ftml_viewer_components::remote::get_server_url;

#[cfg(any(doc, not(feature = "ts")))]
#[wasm_bindgen(start)]
pub fn run() {
    use flams_ontology::uris::DocumentURI;
    use flams_web_utils::components::Themer;
    use ftml_viewer_components::{FTMLDocumentSetup, FTMLGlobalSetup};
    use leptos::prelude::*;
    use leptos_posthoc::{DomChildrenCont, OriginalNode};
    #[allow(unused_mut)]
    let mut config = tracing_wasm::WASMLayerConfigBuilder::new();

    console_error_panic_hook::set_once();
    //#[cfg(not(debug_assertions))]
    //config.set_max_level(tracing::Level::INFO);
    tracing_wasm::set_as_global_default_with_config(config.build());

    leptos_posthoc::hydrate_body(|orig| {
        leptos_meta::provide_meta_context();
        view!(<Themer attr:style="font-family:inherit;font-size:inherit;font-weight:inherit;line-height:inherit;background-color:inherit;color:inherit;display:contents;"><FTMLGlobalSetup><FTMLDocumentSetup uri=DocumentURI::no_doc()>
            <DomChildrenCont orig cont=ftml_viewer_components::iterate/>
            </FTMLDocumentSetup></FTMLGlobalSetup></Themer>
        )
    });
}

#[cfg(feature = "ts")]
mod ts;
#[cfg(feature = "ts")]
pub use ts::*;

#[cfg(all(feature = "ts", doc))]
pub use ftml_viewer_components::components::TOCElem;
