#[cfg(feature="client")]
fn main() {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    
    let window = web_sys::window().expect("no global `window` exists");
    let query_url = window.get("query_url").expect("variable `query_url` not set!");
    let query_url = query_url.as_string().unwrap();
    log::info!("query_url: {}",query_url);
    
    let graph_name = window.location().search().ok().map(|s| s.trim_start_matches('?').to_string()).unwrap_or_default();
    log::info!("Graph name: {}",graph_name);

    wasm_bindgen_futures::spawn_local(async {
        let graph = Box::new(immt_graphs::GraphApp::new(query_url,graph_name).await);
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|_| graph)
            )
            .await
            .expect("failed to start eframe");
    });
}