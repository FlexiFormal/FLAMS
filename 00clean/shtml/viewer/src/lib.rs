#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use leptos_dyn_dom::{DomChildrenCont, OriginalNode};
use wasm_bindgen::prelude::*;
use leptos::prelude::*;
use shtml_viewer_components::SHTMLDocument;

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
pub fn set_server_url(server_url:String) {
    tracing::debug!("setting server url: {server_url}");
    shtml_viewer_components::config::set_server_url(server_url);
}

#[component]
fn Main(orig: OriginalNode) -> impl IntoView {
    view!(<SHTMLDocument><DomChildrenCont orig cont=shtml_viewer_components::iterate/></SHTMLDocument>)
    /* 
    use immt_web_utils::components::{Popover,PopoverTrigger,DivOrMrow};
    view!(
        //<ConfigProvider>
        <div>"Math: "<math>
            <Popover node_type=DivOrMrow::Mrow>
                    <PopoverTrigger slot><msubsup><mtext>"Trigger"</mtext><mi>"𝛽"</mi><mi>"𝛼"</mi></msubsup></PopoverTrigger>
                <div>"Content"</div>
            </Popover>
        </math>
        </div>
        <div>"Text: "<Popover node_type=DivOrMrow::Div>
            <PopoverTrigger slot>"Trigger"</PopoverTrigger>
            <div>"Content"</div>
        </Popover></div>
        <DomChildren orig />
        //</ConfigProvider>
    )
    */
}
