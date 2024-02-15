use std::any::Any;
use std::future::Future;
use std::path::Path;
use async_trait::async_trait;
use immt_api::formats::{Format, FormatExtension, FormatId};
use immt_api::formats::building::{BuildResult, ComplexTaskStep, SourceTaskStep};
use immt_api::Str;
use immt_system::controller::ControllerBuilder;

const ID:FormatId = FormatId::new_unchecked(*b"SHTM");
const EXTENSIONS:&[&str] = &["html"];

pub struct SHMLTaskStep;
#[async_trait]
impl SourceTaskStep for SHMLTaskStep {
    async fn run(&self, file: &Path) -> BuildResult {
        // Do Something
        BuildResult::Final
    }
}
#[async_trait]
impl ComplexTaskStep for SHMLTaskStep {
    async fn run(&self, input: Box<dyn Any+Send>) -> BuildResult {
        match input.downcast_ref::<&Str>() {
            None => BuildResult::Err("Expected a string".into()),
            Some(_) => {
                // Do Something
                BuildResult::Final
            }
        }
    }
}


pub fn register(controller:&mut ControllerBuilder) {
    let format = immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(SHTMLExtension));
    controller.register_format(format);
}

pub struct SHTMLExtension;
impl FormatExtension for SHTMLExtension {
    fn get_task(&self, source: &Path) -> Option<immt_api::formats::building::BuildTask> {
        None // todo!()
    }
}