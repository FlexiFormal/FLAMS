use immt_api::formats::{Format, FormatId};
use immt_system::controller::ControllerBuilder;

const FORMAT:Format = immt_api::formats::Format::new(
    FormatId::new_unchecked(*b"SHTM"),
    &["html"]
);

pub fn register(controller:&mut ControllerBuilder) {
    controller.register_format(FORMAT);
}