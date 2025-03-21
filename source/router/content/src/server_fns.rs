use flams_ontology::{
    SlideElement,
    languages::Language,
    narration::{LOKind, exercises::Quiz, notations::Notation},
    uris::{ArchiveId, DocumentElementURI, DocumentURI, SymbolURI, URI},
};
use flams_utils::CSS;
use ftml_viewer_components::components::{TOCElem, omdoc::AnySpec};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::uris::{DocURIComponents, SymURIComponents, URIComponents};

#[server(
  prefix="/content",
  endpoint="document",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
pub async fn document(
    uri: Option<DocumentURI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
) -> Result<(DocumentURI, Vec<CSS>, String), ServerFnError<String>> {
    let Result::<DocURIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::document(uri).await
}

#[server(
  prefix="/content",
  endpoint="toc",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
pub async fn toc(
    uri: Option<DocumentURI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
) -> Result<(Vec<CSS>, Vec<TOCElem>), ServerFnError<String>> {
    let Result::<DocURIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::toc(uri).await
}

#[server(
  prefix="/content",
  endpoint="fragment",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn fragment(
    uri: Option<URI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
    e: Option<String>,
    m: Option<String>,
    s: Option<String>,
) -> Result<(URI, Vec<CSS>, String), ServerFnError<String>> {
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::fragment(uri).await
}

#[server(
  prefix="/content",
  endpoint="los",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn los(
    uri: Option<SymbolURI>,
    a: Option<ArchiveId>,
    p: Option<String>,
    m: Option<String>,
    s: Option<String>,
    exercises: bool,
) -> Result<Vec<(DocumentElementURI, LOKind)>, ServerFnError<String>> {
    let Result::<SymURIComponents, _>::Ok(comps) = (uri, a, p, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::los(uri, exercises).await
}

#[server(
  prefix="/content",
  endpoint="notations",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn notations(
    uri: Option<URI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
    e: Option<String>,
    m: Option<String>,
    s: Option<String>,
) -> Result<Vec<(DocumentElementURI, Notation)>, ServerFnError<String>> {
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::notations(uri).await
}

#[server(
  prefix="/content",
  endpoint="omdoc",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn omdoc(
    uri: Option<URI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
    e: Option<String>,
    m: Option<String>,
    s: Option<String>,
) -> Result<(Vec<CSS>, AnySpec), ServerFnError<String>> {
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::omdoc(uri).await
}

#[server(
  prefix="/content",
  endpoint="quiz",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn get_quiz(
    uri: Option<DocumentURI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
) -> Result<Quiz, ServerFnError<String>> {
    let Result::<DocURIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::get_quiz(uri).await
}

#[server(
  prefix="/content",
  endpoint="solution",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn solution(
    uri: Option<URI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
    e: Option<String>,
) -> Result<String, ServerFnError<String>> {
    use flams_ontology::uris::NarrativeURI;
    use flams_utils::Hexable;
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, None, None).try_into()
    else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(URI::Narrative(NarrativeURI::Element(uri))) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    tokio::task::spawn_blocking(move || {
        let s = server::get_solution(&uri)?;
        s.as_hex().map_err(|e| e.to_string().into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[server(
  prefix="/content",
  endpoint="slides",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub async fn slides_view(
    uri: Option<URI>,
    rp: Option<String>,
    a: Option<ArchiveId>,
    p: Option<String>,
    l: Option<Language>,
    d: Option<String>,
    e: Option<String>,
    m: Option<String>,
    s: Option<String>,
) -> Result<(Vec<CSS>, Vec<SlideElement>), ServerFnError<String>> {
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::slides(uri).await
}

#[cfg(feature = "ssr")]
mod server {
    use crate::ssr::{backend, insert_base_url};
    use flams_ontology::{
        Checked, SlideElement,
        content::{ContentReference, declarations::Declaration},
        narration::{
            DocumentElement, LOKind, NarrationTrait, NarrativeReference,
            exercises::{Exercise, Quiz, Solutions},
            notations::Notation,
            paragraphs::LogicalParagraph,
        },
        rdf::ontologies::ulo2,
        uris::{
            ContentURI, DocumentElementURI, DocumentURI, NarrativeURI, SymbolURI, URI,
            URIOrRefTrait,
        },
    };
    use flams_system::backend::{Backend, GlobalBackend, rdf::sparql};
    use flams_utils::{CSS, vecmap::VecSet};
    use ftml_viewer_components::components::{
        TOCElem,
        omdoc::{
            AnySpec,
            narration::{DocumentElementSpec, DocumentSpec},
        },
    };
    use leptos::prelude::*;

    pub async fn document(
        uri: DocumentURI,
    ) -> Result<(DocumentURI, Vec<CSS>, String), ServerFnError<String>> {
        let Some((css, doc)) = backend!(get_html_body!(&uri, true)) else {
            return Err("document not found".to_string().into());
        };

        let html = format!(
            "<div{}</div>",
            doc.strip_prefix("<body")
                .and_then(|s| s.strip_suffix("</body>"))
                .unwrap_or("")
        );
        Ok((uri, insert_base_url(css), html))
    }

    pub async fn toc(uri: DocumentURI) -> Result<(Vec<CSS>, Vec<TOCElem>), ServerFnError<String>> {
        let Some(doc) = backend!(get_document!(&uri)) else {
            return Err("document not found".to_string().into());
        };
        Ok(crate::toc::from_document(&doc).await)
    }

    pub async fn fragment(uri: URI) -> Result<(URI, Vec<CSS>, String), ServerFnError<String>> {
        match &uri {
            URI::Narrative(NarrativeURI::Document(duri)) => {
                let Some((css, html)) = backend!(get_html_body!(duri, false)) else {
                    return Err("document not found".to_string().into());
                };
                Ok((uri, insert_base_url(filter_paras(css)), html))
            }
            URI::Narrative(NarrativeURI::Element(euri)) => {
                let Some(e) = backend!(get_document_element!(euri)) else {
                    return Err("element not found".to_string().into());
                };
                match e.as_ref() {
                    DocumentElement::Paragraph(LogicalParagraph { range, .. })
                    | DocumentElement::Exercise(Exercise { range, .. }) => {
                        let Some((css, html)) =
                            backend!(get_html_fragment!(euri.document(), *range))
                        else {
                            return Err("document element not found".to_string().into());
                        };
                        Ok((uri, insert_base_url(filter_paras(css)), html))
                    }
                    DocumentElement::Section(flams_ontology::narration::sections::Section {
                        range,
                        ..
                    }) => {
                        let Some((css, html)) =
                            backend!(get_html_fragment!(euri.document(), *range))
                        else {
                            return Err("document element not found".to_string().into());
                        };
                        Ok((uri, insert_base_url(filter_paras(css)), html))
                    }
                    _ => return Err("not a paragraph".to_string().into()),
                }
            }
            URI::Content(ContentURI::Symbol(suri)) => get_definitions(suri.clone())
                .await
                .ok_or_else(|| "No definition found".to_string().into())
                .map(|(css, b)| (uri, insert_base_url(filter_paras(css)), b)),
            URI::Base(_) => return Err("TODO: base".to_string().into()),
            URI::Archive(_) => return Err("TODO: archive".to_string().into()),
            URI::Path(_) => return Err("TODO: path".to_string().into()),
            URI::Content(ContentURI::Module(_)) => return Err("TODO: module".to_string().into()),
        }
    }

    pub async fn los(
        uri: SymbolURI,
        exercises: bool,
    ) -> Result<Vec<(DocumentElementURI, LOKind)>, ServerFnError<String>> {
        let Ok(v) = tokio::task::spawn_blocking(move || {
            GlobalBackend::get()
                .triple_store()
                .los(&uri, exercises)
                .map(|i| i.collect())
                .unwrap_or_default()
        })
        .await
        else {
            return Err("internal error".to_string().into());
        };
        Ok(v)
    }

    pub async fn notations(
        uri: URI,
    ) -> Result<Vec<(DocumentElementURI, Notation)>, ServerFnError<String>> {
        let r = match uri {
            URI::Content(ContentURI::Symbol(uri)) => {
                tokio::task::spawn_blocking(move || {
                    Ok(backend!(get_notations SYNC!(&uri)).unwrap_or_default())
                })
                .await
            }
            URI::Narrative(NarrativeURI::Element(uri)) => {
                tokio::task::spawn_blocking(move || {
                    Ok(backend!(get_var_notations SYNC!(&uri)).unwrap_or_default())
                })
                .await
            }
            _ => return Err(format!("Not a symbol or variable URI: {uri}").into()),
        };
        let Ok(Ok(v)) = r else {
            return Err("internal error".to_string().into());
        };
        Ok(v.0)
    }

    pub async fn omdoc(uri: URI) -> Result<(Vec<CSS>, AnySpec), ServerFnError<String>> {
        let mut css = VecSet::default();
        match uri {
            uri @ (URI::Base(_) | URI::Archive(_) | URI::Path(_)) => {
                Ok((insert_base_url(css.0), AnySpec::Other(uri.to_string())))
            }
            URI::Narrative(NarrativeURI::Document(uri)) => {
                let Some(doc) = backend!(get_document!(&uri)) else {
                    return Err("document not found".to_string().into());
                };
                let (css, r) = backend!(backend => {
                  let r = DocumentSpec::from_document(&doc, backend,&mut css);
                  (css,r)
                }{
                  tokio::task::spawn_blocking(move || {
                    let r = DocumentSpec::from_document(&doc, backend,&mut css);
                    (css,r)
                  }).await.map_err(|e| e.to_string())?
                });
                Ok((insert_base_url(css.0), r.into()))
            }
            URI::Narrative(NarrativeURI::Element(uri)) => {
                let Some(e): Option<NarrativeReference<DocumentElement<Checked>>> =
                    backend!(get_document_element!(&uri))
                else {
                    return Err("document element not found".to_string().into());
                };
                let (css, r) = backend!(backend => {
                  let r = DocumentElementSpec::from_element(e.as_ref(),backend, &mut css);
                  (css,r)
                }{
                  tokio::task::spawn_blocking(move || {
                    let r = DocumentElementSpec::from_element(e.as_ref(),backend,&mut css);
                    (css,r)
                  }).await.map_err(|e| e.to_string())?
                });
                let Some(r) = r else {
                    return Err("element not found".to_string().into());
                };
                Ok((insert_base_url(css.0), r.into()))
            }
            URI::Content(ContentURI::Module(uri)) => {
                let Some(m) = backend!(get_module!(&uri)) else {
                    return Err("module not found".to_string().into());
                };
                let r = backend!(backend => {
                  AnySpec::from_module_like(&m, backend)
                }{
                  tokio::task::spawn_blocking(move || {
                    AnySpec::from_module_like(&m, backend)
                  }).await.map_err(|e| e.to_string())?
                });
                Ok((Vec::new(), r))
            }
            URI::Content(ContentURI::Symbol(uri)) => {
                let Some(s): Option<ContentReference<Declaration>> =
                    backend!(get_declaration!(&uri))
                else {
                    return Err("declaration not found".to_string().into());
                };
                return Err(format!("TODO: {uri}").into());
            }
        }
    }

    pub async fn get_quiz(uri: DocumentURI) -> Result<Quiz, ServerFnError<String>> {
        use flams_system::backend::docfile::QuizExtension;
        let Some(doc) = backend!(get_document!(&uri)) else {
            return Err("document not found".to_string().into());
        };
        tokio::task::spawn_blocking(move || {
            let be = if flams_system::settings::Settings::get().lsp {
                let Some(state) = flams_lsp::STDIOLSPServer::global_state() else {
                    return Err::<_, ServerFnError<String>>("no lsp server".to_string().into());
                };
                doc.as_quiz(state.backend())
            } else {
                doc.as_quiz(flams_system::backend::GlobalBackend::get())
            };
            let mut be = be.map_err(|e| e.to_string())?;
            be.css = insert_base_url(std::mem::take(&mut be.css));
            Ok(be)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub async fn slides(uri: URI) -> Result<(Vec<CSS>, Vec<SlideElement>), ServerFnError<String>> {
        fn from_children(
            top: &DocumentURI,
            children: &[DocumentElement<Checked>],
            css: &mut VecSet<CSS>,
            backend: &impl Backend,
        ) -> Result<Vec<SlideElement>, String> {
            let mut stack = smallvec::SmallVec::<_, 2>::default();
            let mut ret = Vec::new();
            let mut curr = children.iter();

            loop {
                let Some(next) = curr.next() else {
                    if let Some((a, b, c)) = stack.pop() {
                        curr = a;
                        if let Some(mut b) = b {
                            std::mem::swap(&mut ret, &mut b);
                            ret.push(SlideElement::Section {
                                title: c,
                                children: b,
                            });
                        }
                        continue;
                    }
                    break;
                };
                match next {
                    DocumentElement::Slide { range, uri, .. } => {
                        let Some((c, html)) = backend.get_html_fragment(top, *range) else {
                            return Err(format!("Missing fragment for slide {uri}"));
                        };
                        for c in c {
                            css.insert(c);
                        }
                        ret.push(SlideElement::Slide { html });
                    }
                    DocumentElement::Paragraph(p) => {
                        let Some((c, html)) = backend.get_html_fragment(top, p.range) else {
                            return Err(format!("Missing fragment for paragraph {}", p.uri));
                        };
                        for c in c {
                            css.insert(c);
                        }
                        ret.push(SlideElement::Paragraph { html });
                    }
                    DocumentElement::DocumentReference { target, .. } => {
                        ret.push(SlideElement::Inputref {
                            uri: target.id().into_owned(),
                        })
                    }
                    DocumentElement::Section(s) => {
                        let title = if let Some(t) = s.title {
                            let Some((c, html)) = backend.get_html_fragment(top, t) else {
                                return Err(format!("Missing title for section {}", s.uri));
                            };
                            for c in c {
                                css.insert(c);
                            }
                            Some(html)
                        } else {
                            None
                        };
                        stack.push((
                            std::mem::replace(&mut curr, s.children().iter()),
                            Some(std::mem::replace(&mut ret, Vec::new())),
                            title,
                        ));
                    }
                    o => {
                        let chs = o.children();
                        if !chs.is_empty() {
                            stack.push((std::mem::replace(&mut curr, chs.iter()), None, None));
                        }
                    }
                }
            }
            Ok(ret)
        }

        let Some(doe) = (match uri {
            URI::Narrative(NarrativeURI::Document(uri)) => {
                backend!(get_document!(&uri)).map(either::Either::Left)
            }
            URI::Narrative(NarrativeURI::Element(uri)) => {
                backend!(get_document_element!(&uri)).map(either::Either::Right)
            }
            _ => return Err("Not a narrative URI".to_string().into()),
        }) else {
            return Err("Element not found".to_string().into());
        };
        tokio::task::spawn_blocking(move || {
            let (chs, top) = match &doe {
                either::Either::Left(d) => (d.children(), d.uri()),
                either::Either::Right(e) => {
                    let e: &NarrativeReference<DocumentElement<Checked>> = e;
                    (e.as_ref().children(), e.top().uri())
                }
            };
            let mut css = VecSet::default();

            let r = if flams_system::settings::Settings::get().lsp {
                let Some(state) = flams_lsp::STDIOLSPServer::global_state() else {
                    return Err("no lsp server".to_string());
                };
                from_children(top, chs, &mut css, state.backend())
            } else {
                from_children(
                    top,
                    chs,
                    &mut css,
                    flams_system::backend::GlobalBackend::get(),
                )
            }?;
            Ok((insert_base_url(css.0), r))
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
    }

    pub fn get_solution(uri: &DocumentElementURI) -> Result<Solutions, ServerFnError<String>> {
        use flams_system::backend::Backend;
        match backend!(get_document_element(&uri)) {
            Some(rf) => {
                let e: &Exercise<Checked> = rf.as_ref();
                let Some(sol) = backend!(get_reference(&e.solutions)) else {
                    return Err("solutions not found".to_string().into());
                };
                Ok(sol)
            }
            _ => Err(format!("Exercise {uri} not found").into()),
        }
    }

    async fn get_definitions(uri: SymbolURI) -> Option<(Vec<CSS>, String)> {
        let b = GlobalBackend::get();
        let query = sparql::Select {
            subject: sparql::Var('x'),
            pred: ulo2::DEFINES.into_owned(),
            object: uri.to_iri(),
        }
        .into();
        //println!("Getting definitions using query: {}",query);
        let iter = b
            .triple_store()
            .query(query)
            .map(|r| r.into_uris())
            .unwrap_or_default()
            .collect::<Vec<_>>();
        for uri in iter {
            if let Some(def) = b.get_document_element_async(&uri).await {
                let LogicalParagraph { range, .. } = def.as_ref();
                if let Some((css, r)) = b.get_html_fragment_async(uri.document(), *range).await {
                    return Some((insert_base_url(filter_paras(css)), r));
                }
            }
        }
        None
    }

    pub(crate) fn filter_paras(mut v: Vec<CSS>) -> Vec<CSS> {
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
        v.retain(|c| match c {
            CSS::Class { name, .. } => !CSSS.iter().any(|s| name.starts_with(s)),
            _ => true,
        });
        v
    }
}
