#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//#![feature(file_buffered)]
//#![feature(lazy_type_alias)]

pub mod backend;
pub mod building;
pub mod formats;
#[cfg(feature = "tokio")]
pub mod logging;
#[cfg(feature = "tantivy")]
pub mod search;
pub mod settings;

use std::future::Future;

use backend::GlobalBackend;
use building::queue_manager::QueueManager;
use flams_utils::unwrap;
use formats::FLAMSExtension;
use settings::SettingsSpec;

#[cfg(feature = "tokio")]
static RT: std::sync::OnceLock<tokio::runtime::Handle> = std::sync::OnceLock::new();
#[cfg(feature = "tokio")]
pub fn async_bg<F: Future<Output = ()> + Send + 'static>(f: F) {
    unwrap!(RT.get()).spawn(f);
}

/// #### Panics
pub fn initialize(settings: SettingsSpec) {
    #[cfg(feature = "tokio")]
    {
        let _ = RT.get_or_init(|| tokio::runtime::Handle::current());
    }

    settings::Settings::initialize(settings);
    let settings = settings::Settings::get();
    if settings.lsp {
        use tracing::Level;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::Layer;
        #[cfg(feature = "tokio")]
        let logger = logging::LogStore::new();
        let debug = settings.debug;
        let level = if debug { Level::TRACE } else { Level::INFO };

        let l = tracing_subscriber::fmt::layer()
            //.with_max_level(Level::INFO)//(if debug {Level::TRACE} else {Level::INFO})
            .with_ansi(false)
            .with_target(true)
            .with_writer(std::io::stderr)
            .with_filter(tracing::level_filters::LevelFilter::from(Level::INFO)); //.init();
        #[cfg(feature = "tokio")]
        let sub = tracing_subscriber::registry()
            .with(logger.with_filter(tracing::level_filters::LevelFilter::from(level)))
            .with(l);
        #[cfg(not(feature = "tokio"))]
        let sub = tracing_subscriber::registry().with(l);
        tracing::subscriber::set_global_default(sub)
            .expect("Failed to set global default logging subscriber");
    } else {
        #[cfg(feature = "tokio")]
        logging::LogStore::initialize();
    }
    tracing::info_span!(target:"initializing",parent:None,"initializing").in_scope(move || {
        #[cfg(feature = "gitlab")]
        {
            if let Some(url) = &settings.gitlab_url {
                let cfg = flams_git::gl::GitlabConfig::new(
                    url.clone(),
                    settings.gitlab_token.as_ref().map(ToString::to_string),
                    settings.gitlab_app_id.as_ref().map(ToString::to_string),
                    settings.gitlab_app_secret.as_ref().map(ToString::to_string),
                );
                flams_git::gl::GLInstance::global().clone().load(cfg);
            }
        }
        GlobalBackend::initialize();
        QueueManager::initialize(settings.num_threads);
        FLAMSExtension::initialize();
    });
}
