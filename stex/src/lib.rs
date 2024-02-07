use immt_api::formats::{Format, FormatId};
use immt_system::controller::ControllerBuilder;

const FORMAT:Format = immt_api::formats::Format::new(
    FormatId::new_unchecked(*b"sTeX"),
    &["tex", "ltx"]
);

pub fn register(controller:&mut ControllerBuilder) {
    controller.register_format(FORMAT);
}