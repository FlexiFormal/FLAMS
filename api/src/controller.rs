use crate::backend::manager::{ArchiveManager, ArchiveManagerAsync};

pub trait Controller {
    fn archives(&self) -> &ArchiveManager;
    fn log_file(&self) -> &std::path::Path;
    fn build_queue(&self) -> &crate::building::buildqueue::BuildQueue;
}
#[cfg(feature = "tokio")]
pub trait ControllerAsync {
    fn archives(&self) -> &ArchiveManagerAsync;
    fn log_file(&self) -> &std::path::Path;
    fn build_queue(&self) -> &crate::building::buildqueue::BuildQueue;
}