use immt_api::formats::{Format, FormatExtension, FormatId};
use immt_system::controller::ControllerBuilder;

const ID:FormatId = FormatId::new_unchecked(*b"SHTM");
const EXTENSIONS:&[&str] = &["html"];


pub fn register(controller:&mut ControllerBuilder) {
    let format = immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(SHTMLExtension));
    controller.register_format(format);
}

pub struct SHTMLExtension;
impl FormatExtension for SHTMLExtension {

}