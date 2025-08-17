#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod archive_iter;
pub mod archive_json;
pub mod cache;
pub mod formats;
pub mod manifest;
pub mod mathhub;
pub mod source_files;
pub mod utils;

use crate::{
    archive_json::{ArchiveIndex, Institution},
    formats::{BuildTargetId, SourceFormatId},
    manifest::RepositoryData,
    source_files::{FileStates, SourceDir},
    utils::ignore_source::IgnoreSource,
};
use ftml_ontology::{
    domain::modules::Module,
    narrative::{DocumentRange, documents::Document},
    utils::Css,
};
use ftml_uris::{ArchiveId, ArchiveUri, Language, UriPath, UriWithArchive};
use std::path::{Path, PathBuf};

pub trait MathArchive {
    fn uri(&self) -> &ArchiveUri;
    fn path(&self) -> &Path;
    fn is_meta(&self) -> bool;

    #[inline]
    fn id(&self) -> &ArchiveId {
        self.uri().archive_id()
    }

    #[cfg(feature = "gitlab")]
    pub fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
        let gl = crate::settings::Settings::get().gitlab_url.as_ref()?;
        self.is_managed
            .get_or_init(|| {
                let Ok(repo) = flams_git::repos::GitRepo::open(self.path()) else {
                    return None;
                };
                gl.host_str().and_then(|s| repo.is_managed(s))
            })
            .as_ref()
    }

    fn load_module(&self, path: Option<&UriPath>, name: &str) -> Option<Module>;
    fn load_document(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
    ) -> Option<Document>;
    fn load_html(&self, path: Option<&UriPath>, name: &str, language: Language) -> Option<String>;
    fn load_html_body(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        full: bool,
    ) -> Option<(Vec<Css>, String)>;
    fn load_html_fragment(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<Css>, String)>;
    /*fn load_reference<T: flams_ontology::Resourcable>(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<T>;
    fn load<D: BuildArtifact>(&self, relative_path: &str) -> Result<D, std::io::Error>;
    */
}

pub trait ExternalArchive: Send + Sync + MathArchive + std::any::Any {
    #[inline]
    fn local_out(&self) -> Option<&dyn LocallyBuilt> {
        None
    }
    #[inline]
    fn buildable(&self) -> Option<&dyn BuildableArchive> {
        None
    }
}

pub enum Archive {
    Local(Box<LocalArchive>),
    Ext(&'static ArchiveKind, Box<dyn ExternalArchive>),
}

#[derive(Copy, Clone, Debug)]
pub struct ArchiveKind {
    pub name: &'static str,
    make_new: fn(RepositoryData, &Path) -> Option<Box<dyn ExternalArchive>>,
}

impl ArchiveKind {
    #[inline]
    pub fn all() -> impl Iterator<Item = &'static Self> {
        inventory::iter.into_iter()
    }
    #[must_use]
    pub fn get(name: &str) -> Option<&'static Self> {
        Self::all().find(|e| e.name == name)
    }
}
inventory::collect!(ArchiveKind);
#[macro_export]
macro_rules! archive_kind {
    ($i:ident { $($t:tt)* }) => {
        pub static $i : $crate::ArchiveKind = $crate::ArchiveKind { $($t)* };
        $crate::formats::__reexport::submit!{ $i }
    };
}

pub trait BuildableArchive: MathArchive {
    fn file_state(&self) -> FileStates;
    fn formats(&self) -> &[SourceFormatId];
    fn get_log(&self, relative_path: &str, target: BuildTargetId) -> PathBuf;
    //fn save_omdoc_result(&self, top: &Path, result: &OMDocResult)
    /*
    * pub fn save(
        &self,
        relative_path: &str,
        log: Either<String, PathBuf>,
        from: BuildTargetId,
        result: Option<BuildResultArtifact>,
    )
    */

    #[cfg(feature = "rdf")]
    fn submit_triples(
        &self,
        in_doc: &ftml_uris::DocumentUri,
        rel_path: &str,
        //relational: &RDFStore,
        load: bool,
        iter: &mut dyn Iterator<Item = ulo::rdf_types::Triple>,
    );
}

pub trait LocallyBuilt: BuildableArchive {
    fn out_dir(&self) -> &Path;
}

pub struct LocalArchive {
    pub(crate) uri: ArchiveUri,
    pub(crate) out_path: PathBuf,
    pub(crate) attributes: Vec<(Box<str>, Box<str>)>,
    pub(crate) formats: Vec<SourceFormatId>,
    //pub dependencies: Box<[ArchiveId]>,
    pub(crate) file_state: parking_lot::RwLock<SourceDir>,
    pub(crate) institutions: Box<[Institution]>,
    pub(crate) index: Box<[ArchiveIndex]>,
    pub ignore: IgnoreSource,
    #[cfg(feature = "gitlab")]
    pub(super) is_managed: std::sync::OnceLock<Option<git_url_parse::GitUrl>>,
}
impl MathArchive for LocalArchive {
    #[inline]
    fn uri(&self) -> &ArchiveUri {
        &self.uri
    }
    fn path(&self) -> &Path {
        self.out_path
            .parent()
            .expect("out path of an archive *must* have a parent")
    }

    fn is_meta(&self) -> bool {
        self.uri.archive_id().is_meta()
    }
    fn load_document(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
    ) -> Option<Document> {
        todo!()
    }
    fn load_html(&self, path: Option<&UriPath>, name: &str, language: Language) -> Option<String> {
        todo!()
    }
    fn load_html_body(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        full: bool,
    ) -> Option<(Vec<Css>, String)> {
        todo!()
    }
    fn load_html_fragment(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<Css>, String)> {
        todo!()
    }
    fn load_module(&self, path: Option<&UriPath>, name: &str) -> Option<Module> {
        todo!()
    }
}
impl BuildableArchive for LocalArchive {
    #[inline]
    fn file_state(&self) -> FileStates {
        self.file_state.read().state().clone()
    }

    #[inline]
    fn formats(&self) -> &[SourceFormatId] {
        &self.formats
    }
    fn get_log(&self, relative_path: &str, target: BuildTargetId) -> PathBuf {
        todo!()
    }
    #[cfg(feature = "rdf")]
    fn submit_triples(
        &self,
        in_doc: &ftml_uris::DocumentUri,
        rel_path: &str,
        //relational: &RDFStore,
        load: bool,
        iter: &mut dyn Iterator<Item = ulo::rdf_types::Triple>,
    ) {
        todo!()
    }
}

impl LocallyBuilt for LocalArchive {
    #[inline]
    fn out_dir(&self) -> &Path {
        &self.out_path
    }
}
impl LocalArchive {
    fn escape_module_name(in_path: &Path, name: &str) -> PathBuf {
        in_path.join(name.replace('*', "__AST__"))
    }

    #[inline]
    #[must_use]
    pub fn source_dir_of(p: &Path) -> PathBuf {
        p.join("source")
    }

    #[inline]
    #[must_use]
    pub fn source_dir(&self) -> PathBuf {
        Self::source_dir_of(self.path())
    }

    #[inline]
    #[must_use]
    fn out_dir_of(p: &Path) -> PathBuf
    where
        Self: Sized,
    {
        p.join(".flams")
    }

    #[inline]
    pub fn with_sources<R>(&self, f: impl FnOnce(&SourceDir) -> R) -> R {
        f(&self.file_state.read())
    }

    pub(crate) fn update_sources(&self) {
        let mut state = self.file_state.write();
        state.update(self.uri(), self.path(), &self.ignore, self.formats());
    }

    pub(crate) fn get_filepath(
        &self,
        path: Option<&UriPath>,
        name: &str,
        language: Language,
        filename: &str,
    ) -> Option<PathBuf> {
        let out = path.map_or_else(
            || self.out_dir().to_path_buf(),
            |n| {
                n.steps()
                    .fold(self.out_dir().to_path_buf(), |p, n| p.join(n))
            },
        );

        for d in std::fs::read_dir(&out).ok()? {
            let Ok(dir) = d else { continue };
            let Ok(m) = dir.metadata() else { continue };
            if !m.is_dir() {
                continue;
            }
            let dname = dir.file_name();
            let Some(d) = dname.to_str() else { continue };
            if !d.starts_with(name) {
                continue;
            }
            let rest = &d[name.len()..];
            if !rest.is_empty() && !rest.starts_with('.') {
                continue;
            }
            let rest = rest.strip_prefix('.').unwrap_or(rest);
            if rest.contains('.') {
                let lang: &'static str = language.into();
                if !rest.starts_with(lang) {
                    continue;
                }
            }
            let p = dir.path().join(filename);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
}
