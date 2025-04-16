mod ignore_regex;
mod iter;
pub mod manager;
pub mod source_files;

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
    formats::{BuildTargetId, OMDocResult, SourceFormatId},
};

use super::{docfile::PreDocFile, rdf::RDFStore, BackendChange};

#[derive(Debug)]
pub(super) struct RepositoryData {
    pub(super) uri: ArchiveURI,
    pub(super) attributes: VecMap<Box<str>, Box<str>>,
    pub(super) formats: VecSet<SourceFormatId>,
    pub(super) dependencies: Box<[ArchiveId]>,
    pub(super) institutions: Box<[Institution]>,
    pub(super) index: Box<[ArchiveIndex]>,
}

/*
#[cfg(feature="zip")]
#[derive(Debug)]
pub(super) struct ZipFile {
    path:Option<std::path::PathBuf>
}

#[cfg(feature="zip")]
impl Drop for ZipFile {
    fn drop(&mut self) {
        if let Some(p) = self.path.take() {
            let _ = std::fs::remove_file(p);
        }
    }
}
*/

#[cfg(feature = "zip")]
mod zip {
    use std::path::PathBuf;

    use tokio::io::AsyncWriteExt;

    pub(super) struct ZipStream {
        handle: tokio::task::JoinHandle<()>,
        stream: tokio_util::io::ReaderStream<tokio::io::ReadHalf<tokio::io::SimplexStream>>,
    }
    impl ZipStream {
        pub(super) fn new(p: PathBuf) -> Self {
            let (reader, writer) = tokio::io::simplex(1024);
            let stream = tokio_util::io::ReaderStream::new(reader);
            let handle = tokio::task::spawn(Self::zip(p, writer));
            Self { handle, stream }
        }
        async fn zip(p: PathBuf, writer: tokio::io::WriteHalf<tokio::io::SimplexStream>) {
            let comp = async_compression::tokio::write::GzipEncoder::with_quality(
                writer,
                async_compression::Level::Best,
            );
            let mut tar = tokio_tar::Builder::new(comp);
            let _ = tar.append_dir_all(".", &p).await;
            let mut comp = match tar.into_inner().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to zip: {e}");
                    return;
                }
            };
            //let _ = comp.flush().await;
            let _ = comp.shutdown().await;
            tracing::info!("Finished zipping {}", p.display());
        }
    }
    impl Drop for ZipStream {
        fn drop(&mut self) {
            tracing::info!("Dropping");
            self.handle.abort();
        }
    }
    impl futures::Stream for ZipStream {
        type Item = std::io::Result<tokio_util::bytes::Bytes>;
        #[inline]
        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            unsafe { self.map_unchecked_mut(|f| &mut f.stream).poll_next(cx) }
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.stream.size_hint()
        }
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
    //#[cfg(feature="zip")]
    //pub(super) zip_file: std::sync::Arc<std::sync::OnceLock<Option<ZipFile>>>
}
impl LocalArchive {
    #[inline]
    #[must_use]
    pub fn out_dir_of(p: &Path) -> PathBuf {
        p.join(".flams")
    }

    #[cfg(feature = "zip")]
    /// #### Errors
    pub async fn unzip_from_remote(id: ArchiveId, url: &str) -> Result<(), ()> {
        use flams_utils::PathExt;
        use futures::TryStreamExt;
        let resp = match reqwest::get(url).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error contacting remote: {e}");
                return Err(());
            }
        };
        let status = resp.status().as_u16();
        if (400..=599).contains(&status) {
            let text = resp.text().await;
            tracing::error!("Error response from remote: {text:?}");
            return Err(());
        }
        let stream = resp.bytes_stream().map_err(std::io::Error::other);
        let stream = tokio_util::io::StreamReader::new(stream);
        let decomp = async_compression::tokio::bufread::GzipDecoder::new(stream);
        let dest = crate::settings::Settings::get()
            .temp_dir()
            .join(flams_utils::hashstr("download", &id));

        let mut tar = tokio_tar::Archive::new(decomp);
        if let Err(e) = tar.unpack(&dest).await {
            tracing::error!("Error unpacking stream: {e}");
            let _ = tokio::fs::remove_dir_all(dest).await;
            return Err(());
        };
        let mh = flams_utils::unwrap!(crate::settings::Settings::get().mathhubs.first());
        let mhdest = mh.join(id.as_ref());
        if let Err(e) = tokio::fs::create_dir_all(&mhdest).await {
            tracing::error!("Error moving to MathHub: {e}");
            return Err(());
        }
        if mhdest.exists() {
            let _ = tokio::fs::remove_dir_all(&mhdest).await;
        }
        match tokio::task::spawn_blocking(move || dest.rename_safe(&mhdest)).await {
            Ok(Ok(())) => Ok(()),
            Err(e) => {
                tracing::error!("Error moving to MathHub: {e}");
                Err(())
            }
            Ok(Err(e)) => {
                tracing::error!("Error moving to MathHub: {e:#}");
                Err(())
            }
        }
    }

    #[cfg(feature = "zip")]
    pub fn zip(&self) -> impl futures::Stream<Item = std::io::Result<tokio_util::bytes::Bytes>> {
        let dir_path = flams_utils::unwrap!(self.out_path.parent()).to_path_buf();
        zip::ZipStream::new(dir_path)
    }

    #[cfg(not(feature = "gitlab"))]
    #[inline]
    pub const fn is_managed(&self) -> Option<&str> {
        None
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

    #[inline]
    #[must_use]
    pub fn source_dir_of(p: &Path) -> PathBuf {
        p.join("source")
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        self.out_path.parent().unwrap_or_else(|| unreachable!())
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
    pub fn out_dir(&self) -> &Path {
        &self.out_path
    } //self.path().join(".flams") }

    #[inline]
    #[must_use]
    pub fn source_dir(&self) -> PathBuf {
        Self::source_dir_of(self.path())
    }

    #[inline]
    #[must_use]
    pub fn is_meta(&self) -> bool {
        self.data.uri.archive_id().is_meta()
    }

    #[inline]
    #[must_use]
    pub fn uri(&self) -> ArchiveURIRef {
        self.data.uri.archive_uri()
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> &ArchiveId {
        self.data.uri.archive_id()
    }

    #[inline]
    #[must_use]
    pub fn formats(&self) -> &[SourceFormatId] {
        self.data.formats.0.as_slice()
    }

    #[inline]
    #[must_use]
    pub const fn attributes(&self) -> &VecMap<Box<str>, Box<str>> {
        &self.data.attributes
    }

    #[inline]
    #[must_use]
    pub const fn dependencies(&self) -> &[ArchiveId] {
        &self.data.dependencies
    }

    #[inline]
    pub fn with_sources<R>(&self, f: impl FnOnce(&SourceDir) -> R) -> R {
        f(&self.file_state.read())
    }

    pub(crate) fn update_sources(&self, sender: &ChangeSender<BackendChange>) {
        let mut state = self.file_state.write();
        state.update(
            self.uri(),
            self.path(),
            sender,
            &self.ignore,
            self.formats(),
        );
    }

    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>> {
        let out = path
            .map_or_else(
                || self.out_dir().join(".modules"),
                |n| {
                    n.steps()
                        .iter()
                        .fold(self.out_dir().to_path_buf(), |p, n| p.join(n.as_ref()))
                        .join(".modules")
                },
            )
            .join(name.as_ref());
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

    fn submit_triples(
        &self,
        in_doc: &DocumentURI,
        rel_path: &str,
        relational: &RDFStore,
        load: bool,
        iter: impl Iterator<Item = flams_ontology::rdf::Triple>,
    ) {
        let out = rel_path
            .split('/')
            .fold(self.out_dir().to_path_buf(), |p, s| p.join(s));
        let _ = std::fs::create_dir_all(&out);
        let out = out.join("index.ttl");
        relational.export(iter, &out, in_doc);
        if load {
            relational.load(&out, in_doc.to_iri());
        }
    }

    fn get_filepath(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        filename: &str,
    ) -> Option<PathBuf> {
        let out = path.map_or_else(
            || self.out_dir().to_path_buf(),
            |n| {
                n.steps()
                    .iter()
                    .fold(self.out_dir().to_path_buf(), |p, n| p.join(n.as_ref()))
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
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument> {
        self.get_filepath(path, name, language, "doc")
            .and_then(|p| PreDocFile::read_from_file(&p))
    }

    pub fn load_html_body(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        full: bool,
    ) -> Option<(Vec<CSS>, String)> {
        self.get_filepath(path, name, language, "ftml")
            .and_then(|p| OMDocResult::load_html_body(&p, full))
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_body_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
        full: bool,
    ) -> Option<impl std::future::Future<Output = Option<(Vec<CSS>, String)>> + 'a> {
        let p = self.get_filepath(path, name, language, "ftml")?;
        Some(OMDocResult::load_html_body_async(p, full))
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_full_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
    ) -> Option<impl std::future::Future<Output = Option<String>> + 'a> {
        let p = self.get_filepath(path, name, language, "ftml")?;
        Some(OMDocResult::load_html_full_async(p))
    }

    pub fn load_html_full(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<String> {
        let p = self.get_filepath(path, name, language, "ftml")?;
        OMDocResult::load_html_full(p)
    }

    pub fn load_html_fragment(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<(Vec<CSS>, String)> {
        self.get_filepath(path, name, language, "ftml")
            .and_then(|p| OMDocResult::load_html_fragment(&p, range))
    }
    pub fn load_reference<T: flams_ontology::Resourcable>(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
        range: DocumentRange,
    ) -> eyre::Result<T> {
        let Some(p) = self.get_filepath(path, name, language, "ftml") else {
            return Err(eyre::eyre!("File not found"));
        };
        OMDocResult::load_reference(&p, range)
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_fragment_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<impl std::future::Future<Output = Option<(Vec<CSS>, String)>> + 'a> {
        let p = self.get_filepath(path, name, language, "ftml")?;
        Some(OMDocResult::load_html_fragment_async(p, range))
    }

    /// ### Errors
    pub fn load<D: BuildArtifact>(&self, relative_path: &str) -> Result<D, std::io::Error> {
        let p = self
            .out_dir()
            .join(relative_path)
            .join(D::get_type_id().name());
        if p.exists() {
            D::load(&p)
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cognitive_complexity)]
    fn save_omdoc_result(&self, top: &Path, result: &OMDocResult) {
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
                || self.out_dir().join(".modules"),
                |n| {
                    n.steps()
                        .iter()
                        .fold(self.out_dir().to_path_buf(), |p, n| p.join(n.as_ref()))
                        .join(".modules")
                },
            );
            //.join(name.to_string());
            err!(std::fs::create_dir_all(&out));
            let out = out.join(name.first_name().as_ref());
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

    pub fn get_log(&self, relative_path: &str, target: BuildTargetId) -> PathBuf {
        self.out_dir()
            .join(relative_path)
            .join(target.name())
            .with_extension("log")
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn save(
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
                    self.save_omdoc_result(&top, e);
                    return;
                }
                let p = top.join(d.get_type().name());
                err!(d.write(&p));
            }
            None | Some(BuildResultArtifact::None) => (),
        }
    }
}

#[non_exhaustive]
pub enum Archive {
    Local(LocalArchive),
}
impl std::fmt::Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(a) => a.id().fmt(f),
        }
    }
}
impl Archive {
    #[inline]
    pub fn get_log(&self, relative_path: &str, target: BuildTargetId) -> PathBuf {
        match self {
            Self::Local(a) => a.get_log(relative_path, target),
        }
    }

    #[inline]
    pub fn with_sources<R>(&self, f: impl FnOnce(&SourceDir) -> R) -> R {
        match self {
            Self::Local(a) => a.with_sources(f),
        }
    }

    pub fn submit_triples(
        &self,
        in_doc: &DocumentURI,
        rel_path: &str,
        relational: &RDFStore,
        load: bool,
        iter: impl Iterator<Item = flams_ontology::rdf::Triple>,
    ) {
        match self {
            Self::Local(a) => a.submit_triples(in_doc, rel_path, relational, load, iter),
        }
    }

    #[inline]
    #[must_use]
    const fn data(&self) -> &RepositoryData {
        match self {
            Self::Local(a) => &a.data,
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
    pub const fn attributes(&self) -> &VecMap<Box<str>, Box<str>> {
        &self.data().attributes
    }

    #[inline]
    #[must_use]
    pub const fn dependencies(&self) -> &[ArchiveId] {
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
        }
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_body_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
        full: bool,
    ) -> Option<impl std::future::Future<Output = Option<(Vec<CSS>, String)>> + 'a> {
        match self {
            Self::Local(a) => a.load_html_body_async(path, name, language, full),
        }
    }
    #[cfg(feature = "tokio")]
    pub fn load_html_full_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
    ) -> Option<impl std::future::Future<Output = Option<String>> + 'a> {
        match self {
            Self::Local(a) => a.load_html_full_async(path, name, language),
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
        }
    }

    #[cfg(feature = "tokio")]
    pub fn load_html_fragment_async<'a>(
        &self,
        path: Option<&'a Name>,
        name: &'a NameStep,
        language: Language,
        range: DocumentRange,
    ) -> Option<impl std::future::Future<Output = Option<(Vec<CSS>, String)>> + 'a> {
        match self {
            Self::Local(a) => a.load_html_fragment_async(path, name, language, range),
        }
    }

    fn load_document(
        &self,
        path: Option<&Name>,
        name: &NameStep,
        language: Language,
    ) -> Option<UncheckedDocument> {
        match self {
            Self::Local(a) => a.load_document(path, name, language),
        }
    }
    fn load_module(&self, path: Option<&Name>, name: &NameStep) -> Option<OpenModule<Unchecked>> {
        match self {
            Self::Local(a) => a.load_module(path, name),
        }
    }

    /// ### Errors
    #[inline]
    pub fn load<D: BuildArtifact>(&self, relative_path: &str) -> Result<D, std::io::Error> {
        match self {
            Self::Local(a) => a.load(relative_path),
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
        }
    }
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
                new.insert(Archive::Local(a), f);
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
