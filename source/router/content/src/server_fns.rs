use ftml_ontology::{
    narrative::{
        documents::TocElem,
        elements::{
            Notation, ParagraphOrProblemKind, SlideElement,
            problems::{ProblemFeedbackJson, ProblemResponse, SolutionData, quizzes::Quiz},
        },
    },
    utils::Css,
};
use ftml_uris::{
    ArchiveId, DocumentElementUri, DocumentUri, FtmlUri, IsDomainUri, IsNarrativeUri, Language,
    NarrativeUri, PathUri, SimpleUriName, SymbolUri, Uri, UriName, UriPath, UriWithArchive,
    UriWithPath,
};
use leptos::prelude::*;
use std::str::FromStr;

#[cfg(feature = "ssr")]
use ftml_uris::components::{DocumentUriComponents, UriComponents};

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="document",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    pub async fn document(
        uri: DocumentUri
    ) -> Result<(DocumentUri, Box<[Css]>, Box<str>),
        ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>
    > {
        let uri = uri?.parse(flams_router_base::uris::get_uri)?;
        server::document(uri).await

        /*
        let Result::<DocumentUriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };

        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::document(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
         */
    }
}

#[server(
  prefix="/content",
  endpoint="document_of",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
pub async fn document_of(uri: Uri) -> Result<DocumentUri, ServerFnError<String>> {
    use flams_math_archives::backend::LocalBackend;
    tokio::task::spawn_blocking(move || {
        let m = match uri {
            Uri::Base(_) | Uri::Archive(_) | Uri::Path(_) => {
                return Err("not in a document".to_string().into());
            }
            Uri::Document(d) => return Ok(d),
            Uri::DocumentElement(d) => return Ok(d.document_uri().clone()),
            Uri::Module(ref m) => m,
            Uri::Symbol(ref s) => s.module_uri(),
        };
        flams_math_archives::backend::GlobalBackend.with_local_archive(m.archive_id(), |o| {
            let Some(archive) = o else {
                return Err(format!("no local archive {} found", m.archive_id()).into());
            };
            let mut mname = m.module_name().first();
            let mut file = archive.source_dir();
            let maybe_step = if let Some(path) = m.path() {
                let mut steps = path.steps();
                let _ = steps.next_back();
                for step in steps {
                    file = file.join(step);
                }
                path.steps().next_back()
            } else {
                None
            };
            if let Some(step) = maybe_step {
                if let Ok(mut d) = std::fs::read_dir(file.join(step)) {
                    if let Some(rp) =
                        d.find_map::<String, _>(|p| {
                            p.ok().and_then(|p| {
                                let fnm = p.file_name();
                                let name = fnm.as_os_str().as_encoded_bytes();
                                let Some(name) = name.strip_prefix(mname.as_bytes()) else {
                                    return None;
                                };
                                let Some(name) = name.strip_prefix(&[b'.']) else {
                                    return None;
                                };
                                let Some(lang) = name.strip_suffix(b".tex") else {
                                    return None;
                                };
                                if Language::from_str(std::str::from_utf8(lang).ok()?).is_ok() {
                                    Some(
                                        p.path().as_os_str().to_str()?.strip_prefix(
                                            archive.source_dir().as_os_str().to_str()?,
                                        )?[1..]
                                            .to_string(),
                                    )
                                } else {
                                    None
                                }
                            })
                        })
                    {
                        return DocumentUri::from_archive_relpath(m.archive_uri().clone(), &rp)
                            .map_err(|e| e.to_string().into());
                    }
                    mname = step;
                };
            }
            if let Ok(mut d) = std::fs::read_dir(file) {
                if let Some(rp) = d.find_map::<String, _>(|p| {
                    p.ok().and_then(|p| {
                        let fnm = p.file_name();
                        let name = fnm.as_os_str().as_encoded_bytes();
                        let Some(name) = name.strip_prefix(mname.as_bytes()) else {
                            return None;
                        };
                        let Some(name) = name.strip_prefix(&[b'.']) else {
                            return None;
                        };
                        let Some(lang) = name.strip_suffix(b".tex") else {
                            return None;
                        };
                        if Language::from_str(std::str::from_utf8(lang).ok()?).is_ok() {
                            Some(
                                p.path()
                                    .as_os_str()
                                    .to_str()?
                                    .strip_prefix(archive.source_dir().as_os_str().to_str()?)?[1..]
                                    .to_string(),
                            )
                        } else {
                            None
                        }
                    })
                }) {
                    return DocumentUri::from_archive_relpath(m.archive_uri().clone(), &rp)
                        .map_err(|e| e.to_string().into());
                }
            };
            Err("Not found".to_string().into())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="toc",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    pub async fn toc(
        uri: DocumentUri
    ) -> Result<(Box<[Css]>, Box<[TocElem]>), ServerFnError<String>> {
        let Result::<DocumentUriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };

        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::toc(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
}

#[server(
prefix="/domain",
endpoint="module",
input=server_fn::codec::GetUrl,
output=server_fn::codec::Json
)]
pub async fn get_module(
    uri: Option<ftml_uris::ModuleUri>,
    a: Option<ftml_uris::ArchiveId>,
    p: Option<String>,
    m: Option<String>,
) -> Result<
    ftml_ontology::domain::modules::Module,
    ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
> {
    use flams_math_archives::backend::LocalBackend;
    use flams_system::TokioEngine;
    let Some(uri) = uri.or_else(|| {
        let a = flams_router_base::uris::get_uri(&a?)?;
        let p: PathUri = if let Some(p) = p {
            a / p.parse::<UriPath>().ok()?
        } else {
            a.into()
        };
        Some(p | m?.parse().ok()?)
    }) else {
        return Err(ftml_backend::BackendError::NotFound(
            ftml_uris::UriKind::Archive,
        ));
    };
    flams_system::backend::backend()
        .get_module_async::<TokioEngine>(&uri)
        .await
        .map_err(|_| ftml_backend::BackendError::NotFound(ftml_uris::UriKind::Module))
}

ftml_uris::compfun! {
    #[server(
    prefix="/domain",
    endpoint="document",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn get_document(uri:DocumentUri) -> Result<
        ftml_ontology::narrative::documents::Document,
        ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
    > {
        use flams_math_archives::backend::LocalBackend;
        use flams_system::TokioEngine;
        // TODO this actually already returns proper errors
        let comps = uri?;
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => flams_system::backend::backend().get_document_async::<TokioEngine>(&uri).await.map_err(|e| ftml_backend::BackendError::ToDo(e.to_string())),
            Err(e) => Err(ftml_backend::BackendError::NotFound(ftml_uris::UriKind::Document)),
        }
    }
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="fragment",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn fragment(uri:Uri,
        context: Option<NarrativeUri>
    ) -> Result<(Uri, Box<[Css]>, Box<str>),ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>> {
        // TODO this actually already returns proper errors
        let comps = uri?;
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::fragment(uri, context).await.map_err(|e| ftml_backend::BackendError::ToDo(e.to_string())),
            Err(e) => Err(ftml_backend::BackendError::NotFound(ftml_uris::UriKind::Archive)),
        }
    }
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="los",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn los(
        uri: SymbolUri,
        problems: bool
    ) -> Result<Vec<(DocumentElementUri, ParagraphOrProblemKind)>, ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>> {
        let uri = uri?.parse(flams_router_base::uris::get_uri)?;
        server::los(uri, problems).await.map_err(|e| ftml_backend::BackendError::ToDo(e.to_string()))
        /*let Result::<SymbolUriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::los(uri, problems).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }*/
    }
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="notations",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn notations(
        uri: Uri
    ) -> Result<Vec<(DocumentElementUri, Notation)>, ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>> {
        let uri = uri?.parse(flams_router_base::uris::get_uri)?;
        server::notations(uri).await.map_err(|e| ftml_backend::BackendError::ToDo(e.to_string()))
        /*let Result::<UriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::notations(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }*/
    }
}
/*
ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="omdoc",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn omdoc(
        uri: Uri
    ) -> Result<(Vec<Css>, OMDoc), ServerFnError<String>> {
        let Result::<UriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::omdoc(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
} */

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="title",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn title(
        uri: Uri
    ) -> Result<(Box<[Css]>, Box<str>), ServerFnError<String>> {
        let Result::<UriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::title(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="quiz",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn get_quiz(
        uri: DocumentUri
    ) -> Result<Quiz, ServerFnError<String>> {
        let Result::<DocumentUriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::get_quiz(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
}

#[server(prefix = "/content", endpoint = "grade_enc",
    input=server_fn::codec::Json,
    output=server_fn::codec::Json
)]
pub async fn grade_enc(
    submissions: Vec<(String, Vec<Option<ProblemResponse>>)>,
) -> Result<Vec<Vec<ProblemFeedbackJson>>, ServerFnError<String>> {
    tokio::task::spawn_blocking(move || {
        let mut ret = Vec::new();
        for (sol, resps) in submissions {
            let mut ri = Vec::new();
            let sol = ftml_ontology::narrative::elements::problems::Solutions::from_jstring(&sol)
                .ok_or_else(|| format!("Invalid solution string: {sol}"))?;
            for resp in resps {
                let r = if let Some(resp) = resp {
                    sol.check_response(&resp).ok_or_else(|| {
                        "Response {resp:?} does not match solution {sol:?}".to_string()
                    })?
                } else {
                    sol.default_feedback()
                };
                ri.push(r.to_json());
            }
            ret.push(ri)
        }
        Ok(ret)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[server(prefix = "/content", endpoint = "grade",
    input=server_fn::codec::Json,
    output=server_fn::codec::Json
)]
pub async fn grade(
    submissions: Vec<(Box<[SolutionData]>, Vec<Option<ProblemResponse>>)>,
) -> Result<Vec<Vec<ProblemFeedbackJson>>, ServerFnError<String>> {
    tokio::task::spawn_blocking(move || {
        let mut ret = Vec::new();
        for (sol, resps) in submissions {
            let mut ri = Vec::new();
            let sol = ftml_ontology::narrative::elements::problems::Solutions::from_solutions(sol);
            for resp in resps {
                let r = if let Some(resp) = resp {
                    sol.check_response(&resp).ok_or_else(|| {
                        "Response {resp:?} does not match solution {sol:?}".to_string()
                    })?
                } else {
                    sol.default_feedback()
                };
                ri.push(r.to_json());
            }
            ret.push(ri)
        }
        Ok(ret)
    })
    .await
    .map_err(|e| e.to_string())?
}

ftml_uris::compfun! {
    #[server(prefix = "/content", endpoint = "solution",
        input=server_fn::codec::GetUrl
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn solution(
        uri: Uri
    ) -> Result<String, ServerFnError<String>> {
        use ftml_uris::NarrativeUri;
        use ftml_ontology::utils::Hexable;
        use flams_web_utils::blocking_server_fn;
        let Result::<UriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(Uri::DocumentElement(uri)) => {
                let s = server::get_solution(&uri).await?;
                s.to_jstring().ok_or_else(|| "invalid solution".to_string().into())
            },
            Ok(u) => Err(format!("Invalid document element uri: {u}").into()),
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
}

ftml_uris::compfun! {
    #[server(
    prefix="/content",
    endpoint="slides",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
    )]
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub async fn slides_view(
        uri: Uri
    ) -> Result<(Box<[Css]>, Box<[SlideElement]>), ServerFnError<String>> {
        let Result::<UriComponents, _>::Ok(comps) = uri else {
            return Err("invalid uri components".to_string().into());
        };
        match comps.parse(flams_router_base::uris::get_uri) {
            Ok(uri) => server::slides(uri).await,
            Err(e) => Err(format!("Invalid uri: {e}").into()),
        }
    }
}

#[cfg(feature = "ssr")]
mod server {
    use crate::ssr::insert_base_url;
    use flams_math_archives::backend::{GlobalBackend, LocalBackend};
    use flams_system::{TokioEngine, backend::backend};
    use flams_utils::{unwrap, vecmap::VecSet};
    use flams_web_utils::{blocking_server_fn, not_found};
    use ftml_backend::BackendError;
    use ftml_ontology::{
        narrative::{
            Narrative,
            documents::TocElem,
            elements::{
                DocumentElement, LogicalParagraph, Notation, ParagraphOrProblemKind, Problem,
                Section, SlideElement,
                problems::{ProblemData, Solutions, quizzes::Quiz},
            },
        },
        utils::Css,
    };
    use ftml_uris::{
        DocumentElementUri, DocumentUri, FtmlUri, IsNarrativeUri, NarrativeUri, SymbolUri, Uri,
    };
    use leptos::prelude::*;

    pub async fn document(
        uri: DocumentUri,
    ) -> Result<
        (DocumentUri, Box<[Css]>, Box<str>),
        ftml_backend::BackendError<leptos::server_fn::error::ServerFnErrorErr>,
    > {
        let (css, doc) = backend()
            .get_html_body_async::<TokioEngine>(&uri)
            .await
            .map_err(|e| ftml_backend::BackendError::ToDo(e.to_string()))?;
        let html = format!(
            "<div{}</div>",
            doc.strip_prefix("<body")
                .and_then(|s| s.strip_suffix("</body>"))
                .unwrap_or("")
        );
        Ok((uri, insert_base_url(css), html.into_boxed_str()))
    }

    pub async fn toc(
        uri: DocumentUri,
    ) -> Result<(Box<[Css]>, Box<[TocElem]>), ServerFnError<String>> {
        let doc = backend()
            .get_document_async::<TokioEngine>(&uri)
            .await
            .map_err(|e| e.to_string())?;
        Ok(crate::toc::from_document(doc).await)
    }

    pub async fn fragment(
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> Result<(Uri, Box<[Css]>, Box<str>), BackendError<ServerFnErrorErr>> {
        use ftml_uris::UriKind;
        match &uri {
            Uri::Document(duri) => {
                let Ok((css, html)) = backend()
                    .get_html_body_inner_async::<TokioEngine>(duri)
                    .await
                else {
                    not_found!();
                    return Err(BackendError::NotFound(UriKind::Document));
                };
                Ok((uri, insert_base_url(filter_paras(css)), html))
            }
            Uri::DocumentElement(euri) => {
                let Ok(e) = backend()
                    .get_document_element_async::<TokioEngine>(euri)
                    .await
                else {
                    not_found!();
                    return Err(BackendError::NotFound(UriKind::DocumentElement));
                };
                match &*e {
                    DocumentElement::Paragraph(LogicalParagraph { range, .. })
                    | DocumentElement::Problem(Problem { range, .. }) => {
                        let Ok((css, html)) = backend()
                            .get_html_fragment_async::<TokioEngine>(euri.document_uri(), *range)
                            .await
                        else {
                            not_found!();
                            return Err(BackendError::HtmlNotFound);
                        };
                        Ok((uri, insert_base_url(filter_paras(css)), html))
                    }
                    DocumentElement::Section(Section { range, .. }) => {
                        let Ok((css, html)) = backend()
                            .get_html_fragment_async::<TokioEngine>(euri.document_uri(), *range)
                            .await
                        else {
                            not_found!();
                            return Err(BackendError::HtmlNotFound);
                        };
                        Ok((uri, insert_base_url(filter_paras(css)), html))
                    }
                    _ => return Err(BackendError::NoFragment),
                }
            }
            Uri::Symbol(suri) => get_definitions(suri.clone(), context)
                .await
                .ok_or_else(|| {
                    not_found!();
                    BackendError::NoDefinition
                })
                .map(|(css, b)| (uri, insert_base_url(filter_paras(css)), b)),
            Uri::Base(_) => Err(BackendError::ToDo("base uri".to_string())),
            Uri::Archive(_) => Err(BackendError::ToDo("archive uri".to_string())),
            Uri::Path(_) => Err(BackendError::ToDo("path uri".to_string())),
            Uri::Module(_) => Err(BackendError::ToDo("module uri".to_string())),
        }
    }

    pub async fn los(
        uri: SymbolUri,
        problems: bool,
    ) -> Result<Vec<(DocumentElementUri, ParagraphOrProblemKind)>, ServerFnError<String>> {
        blocking_server_fn(move || {
            Ok(GlobalBackend
                .triple_store()
                .los(&uri, problems)
                .map(|i| i.collect())
                .unwrap_or_default())
        })
        .await
    }

    pub async fn notations(
        uri: Uri,
    ) -> Result<Vec<(DocumentElementUri, Notation)>, ServerFnError<String>> {
        let v = match uri {
            Uri::Symbol(uri) => {
                blocking_server_fn(move || Ok(backend().get_notations(&uri).collect::<Vec<_>>()))
                    .await
            }
            Uri::DocumentElement(uri) => {
                blocking_server_fn(move || {
                    Ok(backend().get_var_notations(&uri).collect::<Vec<_>>())
                })
                .await
            }
            _ => return Err(format!("Not a symbol or variable URI: {uri}").into()),
        }?;
        Ok(v)
    }

    pub async fn title(uri: Uri) -> Result<(Box<[Css]>, Box<str>), ServerFnError<String>> {
        match uri {
            uri @ (Uri::Base(_)
            | Uri::Archive(_)
            | Uri::Path(_)
            | Uri::Module(_)
            | Uri::Symbol(_)) => {
                Err(format!("Not a URI of an element that can have a title: {uri}").into())
            }
            Uri::Document(uri) => {
                let Ok(doc) = backend().get_document_async::<TokioEngine>(&uri).await else {
                    not_found!("Document {uri} not found");
                };
                Ok((
                    Vec::new().into_boxed_slice(),
                    doc.title.clone().unwrap_or_default(),
                ))
            }
            Uri::DocumentElement(uri) => {
                let Ok(e) = backend()
                    .get_document_element_async::<TokioEngine>(&uri)
                    .await
                else {
                    not_found!("Document Element {uri} not found");
                };
                match &*e {
                    DocumentElement::Section(Section { title, .. })
                    | DocumentElement::Paragraph(LogicalParagraph { title, .. }) => {
                        let Some(title) = title else {
                            return Ok((
                                Vec::new().into_boxed_slice(),
                                String::new().into_boxed_str(),
                            ));
                        };
                        return Ok((Vec::new().into_boxed_slice(), title.clone()));
                        // TODO get CSS
                        /*
                        backend()
                            .get_html_fragment_async(uri.document_uri(), *title)
                            .await
                            .ok_or_else(|| format!("Error retrieving title").into())
                             */
                    }
                    DocumentElement::Problem(Problem { data, .. }) => Ok((
                        Vec::new().into_boxed_slice(),
                        data.title.clone().unwrap_or_default(),
                    )),
                    _ => Err(format!("Narrative element has no title").into()),
                }
            }
        }
    }

    /*
    pub async fn omdoc(uri: Uri) -> Result<(Vec<Css>, OMDoc), ServerFnError<String>> {
        let mut css = VecSet::default();
        match uri {
            uri @ (Uri::Base(_) | Uri::Archive(_) | Uri::Path(_)) => {
                Ok((insert_base_url(css.0), OMDoc::Other(uri.to_string())))
            }
            Uri::Document(uri) => {
                let Some(doc) = backend!(get_document!(&uri)) else {
                    not_found!("Document {uri} not found");
                };
                let (css, r) = backend!(backend => {
                  let r = OMDocDocument::from_document(&doc, backend,&mut css);
                  (css,r)
                }{
                  blocking_server_fn(move || {
                    let r = OMDocDocument::from_document(&doc, backend,&mut css);
                    Ok((css,r))
                  }).await?
                });
                Ok((insert_base_url(css.0), r.into()))
            }
            Uri::DocumentElement(uri) => {
                let Some(e): Option<NarrativeReference<DocumentElement<Checked>>> =
                    backend!(get_document_element!(&uri))
                else {
                    not_found!("Document Element {uri} not found");
                };
                let (css, r) = backend!(backend => {
                  let r = OMDocDocumentElement::from_element(e.as_ref(),backend, &mut css);
                  (css,r)
                }{
                  blocking_server_fn(move || {
                    let r = OMDocDocumentElement::from_element(e.as_ref(),backend,&mut css);
                    Ok((css,r))
                  }).await?
                });
                let Some(r) = r else {
                    not_found!("Document Element {uri} not found");
                };
                Ok((insert_base_url(css.0), r.into()))
            }
            Uri::Module(uri) => {
                let Some(m) = backend!(get_module!(&uri)) else {
                    not_found!("Module {uri} not found");
                };
                let r = backend!(backend => {
                  OMDoc::from_module_like(&m, backend)
                }{
                  blocking_server_fn(move || {
                    Ok(OMDoc::from_module_like(&m, backend))
                  }).await?
                });
                Ok((Vec::new(), r))
            }
            Uri::Symbol(uri) => {
                let Some(s): Option<ContentReference<Declaration>> =
                    backend!(get_declaration!(&uri))
                else {
                    not_found!("Declaration {uri} not found");
                };
                return Err(format!("TODO: {uri}").into());
            }
        }
    }
    */

    pub async fn get_quiz(uri: DocumentUri) -> Result<Quiz, ServerFnError<String>> {
        let Ok(doc) = backend().get_document_async::<TokioEngine>(&uri).await else {
            not_found!("Document {uri} not found");
        };
        blocking_server_fn(move || {
            let be = doc.as_quiz(
                &|d| backend().get_document(d).ok(),
                &|d, r| backend().get_html_fragment(d, r).ok(),
                &|d, r| backend().get_reference(&r.with_doc(d.clone())).ok(),
                &|d, r| backend().get_reference(&r.with_doc(d.clone())).ok(),
            );
            let mut be = be.map_err(|e| format!("{e:#}"))?;
            be.css = insert_base_url(std::mem::take(&mut be.css));
            Ok(be)
        })
        .await
    }

    pub async fn slides(
        uri: Uri,
    ) -> Result<(Box<[Css]>, Box<[SlideElement]>), ServerFnError<String>> {
        fn from_children(
            top: &DocumentUri,
            children: &[DocumentElement],
            css: &mut VecSet<Css>,
            backend: &impl LocalBackend,
        ) -> Result<Vec<SlideElement>, String> {
            let mut stack =
                smallvec::SmallVec::<(_, _, _, Option<DocumentElementUri>), 2>::default();
            let mut ret = Vec::new();
            let mut curr = children.iter();

            loop {
                let Some(next) = curr.next() else {
                    if let Some((a, b, c, u)) = stack.pop() {
                        curr = a;
                        if let Some(mut b) = b {
                            std::mem::swap(&mut ret, &mut b);
                            ret.push(SlideElement::Section {
                                title: c,
                                children: b,
                                uri: unwrap!(u),
                            });
                        }
                        continue;
                    }
                    break;
                };
                match next {
                    DocumentElement::Slide { range, uri, .. } => {
                        let Ok((c, html)) = backend.get_html_fragment(top, *range) else {
                            return Err(format!("Missing fragment for slide {uri}"));
                        };
                        for c in c {
                            css.insert(c);
                        }
                        ret.push(SlideElement::Slide {
                            html,
                            uri: uri.clone(),
                        });
                    }
                    DocumentElement::Paragraph(p) => {
                        let Ok((c, html)) = backend.get_html_fragment(top, p.range) else {
                            return Err(format!("Missing fragment for paragraph {}", p.uri));
                        };
                        for c in c {
                            css.insert(c);
                        }
                        ret.push(SlideElement::Paragraph {
                            html,
                            uri: p.uri.clone(),
                        });
                    }
                    DocumentElement::DocumentReference { target, .. } => {
                        ret.push(SlideElement::Inputref {
                            uri: target.clone(),
                        })
                    }
                    e @ DocumentElement::Section(s) => {
                        let title = s.title.clone();
                        stack.push((
                            std::mem::replace(&mut curr, e.children_lt().unwrap_or(&[]).iter()),
                            Some(std::mem::replace(&mut ret, Vec::new())),
                            title,
                            Some(s.uri.clone()),
                        ));
                    }
                    o => {
                        let chs = o.children_lt().unwrap_or(&[]);
                        if !chs.is_empty() {
                            stack.push((
                                std::mem::replace(&mut curr, chs.iter()),
                                None,
                                None,
                                None,
                            ));
                        }
                    }
                }
            }
            Ok(ret)
        }

        let Ok(doe) = (match &uri {
            Uri::Document(uri) => backend()
                .get_document_async::<TokioEngine>(uri)
                .await
                .map(either::Either::Left),
            Uri::DocumentElement(uri) => backend()
                .get_document_element_async::<TokioEngine>(uri)
                .await
                .map(either::Either::Right),
            _ => return Err("Not a narrative URI".to_string().into()),
        }) else {
            not_found!("Element {uri} not found");
        };
        blocking_server_fn(move || {
            let (chs, top) = match &doe {
                either::Either::Left(d) => (&*d.elements, &d.uri),
                either::Either::Right(e) => {
                    let e: &DocumentElement = e;
                    (
                        e.children_lt().unwrap_or(&[]),
                        e.element_uri().expect("has a uri").document_uri(),
                    )
                }
            };
            let mut css = VecSet::default();
            let r = from_children(top, chs, &mut css, backend())?.into_boxed_slice();
            Ok((insert_base_url(css.0.into_boxed_slice()), r))
        })
        .await
    }

    pub async fn get_solution(uri: &DocumentElementUri) -> Result<Solutions, String> {
        use flams_math_archives::backend::LocalBackend;
        match backend()
            .get_typed_document_element_async::<TokioEngine, _>(&uri)
            .await
        {
            Ok(rf) => {
                let sol = match blocking_server_fn(move || {
                    let e: &Problem = &*rf;
                    backend()
                        .get_reference(&rf.data.solutions.with_doc(e.uri.document_uri().clone()))
                        .map_err(|e| e.to_string())
                })
                .await
                {
                    Ok(sol) => sol,
                    Err(e) => return Err(format!("solutions not found: {e}")),
                };
                Ok(sol)
            }
            _ => not_found!("Problem {uri} not found"),
        }
    }

    async fn get_definitions(
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> Option<(Box<[Css]>, Box<str>)> {
        fn iter(
            uri: &SymbolUri,
            context: Option<NarrativeUri>,
        ) -> impl Iterator<Item = DocumentElementUri> {
            use flams_math_archives::triple_store::sparql::QueryResult;
            let iri = uri.to_iri();
            let i = iri.clone();
            let base = GlobalBackend
                .triple_store()
                .query(
                    flams_math_archives::sparql!(SELECT DISTINCT ?x WHERE {
                        ?x ulo:defines i.
                    })
                    .into(),
                )
                .map(QueryResult::into_uris)
                .unwrap_or_default();
            match context {
                None => either::Left(base),
                Some(ctx) => {
                    let lang = ctx.language();
                    let language = format!(
                        "SELECT DISTINCT ?x WHERE {{ ?x ulo:defines <{}>. ?d (ulo:contains|dc:hasPart)* ?x. ?d dc:language \"{}\". }}",
                        iri.as_str(),
                        lang
                    );
                    either::Right(
                        ctx.ancestors()
                            .flat_map(move |uri| {
                                let query = if matches!(uri,Uri::Document(_)|Uri::DocumentElement(_)) {
                                    format!(
                                        "SELECT DISTINCT ?a WHERE {{ <{}> (ulo:contains|dc:hasPart)* ?x. ?x ulo:defines <{}>. }}",
                                        uri.to_iri().as_str(),
                                        iri.as_str()
                                    )
                                } else {
                                    format!(
                                        "SELECT DISTINCT ?a WHERE {{ <{}> (ulo:contains|dc:hasPart)* ?x. ?x ulo:defines <{}>. ?d (ulo:contains|dc:hasPart)* ?x. ?d dc:language \"{}\" }}",
                                        uri.to_iri().as_str(),
                                        iri.as_str(),
                                        lang
                                    )
                                };
                                GlobalBackend
                                    .triple_store()
                                    .query_str(query)
                                    .map(QueryResult::into_uris)
                                    .unwrap_or_default()
                            })
                            .chain(
                                GlobalBackend
                                    .triple_store()
                                    .query_str(language)
                                    .map_err(|e| {
                                        println!("Error: {e}");
                                        e
                                    })
                                    .map(QueryResult::into_uris)
                                    .unwrap_or_default()
                            )
                            .chain(base),
                    )
                }
            }
        }
        tokio::task::spawn_blocking(move || {
            for uri in iter(&uri, context) {
                if let Ok(def) = backend().get_typed_document_element(&uri) {
                    let LogicalParagraph { range, .. } = &*def;
                    if let Ok((css, r)) = backend().get_html_fragment(uri.document_uri(), *range) {
                        return Some((insert_base_url(filter_paras(css)), r));
                    }
                }
            }
            None
        })
        .await
        .ok()
        .flatten()
    }

    pub(crate) fn filter_paras(v: Box<[Css]>) -> Box<[Css]> {
        const CSSS: [&str; 11] = [
            "ftml-part",
            "ftml-chapter",
            "ftml-section",
            "ftml-subsection",
            "ftml-subsubsection",
            "ftml-paragraph",
            "ftml-definition",
            "ftml-assertion",
            "ftml-example",
            "ftml-problem",
            "ftml-subproblem",
        ];
        let mut v = v.into_vec();
        v.retain(|c| match c {
            Css::Class { name, .. } => !CSSS.iter().any(|s| name.starts_with(s)),
            _ => true,
        });
        v.into_boxed_slice()
    }
}

#[server(prefix = "/content/legacy", endpoint = "uris")]
pub async fn uris(uris: Vec<String>) -> Result<Vec<Option<Uri>>, ServerFnError<String>> {
    use flams_math_archives::{
        MathArchive,
        backend::{GlobalBackend, LocalBackend},
    };
    use ftml_uris::{ArchiveUri, BaseUri, ModuleUri};

    const MATHHUB: &str = "http://mathhub.info";
    const META: &str = "http://mathhub.info/sTeX/meta";
    const URTHEORIES: &str = "http://cds.omdoc.org/urtheories";

    macro_rules! cnst {
        ($($name:ident:$tp:ty = $e:expr;)*) => {
            $( static $name: std::sync::LazyLock<$tp> = std::sync::LazyLock::new(|| $e); )*
        }
    }

    cnst! {
      MATHHUB_INFO: BaseUri = BaseUri::from_str("http://mathhub.info/:sTeX").expect("is valid");
      META_URI: ArchiveUri = ftml_uris::metatheory::URI.archive_uri().clone();//ArchiveUri::new(MATHHUB_INFO.clone(),ArchiveId::new("sTeX/meta-inf"));
      UR_URI: ArchiveUri = BaseUri::from_str("http://cds.omdoc.org").expect("is valid") & ArchiveId::new("MMT/urtheories").expect("is valid");
      MY_ARCHIVE: ArchiveUri = BaseUri::from_str("http://mathhub.info").expect("is valid") & ArchiveId::new("my/archive").expect("is valid");
      INJECTING: ArchiveUri = MATHHUB_INFO.clone() & ArchiveId::new("Papers/22-CICM-Injecting-Formal-Mathematics").expect("is valid");
      TUG: ArchiveUri = MATHHUB_INFO.clone() & ArchiveId::new("Papers/22-TUG-sTeX").expect("is valid");
    }

    fn split(p: &str) -> Option<(ArchiveUri, usize)> {
        if p.starts_with(META) {
            return Some((META_URI.clone(), 29));
        }
        if p == URTHEORIES {
            return Some((UR_URI.clone(), 31));
        }
        if p == "http://mathhub.info/my/archive" {
            return Some((MY_ARCHIVE.clone(), 30));
        }
        if p == "http://kwarc.info/Papers/stex-mmt/paper" {
            return Some((INJECTING.clone(), 34));
        }
        if p == "http://kwarc.info/Papers/tug/paper" {
            return Some((TUG.clone(), 34));
        }
        if p.starts_with("file://") {
            return Some((ArchiveUri::no_archive(), 7));
        }
        if let Some(mut p) = p.strip_prefix(MATHHUB) {
            let mut i = MATHHUB.len();
            if let Some(s) = p.strip_prefix('/') {
                p = s;
                i += 1;
            }
            return split_old(p, i);
        }
        GlobalBackend.with_archives(|tree| {
            tree.iter().find_map(|a| {
                let base = a.uri();
                let base = base.base().as_str();
                if p.starts_with(base) {
                    let l = base.len();
                    let np = &p[l..];
                    let id = a.id().as_ref();
                    if np.starts_with(id) {
                        Some((a.uri().clone(), l + id.len()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    fn split_old(p: &str, len: usize) -> Option<(ArchiveUri, usize)> {
        GlobalBackend.with_archives(|tree| {
            tree.iter().find_map(|a| {
                if p.starts_with(a.id().as_ref()) {
                    let mut l = a.id().as_ref().len();
                    let np = &p[l..];
                    if np.starts_with('/') {
                        l += 1;
                    }
                    Some((a.uri().clone(), len + l))
                } else {
                    None
                }
            })
        })
    }

    fn get_doc_uri(pathstr: &str) -> Option<DocumentUri> {
        let pathstr = pathstr.strip_suffix(".tex").unwrap_or(pathstr);
        let (p, mut m) = pathstr.rsplit_once('/')?;
        let (a, l) = split(p)?;
        let mut path = if l < p.len() { &p[l..] } else { "" };
        if path.starts_with('/') {
            path = &path[1..];
        }
        let lang = Language::from_rel_path(m);
        m = m.strip_suffix(&format!(".{lang}")).unwrap_or(m);
        Some((a / path.parse::<UriPath>().ok()?) & (m.parse::<SimpleUriName>().ok()?, lang))
    }

    fn get_mod_uri(pathstr: &str) -> Option<ModuleUri> {
        let (mut p, mut m) = pathstr.rsplit_once('?')?;
        m = m.strip_suffix("-module").unwrap_or(m);
        if p.bytes().last() == Some(b'/') {
            p = &p[..p.len() - 1];
        }
        let (a, l) = split(p)?;
        let mut path = if l < p.len() { &p[l..] } else { "" };
        if path.starts_with('/') {
            path = &path[1..];
        }
        Some((a / path.parse::<UriPath>().ok()?) | m.parse::<UriName>().ok()?)
    }

    fn get_sym_uri(pathstr: &str) -> Option<SymbolUri> {
        let (m, s) = match pathstr.split_once('[') {
            Some((m, s)) => {
                let (m, _) = m.rsplit_once('?')?;
                let (a, b) = s.rsplit_once(']')?;
                let am = get_mod_uri(a)?;
                let name = am.module_name() / &b.parse().ok()?;
                let module = get_mod_uri(m)?;
                return Some(module | name);
            }
            None => pathstr.rsplit_once('?')?,
        };
        let m = get_mod_uri(m)?;
        Some(m | s.parse::<UriName>().ok()?)
    }

    Ok(uris
        .into_iter()
        .map(|s| {
            get_sym_uri(&s).map_or_else(
                || {
                    get_mod_uri(&s)
                        .map_or_else(|| get_doc_uri(&s).map(Into::into), |s| Some(s.into()))
                },
                |s| Some(s.into()),
            )
        })
        .collect())
}
