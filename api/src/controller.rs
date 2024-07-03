use crate::backend::manager::{ArchiveManager, ArchiveManagerAsync};
use crate::utils::settings::Settings;

pub trait Controller:Send {
    fn archives(&self) -> &ArchiveManager;
    fn log_file(&self) -> &std::path::Path;
    fn build_queue(&self) -> &crate::building::buildqueue::BuildQueue;
    fn settings(&self) -> &Settings;
}
#[cfg(feature = "tokio")]
pub trait ControllerAsync:Send+Sync {
    fn archives(&self) -> &ArchiveManagerAsync;
    fn log_file(&self) -> &std::path::Path;
    fn build_queue(&self) -> &crate::building::buildqueue::BuildQueue;
    fn settings(&self) -> &Settings;
}