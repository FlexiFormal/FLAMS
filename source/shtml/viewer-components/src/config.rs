
use immt_ontology::uris::DocumentURI;
use leptos::prelude::*;
use immt_utils::CSS;

#[cfg(feature="hydrate")]
use immt_ontology::{uris::{ArchiveId, URI},languages::Language};

use crate::components::TOCElem;

pub const DEFAULT_SERVER_URL:&str = "https://immt.mathhub.info";


// CSR --------------------------------------------------------


#[cfg(feature="csr")]
pub struct ServerConfig {
  pub server_url:RwSignal<String>,
  //pub get_inputref:RwSignal<FutFun<String,Option<(Vec<CSS>,String)>>>
}

#[cfg(feature="csr")]
impl ServerConfig {
  fn inpuref_url(uri:&str) -> String {
    format!("{}/content/fragment?uri={}",
      server_config.server_url.get(),
      urlencoding::encode(uri)
    )
  }
  fn fulldoc_url(uri:&str) -> String {
    format!("{}/content/document?uri={}",
      server_config.server_url.get(),
      urlencoding::encode(uri)
    )
  }
  fn toc_url(uri:&str) -> String {
    format!("{}/content/toc?uri={}",
      server_config.server_url.get(),
      urlencoding::encode(uri)
    )
  }

  async fn remote<T:for<'a> serde::Deserialize<'a>>(url:String) -> Result<T,String> {
    send_wrapper::SendWrapper::new(Box::pin(async move {
      reqwasm::http::Request::get(&url).send().await.map_err(|e| e.to_string())?
        .json::<T>().await.map_err(|e| e.to_string())
    })).await
  }

  pub async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
    Self::remote(Self::inpuref_url(&doc.to_string())).await
  }

  pub async fn full_doc(&self,doc:DocumentURI) -> Result<(DocumentURI,Vec<CSS>,String),String> {
    Self::remote(Self::fulldoc_url(&doc.to_string())).await
  }
  pub async fn get_toc(&self,doc:DocumentURI) -> Result<(Vec<CSS>,Vec<TOCElem>),String> {
    Self::remote(Self::toc_url(&doc.to_string())).await
  }
}

#[cfg(feature="csr")]
impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      server_url:RwSignal::new(DEFAULT_SERVER_URL.to_string()),
      //get_inputref:RwSignal::new(fn_into_future(Self::default_inputref))
    }
  }
}

#[cfg(feature="csr")]
#[allow(clippy::missing_panics_doc)]
pub fn set_server_url(s:String) {
    server_config.server_url.set(s);
}

// hydrate ---------------------------------------------------

#[cfg(feature="hydrate")]
pub struct ServerConfig {
  pub get_inputref:RwSignal<
    Option<
      fn(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(Vec<CSS>,String),leptos::prelude::ServerFnError<String>>> + Send>>
    >
  >
}

#[cfg(feature="hydrate")]
impl ServerConfig {
  pub(crate) async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
    let Some(f) = self.get_inputref.get_untracked() else {
      panic!("Uninitialized shtml-viewer!!")
    };
    f(Some(URI::Narrative(doc.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string())
  }
  pub fn initialize(
    inputref:fn(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(Vec<CSS>,String),leptos::prelude::ServerFnError<String>>> + Send>>
  ) {
    server_config.get_inputref.set(Some(inputref));
  }
}
#[cfg(feature="hydrate")]
impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      get_inputref:RwSignal::new(None)
    }
  }
}

// SSR ---------------------------------------------------

#[cfg(all(feature="ssr",not(feature="csr"),not(feature="hydrate")))]
pub struct ServerConfig {}
#[cfg(all(feature="ssr",not(feature="csr"),not(feature="hydrate")))]
impl Default for ServerConfig {
  fn default() -> Self {
    Self {}
  }
}
#[cfg(all(feature="ssr",not(feature="csr"),not(feature="hydrate")))]
impl ServerConfig {
  pub(crate) async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
    unreachable!()
  }
}

// --------------------------------------

lazy_static::lazy_static! {
  pub static ref server_config:ServerConfig = ServerConfig::default();
}

/*
type BoxFuture<T> = SendWrapper<std::pin::Pin<Box<dyn Future<Output = T> + 'static>>>;

pub type FutFun<I,O> = std::sync::Arc<dyn Fn(I) -> BoxFuture<O> + Send + Sync>;
pub type FutFunRef<I,O> = std::sync::Arc<dyn Fn(&I) -> BoxFuture<O> + Send + Sync>;

pub fn fn_into_future<I,O,F:Future<Output=O> + 'static>(f: impl Fn(I) -> F +  Send + Sync + 'static) -> FutFun<I,O> {
    std::sync::Arc::new(move |i| SendWrapper::new(Box::pin(f(i))))
}
pub fn fn_into_future_ref<I:?Sized,O:'static,F:Future<Output=O> + 'static>(f: impl Fn(&I) -> F + 'static + Send + Sync) -> FutFunRef<I,O> {
  std::sync::Arc::new(move |i| SendWrapper::new(Box::pin(f(i))))
}
*/