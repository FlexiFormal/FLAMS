
use immt_ontology::uris::DocumentURI;
use leptos::prelude::*;
use std::future::Future;
use send_wrapper::SendWrapper;
use immt_utils::CSS;

#[cfg(feature="hydrate")]
use immt_ontology::{uris::{ArchiveId, URI},languages::Language};

pub const DEFAULT_SERVER_URL:&str = "https://immt.mathhub.info";


// CSR --------------------------------------------------------

#[cfg(feature="csr")]
pub fn inpuref_url(uri:&str) -> String {
  format!("{}/content/fragment?uri={}",
    server_config.server_url.get(),
    urlencoding::encode(uri)
  )
}

#[cfg(feature="csr")]
pub(crate) struct ServerConfig {
  pub server_url:RwSignal<String>,
  pub get_inputref:RwSignal<FutFun<String,Option<(Vec<CSS>,String)>>>
}

#[cfg(feature="csr")]
impl ServerConfig {
  #[allow(clippy::future_not_send)]
  #[allow(clippy::similar_names)]
  pub async fn default_inputref(uri:String) -> Option<(Vec<CSS>,String)> {
    let url = inpuref_url(&uri);
    let res = reqwasm::http::Request::get(&url).send().await.ok()?;
    res.json().await.ok()
  }
  pub(crate) async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
    let f = self.get_inputref.get_untracked();
    f(doc.to_string()).await.map_or_else(
      || Err("failed to get inputref".to_string()),
      Ok
    )
  }
}

#[cfg(feature="csr")]
impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      server_url:RwSignal::new(DEFAULT_SERVER_URL.to_string()),
      get_inputref:RwSignal::new(fn_into_future(Self::default_inputref))
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
  pub(crate) static ref server_config:ServerConfig = ServerConfig::default();
}

type BoxFuture<T> = SendWrapper<std::pin::Pin<Box<dyn Future<Output = T> + 'static>>>;

pub type FutFun<I,O> = std::sync::Arc<dyn Fn(I) -> BoxFuture<O> + Send + Sync>;
pub type FutFunRef<I,O> = std::sync::Arc<dyn Fn(&I) -> BoxFuture<O> + Send + Sync>;

pub fn fn_into_future<I,O,F:Future<Output=O> + 'static>(f: impl Fn(I) -> F +  Send + Sync + 'static) -> FutFun<I,O> {
    std::sync::Arc::new(move |i| SendWrapper::new(Box::pin(f(i))))
}
pub fn fn_into_future_ref<I:?Sized,O:'static,F:Future<Output=O> + 'static>(f: impl Fn(&I) -> F + 'static + Send + Sync) -> FutFunRef<I,O> {
  std::sync::Arc::new(move |i| SendWrapper::new(Box::pin(f(i))))
}