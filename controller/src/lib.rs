use std::fs::DirEntry;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use tracing::instrument;
use immt_api::backend::manager::{ArchiveManager};
use immt_api::backend::relational::RelationalManager;
use immt_api::building::buildqueue::BuildQueue;
use immt_api::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};
pub use immt_api::controller::{Controller};
use immt_api::extensions::{ExtensionId, FormatExtension, MMTExtension};
use immt_api::utils::asyncs::{background, ChangeListener};
use immt_core::ontology::rdf::terms::NamedNode;
use immt_core::utils::logs::LogFileLine;
use immt_core::utils::triomphe::Arc;
use immt_shtml::SHTMLExtension;
use crate::logging::LogStore;

pub mod logging;

#[derive(Debug)]
struct ControllerI {
    log: LogStore,
    relational: RelationalManager,
    archives: ArchiveManager,
    extensions:HMap<ExtensionId,Box<dyn MMTExtension>>,
    format_extensions:HMap<SourceFormatId,SourceFormat>,
    queue:BuildQueue,
    settings: Settings
}


#[derive(Clone,Debug)]
pub struct BaseController(Arc<ControllerI>);
impl BaseController {

    #[instrument(level = "info",
        target = "controller",
        name = "Booting",
        skip(settings)
    )]
    pub fn new(settings: SettingsSpec) -> Self {
        assert!(CONTROLLER.get().is_none());
        let settings: Settings = settings.into();

        let log = logging::tracing(&settings.log_dir,
            if settings.debug { tracing::Level::DEBUG } else {tracing::Level::INFO},
            tracing_appender::rolling::Rotation::NEVER
        );
        let mut extensions = HMap::default();
        let mut format_extensions = HMap::default();
        for e in BaseController::load_extensions() {
            if let Some(e) = e.as_formats() {
                for f in e.formats() {
                    format_extensions.insert(f.id, f);
                }
            };
            extensions.insert(e.name(),e);
        }
        let mut build_targets:Vec<_> = format_extensions.iter().flat_map(|(_,i)| i.targets).copied().collect();
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
        let queue = BuildQueue::new(&settings);

        let relational = RelationalManager::default();
        let archives = ArchiveManager::default();

        let ctrl = ControllerI { log, relational, extensions, archives,queue,
            format_extensions:format_extensions.into(), settings
        };
        let ctrl = Self(Arc::new(ctrl));

        tracing::info_span!(target:"controller","Loading extensions").in_scope(|| {
            for (_,ext) in ctrl.0.extensions.iter() {
                tracing::info!(target:"controller","Loading {}",ext.name());
                ext.on_plugin_load(&ctrl)
            }
        });
        let source_formats = ctrl.0.format_extensions.values().cloned().collect::<Vec<_>>();
        for p in &ctrl.0.settings.mathhubs {
            ctrl.0.relational.add_quads(ctrl.0.archives.load(p, source_formats.as_slice()).into_iter())
        }
        let ret = ctrl.clone();
        background(move || { ctrl.load_relational() });
        CONTROLLER.set(ret.clone()).expect("Controller already set");
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

    fn load_extensions() -> Box<[Box<dyn MMTExtension>]> {
        use immt_stex::STeXExtension;
        let mut target: Vec<Box<dyn MMTExtension>> = vec![
            Box::new(CheckExtension {}),
            Box::new(SHTMLExtension {}),
            Box::new(STeXExtension {}),
        ];
        if let Some(d) = immt_api::config_dir() {
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

impl Default for BaseController {
    fn default() -> Self {
        Self::new(SettingsSpec::default())
    }
}


impl Controller for BaseController {
    fn relations(&self) -> &RelationalManager { &self.0.relational }
    fn archives(&self) -> &ArchiveManager { &self.0.archives }
    fn log_file(&self) -> &Path { self.0.log.log_file() }
    fn build_queue(&self) -> &BuildQueue { &self.0.queue }
    fn settings(&self) -> &Settings { &self.0.settings }
    fn get_format(&self, id:SourceFormatId) -> Option<&SourceFormat> {
        self.0.format_extensions.get(&id)
    }
    fn get_target(&self, id: BuildTargetId) -> Option<&BuildTarget> {
        self.0.format_extensions.values().find_map(|fmt| fmt.targets.iter().find(|tg| tg.id == id))
    }
    fn get_extension(&self, id: ExtensionId) -> Option<&dyn MMTExtension> {
        self.0.extensions.get(&id).map(|b| &**b)
    }
}


pub use Controller as ControllerTrait;
use immt_api::checking::CheckExtension;
use immt_api::HMap;
use immt_api::utils::settings::{Settings};
use immt_core::building::formats::{BuildTargetId, SourceFormatId};
use immt_core::utils::settings::SettingsSpec;


static CONTROLLER: std::sync::OnceLock<BaseController> = std::sync::OnceLock::new();

pub fn controller() -> &'static BaseController {
    CONTROLLER.get().expect("Controller not set")
}

/*
#[test]
fn test() {
    let ctrl = BaseController::default();
}

 */