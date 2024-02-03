use std::path::Path;
use crate::backend::archive_manager::ArchiveManager;
use std::path::PathBuf;
use crate::utils::problems::ProblemHandler;

pub mod archives;
pub mod archive_manager;

pub struct ControllerBuilder {
    main_mh:PathBuf,handler:Option<ProblemHandler>
}
impl ControllerBuilder {
    pub fn build(self) -> Controller {
        let handler = self.handler.unwrap_or_else(ProblemHandler::new);
        Controller {
            mgr:archive_manager::ArchiveManager::new(&self.main_mh,&handler),
            main_mh:self.main_mh,handler
        }
    }
    pub fn with_handler(mut self,handler:ProblemHandler) -> Self {
        self.handler = Some(handler);
        self
    }
}

pub struct Controller {
    mgr:archive_manager::ArchiveManager,
    main_mh:PathBuf,
    handler:ProblemHandler
}
impl Controller {
    pub fn new<S:AsRef<Path>+Into<PathBuf>>(mh:S) -> ControllerBuilder {
        ControllerBuilder {
            main_mh:mh.into(),handler:None
        }
    }
    pub fn archives(&self) -> &ArchiveManager { &self.mgr }
    pub fn mathhub(&self) -> &Path { &self.main_mh }
}