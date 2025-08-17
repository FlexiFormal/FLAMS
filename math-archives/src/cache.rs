use ftml_ontology::{
    domain::modules::Module,
    narrative::{DocumentRange, documents::Document},
    utils::Css,
};
use ftml_uris::{DocumentUri, ModuleUri};

use crate::utils::lazy_file::{BytesField, LazyField, LazyFile, StringField};

pub struct BackendCache {
    modules: ftml_ontology::utils::awaitable::AsyncCache<ModuleUri, Module, BackendError>,
    documents: ftml_ontology::utils::awaitable::AsyncCache<
        DocumentUri,
        triomphe::Arc<parking_lot::RwLock<DocumentFile>>,
        BackendError,
    >,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BackendError {
    #[error("{0}")]
    Channel(#[from] ftml_ontology::utils::awaitable::ChannelError),
}

struct DocumentFile {
    reader: LazyFile<5>,
    body: LazyField<DocumentRange, 0>,
    css: LazyField<Box<[Css]>, 1>,
    data: BytesField<2>,
    document: LazyField<Document, 3>,
    html: StringField<4>,
}
