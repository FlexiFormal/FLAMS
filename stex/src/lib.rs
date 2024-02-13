pub mod quickparse;

#[cfg(test)]
#[doc(hidden)]
mod test;

use immt_api::formats::{Format, FormatExtension, FormatId};
use immt_system::controller::ControllerBuilder;

pub const ID : FormatId = FormatId::new_unchecked(*b"sTeX");
pub const EXTENSIONS : &[&str] = &["tex", "ltx"];

pub fn register(controller:&mut ControllerBuilder) {
    immt_shtml::register(controller);
    let format = immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(STeXExtension));
    controller.register_format(format);
}

pub struct STeXExtension;
impl FormatExtension for STeXExtension {

}