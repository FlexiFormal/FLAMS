#![recursion_limit = "256"]
//#![feature(let_chains)]
/*! Foo Bar
 *
 * See [endpoints] for public API endpoints
*/
#![cfg_attr(docsrs, feature(doc_cfg))]

/*#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");
*/

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use tracing_subscriber::prelude::*;
    fn filter(lvl: tracing::Level) -> tracing_subscriber::filter::Targets {
        tracing_subscriber::filter::Targets::new()
            .with_target("ftml_dom", lvl)
            .with_target("ftml_components", lvl)
            .with_target("ftml_parser", lvl)
            .with_target("ftml_backend", lvl)
            .with_target("ssr_example", lvl)
            .with_target(
                "leptos_posthoc",
                tracing_subscriber::filter::LevelFilter::ERROR,
            )
    }
    console_error_panic_hook::set_once();
    tracing_subscriber::registry()
        .with(tracing_wasm::WASMLayer::default())
        .with(filter(tracing::Level::WARN))
        .init();
    leptos::mount::hydrate_body(flams_router_dashboard::Main);
}

#[cfg(any(doc, feature = "docs"))]
pub mod endpoints;
