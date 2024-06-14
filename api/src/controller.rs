use crate::backend::manager::ArchiveManager;

pub trait Controller {
    fn archives(&self) -> &ArchiveManager;
    fn log_file(&self) -> &std::path::Path;
}