#![allow(clippy::unused_async)]
pub mod uris;
pub mod toc;

use immt_ontology::{content::{declarations::{structures::Extension, Declaration}, ContentReference}, languages::Language, narration::{exercises::Exercise, notations::Notation, paragraphs::{LogicalParagraph, ParagraphKind}, DocumentElement, LOKind, NarrativeReference}, uris::{ArchiveId, ContentURI, DocumentElementURI, DocumentURI, NarrativeURI, SymbolURI, URIOrRefTrait, URI}, Checked};
use immt_utils::{vecmap::VecSet, CSS};
use immt_web_utils::do_css;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shtml_viewer_components::{components::{omdoc::{narration::{DocumentElementSpec, DocumentSpec}, AnySpec,OMDocSource}, TOCElem, TOCSource}, DocumentString};
use uris::{DocURIComponents, SymURIComponents, URIComponents};
use crate::{users::Login, utils::from_server_clone};

macro_rules! backend {
  ($fn:ident!($($args:tt)*)) => {
    if immt_system::settings::Settings::get().lsp {
      let Some(state) = crate::server::lsp::STDIOLSPServer::global_state() else {
        return Err("no lsp server".to_string().into())
      };
      state.backend().$fn($($args)*)
    } else {
      ::paste::paste!{ 
        immt_system::backend::GlobalBackend::get().[<$fn _async>]($($args)*).await
      }
    }
  };
  ($fn:ident SYNC!($($args:tt)*)) => {
    if immt_system::settings::Settings::get().lsp {
      let Some(state) = crate::server::lsp::STDIOLSPServer::global_state() else {
        return Err::<_,ServerFnError<String>>("no lsp server".to_string().into())
      };
      state.backend().$fn($($args)*)
    } else {
        immt_system::backend::GlobalBackend::get().$fn($($args)*)
    }
  };
  ($fn:ident($($args:tt)*)) => {
    if immt_system::settings::Settings::get().lsp {
      crate::server::lsp::STDIOLSPServer::global_state().and_then(
        |state| state.backend().$fn($($args)*)
      )
    } else {
      immt_system::backend::GlobalBackend::get().$fn($($args)*)
    }
  };
  ($b:ident => {$($lsp:tt)*}{$($global:tt)*}) => {
    if immt_system::settings::Settings::get().lsp {
      let Some(state) = crate::server::lsp::STDIOLSPServer::global_state() else {
        return Err("no lsp server".to_string().into())
      };
      let $b = state.backend();
      $($lsp)*
    } else {
      let $b = immt_system::backend::GlobalBackend::get();
      $($global)*
    }
  };
}

pub(crate) use backend;

#[server(
  prefix="/content",
  endpoint="document",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
pub async fn document(
  uri:Option<DocumentURI>,
  rp:Option<String>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  d:Option<String>
) -> Result<(DocumentURI,Vec<CSS>, String),ServerFnError<String>> {
  use immt_system::backend::Backend;
  let Result::<DocURIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let Some((css,doc)) = backend!(get_html_body!(&uri,true)) else {
    return Err("document not found".to_string().into())
  };

  let html = format!("<div{}</div>",doc.strip_prefix("<body").and_then(|s| s.strip_suffix("</body>")).unwrap_or(""));
  Ok((uri,css,html))
}

#[server(
  prefix="/content",
  endpoint="toc",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
pub async fn toc(
  uri:Option<DocumentURI>,
  rp:Option<String>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  d:Option<String>
) -> Result<(Vec<CSS>, Vec<TOCElem>),ServerFnError<String>> {
  use immt_system::backend::Backend;
  let Result::<DocURIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let Some(doc) = backend!(get_document!(&uri)) else {
    return Err("document not found".to_string().into())
  };
  Ok(toc::from_document(&doc).await)
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
  uri:Option<URI>,
  rp:Option<String>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  d:Option<String>,
  e:Option<String>,
  m:Option<String>,
  s:Option<String>
) -> Result<(Vec<CSS>, String),ServerFnError<String>> {
  use immt_system::backend::Backend;
  let Result::<URIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d,e,m,s).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  match uri {
    URI::Narrative(NarrativeURI::Document(uri)) => {
      let Some((css,html)) = backend!(get_html_body!(&uri,false)) else {
        return Err("document not found".to_string().into())
      };
      Ok((css,html))
    }
    URI::Narrative(NarrativeURI::Element(uri)) => {
      let Some(e) = backend!(get_document_element!(&uri)) else {
        return Err("element not found".to_string().into())
      };
      match e.as_ref() {
        DocumentElement::Paragraph(LogicalParagraph{range,..}) |
        DocumentElement::Exercise(Exercise{range,..}) => {
          let Some((css,html)) = backend!(get_html_fragment!(uri.document(),*range)) else {
            return Err("document element not found".to_string().into())
          };
          Ok((css,html))
        }
        _ => return Err("not a paragraph".to_string().into())
      }
    }
    URI::Content(ContentURI::Symbol(uri)) => {
      get_definitions(uri).await.ok_or_else(||
        "No definition found".to_string().into()
      )
    }
    _ => return Err(format!("TODO").into())
  }
}

#[cfg(feature="ssr")]
async fn get_definitions(uri:SymbolURI) -> Option<(Vec<CSS>,String)> {
  use immt_ontology::{rdf::ontologies::ulo2, uris::DocumentElementURI};
  use immt_system::backend::{rdf::sparql, Backend, GlobalBackend};
  let b = GlobalBackend::get();
  let query = sparql::Select { 
    subject: sparql::Var('x'),
    pred: ulo2::DEFINES.into_owned(),
    object: uri.to_iri()
  }.into();
  //println!("Getting definitions using query: {}",query);
  let mut iter = b.triple_store().query(query).map(|r| r.into_uris()).unwrap_or_default().collect::<Vec<_>>();
  for uri in iter {
    if let Some(def) = b.get_document_element_async(&uri).await {
      let LogicalParagraph{range,..} = def.as_ref();
      if let Some(r) = b.get_html_fragment_async(uri.document(), *range).await {
        return Some(r)
      }
    }
  }
  None
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
  uri:Option<SymbolURI>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  m:Option<String>,
  s:Option<String>,
  exercises:bool
) -> Result<Vec<(DocumentElementURI,LOKind)>,ServerFnError<String>> {
  use immt_ontology::{rdf::ontologies::ulo2, uris::DocumentElementURI};
  use immt_system::backend::{rdf::sparql, Backend, GlobalBackend};
  let Result::<SymURIComponents,_>::Ok(comps) = (uri,a,p,l,m,s).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };  
  let Ok(v) = tokio::task::spawn_blocking(move || {
    GlobalBackend::get().triple_store().los(&uri,exercises).map(|i| i.collect()).unwrap_or_default()
  }).await else {
    return Err("internal error".to_string().into())
  };
  Ok(v)
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
  uri:Option<URI>,
  rp:Option<String>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  d:Option<String>,
  e:Option<String>,
  m:Option<String>,
  s:Option<String>
) -> Result<Vec<(DocumentElementURI,Notation)>,ServerFnError<String>> {
  use immt_ontology::{rdf::ontologies::ulo2, uris::DocumentElementURI};
  use immt_system::backend::{rdf::sparql, Backend, GlobalBackend};

  let Result::<URIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d,e,m,s).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let r = match uri {
    URI::Content(ContentURI::Symbol(uri)) => 
      tokio::task::spawn_blocking(move || Ok(backend!(get_notations SYNC!(&uri)).unwrap_or_default())).await,
    URI::Narrative(NarrativeURI::Element(uri)) =>
      tokio::task::spawn_blocking(move || Ok(backend!(get_var_notations SYNC!(&uri)).unwrap_or_default())).await,
    _ => return Err(format!("Not a symbol or variable URI: {uri}").into())
  };
  let Ok(Ok(v)) = r else {
    return Err("internal error".to_string().into())
  };
  Ok(v.0)
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
  uri:Option<URI>,
  rp:Option<String>,
  a:Option<ArchiveId>,
  p:Option<String>,
  l:Option<Language>,
  d:Option<String>,
  e:Option<String>,
  m:Option<String>,
  s:Option<String>
) -> Result<(Vec<CSS>,AnySpec),ServerFnError<String>> {
  use immt_system::backend::Backend;

  let Result::<URIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d,e,m,s).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let mut css = VecSet::default();
  match uri {
    uri @ (URI::Base(_) | URI::Archive(_) | URI::Path(_)) => Ok((css.0,AnySpec::Other(uri.to_string()))),
    URI::Narrative(NarrativeURI::Document(uri)) => {
      let Some(doc) = backend!(get_document!(&uri)) else {
        return Err("document not found".to_string().into())
      };
      let (css,r) = backend!(backend => {
        let r = DocumentSpec::from_document(&doc, backend,&mut css);
        (css,r)
      }{
        tokio::task::spawn_blocking(move || {
          let r = DocumentSpec::from_document(&doc, backend,&mut css);
          (css,r)
        }).await.map_err(|e| e.to_string())?
      });
      Ok((css.0,r.into()))
    }
    URI::Narrative(NarrativeURI::Element(uri)) => {
      let Some(e)
        : Option<NarrativeReference<DocumentElement<Checked>>>
        = backend!(get_document_element!(&uri)) else {
        return Err("document element not found".to_string().into())
      };
      let (css,r) = backend!(backend => {
        let r = DocumentElementSpec::from_element(e.as_ref(),backend, &mut css);
        (css,r)
      }{
        tokio::task::spawn_blocking(move || {
          let r = DocumentElementSpec::from_element(e.as_ref(),backend,&mut css);
          (css,r)
        }).await.map_err(|e| e.to_string())?
      });
      let Some(r) = r else {
        return Err("element not found".to_string().into())
      };
      Ok((css.0,r.into()))
    }
    URI::Content(ContentURI::Module(uri)) => {
      let Some(m) = backend!(get_module!(&uri)) else {
        return Err("module not found".to_string().into())
      };
      let r = backend!(backend => {
        AnySpec::from_module_like(&m, backend)
      }{
        tokio::task::spawn_blocking(move || {
          AnySpec::from_module_like(&m, backend)
        }).await.map_err(|e| e.to_string())?
      });
      Ok((Vec::new(),r))
    }
    URI::Content(ContentURI::Symbol(uri)) => {
      let Some(s)
        : Option<ContentReference<Declaration>>
        = backend!(get_declaration!(&uri)) else {
        return Err("declaration not found".to_string().into())
      };
      todo!()
    }
  }
}

#[component(transparent)]
pub fn URITop() -> impl IntoView {
  use immt_web_utils::components::Themer;
  use leptos_meta::Stylesheet;
  use uris::URIComponentsTrait;
  view!{
    <Stylesheet id="leptos" href="/pkg/immt.css"/>
    <Themer>//<Login>
      <div style="min-height:100vh;color:black;">{
        use_query_map().with_untracked(|m| m.as_doc().map_or_else(
          || view!("TODO").into_any(),
          |doc| view!(<Document doc/>).into_any()
        ))
      }</div>//</Login>
    </Themer>
  }
}

#[component]
pub fn Document(doc:DocURIComponents) -> impl IntoView {
  from_server_clone(false,
    move || doc.clone().into_args(document), 
    |(uri,css,html)| view!{<div>{
        for css in css { do_css(css); }
        view!(<DocumentString html uri toc=TOCSource::Get omdoc=OMDocSource::Get/>)
    }</div>})
}