mod ignore_regex;
//mod inventory;
//pub use inventory::InventoriedArchive;
mod iter;
pub mod manager;
pub mod source_files;
#[cfg(feature = "zip")]
mod zip;

#[cfg(feature = "tokio")]
use std::future::Future;
use std::path::{Path, PathBuf};

use either::Either;
use flams_ontology::{
    archive_json::{ArchiveIndex, Institution},
    content::modules::OpenModule,
    file_states::FileStateSummary,
    languages::Language,
    narration::documents::UncheckedDocument,
    uris::{
        ArchiveId, ArchiveURI, ArchiveURIRef, ArchiveURITrait, DocumentURI, Name, NameStep,
        PathURITrait, URIOrRefTrait, URIRefTrait,
    },
    DocumentRange, Unchecked,
};
use flams_utils::{
    change_listener::ChangeSender,
    prelude::{TreeChild, TreeLike},
    vecmap::{VecMap, VecSet},
    CSS,
};
use ignore_regex::IgnoreSource;
use iter::ArchiveIterator;
use manager::MaybeQuads;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use source_files::{FileStates, SourceDir};
use spliter::ParallelSpliterator;
use tracing::instrument;

use crate::{
    building::{BuildArtifact, BuildResultArtifact},
    formats::{BuildArtifactTypeId, BuildTargetId, OMDocResult, SourceFormatId},
};

use super::{docfile::PreDocFile, rdf::RDFStore, BackendChange};

#[derive(Debug)]
pub struct RepositoryData {
    pub(super) uri: ArchiveURI,
    pub(super) attributes: VecMap<Box<str>, Box<str>>,
    pub(super) formats: VecSet<SourceFormatId>,
    pub(super) dependencies: Box<[ArchiveId]>,
    pub(super) institutions: Box<[Institution]>,
    pub(super) index: Box<[ArchiveIndex]>,
}

pub trait ArchiveBase: std::fmt::Debug {
    fn data(&self) -> &RepositoryData;
    fn files(&self) -> &parking_lot::RwLock<SourceDir>;
    fn update_sources(&self, sender: &ChangeSender<BackendChange>);

    #[cfg(feature = "gitlab")]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl>;
    #[cfg(not(feature = "gitlab"))]
    #[inline]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
        None
    }
}

type Fut<T> = Option<std::pin::Pin<Box<dyn Future<Output = T> + Send + 'static>>>;

pub trait ArchiveTrait: ArchiveBase + Send + Sync {
    //fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>>;
    fn load_html_full(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<String>;

    #[cfg(feature = "tokio")]
    fn load_html_full_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Fut<Option<String>>;

    fn load_html_body(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<(Vec<CSS>, String)>;

    #[cfg(feature = "tokio")]
    fn load_html_body_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Fut<Option<(Vec<CSS>, String)>>;

    fn load_html_fragment(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<CSS>, String)>;

    #[cfg(feature = "tokio")]
    fn load_html_fragment_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Fut<Option<(Vec<CSS>, String)>>;

    /// ### Errors
    fn load_reference_blob(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<Vec<u8>>;

    fn load_document(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument>;
    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>>;

    #[inline]
    #[must_use]
    fn is_meta(&self) -> bool {
        self.data().uri.archive_id().is_meta()
    }

    #[inline]
    #[must_use]
    fn uri(&self) -> ArchiveURIRef {
        self.data().uri.archive_uri()
    }

    #[inline]
    #[must_use]
    fn id(&self) -> &ArchiveId {
        self.data().uri.archive_id()
    }

    #[inline]
    #[must_use]
    fn formats(&self) -> &[SourceFormatId] {
        self.data().formats.0.as_slice()
    }

    #[inline]
    #[must_use]
    fn attributes(&self) -> &VecMap<Box<str>, Box<str>> {
        &self.data().attributes
    }
}

pub trait HasLocalOut: ArchiveBase {
    #[must_use]
    fn out_dir(&self) -> &Path;

    fn artifact_path(&self, relative_path: &str, id: BuildArtifactTypeId) -> PathBuf {
        self.out_dir().join(relative_path).join(id.name())
    }

    fn get_log(&self, relative_path: &str, target: BuildTargetId) -> PathBuf {
        self.out_dir()
            .join(relative_path)
            .join(target.name())
            .with_extension("log")
    }

    #[inline]
    #[must_use]
    fn out_dir_of(p: &Path) -> PathBuf
    where
        Self: Sized,
    {
        p.join(".flams")
    }
}

pub trait Buildable: ArchiveTrait {
    fn save(
        &self,
        relative_path: &str,
        log: Either<String, PathBuf>,
        from: BuildTargetId,
        result: Option<BuildResultArtifact>,
    );
    fn submit_triples(
        &self,
        in_doc: &DocumentURI,
        rel_path: &str,
        relational: &RDFStore,
        load: bool,
        iter: std::collections::hash_set::IntoIter<flams_ontology::rdf::Triple>,
    );
}

impl<T: ArchiveBase + Send + Sync> ArchiveTrait for T
where
    T: HasLocalOut,
{
    fn load_html_full(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<String> {
        let p = get_filepath(self.out_dir(), path, name, language, "ftml")?;
        OMDocResult::load_html_full(p)
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_full_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Fut<Option<String>> {
        load_html_full_async(self.out_dir(), path, name, language).map(|f| Box::pin(f) as _)
    }

    fn load_html_body(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<(Vec<CSS>, String)> {
        get_filepath(self.out_dir(), path, name, language, "ftml")
            .and_then(|p| OMDocResult::load_html_body(&p, full))
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_body_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        load_html_body_async(self.out_dir(), path, name, language, full).map(|f| Box::pin(f) as _)
    }

    fn load_html_fragment(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<CSS>, String)> {
        get_filepath(self.out_dir(), path, name, language, "ftml")
            .and_then(|p| OMDocResult::load_html_fragment(&p, range))
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_fragment_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        load_html_fragment_async(self.out_dir(), path, name, language, range)
            .map(|f| Box::pin(f) as _)
    }

    /// ### Errors
    fn load_reference_blob(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<Vec<u8>> {
        let Some(p) = get_filepath(self.out_dir(), path, name, language, "ftml") else {
            return Err(eyre::eyre!("File not found"));
        };
        OMDocResult::load_reference_blob(&p, range)
    }
    #[inline]
    fn load_document(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument> {
        load_document(self.out_dir(), path, name, language)
    }

    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>> {
        let out = path.map_or_else(
            || self.out_dir().join(".modules"),
            |n| {
                n.steps()
                    .iter()
                    .fold(self.out_dir().to_path_buf(), |p, n| p.join(n.as_ref()))
                    .join(".modules")
            },
        );

        let out = escape_module_name(&out, name);
        //.join(Into::<&'static str>::into(language));
        macro_rules! err {
            ($e:expr) => {
                match $e {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Error loading {}: {e}", out.display());
                        return None;
                    }
                }
            };
        }
        if out.exists() {
            let file = err!(std::fs::File::open(&out));
            let file = std::io::BufReader::new(file);
            Some(err!(bincode::serde::decode_from_reader(
                file,
                bincode::config::standard()
            )))
            //OpenModule::from_byte_stream(&mut file).ok()
        } else {
            None
        }
    }
}

impl<T: ArchiveBase + HasLocalOut + Send + Sync> Buildable for T {
    #[allow(clippy::cognitive_complexity)]
    fn save(
        &self,
        relative_path: &str,
        log: Either<String, PathBuf>,
        from: BuildTargetId,
        result: Option<BuildResultArtifact>,
    ) {
        macro_rules! err {
            ($e:expr) => {
                if let Err(e) = $e {
                    tracing::error!("Failed to save [{}]{}: {}", self.id(), relative_path, e);
                    return;
                }
            };
        }
        let top = self.out_dir().join(relative_path);
        err!(std::fs::create_dir_all(&top));
        let logfile = top.join(from.name()).with_extension("log");
        match log {
            Either::Left(s) => {
                err!(std::fs::write(&logfile, s));
            }
            Either::Right(f) => {
                err!(std::fs::rename(&f, &logfile));
            }
        }
        match result {
            Some(BuildResultArtifact::File(t, f)) => {
                let p = top.join(t.name());
                err!(std::fs::rename(&f, &p));
            }
            Some(BuildResultArtifact::Data(d)) => {
                if let Some(e) = d.as_any().downcast_ref::<OMDocResult>() {
                    save_omdoc_result(self.out_dir(), &top, e);
                    return;
                }
                let p = top.join(d.get_type().name());
                err!(d.write(&p));
            }
            None | Some(BuildResultArtifact::None) => (),
        }
    }
    #[inline]
    fn submit_triples(
        &self,
        in_doc: &DocumentURI,
        rel_path: &str,
        relational: &RDFStore,
        load: bool,
        iter: std::collections::hash_set::IntoIter<flams_ontology::rdf::Triple>,
    ) {
        submit_triples(self.out_dir(), in_doc, rel_path, relational, load, iter)
    }
}

pub trait LocalOut: HasLocalOut {
    #[must_use]
    #[inline]
    fn path(&self) -> &Path {
        self.out_dir().parent().unwrap_or_else(|| unreachable!())
    }

    /// ### Errors
    fn load_reference<T: flams_ontology::Resourcable>(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<T> {
        let Some(p) = get_filepath(self.out_dir(), path, name, language, "ftml") else {
            return Err(eyre::eyre!("File not found"));
        };
        OMDocResult::load_reference(&p, range)
    }

    /// ### Errors
    fn load<D: BuildArtifact>(&self, relative_path: &str) -> Result<D, std::io::Error> {
        let p = self.artifact_path(relative_path, D::get_type_id());
        if p.exists() {
            D::load(&p)
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }
}
impl<T: HasLocalOut + ?Sized> LocalOut for T {}

pub trait ExternalArchive: Send + Sync + ArchiveTrait + std::any::Any {
    #[inline]
    fn local_out(&self) -> Option<&dyn HasLocalOut> {
        None
    }
    #[inline]
    fn buildable(&self) -> Option<&dyn Buildable> {
        None
    }
}

#[derive(Debug)]
pub struct LocalArchive {
    pub(super) data: RepositoryData,
    pub(super) out_path: std::sync::Arc<Path>,
    pub(super) ignore: IgnoreSource,
    pub(super) file_state: parking_lot::RwLock<SourceDir>,
    #[cfg(feature = "gitlab")]
    pub(super) is_managed: std::sync::OnceLock<Option<git_url_parse::GitUrl>>,
}
impl ArchiveBase for LocalArchive {
    #[inline]
    fn data(&self) -> &RepositoryData {
        &self.data
    }
    #[inline]
    fn files(&self) -> &parking_lot::RwLock<SourceDir> {
        &self.file_state
    }

    #[cfg(feature = "gitlab")]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
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

    fn update_sources(&self, sender: &ChangeSender<BackendChange>) {
        let mut state = self.file_state.write();
        state.update(
            self.uri(),
            self.path(),
            sender,
            &self.ignore,
            self.formats(),
        );
    }
}
impl HasLocalOut for LocalArchive {
    #[inline]
    fn out_dir(&self) -> &Path {
        &self.out_path
    }
}

impl LocalArchive {
    #[inline]
    #[must_use]
    pub fn source_dir_of(p: &Path) -> PathBuf {
        p.join("source")
    }

    #[inline]
    pub fn with_sources<R>(&self, f: impl FnOnce(&SourceDir) -> R) -> R {
        f(&*self.files().read())
    }

    #[inline]
    pub fn file_state(&self) -> FileStates {
        self.file_state.read().state().clone()
    }

    #[inline]
    pub fn state_summary(&self) -> FileStateSummary {
        self.file_state.read().state().summarize()
    }

    #[inline]
    #[must_use]
    pub fn source_dir(&self) -> PathBuf {
        Self::source_dir_of(self.path())
    }

    #[inline]
    #[must_use]
    pub const fn dependencies(&self) -> &[ArchiveId] {
        &self.data.dependencies
    }
}

//#[non_exhaustive]
pub enum Archive {
    Local(LocalArchive),
    Ext(Box<dyn ExternalArchive>), //Scraped(InventoriedArchive),
}
impl std::fmt::Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(a) => a.id().fmt(f),
            Self::Ext(e) => e.fmt(f), //Self::Scraped(a) => a.id().fmt(f),
        }
    }
}

impl Archive {
    #[inline]
    #[must_use]
    fn data(&self) -> &RepositoryData {
        match self {
            Self::Local(a) => &a.data,
            Self::Ext(a) => a.data(),
        }
    }

    #[inline]
    #[must_use]
    pub fn uri(&self) -> ArchiveURIRef {
        self.data().uri.archive_uri()
    }
    #[inline]
    #[must_use]
    pub fn id(&self) -> &ArchiveId {
        self.data().uri.archive_id()
    }

    #[inline]
    #[must_use]
    pub fn formats(&self) -> &[SourceFormatId] {
        self.data().formats.0.as_slice()
    }

    #[inline]
    #[must_use]
    pub fn attributes(&self) -> &VecMap<Box<str>, Box<str>> {
        &self.data().attributes
    }

    #[inline]
    #[must_use]
    pub fn dependencies(&self) -> &[ArchiveId] {
        &self.data().dependencies
    }

    pub fn load_html_body(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<(Vec<CSS>, String)> {
        match self {
            Self::Local(a) => a.load_html_body(path, name, language, full),
            Self::Ext(a) => a.load_html_body(path, name, language, full), //Self::Scraped(a) => a.load_html_body(path, name, language, full),
        }
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_body_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<impl Future<Output = Option<(Vec<CSS>, String)>>> {
        match self {
            Self::Local(a) => load_html_body_async(a.out_dir(), path, name, language, full)
                .map(either::Either::Left),
            Self::Ext(a) => a
                .load_html_body_async(path, name, language, full)
                .map(either::Either::Right), //Self::Scraped(a) => a.out_dir(),
        }
    }

    pub fn load_html_full(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<String> {
        match self {
            Self::Local(a) => a.load_html_full(path, name, language),
            Self::Ext(a) => a.load_html_full(path, name, language), //Self::Scraped(a) => a.load_html_full(path, name, language),
        }
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_full_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<impl Future<Output = Option<String>>> {
        match self {
            Self::Local(a) => {
                load_html_full_async(a.out_dir(), path, name, language).map(either::Either::Left)
            } //Self::Scraped(a) => a.out_dir(),
            Self::Ext(a) => a
                .load_html_full_async(path, name, language)
                .map(either::Either::Right),
        }
    }

    pub fn load_html_fragment(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<CSS>, String)> {
        match self {
            Self::Local(a) => a.load_html_fragment(path, name, language, range),
            Self::Ext(a) => a.load_html_fragment(path, name, language, range),
        }
    }

    pub fn load_reference<T: flams_ontology::Resourcable>(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<T> {
        match self {
            Self::Local(a) => a.load_reference(path, name, language, range),
            Self::Ext(a) => {
                let bytes = a.load_reference_blob(path, name, language, range)?;
                let r = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())?;
                Ok(r.0)
            }
        }
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_fragment_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<impl Future<Output = Option<(Vec<CSS>, String)>>> {
        match self {
            Self::Local(a) => load_html_fragment_async(a.out_dir(), path, name, language, range)
                .map(either::Either::Left),
            Self::Ext(a) => a
                .load_html_fragment_async(path, name, language, range)
                .map(either::Either::Right),
        }
    }

    fn load_document(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument> {
        match self {
            Self::Local(a) => load_document(a.out_dir(), path, name, language),
            Self::Ext(a) => a.load_document(path, name, language),
        }
    }

    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>> {
        match self {
            Self::Local(a) => a.load_module(path, name),
            Self::Ext(a) => a.load_module(path, name),
        }
    }

    pub fn as_local_out(&self) -> Option<LocalOrLocalOut> {
        match self {
            Self::Local(a) => Some(either::Either::Left(a)),
            Self::Ext(a) => a.local_out().map(either::Either::Right),
        }
    }

    pub fn as_buildable(&self) -> Option<LocalOrBuildable> {
        match self {
            Self::Local(a) => Some(either::Either::Left(a)),
            Self::Ext(a) => a.buildable().map(either::Either::Right),
        }
    }

    #[inline]
    fn update_sources(&self, sender: &ChangeSender<BackendChange>) {
        match self {
            Self::Local(a) => a.update_sources(sender),
            Self::Ext(a) => a.update_sources(sender),
            //Self::Scraped(a) => a.update_sources(sender),
        }
    }

    #[inline]
    pub fn with_sources<R>(&self, f: impl FnOnce(&SourceDir) -> R) -> R {
        match self {
            Self::Local(a) => f(&*a.files().read()),
            Self::Ext(a) => f(&*a.files().read()),
        }
    }

    /*

    #[inline]
    pub fn get_log(&self, relative_path: &str, target: BuildTargetId) -> Option<PathBuf> {
        match self {
            Self::Local(a) => Some(a.get_log(relative_path, target)),
            Self::Ext(_) => None, //Self::Scraped(a) => a.get_log(relative_path, target),
        }
    }


     /// ### Errors
     #[inline]
     pub fn load<D: BuildArtifact>(&self, relative_path: &str) -> Result<D, std::io::Error> {
         match self {
             Self::Local(a) => a.load(relative_path),
             //Self::Scraped(a) => a.load(relative_path),
         }
     }

    pub fn save(
        &self,
        relative_path: &str,
        log: Either<String, PathBuf>,
        from: BuildTargetId,
        result: Option<BuildResultArtifact>,
    ) {
        match self {
            Self::Local(a) => a.save(relative_path, log, from, result),
            //Self::Scraped(a) => a.save(relative_path, log, from, result),
        }
    }
     */
}

#[derive(Debug, Default)]
pub struct ArchiveTree {
    pub archives: Vec<Archive>,
    pub groups: Vec<ArchiveOrGroup>,
    pub index: (VecSet<Institution>, VecSet<ArchiveIndex>),
}

#[derive(Debug)]
pub enum ArchiveOrGroup {
    Archive(ArchiveId),
    Group(ArchiveGroup),
}

impl ArchiveOrGroup {
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &ArchiveId {
        match self {
            Self::Archive(id) => id,
            Self::Group(g) => &g.id,
        }
    }
}

#[derive(Debug)]
pub struct ArchiveGroup {
    pub id: ArchiveId,
    pub children: Vec<ArchiveOrGroup>,
    pub state: FileStates,
}

impl TreeLike for ArchiveTree {
    type RefIter<'a> = std::slice::Iter<'a, ArchiveOrGroup>;
    type Child<'a> = &'a ArchiveOrGroup;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        Some(self.groups.iter())
    }
}

impl TreeLike for &ArchiveGroup {
    type RefIter<'a>
        = std::slice::Iter<'a, ArchiveOrGroup>
    where
        Self: 'a;
    type Child<'a>
        = &'a ArchiveOrGroup
    where
        Self: 'a;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        Some(self.children.iter())
    }
}

impl TreeChild<ArchiveTree> for &ArchiveOrGroup {
    fn children<'a>(&self) -> Option<<ArchiveTree as TreeLike>::RefIter<'a>>
    where
        Self: 'a,
    {
        if let ArchiveOrGroup::Group(a) = self {
            Some(a.children.iter())
        } else {
            None
        }
    }
}

impl TreeChild<&ArchiveGroup> for &ArchiveOrGroup {
    fn children<'a>(&self) -> Option<std::slice::Iter<'a, ArchiveOrGroup>>
    where
        Self: 'a,
    {
        if let ArchiveOrGroup::Group(a) = self {
            Some(a.children.iter())
        } else {
            None
        }
    }
}

impl ArchiveTree {
    #[must_use]
    pub fn find(&self, id: &ArchiveId) -> Option<&ArchiveOrGroup> {
        let mut steps = id.steps().peekable();
        let mut curr = &self.groups;
        while let Some(step) = steps.next() {
            let e = curr.iter().find(|e| e.id().last_name() == step)?;
            /*let Ok(i) = curr.binary_search_by_key(&step, |v| v.id().last_name()) else {
                return None;
            };*/
            if steps.peek().is_none() {
                return Some(e);
            } //{ return Some(&curr[i]); }
            if let ArchiveOrGroup::Group(g) = e {
                //&curr[i] {
                curr = &g.children;
            } else {
                return None;
            }
        }
        None
    }

    #[must_use]
    pub fn get(&self, id: &ArchiveId) -> Option<&Archive> {
        self.archives.iter().find(|a| a.uri().archive_id() == id)
        //self.archives.binary_search_by_key(&id, Archive::id).ok()
        //    .map(|i| &self.archives[i])
    }

    #[instrument(level = "info",
    target = "archives",
    name = "Loading archives",
    fields(path = %path.display()),
    skip_all
    )]
    pub(crate) fn load(
        &mut self,
        path: &Path,
        sender: &ChangeSender<BackendChange>,
        f: impl MaybeQuads,
    ) {
        tracing::info!(target:"archives","Searching for archives");
        let old = std::mem::take(self);
        let old_new_f = parking_lot::Mutex::new((old, Self::default(), f));

        ArchiveIterator::new(path)
            .par_split()
            .into_par_iter()
            .for_each(|a| {
                a.update_sources(sender);
                let mut lock = old_new_f.lock();
                let (old, new, f) = &mut *lock;
                if old.remove_from_list(a.id()).is_none() {
                    sender.lazy_send(|| BackendChange::NewArchive(URIRefTrait::owned(a.uri())));
                }
                new.insert(a, f);
                drop(lock);
                // todo
            });
        let (_old, new, _) = old_new_f.into_inner();
        //news.sort_by_key(|a| a.id()); <- alternative
        *self = new;
        for a in &self.archives {
            for i in &a.data().institutions {
                self.index.0.insert_clone(i);
            }
            for doc in &a.data().index {
                self.index.1.insert_clone(doc);
            }
        }
        // TODO olds
    }

    #[inline]
    fn remove_from_list(&mut self, id: &ArchiveId) -> Option<Archive> {
        if let Ok(i) = self
            .archives
            .binary_search_by_key(&id, |a: &Archive| a.id())
        {
            Some(self.archives.remove(i))
        } else {
            None
        }
    }

    fn _remove(&mut self, id: &ArchiveId) -> Option<Archive> {
        let mut curr = &mut self.groups;
        let mut steps = id.steps();
        while let Some(step) = steps.next() {
            let Ok(i) = curr.binary_search_by_key(&step, |v| v.id().last_name()) else {
                return None;
            };
            if matches!(curr[i], ArchiveOrGroup::Group(_)) {
                let ArchiveOrGroup::Group(g) = &mut curr[i] else {
                    unreachable!()
                };
                curr = &mut g.children;
                continue;
            }
            if steps.next().is_some() {
                return None;
            }
            let ArchiveOrGroup::Archive(a) = curr.remove(i) else {
                unreachable!()
            };
            let Ok(i) = self
                .archives
                .binary_search_by_key(&&a, |a: &Archive| a.id())
            else {
                unreachable!()
            };
            return Some(self.archives.remove(i));
        }
        None
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    #[allow(irrefutable_let_patterns)]
    fn insert(&mut self, archive: Archive, _f: &mut impl MaybeQuads) {
        let id = archive.id().clone();
        let steps = if let Some((group, _)) = id.as_ref().rsplit_once('/') {
            group.split('/')
        } else {
            match self
                .archives
                .binary_search_by_key(&&id, |a: &Archive| a.id())
            {
                Ok(i) => self.archives[i] = archive,
                Err(i) => self.archives.insert(i, archive),
            };
            match self
                .groups
                .binary_search_by_key(&id.as_ref(), |v| v.id().last_name())
            {
                Ok(i) => self.groups[i] = ArchiveOrGroup::Archive(id),
                Err(i) => self.groups.insert(i, ArchiveOrGroup::Archive(id)),
            }
            return;
        };
        let mut curr = &mut self.groups;
        let mut curr_name = String::new();
        for step in steps {
            if curr_name.is_empty() {
                curr_name = step.to_string();
            } else {
                curr_name = format!("{curr_name}/{step}");
            }
            match curr.binary_search_by_key(&step, |v| v.id().last_name()) {
                Ok(i) => {
                    let ArchiveOrGroup::Group(g) = &mut curr[i]
                    // TODO maybe reachable?
                    else {
                        unreachable!()
                    };
                    if let Archive::Local(a) = &archive {
                        g.state.merge_all(a.file_state.read().state());
                    }
                    curr = &mut g.children;
                }
                Err(i) => {
                    let mut state = FileStates::default();
                    if let Archive::Local(a) = &archive {
                        state.merge_all(a.file_state.read().state());
                    }
                    let g = ArchiveGroup {
                        id: ArchiveId::new(&curr_name),
                        children: Vec::new(),
                        state,
                    };
                    curr.insert(i, ArchiveOrGroup::Group(g));
                    let ArchiveOrGroup::Group(g) = &mut curr[i] else {
                        unreachable!()
                    };
                    curr = &mut g.children;
                }
            }
        }

        match self
            .archives
            .binary_search_by_key(&&id, |a: &Archive| a.id())
        {
            Ok(i) => self.archives[i] = archive,
            Err(i) => self.archives.insert(i, archive),
        };
        match curr.binary_search_by_key(&id.last_name(), |v| v.id().last_name()) {
            Ok(i) => curr[i] = ArchiveOrGroup::Archive(id),
            Err(i) => curr.insert(i, ArchiveOrGroup::Archive(id)),
        }
    }
}

// -------------------------------------------------------------------------------------------------

fn escape_module_name(in_path: &Path, name: &NameStep) -> PathBuf {
    static REPLACER: flams_utils::escaping::Escaper<u8, 1> =
        flams_utils::escaping::Escaper([(b'*', "__AST__")]);
    in_path.join(REPLACER.escape(name).to_string())
}

fn submit_triples(
    out: &Path,
    in_doc: &DocumentURI,
    rel_path: &str,
    relational: &RDFStore,
    load: bool,
    iter: impl Iterator<Item = flams_ontology::rdf::Triple>,
) {
    let out = rel_path
        .split('/')
        .fold(out.to_path_buf(), |p, s| p.join(s));
    let _ = std::fs::create_dir_all(&out);
    let out = out.join("index.ttl");
    relational.export(iter, &out, in_doc);
    if load {
        relational.load(&out, in_doc.to_iri());
    }
}

pub(super) fn get_filepath(
    out: &Path,
    path: Option<&Name>,
    name: &NameStep,
    language: Language,
    filename: &str,
) -> Option<PathBuf> {
    let out = path.map_or_else(
        || out.to_path_buf(),
        |n| {
            n.steps()
                .iter()
                .fold(out.to_path_buf(), |p, n| p.join(n.as_ref()))
        },
    );
    let name = name.as_ref();

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

fn load_document(
    out: &Path,
    path: Option<&Name>,
    name: &NameStep,
    language: Language,
) -> Option<UncheckedDocument> {
    get_filepath(out, path, name, language, "doc").and_then(|p| PreDocFile::read_from_file(&p))
}

#[cfg(feature = "tokio")]
#[inline]
fn load_html_body_async(
    out: &Path,
    path: Option<&Name>,
    name: &NameStep,
    language: Language,
    full: bool,
) -> Option<impl Future<Output = Option<(Vec<CSS>, String)>> + 'static> {
    let p = get_filepath(out, path, name, language, "ftml")?;
    Some(OMDocResult::load_html_body_async(p, full))
}

#[cfg(feature = "tokio")]
#[inline]
fn load_html_full_async(
    out: &Path,
    path: Option<&Name>,
    name: &NameStep,
    language: Language,
) -> Option<impl Future<Output = Option<String>> + 'static> {
    let p = get_filepath(out, path, name, language, "ftml")?;
    Some(OMDocResult::load_html_full_async(p))
}

#[cfg(feature = "tokio")]
#[inline]
fn load_html_fragment_async(
    out: &Path,
    path: Option<&Name>,
    name: &NameStep,
    language: Language,
    range: DocumentRange,
) -> Option<impl Future<Output = Option<(Vec<CSS>, String)>> + 'static> {
    let p = get_filepath(out, path, name, language, "ftml")?;
    Some(OMDocResult::load_html_fragment_async(p, range))
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cognitive_complexity)]
fn save_omdoc_result(out: &Path, top: &Path, result: &OMDocResult) {
    macro_rules! err {
        ($e:expr) => {
            match $e {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to save {}: {}", top.display(), e);
                    return;
                }
            }
        };
    }
    macro_rules! er {
        ($e:expr) => {
            if let Err(e) = $e {
                tracing::error!("Failed to save {}: {}", top.display(), e);
                return;
            }
        };
    }
    let p = top.join("ftml");
    result.write(&p);
    let OMDocResult {
        document,
        modules,
        html,
    } = result;
    let p = top.join("doc");
    let file = err!(std::fs::File::create(&p));
    let mut buf = std::io::BufWriter::new(file);

    er!(bincode::serde::encode_into_std_write(
        document,
        &mut buf,
        bincode::config::standard()
    ));
    //er!(document.into_byte_stream(&mut buf));

    #[cfg(feature = "tantivy")]
    {
        let p = top.join("tantivy");
        let file = err!(std::fs::File::create(&p));
        let mut buf = std::io::BufWriter::new(file);
        let ret = document.all_searches(&html.html);
        er!(bincode::serde::encode_into_std_write(
            ret,
            &mut buf,
            bincode::config::standard()
        ));
    }

    for m in modules {
        let path = m.uri.path();
        let name = m.uri.name();
        //let language = m.uri.language();
        let out = path.map_or_else(
            || out.join(".modules"),
            |n| {
                n.steps()
                    .iter()
                    .fold(out.to_path_buf(), |p, n| p.join(n.as_ref()))
                    .join(".modules")
            },
        );
        //.join(name.to_string());
        err!(std::fs::create_dir_all(&out));
        let out = escape_module_name(&out, name.first_name());
        let file = err!(std::fs::File::create(&out));
        let mut buf = std::io::BufWriter::new(file);
        //er!(m.into_byte_stream(&mut buf));
        er!(bincode::serde::encode_into_std_write(
            m,
            &mut buf,
            bincode::config::standard()
        ));
    }
}

// -------------------------------------------------------------------------------------------

impl<'a, A1: ArchiveBase + ?Sized, A2: ArchiveBase + ?Sized> ArchiveBase
    for either::Either<&'a A1, &'a A2>
{
    #[inline]
    fn data(&self) -> &RepositoryData {
        match self {
            either::Either::Left(a) => a.data(),
            either::Either::Right(a) => a.data(),
        }
    }
    #[inline]
    fn files(&self) -> &parking_lot::RwLock<SourceDir> {
        match self {
            either::Either::Left(a) => a.files(),
            either::Either::Right(a) => a.files(),
        }
    }
    #[inline]
    fn update_sources(&self, sender: &ChangeSender<BackendChange>) {
        match self {
            either::Either::Left(a) => a.update_sources(sender),
            either::Either::Right(a) => a.update_sources(sender),
        }
    }

    #[cfg(feature = "gitlab")]
    #[inline]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
        match self {
            either::Either::Left(a) => a.is_managed(),
            either::Either::Right(a) => a.is_managed(),
        }
    }
}

impl<'a, A1: ArchiveTrait + ?Sized, A2: ArchiveTrait + ?Sized> ArchiveTrait
    for either::Either<&'a A1, &'a A2>
{
    #[inline]
    fn load_html_full(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<String> {
        match self {
            either::Either::Left(a) => a.load_html_full(path, name, language),
            either::Either::Right(a) => a.load_html_full(path, name, language),
        }
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_full_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Fut<Option<String>> {
        match self {
            either::Either::Left(a) => a.load_html_full_async(path, name, language),
            either::Either::Right(a) => a.load_html_full_async(path, name, language),
        }
    }

    #[inline]
    fn load_html_body(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<(Vec<CSS>, String)> {
        match self {
            either::Either::Left(a) => a.load_html_body(path, name, language, full),
            either::Either::Right(a) => a.load_html_body(path, name, language, full),
        }
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_body_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        match self {
            either::Either::Left(a) => a.load_html_body_async(path, name, language, full),
            either::Either::Right(a) => a.load_html_body_async(path, name, language, full),
        }
    }

    #[inline]
    fn load_html_fragment(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<CSS>, String)> {
        match self {
            either::Either::Left(a) => a.load_html_fragment(path, name, language, range),
            either::Either::Right(a) => a.load_html_fragment(path, name, language, range),
        }
    }

    #[cfg(feature = "tokio")]
    #[inline]
    fn load_html_fragment_async(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        match self {
            either::Either::Left(a) => a.load_html_fragment_async(path, name, language, range),
            either::Either::Right(a) => a.load_html_fragment_async(path, name, language, range),
        }
    }

    fn load_reference_blob(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<Vec<u8>> {
        match self {
            either::Either::Left(a) => a.load_reference_blob(path, name, language, range),
            either::Either::Right(a) => a.load_reference_blob(path, name, language, range),
        }
    }

    fn load_document(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument> {
        match self {
            either::Either::Left(a) => a.load_document(path, name, language),
            either::Either::Right(a) => a.load_document(path, name, language),
        }
    }

    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>> {
        match self {
            either::Either::Left(a) => a.load_module(path, name),
            either::Either::Right(a) => a.load_module(path, name),
        }
    }
}

pub type LocalOrLocalOut<'a> = either::Either<&'a LocalArchive, &'a dyn HasLocalOut>;
pub type LocalOrBuildable<'a> = either::Either<&'a LocalArchive, &'a dyn Buildable>;

impl<'a> HasLocalOut for LocalOrLocalOut<'a> {
    #[inline]
    fn out_dir(&self) -> &Path {
        match self {
            either::Either::Left(a) => a.out_dir(),
            either::Either::Right(a) => a.out_dir(),
        }
    }
}
impl<'a> Buildable for LocalOrBuildable<'a> {
    #[inline]
    fn save(
        &self,
        relative_path: &str,
        log: Either<String, PathBuf>,
        from: BuildTargetId,
        result: Option<BuildResultArtifact>,
    ) {
        match self {
            either::Either::Left(a) => a.save(relative_path, log, from, result),
            either::Either::Right(a) => a.save(relative_path, log, from, result),
        }
    }
    #[inline]
    fn submit_triples(
        &self,
        in_doc: &DocumentURI,
        rel_path: &str,
        relational: &RDFStore,
        load: bool,
        iter: std::collections::hash_set::IntoIter<flams_ontology::rdf::Triple>,
    ) {
        match self {
            either::Either::Left(a) => a.submit_triples(in_doc, rel_path, relational, load, iter),
            either::Either::Right(a) => a.submit_triples(in_doc, rel_path, relational, load, iter),
        }
    }
}
