#[cfg(feature = "rdf")]
use ftml_ontology::narrative::elements::Notation;
use ftml_ontology::{
    domain::modules::{Module, ModuleLike},
    narrative::{DocDataRef, DocumentRange, documents::Document},
    utils::Css,
};
#[cfg(feature = "rdf")]
use ftml_uris::DocumentElementUri;
use ftml_uris::{
    ArchiveId, DocumentUri, IsNarrativeUri, ModuleUri, NamedUri, SymbolUri, UriPath,
    UriWithArchive, UriWithPath,
};
use futures_util::TryFutureExt;

use crate::{
    Archive, ExternalArchive, LocallyBuilt,
    backend::LocalBackend,
    document_file::DocumentFile,
    manager::{ArchiveManager, ArchiveOrGroup},
    utils::{
        AsyncEngine,
        errors::{ArtifactSaveError, BackendError},
    },
};

static GLOBAL: std::sync::LazyLock<ArchiveManager> =
    std::sync::LazyLock::new(ArchiveManager::default);

#[derive(Debug, Copy, Clone)]
pub struct GlobalBackend;
impl std::ops::Deref for GlobalBackend {
    type Target = ArchiveManager;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &GLOBAL
    }
}

impl GlobalBackend {
    #[inline]
    #[must_use]
    pub fn get(&self) -> &'static ArchiveManager {
        &GLOBAL
    }
    pub fn initialize<A: AsyncEngine>() {
        Self.load(crate::mathhub::mathhubs());
        #[cfg(feature = "rdf")]
        {
            A::background(|| Self.triple_store().load_archives(&Self.all_archives()));
        }
    }

    pub fn reset<A: AsyncEngine>(self) {
        self.reinit(|_| (), crate::mathhub::mathhubs());
        #[cfg(feature = "rdf")]
        {
            A::background(|| Self.triple_store().load_archives(&Self.all_archives()));
        }
    }
}

impl LocalBackend for ArchiveManager {
    type ArchiveIter<'a>
        = &'a [Archive]
    where
        Self: Sized;

    fn save(
        &self,
        in_doc: &ftml_uris::DocumentUri,
        rel_path: Option<&UriPath>,
        log: crate::artifacts::FileOrString,
        from: crate::formats::BuildTargetId,
        result: Option<Box<dyn crate::artifacts::Artifact>>,
    ) -> std::result::Result<(), crate::utils::errors::ArtifactSaveError> {
        self.with_buildable_archive(in_doc.archive_id(), |a| {
            let Some(a) = a else {
                return Err(ArtifactSaveError::NoArchive);
            };
            a.save(
                in_doc,
                rel_path,
                log,
                from,
                result,
                #[cfg(feature = "rdf")]
                self.triple_store(),
                #[cfg(feature = "rdf")]
                true,
            )
        })
    }

    fn with_archive<R>(&self, id: &ArchiveId, f: impl FnOnce(Option<&Archive>) -> R) -> R {
        let tree = self.tree.read();
        f(tree.get(id))
    }

    fn with_archives<R>(&self, f: impl FnOnce(Self::ArchiveIter<'_>) -> R) -> R
    where
        Self: Sized,
    {
        f(&self.all_archives())
    }

    fn with_archive_or_group<R>(
        &self,
        id: &ArchiveId,
        f: impl FnOnce(Option<&ArchiveOrGroup>) -> R,
    ) -> R
    where
        Self: Sized,
    {
        self.with_tree(|t| f(t.get_group_or_archive(id)))
    }

    fn get_document(&self, uri: &DocumentUri) -> Result<Document, BackendError> {
        self.with_doc(
            uri,
            |docfile| docfile.get_document().map_err(Into::into),
            |o| todo!(),
        )
    }

    fn get_document_async<A: AsyncEngine>(
        &self,
        uri: &DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError>> + Send + use<A>
    where
        Self: Sized,
    {
        self.with_doc_async::<A, _, _, _, _, _>(
            uri,
            |docfile| async move { docfile.get_document_async::<A>().await.map_err(Into::into) },
            |o| std::future::ready(todo!()),
        )
    }

    fn get_html_full(&self, uri: &DocumentUri) -> Result<Box<str>, BackendError> {
        self.with_doc(
            uri,
            |docfile| docfile.get_html().map_err(Into::into),
            |o| todo!(),
        )
    }

    fn get_html_body(&self, uri: &DocumentUri) -> Result<(Box<[Css]>, Box<str>), BackendError> {
        self.with_doc(
            uri,
            |docfile| {
                docfile
                    .get_html_body()
                    .map_err(Into::into)
                    .map(|s| (docfile.get_css(), s))
            },
            |o| todo!(),
        )
    }

    fn get_html_body_async<A: AsyncEngine>(
        &self,
        uri: &ftml_uris::DocumentUri,
    ) -> impl Future<Output = Result<(Box<[ftml_ontology::utils::Css]>, Box<str>), BackendError>>
    + Send
    + use<A>
    where
        Self: Sized,
    {
        self.with_doc_async::<A, _, _, _, _, _>(
            uri,
            |docfile| {
                A::block_on(move || {
                    docfile
                        .get_html_body()
                        .map_err(Into::into)
                        .map(|s| (docfile.get_css(), s))
                })
            },
            |o| std::future::ready(todo!()),
        )
    }

    fn get_html_body_inner(
        &self,
        uri: &DocumentUri,
    ) -> Result<(Box<[Css]>, Box<str>), BackendError> {
        self.with_doc(
            uri,
            |docfile| {
                docfile
                    .get_html_body_inner()
                    .map_err(Into::into)
                    .map(|s| (docfile.get_css(), s))
            },
            |o| todo!(),
        )
    }

    fn get_html_body_inner_async<A: AsyncEngine>(
        &self,
        uri: &ftml_uris::DocumentUri,
    ) -> impl Future<Output = Result<(Box<[ftml_ontology::utils::Css]>, Box<str>), BackendError>>
    + Send
    + use<A>
    where
        Self: Sized,
    {
        self.with_doc_async::<A, _, _, _, _, _>(
            uri,
            |docfile| {
                A::block_on(move || {
                    docfile
                        .get_html_body_inner()
                        .map_err(Into::into)
                        .map(|s| (docfile.get_css(), s))
                })
            },
            |o| std::future::ready(todo!()),
        )
    }

    fn get_html_fragment(
        &self,
        uri: &DocumentUri,
        range: DocumentRange,
    ) -> Result<(Box<[Css]>, Box<str>), BackendError> {
        self.with_doc(
            uri,
            |docfile| {
                docfile
                    .get_html_range(range)
                    .map_err(Into::into)
                    .map(|s| (docfile.get_css(), s))
            },
            |o| todo!(),
        )
    }

    fn get_html_fragment_async<A: AsyncEngine>(
        &self,
        uri: &ftml_uris::DocumentUri,
        range: ftml_ontology::narrative::DocumentRange,
    ) -> impl Future<Output = Result<(Box<[ftml_ontology::utils::Css]>, Box<str>), BackendError>>
    + Send
    + use<A> {
        self.with_doc_async::<A, _, _, _, _, _>(
            uri,
            move |docfile| {
                A::block_on(move || {
                    docfile
                        .get_html_range(range)
                        .map_err(Into::into)
                        .map(|s| (docfile.get_css(), s))
                })
            },
            |o| std::future::ready(todo!()),
        )
    }

    fn get_reference<T: bincode::Decode<()>>(&self, rf: &DocDataRef<T>) -> Result<T, BackendError>
    where
        Self: Sized,
    {
        let DocDataRef {
            start,
            end,
            in_doc: uri,
            ..
        } = rf;
        self.with_doc(
            uri,
            |docfile| docfile.get_data(*start, *end).map_err(Into::into),
            |o| todo!(),
        )
    }

    fn get_module(&self, uri: &ModuleUri) -> Result<ModuleLike, BackendError> {
        if uri.is_top() {
            self.modules
                .get_sync(uri.clone(), |uri| {
                    self.load_module(uri.archive_uri(), uri.path(), uri.name().as_ref())
                })
                .map(ModuleLike::Module)
        } else {
            // SAFETY: !uri.is_top()
            let SymbolUri { name, module } =
                unsafe { uri.clone().into_symbol().unwrap_unchecked() };
            let m = self.modules.get_sync(module, |uri| {
                self.load_module(uri.archive_uri(), uri.path(), uri.name().as_ref())
            })?;
            m.as_module_like(&name)
                .ok_or(BackendError::NotFound(ftml_uris::UriKind::Symbol))
        }
    }

    fn get_module_async<A: AsyncEngine>(
        &self,
        uri: &ModuleUri,
    ) -> impl Future<Output = Result<ModuleLike, BackendError>> + Send + use<A>
    where
        Self: Sized,
    {
        if uri.is_top() {
            if let Some(m) = self.modules.has_async(uri) {
                return either::Left(either::Left(m.map_ok(ModuleLike::Module)));
            }
            let lm =
                self.load_module_async::<A>(uri.archive_uri(), uri.path(), uri.name().as_ref());
            either::Left(either::Right(
                self.modules
                    .get(uri.clone(), |_| lm)
                    .map_ok(ModuleLike::Module),
            ))
        } else {
            // SAFETY: !uri.is_top()
            let SymbolUri { name, module } =
                unsafe { uri.clone().into_symbol().unwrap_unchecked() };
            let m = if let Some(m) = self.modules.has_async(&module) {
                either::Left(m)
            } else {
                either::Right(self.load_module_async::<A>(
                    module.archive_uri(),
                    module.path(),
                    module.name().as_ref(),
                ))
            };
            either::Right(m.and_then(move |m| {
                std::future::ready(
                    m.as_module_like(&name)
                        .ok_or(BackendError::NotFound(ftml_uris::UriKind::Symbol)),
                )
            }))
        }
    }

    #[cfg(feature = "rdf")]
    fn get_notations(&self, uri: &SymbolUri) -> impl Iterator<Item = (DocumentElementUri, Notation)>
    where
        Self: Sized,
    {
        use ftml_uris::FtmlUri;
        self.do_notations(uri.to_iri())
    }

    #[cfg(feature = "rdf")]
    fn get_var_notations(
        &self,
        uri: &DocumentElementUri,
    ) -> impl Iterator<Item = (DocumentElementUri, Notation)>
    where
        Self: Sized,
    {
        use ftml_uris::FtmlUri;
        self.do_var_notations(uri.to_iri())
    }
}

impl ArchiveManager {
    fn with_doc<R>(
        &self,
        uri: &DocumentUri,
        then: impl FnOnce(&DocumentFile) -> Result<R, BackendError>,
        other: impl FnOnce(&dyn ExternalArchive) -> Result<R, BackendError>,
    ) -> Result<R, BackendError> {
        if let Some(v) = self.documents.has(uri) {
            let docfile = v?;
            return then(&docfile);
        }
        let file_or_other = self.with_archive(uri.archive_id(), |a| {
            let Some(a) = a else {
                return Err(BackendError::ArchiveNotFound);
            };
            match a {
                Archive::Local(a) => Ok(either::Left(a.document_file(
                    uri.path(),
                    None,
                    &uri.name,
                    uri.language(),
                ))),
                Archive::Ext(_, ext) => other(&**ext).map(either::Right),
            }
        })?;
        match file_or_other {
            either::Left(file) => {
                let docfile = self.documents.get_sync(uri.clone(), |_| {
                    DocumentFile::from_file(file)
                        .map(triomphe::Arc::new)
                        .map_err(Into::into)
                })?;
                then(&docfile)
            }
            either::Right(r) => Ok(r),
        }
    }

    fn with_doc_async<
        A: AsyncEngine,
        R: Send,
        T: Future<Output = Result<R, BackendError>> + Send,
        O: Future<Output = Result<R, BackendError>> + Send,
        Then: FnOnce(triomphe::Arc<DocumentFile>) -> T + Send,
        Other: FnOnce(&dyn ExternalArchive) -> O,
    >(
        &self,
        uri: &DocumentUri,
        then: Then,
        other: Other,
    ) -> impl Future<Output = Result<R, BackendError>> + Send + use<A, R, T, O, Then, Other> {
        if let Some(v) = self.documents.has_async(uri) {
            return either::Right(either::Left(async move {
                match v.await {
                    Ok(f) => then(f).await,
                    Err(e) => Err(e),
                }
            }));
        }
        // TODO: a.document_file blocks; avoid!
        let file_or_other = match self.with_archive(uri.archive_id(), |a| {
            let Some(a) = a else {
                return Err(BackendError::ArchiveNotFound);
            };
            match a {
                Archive::Local(a) => Ok(either::Left(a.document_file(
                    uri.path(),
                    None,
                    &uri.name,
                    uri.language(),
                ))),
                Archive::Ext(_, ext) => Ok(either::Right(other(&**ext))),
            }
        }) {
            Ok(v) => v,
            Err(e) => return either::Left(std::future::ready(Err(e))),
        };
        match file_or_other {
            either::Left(file) => {
                let docfile = self.documents.get(uri.clone(), |_| {
                    A::block_on(move || {
                        DocumentFile::from_file(file)
                            .map(triomphe::Arc::new)
                            .map_err(Into::into)
                    })
                });
                either::Right(either::Right(either::Left(async move {
                    let docfile = docfile.await?;
                    then(docfile).await
                })))
            }
            either::Right(r) => either::Right(either::Right(either::Right(r))),
        }
    }

    #[cfg(feature = "rdf")]
    fn do_notations(
        &self,
        iri: ulo::rdf_types::NamedNode,
    ) -> impl Iterator<Item = (DocumentElementUri, Notation)> {
        let q = crate::sparql!(SELECT DISTINCT ?n WHERE { ?n ulo:notation_for iri. });
        self.triple_store()
            .query(q)
            .expect("Notations query should be valid")
            .into_uris::<DocumentElementUri>()
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|uri| {
                use ftml_ontology::narrative::elements::notations::NotationReference;
                //tracing::warn!("Found {uri}");
                let notation = self
                    .get_typed_document_element::<NotationReference>(&uri)
                    .ok()?;
                //tracing::warn!("Found {notation:?}");
                self.get_reference(&notation.notation.with_doc(uri.document.clone()))
                    .map_err(|e| tracing::error!("Error getting notation {uri}: {e}"))
                    .ok()
                    .map(|n| (uri, n))
            })
    }

    #[cfg(feature = "rdf")]
    fn do_var_notations(
        &self,
        iri: ulo::rdf_types::NamedNode,
    ) -> impl Iterator<Item = (DocumentElementUri, Notation)> {
        let q = crate::sparql!(SELECT DISTINCT ?n WHERE { ?n ulo:notation_for iri. });
        self.triple_store()
            .query(q)
            .expect("Notations query should be valid")
            .into_uris::<DocumentElementUri>()
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|uri| {
                use ftml_ontology::narrative::elements::notations::VariableNotationReference;
                //tracing::warn!("Found {uri}");
                let notation = self
                    .get_typed_document_element::<VariableNotationReference>(&uri)
                    .ok()?;
                //tracing::warn!("Found {notation:?}");
                self.get_reference(&notation.notation.with_doc(uri.document.clone()))
                    .map_err(|e| tracing::error!("Error getting variable notation {uri}: {e}"))
                    .ok()
                    .map(|n| (uri, n))
            })
    }
}
