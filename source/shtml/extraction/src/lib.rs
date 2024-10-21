#![feature(box_patterns)]

mod extractor;
mod rules;
mod tags;
pub mod errors;
pub mod open;

pub mod prelude {
    pub use crate::rules::*;
    pub use super::extractor::*;
    pub use super::tags::SHTMLTag;
}