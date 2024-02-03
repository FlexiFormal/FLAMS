pub mod app;
pub mod error_template;

pub mod backend {
    pub mod archives;
}

#[cfg(feature = "ssr")]
pub mod fileserv;

#[cfg(feature = "ssr")]
pub mod controller {
    use std::path::Path;
    use lazy_static::lazy_static;
    use immt_api::backend::Controller;

    lazy_static! {
        pub static ref CONTROLLER:std::sync::Arc<Controller> = {
            let mh = std::env::var("MATHHUB")
                .unwrap_or_else(|_| "/home/jazzpirate/work/MathHub".to_string());
            std::sync::Arc::new(Controller::new(Path::new(&mh)))
        };
    }
}


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    //leptos::mount_to_body(App);
    leptos::leptos_dom::HydrationCtx::stop_hydrating();
}