//#![feature(box_patterns)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod extractor;
mod rules;
mod tags;
pub mod errors;
pub mod open;

pub mod prelude {
    pub use crate::rules::{FTMLElements,FTMLExtractionRule,RuleSet};
    pub use super::extractor::*;
    pub use flams_ontology::ftml::FTMLKey as FTMLTag;
    pub use crate::tags::{rule,all_rules};
}