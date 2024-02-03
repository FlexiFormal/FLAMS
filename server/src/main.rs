
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).try_init().unwrap();
    setup_env();
    immt_server::controller::CONTROLLER.archives();
    start_server().await;
}

#[cfg(feature = "ssr")]
async fn start_server() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use immt_server::app::*;
    use immt_server::fileserv::file_and_error_handler;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        //.route("/api/*fn_name",get(leptos_axum::handle_server_fns))
        //.route("/special/*fn_name",post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn setup_env() {
    std::env::set_var("RUST_LOG", "info");
    #[cfg(not(debug_assertions))]
    std::env::set_var("LEPTOS_SITE_ROOT", "web");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
