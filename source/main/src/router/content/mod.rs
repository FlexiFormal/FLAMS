#![allow(clippy::unused_async)]
pub mod uris;
pub mod toc;

use immt_ontology::{languages::Language, uris::{ArchiveId, DocumentURI, NarrativeURI, URI}};
use immt_utils::CSS;
use immt_web_utils::do_css;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shtml_viewer_components::components::TOCElem;
use uris::{DocURIComponents, URIComponents};
use crate::{users::Login, utils::from_server_clone};

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
  let Result::<DocURIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let Some((css,doc)) = immt_system::backend::GlobalBackend::get().get_html_body_async(&uri,true).await else {
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
  let Result::<DocURIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  let Some(doc) =immt_system::backend::GlobalBackend::get().get_document_async(&uri).await else {
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
  let Result::<URIComponents,_>::Ok(comps) = (uri,rp,a,p,l,d,e,m,s).try_into() else {
    return Err("invalid uri components".to_string().into())
  };
  let Some(uri) = comps.parse() else {
    return Err("invalid uri".to_string().into())
  };
  match uri {
    URI::Narrative(NarrativeURI::Document(uri)) => {
      let Some((css,html)) = immt_system::backend::GlobalBackend::get().get_html_body_async(&uri,false).await else {
        return Err("document not found".to_string().into())
      };
      Ok((css,html))
    }
    _ => todo!()
  }
}

#[component(transparent)]
pub fn URITop() -> impl IntoView {
  use immt_web_utils::components::Themer;
  use leptos_meta::Stylesheet;
  use uris::URIComponentsTrait;
  view!{
    <Stylesheet id="leptos" href="/pkg/immt.css"/>
    <Themer><Login><div style="min-height:100vh;">{
      use_query_map().with_untracked(|m| m.as_doc().map_or_else(
        || todo!(),
        |doc| view!(<Document doc/>)
      ))
    }</div></Login></Themer>
  }
}

#[component]
pub fn Document(doc:DocURIComponents) -> impl IntoView {
  use shtml_viewer_components::{SHTMLDocument,components::Toc};
  use leptos_dyn_dom::DomStringCont;
  let doccl = doc.clone();
  from_server_clone(false,
    move || doc.clone().into_args(document), |(uri,css,html)| view!{<div>{
      for css in css { do_css(css); }
      let r = Resource::new(|| (),move |()| doccl.clone().into_args(toc));
      let toc_signal = RwSignal::new(None);
      let _f = Effect::new(move || {
        if let Some(Ok((c,t))) = r.get() {
          toc_signal.set(Some((c,t)));
        }
      });

      let on_load = RwSignal::new(false);

      view!{<SHTMLDocument uri on_load>
        <Toc toc=toc_signal/>
        <DomStringCont html on_load cont=shtml_viewer_components::iterate/>
        </SHTMLDocument>}
    }</div>})
}

/*
macro_rules! uri {
  ($(#[$meta:meta])* fn $name:ident($uri:ident : $uritp:ident $(,$n:ident:$t:ty)*) $(-> $ret:ty)? $f:block) => {
    uri!{!! {$(#[$meta])* fn} $name($uri:$uritp $(,$n:$t)*) $(-> $ret)? $f}
  };
  ($(#[$meta:meta])* async fn $name:ident($uri:ident : $uritp:ident $(,$n:ident:$t:ty)*) $(-> $ret:ty)? $f:block) => {
    uri!{!! {$(#[$meta])* async fn} $name($uri:$uritp $(,$n:$t)*) $(-> $ret)? $f}
  };
  (!! {$($pre:tt)*} $name:ident($uri:ident :DocumentURI $(,$n:ident:$t:ty)*) $(-> $ret:ty)? $f:block) => {
    $($pre)* $name(uri:Option<DocumentURI>,a:Option<ArchiveId>,rp:Option<String>,l:Option<Language>,p:Option<String>,d:Option<String>$(,$n:$t)*) $(-> $ret)? {
      let $uri = uri.or_else(|| if let Some(a) = a {
        if let Some(rp) = rp {
          return uris::from_archive_relpath(&a,&rp)
        }
        if let Some(d) = d {uris::get_doc_uri(&a,
          p.map(|p| Name::from_str(&p).unwrap_or_else(|_| unreachable!())),
          l.unwrap_or(Language::English),
          Name::from_str(&d).unwrap_or_else(|_| unreachable!())
        )} else {None}
      } else {None});
      $f
    }
  }
}

uri!{
#[server(
  prefix="/api/fragments",
  endpoint="document",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Json
)]
async fn document(uri:DocumentURI) -> Result<(Vec<CSS>, String),ServerFnError<String>> {
  Ok((Vec::new(),String::new()))
}}

/*
uri!{DOC
  #[component]
  fn Document(uri) -> impl IntoView {
    todo!()
  }
}
*/
*/