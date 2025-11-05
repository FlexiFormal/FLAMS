#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod archive_json;
pub mod archives;
pub mod git;
pub mod search;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ManagerCacheSize {
    pub num_modules: usize,
    pub modules_bytes: usize,
    pub num_documents: usize,
    pub documents_bytes: usize,
    pub relations: usize,
}
impl ManagerCacheSize {
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.modules_bytes + self.documents_bytes
    }
}
