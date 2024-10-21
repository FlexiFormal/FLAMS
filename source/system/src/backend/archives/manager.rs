use std::{ops::Deref, path::Path};

use immt_ontology::{
    content::modules::UncheckedModule, languages::Language, narration::documents::UncheckedDocument, uris::{ArchiveId, ArchiveURITrait, NameStep, PathURIRef, PathURITrait}
};
use immt_utils::change_listener::ChangeSender;
use oxigraph::model::Quad;

use crate::backend::{docfile::PreDocFile, BackendChange};

use super::{Archive, ArchiveTree};

#[derive(Debug)]
pub struct ArchiveManager {
    tree: parking_lot::RwLock<ArchiveTree>,
    change_sender: ChangeSender<BackendChange>,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self {
            tree: parking_lot::RwLock::new(ArchiveTree::default()),
            change_sender: ChangeSender::new(256),
        }
    }
}
impl ArchiveManager {
    #[inline]
    #[must_use]
    pub fn all_archives(&self) -> impl Deref<Target = [Archive]> + '_ {
        parking_lot::RwLockReadGuard::map(self.tree.read(), |s| s.archives.as_slice())
    }

    #[inline]
    pub fn with_tree<R>(&self,f:impl FnOnce(&ArchiveTree) -> R) -> R {
        f(&self.tree.read())
    }

    #[inline]
    pub fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R {
        let tree = self.tree.read();
        f(tree.get(id))
    }

    #[inline]
    pub fn load(&self, path: &Path) {
        self.do_load(path, ());
    }

    #[inline]
    pub fn load_with_quads(&self, path: &Path, add_quad: impl FnMut(Quad) + Send) {
        self.do_load(path, add_quad);
    }
    fn do_load<F: MaybeQuads>(&self, path: &Path, add_quad: F) {
        let mut tree = self.tree.write();
        tree.load(path, &self.change_sender, add_quad);
    }

    pub(crate) fn load_document(
        &self,
        path_uri: PathURIRef,
        language: Language,
        name: &NameStep,
    ) -> Option<UncheckedDocument> {
        let archive = path_uri.archive_id();
        let path = path_uri.path();
        self.with_archive(archive, |a| {
            a.and_then(|a| a.load_document(path, name, language))
        })
    }
    pub(crate) fn load_module(
        &self,
        path_uri: PathURIRef,
        language: Language,
        name: &NameStep,
    ) -> Option<UncheckedModule> {
        let archive = path_uri.archive_id();
        let path = path_uri.path();
        self.with_archive(archive, |a| {
            a.and_then(|a| a.load_module(path, name, language))
        })
    }
}

pub(super) trait MaybeQuads: Send {}
impl MaybeQuads for () {}
impl<F> MaybeQuads for F where F: FnMut(Quad) + Send {}

/*
#[cfg(test)]
mod tests {

    use immt_ontology::source_format;
    use immt_utils::time::measure;

    use super::*;

    source_format!(stex ["tex","ltx"] [] @
        "Semantically annotated LaTeX"
    );

    #[test]
    fn mathhub() {
        use std::fmt::Write;
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
        //.with_max_level(tracing::Level::DEBUG)
        .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        let manager = ArchiveManager::default();
        let (_,t) = measure(|| manager.load(Path::new("/home/jazzpirate/work/MathHub")));

        tracing::info!("Loaded archives in {t}");

        let mut all = String::new();
        for a in &*manager.all_archives() {
        write!(all,"{}, ",a.id()).unwrap();
        }
        tracing::info!("{all}");

        assert_eq!(165,manager.all_archives().len());
    }
}
*/
