
use immt_ontology::{narration::LOKind, uris::{DocumentElementURI, DocumentURI, SymbolURI}};
use leptos::prelude::*;
use immt_utils::CSS;

#[cfg(feature="omdoc")]
use crate::components::omdoc::AnySpec;

#[cfg(any(feature="hydrate",feature="ssr"))]
use immt_ontology::{uris::{ArchiveId, URI},languages::Language};

use crate::components::TOCElem;

pub const DEFAULT_SERVER_URL:&str = "https://immt.mathhub.info";

macro_rules! get {
    ($fn:ident($($arg:expr),*) = $res:pat => { $($code:tt)*}) => {{
        use ::leptos::suspense::Suspense;
        let r = ::leptos::prelude::Resource::new(|| (),move |()| $crate::config::server_config.$fn($($arg),*));
        ::leptos::prelude::view!{
            <Suspense fallback=|| view!(<immt_web_utils::components::Spinner/>)>{move ||
                if let Some(Ok($res)) = r.get() {
                    Some({$($code)*})
                } else {None}
            }</Suspense>
        }
    }}
}

pub(crate) use get;

#[cfg(feature="csr")]
#[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
pub fn set_server_url(s:String) {
    *server_config.server_url.lock() = s;
}

#[cfg(any(feature="hydrate",feature="ssr"))]
#[macro_export]
macro_rules! server_fun{
    ($($ty:ty),* => $ret:ty) => {
        fn($($ty),*) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<$ret,leptos::prelude::ServerFnError<String>>> + Send>>
    };
    (@URI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String> $(,$ty)* => $ret)
    };
    (@DOCURI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<DocumentURI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String> $(,$ty)* => $ret)
    };
    (@SYMURI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<SymbolURI>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>  $(,$ty)* => $ret)
    };
}

pub struct ServerConfig {
    #[cfg(feature="csr")]
    pub server_url:parking_lot::Mutex<String>,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[allow(clippy::type_complexity)]
    pub get_full_doc:std::sync::OnceLock<server_fun!(@DOCURI => (DocumentURI,Vec<CSS>,String))>,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[allow(clippy::type_complexity)]
    pub get_fragment:std::sync::OnceLock<server_fun!(@URI => (Vec<CSS>,String))>,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[allow(clippy::type_complexity)]
    pub get_omdoc:std::sync::OnceLock<server_fun!(@URI => (Vec<CSS>,AnySpec))>,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[allow(clippy::type_complexity)]
    pub get_toc:std::sync::OnceLock<server_fun!(@DOCURI => (Vec<CSS>,Vec<TOCElem>))>,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[allow(clippy::type_complexity)]
    pub get_los:std::sync::OnceLock<server_fun!(@SYMURI => Vec<(DocumentElementURI,LOKind)>)>
}

impl ServerConfig {

    pub fn top_doc_url(&self,uri:&DocumentURI) -> String {
        #[cfg(feature="csr")]
        {format!("{}/?uri={}",self.server_url.lock(),urlencoding::encode(&uri.to_string()))}
        #[cfg(not(feature="csr"))]
        {format!("/?uri={}",urlencoding::encode(&uri.to_string()))}
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::fragment_url(&doc.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_fragment.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(URI::Narrative(doc.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn paragraph(&self,doc:DocumentElementURI) -> Result<(Vec<CSS>,String),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::fragment_url(&doc.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_fragment.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(URI::Narrative(doc.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn definition(&self,uri:SymbolURI) -> Result<(Vec<CSS>,String),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::fragment_url(&uri.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_fragment.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(URI::Content(uri.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn full_doc(&self,uri:DocumentURI) -> Result<(DocumentURI,Vec<CSS>,String),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::fulldoc_url(&uri.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_full_doc.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(uri),None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn get_toc(&self,uri:DocumentURI) -> Result<(Vec<CSS>,Vec<TOCElem>),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::toc_url(&uri.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_toc.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(uri),None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn get_los(&self,uri:SymbolURI) -> Result<Vec<(DocumentElementURI,LOKind)>,String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::los_url(&uri.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_los.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(uri),None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    /// #### Errors
    /// #### Panics
    #[cfg(feature="omdoc")]
    #[allow(unreachable_code)]
    pub async fn omdoc(&self,uri:immt_ontology::uris::URI) -> Result<(Vec<CSS>,AnySpec),String> {
        #[cfg(feature="csr")]
        return Self::remote(Self::omdoc_url(&uri.to_string())).await;
        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.get_omdoc.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return f(Some(uri),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }

    #[cfg(any(feature="hydrate",feature="ssr"))]
    pub fn initialize(
      inputref:server_fun!(@URI => (Vec<CSS>,String)),
      full_doc:server_fun!(@DOCURI => (DocumentURI,Vec<CSS>,String)),
      toc:server_fun!(@DOCURI => (Vec<CSS>,Vec<TOCElem>)),
      omdoc:server_fun!(@URI => (Vec<CSS>,AnySpec)),
      los:server_fun!(@SYMURI => Vec<(DocumentElementURI,LOKind)>)
    ) {
        use leptos::server;

      let _ = server_config.get_fragment.set(inputref);
      let _ = server_config.get_omdoc.set(omdoc);
      let _ = server_config.get_full_doc.set(full_doc);
      let _ = server_config.get_toc.set(toc);
      let _ = server_config.get_los.set(los);
    }
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
        #[cfg(feature="csr")]
        server_url:parking_lot::Mutex::new(DEFAULT_SERVER_URL.to_string()),
        #[cfg(any(feature="hydrate",feature="ssr"))]
        get_fragment:std::sync::OnceLock::new(),
        #[cfg(any(feature="hydrate",feature="ssr"))]
        get_omdoc:std::sync::OnceLock::new(),
        #[cfg(any(feature="hydrate",feature="ssr"))]
        get_full_doc:std::sync::OnceLock::new(),
        #[cfg(any(feature="hydrate",feature="ssr"))]
        get_toc:std::sync::OnceLock::new(),
        #[cfg(any(feature="hydrate",feature="ssr"))]
        get_los:std::sync::OnceLock::new()
    }
  }
}

lazy_static::lazy_static! {
  pub static ref server_config:ServerConfig = ServerConfig::default();
}

// URLs

#[cfg(feature="csr")]
impl ServerConfig {
    #[inline]
    fn fragment_url(uri:&str) -> String {
        format!("{}/content/fragment?uri={}",
            server_config.server_url.lock(),
            urlencoding::encode(uri)
        )
    }

    #[inline]
    fn fulldoc_url(uri:&str) -> String {
        format!("{}/content/document?uri={}",
            server_config.server_url.lock(),
            urlencoding::encode(uri)
        )
    }

    #[inline]
    fn los_url(uri:&str) -> String {
        format!("{}/content/los?uri={}",
            server_config.server_url.lock(),
            urlencoding::encode(uri)
        )
    }

    #[cfg(feature="omdoc")]
    #[inline]
    fn omdoc_url(uri:&str) -> String {
        format!("{}/content/omdoc?uri={}",
            server_config.server_url.lock(),
            urlencoding::encode(uri)
        )
    }

    #[inline]
    fn toc_url(uri:&str) -> String {
        format!("{}/content/toc?uri={}",
            server_config.server_url.lock(),
            urlencoding::encode(uri)
        )
    }

    #[inline]
    async fn remote<T:for<'a> serde::Deserialize<'a>>(url:String) -> Result<T,String> {
        send_wrapper::SendWrapper::new(Box::pin(async move {
        reqwasm::http::Request::get(&url).send().await.map_err(|e| e.to_string())?
            .json::<T>().await.map_err(|e| e.to_string())
        })).await
    }
}