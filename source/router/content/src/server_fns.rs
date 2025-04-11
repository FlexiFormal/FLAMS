use flams_ontology::{
    SlideElement,
    languages::Language,
    narration::{
        LOKind,
        notations::Notation,
        problems::{ProblemFeedbackJson, ProblemResponse, Quiz, SolutionData},
    },
    uris::{ArchiveId, DocumentElementURI, DocumentURI, SymbolURI, URI},
};
use flams_utils::CSS;
use ftml_viewer_components::components::{TOCElem, omdoc::AnySpec};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use flams_router_base::uris::{DocURIComponents, SymURIComponents, URIComponents};

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
    problems: bool,
) -> Result<Vec<(DocumentElementURI, LOKind)>, ServerFnError<String>> {
    let Result::<SymURIComponents, _>::Ok(comps) = (uri, a, p, m, s).try_into() else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(uri) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    server::los(uri, problems).await
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

#[server(prefix = "/content", endpoint = "grade")]
pub async fn grade(
    submissions: Vec<(Box<[SolutionData]>, Vec<ProblemResponse>)>,
) -> Result<Vec<Vec<ProblemFeedbackJson>>, ServerFnError<String>> {
    tokio::task::spawn_blocking(move || {
        let mut ret = Vec::new();
        for (sol, resps) in submissions {
            let mut ri = Vec::new();
            let sol = flams_ontology::narration::problems::Solutions::from_solutions(sol);
            for resp in resps {
                let r = sol.check_response(&resp).ok_or_else(|| {
                    "Response {resp:?} does not match solution {sol:?}".to_string()
                })?;
                ri.push(r.to_json());
            }
            ret.push(ri)
        }
        Ok(ret)
    })
    .await
    .map_err(|e| e.to_string())?
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
    use flams_web_utils::blocking_server_fn;
    let Result::<URIComponents, _>::Ok(comps) = (uri, rp, a, p, l, d, e, None, None).try_into()
    else {
        return Err("invalid uri components".to_string().into());
    };
    let Some(URI::Narrative(NarrativeURI::Element(uri))) = comps.parse() else {
        return Err("invalid uri".to_string().into());
    };
    blocking_server_fn(move || {
        let s = server::get_solution(&uri)?;
        s.as_hex().map_err(|e| e.to_string())
    })
    .await
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
            notations::Notation,
            paragraphs::LogicalParagraph,
            problems::{Problem, Quiz, Solutions},
        },
        rdf::ontologies::ulo2,
        uris::{
            ContentURI, DocumentElementURI, DocumentURI, NarrativeURI, SymbolURI, URI,
            URIOrRefTrait,
        },
    };
    use flams_system::backend::{Backend, GlobalBackend, rdf::sparql};
    use flams_utils::{CSS, vecmap::VecSet};
    use flams_web_utils::blocking_server_fn;
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
                    | DocumentElement::Problem(Problem { range, .. }) => {
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
        problems: bool,
    ) -> Result<Vec<(DocumentElementURI, LOKind)>, ServerFnError<String>> {
        blocking_server_fn(move || {
            Ok(GlobalBackend::get()
                .triple_store()
                .los(&uri, problems)
                .map(|i| i.collect())
                .unwrap_or_default())
        })
        .await
    }

    pub async fn notations(
        uri: URI,
    ) -> Result<Vec<(DocumentElementURI, Notation)>, ServerFnError<String>> {
        let v = match uri {
            URI::Content(ContentURI::Symbol(uri)) => {
                blocking_server_fn(move || {
                    Ok(backend!(get_notations SYNC!(&uri)).unwrap_or_default())
                })
                .await
            }
            URI::Narrative(NarrativeURI::Element(uri)) => {
                blocking_server_fn(move || {
                    Ok(backend!(get_var_notations SYNC!(&uri)).unwrap_or_default())
                })
                .await
            }
            _ => return Err(format!("Not a symbol or variable URI: {uri}").into()),
        }?;
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
                  blocking_server_fn(move || {
                    let r = DocumentSpec::from_document(&doc, backend,&mut css);
                    Ok((css,r))
                  }).await?
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
                  blocking_server_fn(move || {
                    let r = DocumentElementSpec::from_element(e.as_ref(),backend,&mut css);
                    Ok((css,r))
                  }).await?
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
                  blocking_server_fn(move || {
                    Ok(AnySpec::from_module_like(&m, backend))
                  }).await?
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
        blocking_server_fn(move || {
            let be = if flams_system::settings::Settings::get().lsp {
                let Some(state) = flams_lsp::STDIOLSPServer::global_state() else {
                    return Err("no lsp server".to_string());
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
        blocking_server_fn(move || {
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
    }

    pub fn get_solution(uri: &DocumentElementURI) -> Result<Solutions, String> {
        use flams_system::backend::Backend;
        match backend!(get_document_element(&uri)) {
            Some(rf) => {
                let e: &Problem<Checked> = rf.as_ref();
                let Some(sol) = backend!(get_reference(&e.solutions)) else {
                    return Err("solutions not found".to_string());
                };
                Ok(sol)
            }
            _ => Err(format!("Problem {uri} not found")),
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

#[server(prefix = "/content/legacy", endpoint = "uris")]
pub async fn uris(uris: Vec<String>) -> Result<Vec<Option<URI>>, ServerFnError<String>> {
    use flams_ontology::uris::{
        ArchiveURI, ArchiveURITrait, BaseURI, ModuleURI, URIOrRefTrait, URIRefTrait,
    };
    use flams_system::backend::{Backend, GlobalBackend};

    const MATHHUB: &str = "http://mathhub.info";
    const META: &str = "http://mathhub.info/sTeX/meta";
    const URTHEORIES: &str = "http://cds.omdoc.org/urtheories";

    lazy_static::lazy_static! {
      static ref MATHHUB_INFO: BaseURI = BaseURI::new_unchecked("http://mathhub.info/:sTeX");
      static ref META_URI: ArchiveURI = flams_ontology::metatheory::URI.archive_uri().owned();//ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("sTeX/meta-inf"));
      static ref UR_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://cds.omdoc.org"),ArchiveId::new("MMT/urtheories"));
      static ref MY_ARCHIVE: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://mathhub.info"),ArchiveId::new("my/archive"));
      static ref INJECTING: ArchiveURI = ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("Papers/22-CICM-Injecting-Formal-Mathematics"));
      static ref TUG: ArchiveURI = ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("Papers/22-TUG-sTeX"));
    }

    fn split(p: &str) -> Option<(ArchiveURI, usize)> {
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
            return Some((ArchiveURI::no_archive(), 7));
        }
        if let Some(mut p) = p.strip_prefix(MATHHUB) {
            let mut i = MATHHUB.len();
            if let Some(s) = p.strip_prefix('/') {
                p = s;
                i += 1;
            }
            return split_old(p, i);
        }
        GlobalBackend::get().with_archives(|mut tree| {
            tree.find_map(|a| {
                let base = a.uri();
                let base = base.base().as_ref();
                if p.starts_with(base) {
                    let l = base.len();
                    let np = &p[l..];
                    let id = a.id().as_ref();
                    if np.starts_with(id) {
                        Some((a.uri().owned(), l + id.len()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    fn split_old(p: &str, len: usize) -> Option<(ArchiveURI, usize)> {
        GlobalBackend::get().with_archives(|mut tree| {
            tree.find_map(|a| {
                if p.starts_with(a.id().as_ref()) {
                    let mut l = a.id().as_ref().len();
                    let np = &p[l..];
                    if np.starts_with('/') {
                        l += 1;
                    }
                    Some((a.uri().owned(), len + l))
                } else {
                    None
                }
            })
        })
    }

    fn get_doc_uri(pathstr: &str) -> Option<DocumentURI> {
        let pathstr = pathstr.strip_suffix(".tex").unwrap_or(pathstr);
        let (p, mut m) = pathstr.rsplit_once('/')?;
        let (a, l) = split(p)?;
        let mut path = if l < p.len() { &p[l..] } else { "" };
        if path.starts_with('/') {
            path = &path[1..];
        }
        let lang = Language::from_rel_path(m);
        m = m.strip_suffix(&format!(".{lang}")).unwrap_or(m);
        ((a % path).ok()? & (m, lang)).ok()
    }

    fn get_mod_uri(pathstr: &str) -> Option<ModuleURI> {
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
        ((a % path).ok()? | m).ok()
    }

    fn get_sym_uri(pathstr: &str) -> Option<SymbolURI> {
        let (m, s) = match pathstr.split_once('[') {
            Some((m, s)) => {
                let (m, _) = m.rsplit_once('?')?;
                let (a, b) = s.rsplit_once(']')?;
                let am = get_mod_uri(a)?;
                let name = (am.name().clone() / b).ok()?;
                let module = get_mod_uri(m)?;
                return Some(module | name);
            }
            None => pathstr.rsplit_once('?')?,
        };
        let m = get_mod_uri(m)?;
        (m | s).ok()
    }

    Ok(uris
        .into_iter()
        .map(|s| {
            get_sym_uri(&s).map_or_else(
                || {
                    get_mod_uri(&s).map_or_else(
                        || get_doc_uri(&s).map(|d| URI::Narrative(d.into())),
                        |s| Some(URI::Content(s.into())),
                    )
                },
                |s| Some(URI::Content(s.into())),
            )
        })
        .collect())
}
