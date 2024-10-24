use leptos::prelude::*;
use leptos_dyn_dom::{DomChildrenCont, OriginalNode};
use shtml_viewer_components::SHTMLDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    //console_error_panic_hook::set_once();
    #[allow(unused_mut)]
    let mut config = tracing_wasm::WASMLayerConfigBuilder::new();
    //#[cfg(not(debug_assertions))]
    //config.set_max_level(tracing::Level::INFO);
    tracing_wasm::set_as_global_default_with_config(config.build());

    leptos_dyn_dom::hydrate_body(|orig| view!(<Main orig/>));
}

#[wasm_bindgen]
/// sets the server url used to the provided one; by default `https://immt.mathhub.info`.
pub fn set_server_url(server_url: String) {
    tracing::debug!("setting server url: {server_url}");
    shtml_viewer_components::config::set_server_url(server_url);
}

#[component]
fn Main(orig: OriginalNode) -> impl IntoView {
    leptos_meta::provide_meta_context();
    let on_load = RwSignal::new(false);
    view!(<SHTMLDocument on_load>
        <DomChildrenCont orig on_load cont=shtml_viewer_components::iterate/>
        </SHTMLDocument>
    )
}

#[cfg(feature="ts")]
mod ts {
    use shtml_viewer_components::components::{TOCElem, TOC};
    use immt_ontology::uris::DocumentURI;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[derive(Debug,Clone,serde::Serialize,serde::Deserialize,tsify_next::Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    /// Options for rendering an SHTML document
    /// uri: the URI of the document (as string)
    /// toc: if defined, will render a table of contents for the document
    pub struct ShtmlOptions {
        #[tsify(type = "string")]
        pub uri:DocumentURI,
        pub toc:Option<TOCOptions>
    }

    #[derive(Debug,Clone,serde::Serialize,serde::Deserialize,tsify_next::Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    /// Options for rendering a table of contents
    /// `FromBackend` will retrieve it from the remote backend
    /// `Predefined(toc)` will render the provided TOC
    pub enum TOCOptions {
        FromBackend,
        Predefined(Vec<TOCElem>)
    }
    
    #[wasm_bindgen]
    /// render an SHTML document to the provided element
    pub fn render_document(options:ShtmlOptions,to:leptos::web_sys::HtmlElement) -> Result<(),String> {
        todo!()
    }
}

#[cfg(feature="ts")]
pub use ts::*;
