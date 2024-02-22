pub mod parse;

#[cfg(test)]
#[doc(hidden)]
mod test;

use async_trait::async_trait;
use immt_api::formats::building::{
    Backend, BuildData, BuildInfo, BuildResult, BuildStep, TaskStep,
};
use immt_api::formats::{Format, FormatExtension, Id};
use immt_api::uris::DocumentURI;
use immt_api::{CloneStr, FinalStr};
use immt_system::controller::ControllerBuilder;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;

const ID: Id = Id::new_unchecked(*b"SHTM");
const EXTENSIONS: &[&str] = &["html"];

#[derive(Copy, Clone)]
pub struct SHMLTaskStep;

impl TaskStep for SHMLTaskStep {
    fn run(&self, state: &mut BuildData, backend: &Backend<'_>) -> BuildResult {
        // Do Something
        let html = state
            .state
            .get("shtml")
            .and_then(|s| s.downcast_ref::<CloneStr>());
        if let Some(html) = html {
            let html = html.as_ref();
            let doc_uri =
                DocumentURI::from_relpath(state.archive_uri.as_ref(), state.rel_path.as_ref());
            let (s, d) =
                parse::HTMLParser::new(html, state.build_path.as_ref().unwrap(), doc_uri).run(); // TODO no unwrap
            state.document = Some((d, s.into()));
            BuildResult::Success
        } else {
            BuildResult::Err("No shtml".into())
        }
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
