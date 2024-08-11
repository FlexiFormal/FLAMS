use immt_core::building::formats::{BuildTargetId, SourceFormatId};
use crate::backend::Backend;
use crate::building::targets::{BuildTarget, SourceFormat};
use crate::extensions::{ExtensionId, MMTExtension};
use crate::utils::settings::Settings;

pub trait Controller:Send+Sync {
    fn backend(&self) -> &Backend;
    fn log_file(&self) -> &std::path::Path;
    fn build_queue(&self) -> &crate::building::buildqueue::BuildQueue;
    fn settings(&self) -> &Settings;
    fn get_format(&self, id:SourceFormatId) -> Option<&SourceFormat>;
    fn get_target(&self, id:BuildTargetId) -> Option<&BuildTarget>;
    fn get_extension(&self,id:ExtensionId) -> Option<&dyn MMTExtension>;
}