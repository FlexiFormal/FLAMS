use std::path::Path;
use crate::backend::archive_manager::ArchiveManager;
use std::path::PathBuf;
use immt_api::formats::{Format,FormatStore};
use crate::ontology::relational::RelationalManager;
use crate::utils::problems::ProblemHandler;


pub struct ControllerBuilder {
    main_mh:PathBuf,handler:Option<ProblemHandler>,
    formats:FormatStore
}

pub struct Controller {
    mgr:ArchiveManager,
    main_mh:PathBuf,
    handler:ProblemHandler,
    relman:RelationalManager,
    formats:FormatStore
}

impl Controller {
    pub fn builder<S:AsRef<Path>+Into<PathBuf>>(mh:S) -> ControllerBuilder {
        ControllerBuilder {
            main_mh:mh.into(),handler:None,formats:FormatStore::default()
        }
    }
    pub fn archives(&self) -> &ArchiveManager { &self.mgr }
    pub fn mathhub(&self) -> &Path { &self.main_mh }
}

impl ControllerBuilder {
    pub fn build(self) -> Controller {
        let handler = self.handler.unwrap_or_else(ProblemHandler::default);
        let mgr = ArchiveManager::new(&self.main_mh,&handler,&self.formats);
        let relman = RelationalManager::default();
        Controller {
            mgr,
            main_mh:self.main_mh,handler,relman,
            formats:self.formats
        }
    }
    pub fn with_handler(mut self,handler:ProblemHandler) -> Self {
        self.handler = Some(handler);
        self
    }
    pub fn register_format(&mut self,format:Format) {
        self.formats.register(format);
    }
}