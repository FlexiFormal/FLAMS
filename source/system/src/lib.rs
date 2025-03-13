#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//#![feature(file_buffered)]
//#![feature(lazy_type_alias)]

pub mod backend;
pub mod settings;
#[cfg(feature="tokio")]
pub mod logging;
pub mod formats;
pub mod building;
#[cfg(feature="tantivy")]
pub mod search;

use building::queue_manager::QueueManager;
use formats::FLAMSExtension;
use settings::SettingsSpec;
use backend::GlobalBackend;

/// #### Panics
pub fn initialize(settings: SettingsSpec) {
    settings::Settings::initialize(settings);
    let settings = settings::Settings::get();
    if settings.lsp {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::Layer;
        use tracing::Level;
        #[cfg(feature="tokio")]
        let logger = logging::LogStore::new();
        let debug = settings.debug;
        let level = if debug {Level::TRACE} else {Level::INFO};

        let l = tracing_subscriber::fmt::layer()
          //.with_max_level(Level::INFO)//(if debug {Level::TRACE} else {Level::INFO})
          .with_ansi(false)
          .with_target(true)
          .with_writer(std::io::stderr)
          .with_filter(tracing::level_filters::LevelFilter::from(Level::INFO))
          ;//.init();
        #[cfg(feature="tokio")]
        let sub = tracing_subscriber::registry()
          .with(logger.with_filter(tracing::level_filters::LevelFilter::from(level)))
          .with(l);
        #[cfg(not(feature="tokio"))]
        let sub = tracing_subscriber::registry().with(l);
        tracing::subscriber::set_global_default(sub).expect(
          "Failed to set global default logging subscriber"
        );
    } else {
        #[cfg(feature="tokio")]
        logging::LogStore::initialize();
    }
    tracing::info_span!(target:"initializing",parent:None,"initializing").in_scope(move || {
        #[cfg(feature="gitlab")]
        {
            if let Some(url) = &settings.gitlab_url {
                let cfg = flams_git::gl::GitlabConfig::new(
                    url.to_string(),
                    settings.gitlab_token.as_ref().map(ToString::to_string),
                    settings.gitlab_app_id.as_ref().map(ToString::to_string),
                    settings.gitlab_app_secret.as_ref().map(ToString::to_string)
                );
                flams_git::gl::GLInstance::global().clone().load(cfg);
            }
        }
        GlobalBackend::initialize();
        QueueManager::initialize(settings.num_threads);
        FLAMSExtension::initialize();
    });
}