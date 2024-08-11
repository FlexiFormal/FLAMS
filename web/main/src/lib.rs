use leptos::*;

//pub mod test;
pub mod components;
mod home;
mod utils;
pub mod accounts;

#[cfg(feature = "client")]
pub mod client {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (
        #[cfg(feature = "client")]
        {$crate::client::log(&format_args!($($t)*).to_string());}
        #[cfg(feature = "server")]
        {println!($($t)*);}
    )
}


#[cfg(feature = "client")]
#[allow(unused_imports)]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use home::*;
    console_error_panic_hook::set_once();
    leptos_dom::HydrationCtx::stop_hydrating();
}

#[cfg(feature = "server")]
pub mod server {
    use std::future::IntoFuture;
    use std::time::Duration;
    use axum::error_handling::HandleErrorLayer;
    use axum::{Extension, extract};
    use axum::response::IntoResponse;
    use http::Request;
    use leptos::{LeptosOptions, provide_context};
    use leptos_axum::generate_route_list;
    use tower::ServiceExt;
    use tower_http::services::ServeDir;
    use tower_sessions::Expiry;
    use tracing::Instrument;
    use immt_controller::{controller, BaseController};
    use crate::accounts::AccountManager;
    use immt_controller::ControllerTrait;
    /*
    macro_rules! files {
        (@file $name:ident:$path:literal => $local:literal) => {
            #[actix_web::get($path)]
            async fn $name(
                leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
            ) -> actix_web::Result<actix_files::NamedFile> {
                let leptos_options = leptos_options.into_inner();
                let site_root = &leptos_options.site_root;
                Ok(actix_files::NamedFile::open(format!(
                    "{site_root}/{}",$local
                ))?)
            }
        };
        (@file $name:ident:$path:literal) => { files!(@file $name:$path => $path); };
        ($($name:ident:$path:literal$(=> $local:literal)?),*) => {
            $( files!(@file $name:$path$(=> $local)?); )*
            macro_rules! with_files { ($app:expr) => { $app$(.service($name))* }; }
        }
    }

    files![
        favicon:"favicon.ico",
        immt_bg:"pkg/immt_bg.wasm" => "pkg/immt.wasm"
    ];

     */


    /** Endpoints:
       ** | Path | Description |
       * | --- | --- |
       * | `/favicon.ico` | favicon.ico |
       * | `/pkg/_` | serve JS/WASM/CSS |
       * | `/graph_viewer` | serve graph viewer api |
       * | `/assets/` | serve other assets from the `assets` directory |
       * | `/log/ws` | logging websocket |
       * | `/` | home page |
       * | `/login` | [login page](crate::accounts::Login) |
       * | `/mathhub` | [MathHub browser](crate::components::mathhub_tree::ArchiveOrGroups) |
       * | `/graphs` | [Graph viewer](crate::components::graph_viewer::GraphTest) |
       * | `/log` | [Log viewer](crate::components::logging::FullLog) |
       * | `/queue` | [Build Queue](crate::components::logging::FullLog) |
       * | `/settings` | [Settings](crate::components::logging::FullLog) |
       * | `/log/ws` | [Logging websocket](crate::components::logging::LogViewer) |
       * | `/_` | [404 - Not Found](super::NotFound) |
       * | --- | --- |
       * | `/api/backend/archives` | [get archives](crate::components::mathhub_tree::get_archives) |
       * | `/api/backend/files_in` | [files in archive/path](crate::components::mathhub_tree::get_files_in) |
       * | `api/log/full` | [full log file](crate::components::logging::full_log) |
       * | `/api/graph` | [Graph viewer api](crate::components::graph_viewer::get_graph) |
       **/

    lazy_static::lazy_static!{
        pub(crate) static ref ADMIN_PWD: Option<String> = {
            let mut pwd = controller().settings().admin_pwd.as_ref()?.to_string();
            use argon2::{Argon2,PasswordHasher,password_hash::{SaltString,rand_core::OsRng}};
            pwd = Argon2::default().hash_password(pwd.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string();
            Some(pwd)
        };
    }

    async fn server_fn_handle(
        auth_session: axum_login::AuthSession<AccountManager>,
        extract::State(state): extract::State<AppState>,
        request:http::Request<axum::body::Body>
    ) -> impl IntoResponse {
        leptos_axum::handle_server_fns_with_context(move || {
            provide_context(auth_session.clone());
            provide_context(state.leptos_options.clone());
            provide_context(state.db.clone())
        },request).in_current_span().await
    }
    async fn file_and_error_handler(
        extract::State(state): extract::State<AppState>,
        request:http::Request<axum::body::Body>
    ) -> axum::response::Response {
        async fn get_static_file(mut request:http::Request<axum::body::Body>,root:&str) -> Result<http::Response<axum::body::Body>,(http::StatusCode,String)> {
            if request.uri().path().ends_with("immt_bg.wasm") {
                // change to "immt.wasm"
                *request.uri_mut() = http::Uri::builder().path_and_query("/pkg/immt.wasm").build().unwrap();
            }
            //println!("Here: {:?}, {root}",request.uri().path());
            match ServeDir::new(root).precompressed_gzip().precompressed_br().oneshot(request).in_current_span().await {
                Ok(res) => Ok(res.into_response()),
                Err(e) => Err((http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))
            }
        }
        let root = &state.leptos_options.site_root;
        let (parts,body) = request.into_parts();
        let mut static_parts = parts.clone();
        static_parts.headers.clear();
        if let Some(encodings) = parts.headers.get("accept-encoding") {
            static_parts.headers.insert("accept-encoding",encodings.clone());
        }
        let result = get_static_file(http::Request::from_parts(static_parts,axum::body::Body::empty()),root).in_current_span().await.unwrap();
        if result.status() == http::StatusCode::OK {
            result
        } else {
            let handler = leptos_axum::render_app_to_stream(state.leptos_options.clone(),crate::home::Main);
            handler(Request::from_parts(parts,body)).in_current_span().await.into_response()
        }
    }

    #[tracing::instrument(skip_all,target="server")]
    pub async fn run_server(site_root: &str) -> std::io::Result<()> {
        use leptos::*;
        use immt_web_orm::MigratorTrait;
        use leptos_axum::*;
        use axum::*;
        use axum_login::AuthManagerLayerBuilder;
        use crate::utils::WebSocket;
        use tracing::Instrument;


        let ip = controller().settings().ip.clone();
        let port = controller().settings().port;

        let mut leptos_options = get_configuration(None).in_current_span().await.unwrap().leptos_options;
        let site_addr = std::net::SocketAddr::new(ip, port);
        leptos_options.site_root = site_root.to_string();
        leptos_options.output_name = "immt".to_string();
        leptos_options.site_addr = site_addr;

        let db = sea_orm::Database::connect(format!("sqlite:{}?mode=rwc", controller().settings().database.display()))
            .in_current_span().await
            .expect("Failed to connect to user database");
        immt_web_orm::Migrator::up(&db, None).in_current_span().await.expect("Failed to migrate database");

        let span = tracing::Span::current();

        let routes = generate_route_list(crate::home::Main);
        let session_store = tower_sessions::MemoryStore::default();//tower_sessions_sqlx_store::SqliteStore::new(db.)
        let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::seconds(60 * 60 * 24 * 7)));
        let auth_service = tower::ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|_| async {http::StatusCode::BAD_REQUEST}))
            .layer(AuthManagerLayerBuilder::new(AccountManager(db.clone()),session_layer).build());

        #[derive(Clone)]
        struct MySpan(tracing::Span);
        impl<A> tower_http::trace::MakeSpan<A> for MySpan {
            fn make_span(&mut self, r: &http::Request<A>) -> tracing::Span {
                let _e = self.0.enter();
                tower_http::trace::DefaultMakeSpan::default().make_span(r)
            }
        }

        let app = axum::Router::<AppState>::new()
            .route("/dashboard/queue/ws",axum::routing::get(crate::components::queue::QueueSocket::ws_handler))
            .route("/dashboard/log/ws",axum::routing::get(crate::components::logging::LogSocket::ws_handler))
            .route("/api/*fn_name", axum::routing::get(server_fn_handle).post(server_fn_handle))
            .route("/content/html",axum::routing::get(crate::components::content::server::get_html).post(crate::components::content::server::get_html))
            .route("/content/*fn_name", axum::routing::get(server_fn_handle).post(server_fn_handle))
            .leptos_routes_with_handler(routes,axum::routing::get(move |
                auth_session: axum_login::AuthSession<AccountManager>,
                extract::State(state): extract::State<AppState>,
                request:http::Request<axum::body::Body>| { async move {
                let handler = leptos_axum::render_app_to_stream_with_context(state.leptos_options.clone(),move || {
                    provide_context(auth_session.clone());
                    provide_context(state.db.clone())
                },crate::home::Main);
                handler(request).in_current_span().await.into_response()
            }.in_current_span()}))
            .fallback(file_and_error_handler)
            .layer(auth_service)
            .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(MySpan(span)));
        let app : Router<()> = app.with_state(AppState {leptos_options,db});
        axum::serve(tokio::net::TcpListener::bind(&site_addr).in_current_span().await.expect("Failed to initialize TCP listener"),
                    app.into_make_service_with_connect_info::<std::net::SocketAddr>()
        ).into_future().in_current_span().await

        /*
        HttpServer::new(move || {
            let site_root = &leptos_options.site_root;
            let routes = generate_route_list(Main);
            //println!("listening on http://{}:{}", ip,port);

            let app = with_files!(actix_web::App::new()
                .wrap(middleware::Compress::default())
            );

            const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;


                // serve JS/WASM/CSS from `pkg`
            let app = app.route("/log/ws", web::get().to(crate::components::logging::LogViewer::start))
                .service(Files::new("/pkg", format!("{site_root}/pkg")))
                .service(Files::new("/graph_viewer", format!("{site_root}/graphs")))
                // serve other assets from the `assets` directory
                .service(Files::new("/assets", site_root))
                .leptos_routes(leptos_options.to_owned(), routes.to_owned(), Main)
                .app_data(web::Data::new(leptos_options.to_owned()))
                .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim));
            #[cfg(feature="accounts")]
            let app = {
                app.app_data(web::Data::new(db.clone()))
                    .wrap(actix_identity::IdentityMiddleware::default())
                    .wrap(
                        actix_session::SessionMiddleware::builder(redis.clone(),secret_key.clone())
                            .session_lifecycle(actix_session::config::PersistentSession::default().session_ttl(cookie::time::Duration::seconds(SECS_IN_WEEK).into()))
                            .cookie_secure(secure)
                            .build()
                    )
            };
            app
        })
            .bind(format!("{}:{}", ip, port))?
            .run()
            .await

         */
    }

    #[derive(Clone)]
    pub(crate) struct AppState {
        pub leptos_options:LeptosOptions,
        pub db: sea_orm::DatabaseConnection
    }
    impl axum::extract::FromRef<AppState> for LeptosOptions {
        fn from_ref(input: &AppState) -> Self {
            input.leptos_options.clone()
        }
    }
}
