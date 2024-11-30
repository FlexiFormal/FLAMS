#![feature(box_patterns)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod extractor;
mod rules;
mod tags;
pub mod errors;
pub mod open;

pub mod prelude {
    pub use crate::rules::{SHTMLElements,SHTMLExtractionRule,RuleSet};
    pub use super::extractor::*;
    pub use immt_ontology::shtml::SHTMLKey as SHTMLTag;
    pub use crate::tags::{rule,all_rules};
}