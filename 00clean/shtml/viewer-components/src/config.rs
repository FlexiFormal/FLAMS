use leptos::prelude::*;
use std::future::Future;
use send_wrapper::SendWrapper;
use immt_utils::CSS;

pub const DEFAULT_SERVER_URL:&str = "https://immt.mathhub.info";

pub fn inpuref_url(uri:&str) -> String {
  format!("{}/fragment?uri={}",
    server_config.server_url.get(),
    urlencoding::encode(uri)
  )
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

#[allow(clippy::missing_panics_doc)]
pub fn set_server_url(s:String) {
    server_config.server_url.set(s);
}

#[allow(clippy::type_complexity)]
pub(crate) struct ServerConfig {
    pub server_url:RwSignal<String>,
    pub get_inputref:RwSignal<FutFun<String,Option<(Vec<CSS>,String)>>>
}

lazy_static::lazy_static! {
    pub(crate) static ref server_config:ServerConfig = ServerConfig {
        server_url:RwSignal::new(DEFAULT_SERVER_URL.to_string()),
        get_inputref:RwSignal::new(fn_into_future(defaults::get_inputref))
    };
}

#[cfg(feature="csr")]
mod defaults {
    use immt_utils::CSS;

  #[allow(clippy::future_not_send)]
  #[allow(clippy::similar_names)]
  pub async fn get_inputref(uri:String) -> Option<(Vec<CSS>,String)> {
    let url = super::inpuref_url(&uri);
    let res = reqwasm::http::Request::get(&url).send().await.ok()?;
    res.json().await.ok()
  }
}

#[cfg(not(feature="csr"))]
mod defaults {
  pub async fn get_inputref(uri:String) -> Option<String> {
    todo!()
  }
}