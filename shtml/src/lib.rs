use immt_api::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};
use immt_api::controller::Controller;
use immt_api::core::building::formats::ShortId;
use immt_api::extensions::{FormatExtension, MMTExtension};

/*
immt_api::export_plugin!(register);
unsafe extern "C" fn register() -> Box<dyn MMTExtension> {
    Box::new(SHTMLExtension {})//engine:None})
}
 */
#[derive(Debug)]
pub struct SHTMLExtension {}

pub const SHTML_FORMAT: BuildDataFormat = BuildDataFormat{id:ShortId::new("sHTML"),file_extensions:&["html","xhtml"],
    description:"Semantically annotated HTML"
};

pub const SHTML_OMDOC: BuildTarget = BuildTarget{id:SHTML_FORMAT.id, requires:&[SHTML_FORMAT],
    produces:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
    description:"Extract (content and narrative) OMDoc from sHTML",
    extension:Some(SHTML_FORMAT.id)
};

pub const SHTML_IMPORT: SourceFormat = SourceFormat {
    id:ShortId::new("sHTML"),
    file_extensions: & ["html", "xhtml"],
    targets:&[SHTML_OMDOC,BuildTarget::CHECK],
    description:"Import (s)HTML files",
    extension:Some(SHTML_FORMAT.id)
};

impl MMTExtension for SHTMLExtension {
    fn name(&self) -> ShortId {SHTML_FORMAT.id }
    fn test(&self, _controller: &mut dyn Controller) -> bool { true }
    fn test2(&self, _controller: &mut dyn Controller) -> bool { true }
    fn as_formats(&self) -> Option<&dyn FormatExtension> { Some(self) }
}
impl FormatExtension for SHTMLExtension {
    fn formats(&self) -> Vec<SourceFormat> { vec![SHTML_IMPORT] }
    fn sandbox(&self, _controller: &mut dyn Controller) -> Box<dyn MMTExtension> {
        todo!()
    }
}