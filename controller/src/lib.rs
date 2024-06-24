use std::fs::DirEntry;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use tracing::instrument;
use immt_api::backend::manager::{ArchiveManager, ArchiveManagerAsync};
use immt_api::backend::relational::RelationalManager;
use immt_api::building::buildqueue::BuildQueue;
use immt_api::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};
pub use immt_api::controller::{Controller, ControllerAsync};
use immt_api::extensions::{FormatExtension, MMTExtension};
use immt_api::utils::asyncs::{background, ChangeListener};
use immt_core::ontology::rdf::terms::NamedNode;
use immt_core::utils::logs::LogFileLine;
use immt_core::utils::triomphe::Arc;
use immt_shtml::SHTMLExtension;
use crate::logging::LogStore;

pub mod logging;

struct ControllerI {
    log: LogStore,
    relational: RelationalManager,
    archives: ArchiveManager,
    extensions:Box<[Box<dyn MMTExtension>]>,
    format_extensions:Box<[u8]>,
    queue:BuildQueue
}

lazy_static::lazy_static! {
    static ref CONFIG_DIR: Option<PathBuf> = {
        simple_home_dir::home_dir().map(|d| d.join(".immt"))
    };
}

#[derive(Clone)]
pub struct BaseController(Arc<ControllerI>);
impl BaseController {

    #[instrument(level = "info",
        target = "controller",
        name = "Booting",
        skip(mathhubs,debug)
    )]
    #[cfg(not(feature="async"))]
    pub fn new<'a,Mhs:Iterator<Item=&'a Path>>(debug:bool,mathhubs:Mhs) -> Self {
        assert!(CONTROLLER.get().is_none());

        let log = logging::tracing(
            if debug { tracing::Level::DEBUG } else {tracing::Level::INFO},
            tracing_appender::rolling::Rotation::NEVER
        );
        let extensions = Self::load_extensions();
        let mut format_extensions = Vec::new();
        let source_formats = extensions.iter().enumerate().filter_map(|(i,e)| e.as_formats().map(|e| (i,e))).flat_map(|(i,e)| {
            format_extensions.push(i as u8);
            e.formats()
        }).collect::<Vec<_>>();
        let mut build_targets:Vec<_> = source_formats.iter().flat_map(|i| i.targets).copied().collect();
        build_targets.insert(0,BuildTarget::CHECK);
        let mut build_data_formats = vec![
            BuildDataFormat::PDF
        ];
        for t in &build_targets {
            for c in t.requires.iter().chain(t.produces.iter()) {
                if !build_data_formats.iter().any(|b| b.id == c.id) {
                    build_data_formats.push(c.clone())
                }
            }
        }
        let queue = BuildQueue::new(source_formats.into(),build_data_formats.into(),build_targets.into());

        let relational = RelationalManager::default();
        let archives = ArchiveManager::default();

        let ctrl = ControllerI { log, relational, extensions, archives,queue,
            format_extensions:format_extensions.into()
        };
        let ctrl = Self(Arc::new(ctrl));

        tracing::info_span!(target:"controller","Loading extensions").in_scope(|| {
            for ext in ctrl.0.extensions.iter() {
                tracing::info!(target:"controller","Loading {}",ext.name());
                ext.on_plugin_load(&ctrl)
            }
        });
        for p in mathhubs {
            ctrl.0.relational.add_quads(ctrl.0.archives.load_par(p, ctrl.0.queue.formats()).into_iter())
        }
        let ret = ctrl.clone();
        background(move || { ctrl.load_relational() });
        CONTROLLER.set(ret.clone()).unwrap();
        ret
    }

    fn is_dll_file(e:std::io::Result<DirEntry>) -> Option<DirEntry> {
        const DLL_EXT: &'static str = if cfg!(windows) { "dll" } else { "so" };

        let e = e.ok()?;
        let ft = e.file_type().ok()?;
        if ft.is_file() && e.path().extension()? == DLL_EXT {
            Some(e)
        } else { None }
    }

    fn do_dir(path:PathBuf,target:&mut Vec<Box<dyn MMTExtension>>) {
        /*
        use libloading::*;
        if path.is_dir() {
            for e in path.read_dir().unwrap().filter_map(Self::is_dll_file) {
                unsafe {
                    let lib = Library::new(e.path()).unwrap();
                    let registry : Symbol<*mut immt_api::extensions::ExtensionDeclaration> = lib.get(b"extension_declaration").unwrap();
                    let plugin = (registry.as_ref().unwrap().register)();
                    target.push(plugin)
                }
            }
        }
         */
    }

    pub fn config_dir() -> Option<&'static Path> { CONFIG_DIR.as_ref().map(|a| a.as_path())}

    fn load_extensions() -> Box<[Box<dyn MMTExtension>]> {
        use immt_stex::STeXExtension;
        let mut target: Vec<Box<dyn MMTExtension>> = vec![
            Box::new(SHTMLExtension {}),
            Box::new(STeXExtension {}),
        ];
        if let Some(d) = Self::config_dir() {
            let path = d.join("plugins");
            Self::do_dir(path,&mut target)
        }
        if let Ok(p) = std::env::current_exe() {
            if let Some(path) = p.parent().map(|p| p.join("plugins")) {
                Self::do_dir(path,&mut target)
            }
        }
        target.into()
    }

    fn load_relational(&self) {
        self.0.relational.load_archives(&self.0.archives)
    }
    pub fn log_listener(&self) -> ChangeListener<LogFileLine<String>> {
        self.0.log.listener()
    }
}

#[cfg(not(feature="async"))]
impl Default for BaseController {
    fn default() -> Self {
        Self::new(cfg!(debug_assertions),immt_api::MATHHUB_PATHS.iter().map(|p| p.deref()))
    }
}


impl Controller for BaseController {
    fn archives(&self) -> &ArchiveManager { &self.0.archives }
    fn log_file(&self) -> &Path { self.0.log.log_file() }
    fn build_queue(&self) -> &BuildQueue { &self.0.queue }
}

#[cfg(not(feature="async"))]
pub type MainController = BaseController;
#[cfg(feature="async")]
pub type MainController = BaseControllerAsync;

#[cfg(not(feature="async"))]
pub use Controller as ControllerTrait;
#[cfg(feature="async")]
pub use ControllerAsync as ControllerTrait;


static CONTROLLER: std::sync::OnceLock<MainController> = std::sync::OnceLock::new();

pub fn controller() -> &'static MainController {
    CONTROLLER.get().expect("Controller not set")
}

#[cfg(feature="async")]
#[derive(Debug)]
struct ControllerIAsync {
    log: LogStore,
    relational: RelationalManager,
    archives: ArchiveManagerAsync, //RwLock<ArchiveManager>,
    extensions:Box<[Box<dyn MMTExtension>]>,
    format_extensions:Box<[u8]>,
    queue:BuildQueue
}

#[cfg(feature="async")]
#[derive(Clone,Debug)]
pub struct BaseControllerAsync(Arc<ControllerIAsync>);

#[cfg(feature="async")]
impl BaseControllerAsync {
    pub fn config_dir() -> Option<&'static Path> { BaseController::config_dir() }

    #[instrument(level = "info",
        target = "controller",
        name = "Booting",
        skip(mathhubs,debug)
    )]
    pub async fn new<'a,Mhs:Iterator<Item=&'a Path>>(debug:bool,mathhubs:Mhs) -> Self {
        assert!(CONTROLLER.get().is_none());
        let log = logging::tracing(
            if debug { tracing::Level::DEBUG } else {tracing::Level::INFO},
            tracing_appender::rolling::Rotation::NEVER
        );
        let extensions = BaseController::load_extensions();
        let mut format_extensions = Vec::new();
        let source_formats = extensions.iter().enumerate().filter_map(|(i,e)| e.as_formats().map(|e| (i,e))).flat_map(|(i,e)| {
            format_extensions.push(i as u8);
            e.formats()
        }).collect::<Vec<_>>();
        let mut build_targets:Vec<_> = source_formats.iter().flat_map(|i| i.targets).copied().collect();
        build_targets.insert(0,BuildTarget::CHECK);
        let mut build_data_formats = vec![
            BuildDataFormat::PDF
        ];
        for t in &build_targets {
            for c in t.requires.iter().chain(t.produces.iter()) {
                if !build_data_formats.iter().any(|b| b.id == c.id) {
                    build_data_formats.push(c.clone())
                }
            }
        }
        let queue = BuildQueue::new(source_formats.into(),build_data_formats.into(),build_targets.into());

        let relational = RelationalManager::default();
        let archives = ArchiveManagerAsync::default();

        let ctrl = ControllerIAsync { log, relational, extensions, archives,queue,
            format_extensions:format_extensions.into()
        };
        let ctrl = Self(Arc::new(ctrl));

        tracing::info_span!(target:"controller","Loading extensions").in_scope(|| {
            for ext in ctrl.0.extensions.iter() {
                tracing::info!(target:"controller","Loading {}",ext.name());
                ext.on_plugin_load_async(&ctrl)
            }
        });
        for p in mathhubs {
            let formats = ctrl.0.queue.formats().into();
            ctrl.0.relational.add_quads(ctrl.0.archives.load(p.to_owned(), formats).await.into_iter())
        }
        let ret = ctrl.clone();
        tokio::spawn(async move {ctrl.load_relational().await});
        CONTROLLER.set(ret.clone()).expect("Controller already set");
        ret
    }

    pub async fn default() -> Self {
        Self::new(cfg!(debug_assertions),immt_api::MATHHUB_PATHS.iter().map(|p| p.deref())).await
    }

    async fn load_relational(&self) {
        self.0.relational.load_archives_async(&self.0.archives).await
    }
    pub fn log_listener(&self) -> ChangeListener<LogFileLine<String>> {
        self.0.log.listener()
    }
}

#[cfg(feature="async")]
impl ControllerAsync for BaseControllerAsync {
    fn archives(&self) -> &ArchiveManagerAsync { &self.0.archives }
    fn log_file(&self) -> &Path { self.0.log.log_file() }
    fn build_queue(&self) -> &BuildQueue { &self.0.queue }
}



#[test]
fn test() {
    let ctrl = BaseController::default();
}