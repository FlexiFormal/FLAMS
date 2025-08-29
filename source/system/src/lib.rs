#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//#![feature(file_buffered)]
//#![feature(lazy_type_alias)]

pub mod backend;
//pub mod formats;
pub mod building;
#[cfg(feature = "tokio")]
pub mod logging;
pub mod settings;

#[cfg(feature = "zip")]
pub mod zip;

use building::queue_manager::QueueManager;
use flams_math_archives::{
    artifacts::Artifact, backend::AnyBackend, utils::AsyncEngine, LocalArchive,
};
use ftml_uris::{DocumentUri, UriPath};
use settings::SettingsSpec;

pub use inventory::submit as register_exension;

pub struct FlamsExtension {
    pub name: &'static str,
    pub on_start: fn(),
    pub on_build_result: fn(&AnyBackend, &DocumentUri, &UriPath, &dyn Artifact),
}

#[cfg(feature = "tokio")]
pub struct TokioEngine;
#[cfg(feature = "tokio")]
impl AsyncEngine for TokioEngine {
    fn background(f: impl FnOnce() + Send + 'static) {
        let span = tracing::Span::current();
        tokio::task::spawn_blocking(move || span.in_scope(f));
    }
    async fn block_on<R: Send + Sync + 'static>(
        f: impl FnOnce() -> R + Send + Sync + 'static,
    ) -> R {
        tokio::task::spawn_blocking(f).await.expect("this is a bug")
    }
}

inventory::collect!(FlamsExtension);

/// #### Panics
pub fn initialize<A: AsyncEngine>(settings: SettingsSpec) {
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
        backend::initialize::<A>();
        QueueManager::initialize(settings.num_threads);
        for e in inventory::iter::<FlamsExtension>() {
            A::background(|| {
                tracing::info_span!("Initializing", extension = e.name).in_scope(|| (e.on_start)());
            });
        }
    });
}

#[cfg(feature = "gitlab")]
pub trait LocalArchiveExt {
    fn is_managed(&self) -> Option<&flams_git::GitUrl>;
}

#[cfg(feature = "gitlab")]
impl LocalArchiveExt for LocalArchive {
    fn is_managed(&self) -> Option<&flams_git::GitUrl> {
        let gl = crate::settings::Settings::get().gitlab_url.as_ref()?;
        self.git_url(gl)
    }
}
