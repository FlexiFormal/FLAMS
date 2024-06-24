use leptos::*;
use leptos_router::*;

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
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use home::*;
    console_error_panic_hook::set_once();
    leptos_dom::HydrationCtx::stop_hydrating();
}

#[cfg(feature = "server")]
pub mod server {
    use std::time::Duration;
    use argon2::PasswordHasher;
    use immt_controller::MainController;
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
            let mut pwd = std::env::var("IMMT_ADMIN_PWD").ok()?;
            #[cfg(feature="accounts")]
            {
                use argon2::{Argon2,password_hash::{SaltString,rand_core::OsRng}};
                pwd = Argon2::default().hash_password(pwd.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string();
            }
            Some(pwd)
        };
    }

    pub async fn run_server<S: AsRef<str> + std::fmt::Display>(ip: S, port: u16, site_root: &str) -> std::io::Result<()> {
        use actix_files::Files;
        use actix_web::*;
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use crate::utils::ws::WS;

        let Main = crate::home::MainNew;

        std::env::set_var("LEPTOS_OUTPUT_NAME", "immt");
        let secure = std::env::var("IMMT_HTTPS_ONLY").is_ok_and(|s| s.eq_ignore_ascii_case("true"));

        let mut leptos_options = get_configuration(None).await.unwrap().leptos_options;
        leptos_options.site_addr = std::net::SocketAddr::new(ip.as_ref().parse().unwrap(), port);
        leptos_options.site_root = site_root.to_string();
        leptos_options.output_name = "immt".to_string();

        #[cfg(feature="accounts")]
        let (db,redis,secret_key) = {
            use immt_web_orm::MigratorTrait;
            let db = sea_orm::Database::connect(format!("sqlite:{}/users.sqlite?mode=rwc", MainController::config_dir().expect("Config directory not found").display()))
                .await
                .expect("Failed to connect to user database");
            immt_web_orm::Migrator::up(&db, None).await.expect("Failed to migrate database");

/*
            match tokio::process::Command::new("redis-server").args(["--port","6380"]).spawn().is_ok() {
                true => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    println!("Started Redis.")
                }
                false => panic!("Failed to start Redist Server"),
            };

 */

            (db,
             crate::utils::PseudoRedis::default(),//actix_session::storage::RedisSessionStore::new("redis://127.0.0.1:6380").await.unwrap(),
             cookie::Key::from(b"w<aiwhi3i<wu h<avwiailuwr<laasfknQ$))$(/Z$kljsdfkjbdgkjysd r<ar wrvvwi<qwu3")//cookie::Key::generate()
            )
        };

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
    }
}
