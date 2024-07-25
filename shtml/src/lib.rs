mod docs;
mod parsing;

use immt_api::async_trait::async_trait;
use immt_api::backend::archives::{Archive, Storage};
use immt_api::building::targets::{BuildDataFormat, BuildFormatId, BuildTarget, SourceFormat};
use immt_api::building::tasks::BuildTask;
use immt_api::controller::Controller;
use immt_api::core::building::formats::{BuildTargetId, ShortId, SourceFormatId};
use immt_api::core::uris::documents::DocumentURI;
use immt_api::extensions::{ExtensionId, FormatExtension, MMTExtension};
use crate::parsing::parser::HTMLParser;

/*
immt_api::export_plugin!(register);
unsafe extern "C" fn register() -> Box<dyn MMTExtension> {
    Box::new(SHTMLExtension {})//engine:None})
}
 */
#[derive(Debug)]
pub struct SHTMLExtension {}

const SHTML_SHORT_ID: ShortId = ShortId::new_unchecked("sHTML");

pub const SHTML_FORMAT: BuildDataFormat = BuildDataFormat{id:BuildFormatId::new(SHTML_SHORT_ID),file_extensions:&["html","xhtml"],
    description:"Semantically annotated HTML"
};

pub const SHTML_OMDOC: BuildTarget = BuildTarget{id:BuildTargetId::new(SHTML_SHORT_ID), requires:&[SHTML_FORMAT],
    produces:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
    description:"Extract (content and narrative) OMDoc from sHTML",
    extension:Some(ExtensionId::new(SHTML_SHORT_ID))
};

pub const SHTML_IMPORT: SourceFormat = SourceFormat {
    id:SourceFormatId::new(ShortId::new_unchecked("sHTML")),
    file_extensions: & ["html", "xhtml"],
    targets:&[SHTML_OMDOC,BuildTarget::CHECK],
    description:"Import (s)HTML files",
    extension:Some(ExtensionId::new(SHTML_SHORT_ID))
};

impl MMTExtension for SHTMLExtension {
    fn name(&self) -> ExtensionId { ExtensionId::new(SHTML_SHORT_ID) }
    fn as_formats(&self) -> Option<&dyn FormatExtension> { Some(self) }
}
#[async_trait]
impl FormatExtension for SHTMLExtension {
    fn formats(&self) -> Vec<SourceFormat> { vec![SHTML_IMPORT] }
    fn sandbox(&self, _controller: &mut dyn Controller) -> Box<dyn MMTExtension> {
        todo!()
    }

    fn get_deps(&self, controller: &dyn Controller, task: &BuildTask) {
        todo!()
    }
    fn build(&self, ctrl:&dyn Controller,task: &BuildTask, target: BuildTargetId, index: u8) -> bool {
        match target {
            s if s == SHTML_OMDOC.id => {
                if let Some(path) = ctrl.archives().with_archives(|ars| {
                    ars.iter().find_map(|a| match a {
                        Archive::Physical(ma) if ma.id() == task.archive().id() => {
                            let p = task.rel_path().split('/').fold(ma.out_dir(), |p, c| p.join(c)).join("index.html");
                            if p.exists() { Some(p) } else { None }
                        },
                        _ => None
                    })
                }) {
                    let s = std::fs::read_to_string(path).unwrap();
                    let (path,name) = task.rel_path().rsplit_once('/').unwrap_or(("",task.rel_path()));
                    let (spec,mods) = HTMLParser::new(&s,task.path(),DocumentURI::new(task.archive().to_owned(),path,name),ctrl.archives(),true).run();
                    if let Some(step) = task.find_step(target) {
                        step.set_narrative(ctrl,task, spec);
                        step.set_content(ctrl,task,mods);
                    }
                } else { return false }
            },
            _ => unreachable!()
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        true
    }
}