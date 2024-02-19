use async_trait::async_trait;
use immt_api::formats::building::{
    Backend, BuildData, BuildInfo, BuildResult, BuildStep, TaskStep,
};
use immt_api::formats::{Format, FormatExtension, Id};
use immt_api::FinalStr;
use immt_system::controller::ControllerBuilder;
use std::any::Any;
use std::future::Future;
use std::path::Path;

const ID: Id = Id::new_unchecked(*b"SHTM");
const EXTENSIONS: &[&str] = &["html"];

#[derive(Copy, Clone)]
pub struct SHMLTaskStep;

impl TaskStep for SHMLTaskStep {
    fn run(&self, state: &mut BuildData, backend: &Backend<'_>) -> BuildResult {
        // Do Something
        BuildResult::Success
    }
}

pub fn register(controller: &mut ControllerBuilder) {
    let format = immt_api::formats::Format::new(ID, EXTENSIONS, Box::new(SHTMLExtension));
    controller.register_format(format);
}

pub struct SHTMLExtension;

impl FormatExtension for SHTMLExtension {
    fn get_task(&self, info: &mut BuildInfo, backend: &Backend<'_>) -> Vec<BuildStep> {
        vec![]
    }
}
