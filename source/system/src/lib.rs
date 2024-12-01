#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![feature(lazy_type_alias)]

pub mod backend;
pub mod settings;
pub mod logging;
pub mod formats;
pub mod building;

use building::queue_manager::QueueManager;
use formats::IMMTExtension;
#[cfg(feature="tokio")]
use immt_utils::background;
use settings::SettingsSpec;
use backend::GlobalBackend;

static LOG : std::sync::OnceLock<logging::LogStore> = std::sync::OnceLock::new();

pub fn initialize(settings: SettingsSpec) {
    settings::Settings::initialize(settings);
    let settings = settings::Settings::get();
    if !settings.lsp {
        let _ = LOG.get_or_init(|| {
            logging::tracing(
                &settings.log_dir,
                if settings.debug { tracing::Level::DEBUG } else {tracing::Level::INFO},
                tracing_appender::rolling::Rotation::NEVER
            )
        });
    }
    let backend = GlobalBackend::get().manager();
    let mhs = &*settings.mathhubs;
    for p in mhs {
        backend.load(p);
    }
    let f = || {
        let backend = GlobalBackend::get();
        backend.triple_store().load_archives(&backend.all_archives());
    };
    #[cfg(feature="tokio")]
    background(f);
    #[cfg(not(feature="tokio"))]
    f();
    QueueManager::initialize(settings.num_threads);
    for e in IMMTExtension::all() {
        let span = tracing::info_span!("Initializing",extension=e.name());
        let f = move || {
            span.in_scope(||
                (e.on_start())()
            );
        };
        #[cfg(feature="tokio")]
        background(f);
        #[cfg(not(feature="tokio"))]
        f();
    }
}

/// ### Panics
pub fn logger() -> &'static logging::LogStore {
    LOG.get().expect("log should be initialized")
}
