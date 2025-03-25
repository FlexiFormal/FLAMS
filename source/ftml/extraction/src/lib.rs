//#![feature(box_patterns)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod errors;
mod extractor;
pub mod open;
mod rules;
mod tags;

pub mod prelude {
    pub use super::extractor::*;
    pub use crate::rules::{FTMLElements, FTMLExtractionRule, RuleSet};
    pub use crate::tags::{all_rules, rule};
    pub use flams_ontology::ftml::FTMLKey as FTMLTag;
}
