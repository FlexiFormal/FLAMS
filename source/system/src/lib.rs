//! This Crate is the back bone of IMMT System, This consists of a Work in Progress Build Tool to build the archives
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//#![feature(file_buffered)]
//#![feature(lazy_type_alias)]

pub mod backend;
pub mod building;
pub mod formats;
pub mod logging;
pub mod settings;

use backend::GlobalBackend;
use building::queue_manager::QueueManager;
#[cfg(feature = "tokio")]
use flams_utils::background;
use formats::FLAMSExtension;
use settings::SettingsSpec;

<<<<<<< HEAD
static LOG: std::sync::OnceLock<logging::LogStore> = std::sync::OnceLock::new();

pub fn initialize(settings: SettingsSpec) {
    settings::Settings::initialize(settings);
    let settings = settings::Settings::get();
    if !settings.lsp {
        let _ = LOG.get_or_init(|| {
            logging::tracing(
                &settings.log_dir,
                if settings.debug {
                    tracing::Level::DEBUG
                } else {
                    tracing::Level::INFO
                },
                //tracing_appender::rolling::Rotation::NEVER
            )
        });
=======
pub fn initialize(settings: SettingsSpec) {
    settings::Settings::initialize(settings);
    let settings = settings::Settings::get();
    if settings.lsp {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::Layer;
        use tracing::Level;
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
        let sub = tracing_subscriber::registry()
          .with(logger.with_filter(tracing::level_filters::LevelFilter::from(level)))
          .with(l);
        tracing::subscriber::set_global_default(sub).expect(
          "Failed to set global default logging subscriber"
        );
    } else {
        logging::LogStore::initialize();
>>>>>>> origin/devel
    }
    tracing::info_span!(target:"initializing",parent:None,"initializing").in_scope(move || {
        #[cfg(feature = "gitlab")]
        {
            if let Some(url) = &settings.gitlab_url {
                let cfg = flams_git::gl::GitlabConfig::new(
                    url.to_string(),
                    settings.gitlab_token.as_ref().map(ToString::to_string),
                    settings.gitlab_app_id.as_ref().map(ToString::to_string),
                    settings.gitlab_app_secret.as_ref().map(ToString::to_string),
                );
                flams_git::gl::GLInstance::global().clone().load(cfg);
            }
        }
<<<<<<< HEAD
        let backend = GlobalBackend::get().manager();
        let mhs = &*settings.mathhubs;
        for p in mhs.iter().rev() {
            backend.load(p);
        }
        let f = || {
            let backend = GlobalBackend::get();
            backend
                .triple_store()
                .load_archives(&backend.all_archives());
        };
        #[cfg(feature = "tokio")]
        background(f);
        #[cfg(not(feature = "tokio"))]
        f();
        QueueManager::initialize(settings.num_threads);
        for e in FLAMSExtension::all() {
            //let span = tracing::info_span!("Initializing",extension=e.name());
            let f = move || {
                tracing::info_span!("Initializing", extension = e.name())
                    .in_scope(|| (e.on_start())());
            };
            #[cfg(feature = "tokio")]
            background(f);
            #[cfg(not(feature = "tokio"))]
            f();
        }
=======
        GlobalBackend::initialize();
        QueueManager::initialize(settings.num_threads);
        FLAMSExtension::initialize();
>>>>>>> origin/devel
    })
}