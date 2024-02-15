pub mod quickparse;

#[cfg(test)]
#[doc(hidden)]
mod test;

use std::path::Path;
use async_trait::async_trait;
use immt_api::formats::{Format, FormatExtension, Id};
use immt_api::formats::building::{BuildResult, BuildTask, BuildTaskStep, BuildTaskStepKind, SourceTaskStep};
use immt_system::controller::ControllerBuilder;

pub const ID : Id = Id::new_unchecked(*b"sTeX");
pub const EXTENSIONS : &[&str] = &["tex", "ltx"];

pub fn register(controller:&mut ControllerBuilder) {
    immt_shtml::register(controller);
    let format = immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(STeXExtension));
    controller.register_format(format);
}

pub struct PdfLaTeX;
#[async_trait]
impl SourceTaskStep for PdfLaTeX {
    async fn run(&self, file: &Path) -> BuildResult {
        // Do Something
        BuildResult::None
    }
}

pub struct BibTeX;
#[async_trait]
impl SourceTaskStep for BibTeX {
    async fn run(&self, file: &Path) -> BuildResult {
        // Do Something
        BuildResult::None
    }
}

pub struct RusTeX;
#[async_trait]
impl SourceTaskStep for RusTeX {
    async fn run(&self, file: &Path) -> BuildResult {
        // Do Something
        BuildResult::None
    }
}

pub struct STeXExtension;
impl FormatExtension for STeXExtension {
    fn get_task(&self, source: &Path) -> Option<BuildTask> {
        Some(BuildTask {
            steps: vec![
                BuildTaskStep {
                    kind: BuildTaskStepKind::Source(Box::new(PdfLaTeX)),
                    id: Id::new_unchecked(*b"pLTX")
                },
                BuildTaskStep {
                    kind: BuildTaskStepKind::Source(Box::new(BibTeX)),
                    id: Id::new_unchecked(*b"bTeX")
                },
                BuildTaskStep {
                    kind: BuildTaskStepKind::Source(Box::new(RusTeX)),
                    id: Id::new_unchecked(*b"rTeX")
                },
                BuildTaskStep {
                    kind: BuildTaskStepKind::Complex(Box::new(immt_shtml::SHMLTaskStep)),
                    id: Id::new_unchecked(*b"sHTM")
                },
            ],
            state: None
        })
    }
}