#![allow(clippy::unused_async)]
pub mod uris;
pub mod toc;

use immt_ontology::{content::{declarations::{structures::Extension, Declaration}, ContentReference}, languages::Language, narration::{exercises::Exercise, notations::Notation, paragraphs::{LogicalParagraph, ParagraphKind}, DocumentElement, LOKind, NarrativeReference}, uris::{ArchiveId, ContentURI, DocumentElementURI, DocumentURI, NarrativeURI, SymbolURI, URIOrRefTrait, URI}, Checked};
use immt_utils::{vecmap::VecSet, CSS};
use immt_web_utils::do_css;
use leptos::{either::Either, prelude::*};
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
    URI::Base(_) => return Err("TODO: base".to_string().into()),
    URI::Archive(_) => return Err("TODO: archive".to_string().into()),
    URI::Path(_) => return Err("TODO: path".to_string().into()),
    URI::Content(ContentURI::Module(_)) => return Err("TODO: module".to_string().into())
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
  m:Option<String>,
  s:Option<String>,
  exercises:bool
) -> Result<Vec<(DocumentElementURI,LOKind)>,ServerFnError<String>> {
  use immt_ontology::{rdf::ontologies::ulo2, uris::DocumentElementURI};
  use immt_system::backend::{rdf::sparql, Backend, GlobalBackend};
  let Result::<SymURIComponents,_>::Ok(comps) = (uri,a,p,m,s).try_into() else {
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
  use shtml_viewer_components::SHTMLGlobalSetup;
  view!{
    <Stylesheet id="leptos" href="/pkg/immt.css"/>
    <Themer><SHTMLGlobalSetup>//<Login>
      <div style="min-height:100vh;color:black;">{
        use_query_map().with_untracked(|m| m.as_doc().map_or_else(
          || Either::Left(view!("TODO")),
          |doc| Either::Right(view!(<Document doc/>))
        ))
      }</div>//</Login>
    </SHTMLGlobalSetup></Themer>
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

// -------------------------------------------------------------


#[server(prefix="/content/legacy",endpoint="uris")]
pub async fn uris(uris:Vec<String>) -> Result<Vec<Option<URI>>,ServerFnError<String>> {
  use immt_ontology::uris::{BaseURI,ArchiveURI,ArchiveURITrait,URIRefTrait,ModuleURI};
  use immt_system::backend::{GlobalBackend,Backend};

  const MATHHUB: &str = "http://mathhub.info";
  const META: &str = "http://mathhub.info/sTeX/meta";
  const URTHEORIES: &str = "http://cds.omdoc.org/urtheories";

  lazy_static::lazy_static! {
    static ref MATHHUB_INFO: BaseURI = BaseURI::new_unchecked("http://mathhub.info/:sTeX");
    static ref META_URI: ArchiveURI = immt_ontology::metatheory::URI.archive_uri().owned();//ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("sTeX/meta-inf"));
    static ref UR_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://cds.omdoc.org"),ArchiveId::new("MMT/urtheories"));
    static ref MY_ARCHIVE: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://mathhub.info"),ArchiveId::new("my/archive"));
    static ref INJECTING: ArchiveURI = ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("Papers/22-CICM-Injecting-Formal-Mathematics"));
    static ref TUG: ArchiveURI = ArchiveURI::new(MATHHUB_INFO.clone(),ArchiveId::new("Papers/22-TUG-sTeX"));
  }


  fn split(p:&str) -> Option<(ArchiveURI,usize)> {
    if p.starts_with(META) {
        return Some((META_URI.clone(),29))
    }
    if p == URTHEORIES {
        return Some((UR_URI.clone(),31))
    }
    if p == "http://mathhub.info/my/archive" {
        return Some((MY_ARCHIVE.clone(),30))
    }
    if p == "http://kwarc.info/Papers/stex-mmt/paper" {
        return Some((INJECTING.clone(),34))
    }
    if p == "http://kwarc.info/Papers/tug/paper" {
        return Some((TUG.clone(),34))
    }
    if p.starts_with("file://") {
      return Some((ArchiveURI::no_archive(),7))
    }
    if let Some(mut p) = p.strip_prefix(MATHHUB) {
        let mut i = MATHHUB.len();
        if let Some(s) = p.strip_prefix('/') {
            p = s;
            i += 1;
        }
        return split_old(p,i)
    }
    GlobalBackend::get().with_archives(|mut tree|
      tree.find_map(|a| {
        let base = a.uri();
        let base = base.base().as_ref();
        if p.starts_with(base) {
            let l = base.len();
            let np = &p[l..];
            let id = a.id().as_ref();
            if np.starts_with(id) {
                Some((a.uri().owned(),l+id.len()))
            } else {None}
        } else { None }
    }))
  }

  fn split_old(p:&str,len:usize) -> Option<(ArchiveURI,usize)> {
    GlobalBackend::get().with_archives(|mut tree|
      tree.find_map(|a| {
        if p.starts_with(a.id().as_ref()) {
            let mut l = a.id().as_ref().len();
            let np = &p[l..];
            if np.starts_with('/') {
                l += 1;
            }
            Some((a.uri().owned(),len + l))
        } else { None }
    }))
  }

  fn get_doc_uri(pathstr: &str) -> Option<DocumentURI> {
    let pathstr = pathstr.strip_suffix(".tex").unwrap_or(pathstr);
    let (p,mut m) = pathstr.rsplit_once('/')?;
    let (a,l) = split(p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    let lang = Language::from_rel_path(m);
    m = m.strip_suffix(&format!(".{lang}")).unwrap_or(m);
    Some((a % path) & (m,lang))
  }

  fn get_mod_uri(pathstr: &str) -> Option<ModuleURI> {
    let (mut p,mut m) = pathstr.rsplit_once('?')?;
    m = m.strip_suffix("-module").unwrap_or(m);
    if p.bytes().last() == Some(b'/') {
        p = &p[..p.len()-1];
    }
    let (a,l) = split(p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    Some((a % path) | m)
  }

  fn get_sym_uri(pathstr: &str) -> Option<SymbolURI> {
    let (m,s) = match pathstr.split_once('[') {
        Some((m,s)) => {
            let (m,_) = m.rsplit_once('?')?;
            let (a,b) = s.rsplit_once(']')?;
            let am = get_mod_uri(a)?;
            let name = am.name().clone() / b;
            let module = get_mod_uri(m)?;
            return Some(module | name)
        }
        None => pathstr.rsplit_once('?')?
    };
    let m = get_mod_uri(m)?;
    Some(m | s)
  }

  Ok(
    uris.into_iter().map(|s| 
      get_sym_uri(&s).map_or_else(
        || get_mod_uri(&s).map_or_else(
          || get_doc_uri(&s).map(|d| URI::Narrative(d.into())),
          |s| Some(URI::Content(s.into()))
        ),
        |s| Some(URI::Content(s.into()))
      )
    ).collect()
  )
}