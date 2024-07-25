use std::fmt::{Debug, Formatter};
use immt_core::building::formats::{BuildTargetId, ShortId};
use crate::building::targets::{BuildTarget, SourceFormat};
use crate::building::tasks::BuildTask;
use crate::controller::Controller;
use crate::extensions::{ExtensionId, FormatExtension, MMTExtension};

pub const CHECK_EXTENSION:ExtensionId = ExtensionId::new(ShortId::CHECK);

#[derive(Debug)]
pub struct CheckExtension {}

impl MMTExtension for CheckExtension {
    fn name(&self) -> ExtensionId {
        CHECK_EXTENSION
    }
    fn as_formats(&self) -> Option<&dyn FormatExtension> {
        Some(self)
    }
}
impl FormatExtension for CheckExtension {
    fn formats(&self) -> Vec<SourceFormat> { Vec::new() }

    fn sandbox(&self, _controller: &mut dyn Controller) -> Box<dyn MMTExtension> {
        todo!()
    }

    fn get_deps(&self, controller: &dyn Controller, task: &BuildTask) {}

    fn build(&self, ctrl:&dyn Controller,task: &BuildTask, target: BuildTargetId, index: u8) -> bool {
        if target != BuildTarget::CHECK.id {
            unreachable!()
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        true
        //todo!()
    }
}