use immt_api::async_trait::async_trait;
use immt_api::building::queue::BuildTask;
use immt_api::building::targets::{BuildDataFormat, BuildFormatId, BuildTarget, SourceFormat};
use immt_api::controller::Controller;
use immt_api::core::building::formats::{BuildTargetId, ShortId, SourceFormatId};
use immt_api::extensions::{ExtensionId, FormatExtension, MMTExtension};

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
    fn test(&self, _controller: &mut dyn Controller) -> bool { true }
    fn test2(&self, _controller: &mut dyn Controller) -> bool { true }
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
}