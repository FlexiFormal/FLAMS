
use immt_ontology::uris::DocumentURI;
use leptos::prelude::*;
use immt_utils::CSS;

#[cfg(feature="hydrate")]
use immt_ontology::{uris::{ArchiveId, URI},languages::Language};

use crate::components::TOCElem;

pub const DEFAULT_SERVER_URL:&str = "https://immt.mathhub.info";

/*macro_rules! csr {
    ($($t:tt)*) => {
        #[cfg(feature="csr")]
        #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
        $($t)*
    }
}*/


#[cfg(feature="csr")]
#[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
pub fn set_server_url(s:String) {
    server_config.server_url.set(s);
}


pub struct ServerConfig {
    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    pub server_url:RwSignal<String>,
    #[cfg(feature="hydrate")]
    #[cfg_attr(docsrs, doc(cfg(feature = "hydrate")))]
    pub get_inputref:RwSignal<
      Option<
        fn(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(Vec<CSS>,String),leptos::prelude::ServerFnError<String>>> + Send>>
      >
    >
}

impl ServerConfig {
    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    #[inline]
    fn inpuref_url(uri:&str) -> String {
        format!("{}/content/fragment?uri={}",
            server_config.server_url.get(),
            urlencoding::encode(uri)
        )
    }

    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    #[inline]
    fn fulldoc_url(uri:&str) -> String {
        format!("{}/content/document?uri={}",
            server_config.server_url.get(),
            urlencoding::encode(uri)
        )
    }

    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    #[inline]
    fn toc_url(uri:&str) -> String {
        format!("{}/content/toc?uri={}",
            server_config.server_url.get(),
            urlencoding::encode(uri)
        )
    }

    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    #[inline]
    async fn remote<T:for<'a> serde::Deserialize<'a>>(url:String) -> Result<T,String> {
        send_wrapper::SendWrapper::new(Box::pin(async move {
        reqwasm::http::Request::get(&url).send().await.map_err(|e| e.to_string())?
            .json::<T>().await.map_err(|e| e.to_string())
        })).await
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::inpuref_url(&doc.to_string())).await;
        #[cfg(feature="hydrate")]
        {
            let Some(f) = self.get_inputref.get_untracked() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(URI::Narrative(doc.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
        #[cfg(feature="ssr")]
        { unreachable!() }
    }

    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    /// #### Errors
    pub async fn full_doc(&self,doc:DocumentURI) -> Result<(DocumentURI,Vec<CSS>,String),String> {
        Self::remote(Self::fulldoc_url(&doc.to_string())).await
    }

    #[cfg(feature="csr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
    /// #### Errors
    pub async fn get_toc(&self,doc:DocumentURI) -> Result<(Vec<CSS>,Vec<TOCElem>),String> {
        Self::remote(Self::toc_url(&doc.to_string())).await
    }

    #[cfg(feature="hydrate")]
    #[cfg_attr(docsrs, doc(cfg(feature = "hydrate")))]
    pub fn initialize(
      inputref:fn(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(Vec<CSS>,String),leptos::prelude::ServerFnError<String>>> + Send>>
    ) {
      server_config.get_inputref.set(Some(inputref));
    }
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
        #[cfg(feature="csr")]
        server_url:RwSignal::new(DEFAULT_SERVER_URL.to_string()),
        #[cfg(feature="hydrate")]
        get_inputref:RwSignal::new(None)
    }
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