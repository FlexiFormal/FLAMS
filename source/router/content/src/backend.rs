use ftml_uris::{FtmlUri, Uri};

pub struct FtmlBackend;
impl ftml_backend::GlobalBackend for FtmlBackend {
    type Error = <Self::Backend as ftml_backend::FtmlBackend>::Error; // TODO
    #[cfg(feature = "ssr")]
    type Backend = Self;
    #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
    type Backend = ftml_backend::CachedBackend<Self>;
    #[inline]
    fn get() -> &'static Self::Backend {
        #[cfg(feature = "ssr")]
        static SLF: FtmlBackend = FtmlBackend;
        #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
        static SLF: std::sync::LazyLock<ftml_backend::CachedBackend<FtmlBackend>> =
            std::sync::LazyLock::new(|| ftml_backend::CachedBackend::new(FtmlBackend));
        &SLF
    }
}
impl ftml_backend::FlamsBackend for FtmlBackend {
    fn document_link_url(&self, uri: &ftml_uris::DocumentUri) -> String {
        format!("/?uri={}", uri.url_encoded())
    }
    fn resource_link_url(
        &self,
        uri: &ftml_uris::DocumentUri,
        kind: &'static str,
    ) -> Option<String> {
        Some(format!("/doc?uri={}&format={kind}", uri.url_encoded()))
    }
    fn get_fragment(
        &self,
        uri: Option<ftml_uris::Uri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        m: Option<String>,
        l: Option<ftml_uris::Language>,
        e: Option<String>,
        s: Option<String>,
        context: Option<ftml_uris::NarrativeUri>,
    ) -> impl Future<
        Output = Result<
            (ftml_uris::Uri, Box<[ftml_ontology::utils::Css]>, Box<str>),
            ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
        >,
    > {
        super::server_fns::fragment(uri, rp, a, p, d, m, l, e, s, context)
    }

    fn get_module(
        &self,
        uri: Option<ftml_uris::ModuleUri>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        m: Option<String>,
    ) -> impl Future<
        Output = Result<
            ftml_ontology::domain::modules::Module,
            ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
        >,
    > + Send {
        super::server_fns::get_module(uri, a, p, m)
    }

    fn get_document(
        &self,
        uri: Option<ftml_uris::DocumentUri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        l: Option<ftml_uris::Language>,
    ) -> impl Future<
        Output = Result<
            ftml_ontology::narrative::documents::Document,
            ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
        >,
    > {
        super::server_fns::get_document(uri, rp, a, p, d, l)
    }

    fn get_notations(
        &self,
        uri: Option<Uri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        m: Option<String>,
        l: Option<ftml_uris::Language>,
        e: Option<String>,
        s: Option<String>,
    ) -> impl Future<
        Output = Result<
            Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::Notation,
            )>,
            ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
        >,
    > + Send {
        super::server_fns::notations(uri, rp, a, p, d, m, l, e, s)
    }
    fn get_logical_paragraphs(
        &self,
        uri: Option<ftml_uris::SymbolUri>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        m: Option<String>,
        s: Option<String>,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::ParagraphOrProblemKind,
            )>,
            ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
        >,
    > + Send {
        super::server_fns::los(uri, a, p, m, s, problems)
    }
}
