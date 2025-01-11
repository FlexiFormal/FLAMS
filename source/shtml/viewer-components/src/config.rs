
use immt_ontology::{narration::LOKind, uris::{DocumentElementURI, DocumentURI, SymbolURI}};
use leptos::prelude::*;
use immt_utils::CSS;
#[cfg(feature="csr")]
use std::borrow::Cow;

#[cfg(feature="omdoc")]
use crate::components::omdoc::AnySpec;

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
macro_rules! server_fun{
    ($($ty:ty),* => $ret:ty) => {
        fn($($ty),*) -> server_fun_ret!($ret)
    };
    (@URI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String> $(,$ty)* => $ret)
    };
    (@DOCURI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<DocumentURI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String> $(,$ty)* => $ret)
    };
    (@SYMURI$(,$ty:ty)* => $ret:ty) => {
        server_fun!(Option<SymbolURI>,Option<ArchiveId>,Option<String>,Option<String>,Option<String>  $(,$ty)* => $ret)
    };
}

#[cfg(any(feature="hydrate",feature="ssr"))]
macro_rules! server_fun_ret{
    ($ret:ty) => {
        std::pin::Pin<Box<dyn std::future::Future<Output=Result<$ret,leptos::prelude::ServerFnError<String>>> + Send>>
    }
}

#[cfg(all(feature="csr",not(any(feature="hydrate",feature="ssr"))))]
#[macro_export]
macro_rules! server_fun_ret{
    ($ret:ty) => {
        $ret
    }
}

trait ServerFunArgs {
    #[cfg(any(feature="hydrate",feature="ssr"))]
    type DeTupledFun<R>;
    type First:std::hash::Hash+std::fmt::Display+PartialEq+Eq+Clone;
    type Extra;
    #[cfg(feature="csr")]
    fn as_params(e:&Self::Extra) -> Cow<'static,str>;
    #[cfg(any(feature="hydrate",feature="ssr"))]
    fn call<R>(uri:Self::First,extra:Self::Extra,f:&Self::DeTupledFun<R>) -> server_fun_ret!(R);
}

#[cfg(all(feature="csr",not(any(feature="hydrate",feature="ssr"))))]
type URIArgs = URI;
#[cfg(any(feature="hydrate",feature="ssr"))]
type URIArgs = (Option<URI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>,Option<String>,Option<String>,Option<String>);
impl ServerFunArgs for URIArgs {
    #[cfg(any(feature="hydrate",feature="ssr"))]
    type DeTupledFun<R> = server_fun!(@URI => R);
    type First = URI;
    type Extra = ();
    #[cfg(feature="csr")]
    fn as_params(_:&Self::Extra) -> Cow<'static,str> {"".into()}
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[inline]
    fn call<R>(uri:URI,_:(),f:&Self::DeTupledFun<R>) -> server_fun_ret!(R) {
        f(Some(uri),None,None,None,None,None,None,None,None)
    }
}

#[cfg(all(feature="csr",not(any(feature="hydrate",feature="ssr"))))]
type DocURIArgs = DocumentURI;
#[cfg(any(feature="hydrate",feature="ssr"))]
type DocURIArgs = (Option<DocumentURI>,Option<String>,Option<ArchiveId>,Option<String>,Option<Language>,Option<String>);
impl ServerFunArgs for DocURIArgs {
    #[cfg(any(feature="hydrate",feature="ssr"))]
    type DeTupledFun<R> = server_fun!(@DOCURI => R);
    type First = DocumentURI;
    type Extra = ();
    #[cfg(feature="csr")]
    fn as_params(_:&Self::Extra) -> Cow<'static,str> {"".into()}
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[inline]
    fn call<R>(uri:DocumentURI,_:(),f:&Self::DeTupledFun<R>) -> server_fun_ret!(R) {
        f(Some(uri),None,None,None,None,None)
    }
}


#[cfg(all(feature="csr",not(any(feature="hydrate",feature="ssr"))))]
type SymURIArgs = SymbolURI;
#[cfg(any(feature="hydrate",feature="ssr"))]
type SymURIArgs = (Option<SymbolURI>,Option<ArchiveId>,Option<String>,Option<String>,Option<String>);
impl ServerFunArgs for SymURIArgs {
    #[cfg(any(feature="hydrate",feature="ssr"))]
    type DeTupledFun<R> = server_fun!(@SYMURI => R);
    type First = SymbolURI;
    type Extra = ();
    #[cfg(feature="csr")]
    fn as_params(_:&Self::Extra) -> Cow<'static,str> {"".into()}
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[inline]
    fn call<R>(uri:SymbolURI,_:(),f:&Self::DeTupledFun<R>) -> server_fun_ret!(R) {
        f(Some(uri),None,None,None,None)
    }
}


#[cfg(all(feature="csr",not(any(feature="hydrate",feature="ssr"))))]
type LOArgs = (SymbolURI,bool);
#[cfg(any(feature="hydrate",feature="ssr"))]
type LOArgs = (Option<SymbolURI>,Option<ArchiveId>,Option<String>,Option<String>,Option<String>,bool);
impl ServerFunArgs for LOArgs {
    #[cfg(any(feature="hydrate",feature="ssr"))]
    type DeTupledFun<R> = server_fun!(@SYMURI,bool => R);
    type First = SymbolURI;
    type Extra = bool;
    #[cfg(feature="csr")]
    fn as_params(b:&Self::Extra) -> Cow<'static,str> {format!("&exercises={b}").into()}
    #[cfg(any(feature="hydrate",feature="ssr"))]
    #[inline]
    fn call<R>(uri:SymbolURI,b:bool,f:&Self::DeTupledFun<R>) -> server_fun_ret!(R) {
        f(Some(uri),None,None,None,None,b)
    }
}

struct Cache<
    T:ServerFunArgs,
    V:Clone+for<'de> serde::Deserialize<'de>
> {
    #[cfg(any(feature="hydrate",feature="csr"))]
    cache:immt_utils::parking_lot::Mutex<immt_utils::prelude::HMap<T::First,V>>,
    #[cfg(feature="csr")]
    url_frag:&'static str,
    #[cfg(any(feature="hydrate",feature="ssr"))]
    getter:std::sync::OnceLock<T::DeTupledFun<V>>,
    #[cfg(feature="ssr")]
    phantom:std::marker::PhantomData<(T::First,V)>,
    #[cfg(all(feature="csr",not(feature="ssr")))]
    phantom:std::marker::PhantomData<T>
}

impl<
    T:ServerFunArgs,
    V:Clone+for<'de> serde::Deserialize<'de>
> Cache<T,V> {

    #[allow(unused_variables)]
    fn new(frag:&'static str) -> Self {
        Self {
            #[cfg(any(feature="hydrate",feature="csr"))]
            cache:immt_utils::parking_lot::Mutex::new(immt_utils::prelude::HMap::default()),
            #[cfg(feature="csr")]
            url_frag:frag,
            #[cfg(any(feature="hydrate",feature="ssr"))]
            getter:std::sync::OnceLock::new(),
            #[cfg(feature="ssr")]
            phantom:std::marker::PhantomData,
            #[cfg(all(feature="csr",not(feature="ssr")))]
            phantom:std::marker::PhantomData
        }
    }

    #[cfg(feature="csr")]
    #[inline]
    fn url(&self,uri:&str,extra:Cow<'static,str>) -> String {
        format!("{}/content/{}?uri={}{extra}",
            server_config.server_url.lock(),
            self.url_frag,
            urlencoding::encode(uri)
        )
    }

    /// #### Errors
    /// #### Panics
    #[allow(unreachable_code)]
    pub async fn call(&self,key:T::First,extra:T::Extra) -> Result<V,String> {
        #[cfg(any(feature="hydrate",feature="csr"))]
        {{
            let cache = self.cache.lock();
            if let Some(v) = std::collections::HashMap::get(&*cache, &key) {
                return Ok(v.clone())
            }
        }}
        
        #[cfg(feature="csr")]
        {
            let ret: Result<V,_> = ServerConfig::remote(self.url(&key.to_string(),T::as_params(&extra))).await;
            if let Ok(v) = &ret {
                let mut cache = self.cache.lock();
                cache.insert( key.clone(), v.clone());
            }
            return ret
        }

        #[cfg(any(feature="hydrate",feature="ssr"))]
        {
            let Some(f) = self.getter.get() else {
                panic!("Uninitialized shtml-viewer!!")
            };
            return match T::call(key.clone(),extra,f).await {
                Ok(r) => {
                    #[cfg(feature="hydrate")]
                    { std::collections::HashMap::insert(&mut self.cache.lock(),key,r.clone()); }
                    Ok(r)
                }
                Err(e) => Err(e.to_string())
            }
            //return T::call(key,extra,f).await.map_err(|e| e.to_string());
            //return f(Some(URI::Narrative(doc.into())),None,None,None,None,None,None,None,None).await.map_err(|e| e.to_string());
        }
    }
}

pub struct ServerConfig {
    #[cfg(feature="csr")]
    pub server_url:immt_utils::parking_lot::Mutex<String>,
    get_full_doc:Cache<DocURIArgs,(DocumentURI,Vec<CSS>,String)>,
    get_fragment:Cache<URIArgs,(Vec<CSS>,String)>,
    #[cfg(feature="omdoc")]
    get_omdoc:Cache<URIArgs,(Vec<CSS>,AnySpec)>,
    get_toc:Cache<DocURIArgs,(Vec<CSS>,Vec<TOCElem>)>,
    get_los:Cache<LOArgs,Vec<(DocumentElementURI,LOKind)>>,
    #[cfg(feature="omdoc")]
    get_notations:Cache<URIArgs,Vec<(DocumentElementURI,immt_ontology::narration::notations::Notation)>>,
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
    #[inline]
    pub async fn inputref(&self,doc:DocumentURI) -> Result<(Vec<CSS>,String),String> {
        self.get_fragment.call(URI::Narrative(doc.into()),()).await
    }

    /// #### Errors
    /// #### Panics
    #[inline]
    pub async fn paragraph(&self,doc:DocumentElementURI) -> Result<(Vec<CSS>,String),String> {
        self.get_fragment.call(URI::Narrative(doc.into()),()).await
    }

    /// #### Errors
    /// #### Panics
    #[inline]
    pub async fn definition(&self,uri:SymbolURI) -> Result<(Vec<CSS>,String),String> {
        self.get_fragment.call(URI::Content(uri.into()),()).await
    }

    /// #### Errors
    /// #### Panics
    #[inline]
    pub async fn full_doc(&self,uri:DocumentURI) -> Result<(DocumentURI,Vec<CSS>,String),String> {
        self.get_full_doc.call(uri,()).await
    }

    /// #### Errors
    /// #### Panics
    #[inline]
    pub async fn get_toc(&self,uri:DocumentURI) -> Result<(Vec<CSS>,Vec<TOCElem>),String> {
        self.get_toc.call(uri,()).await
    }

    /// #### Errors
    /// #### Panics
    #[inline]
    pub async fn get_los(&self,uri:SymbolURI,exercises:bool) -> Result<Vec<(DocumentElementURI,LOKind)>,String> {
        self.get_los.call(uri,exercises).await
    }

    /// #### Errors
    /// #### Panics
    #[cfg(feature="omdoc")]
    #[inline]
    pub async fn omdoc(&self,uri:immt_ontology::uris::URI) -> Result<(Vec<CSS>,AnySpec),String> {
        self.get_omdoc.call(uri,()).await
    }

    /// #### Errors
    /// #### Panics
    #[cfg(feature="omdoc")]
    #[inline]
    pub async fn notations(&self,uri:immt_ontology::uris::URI) -> Result<Vec<(DocumentElementURI,immt_ontology::narration::notations::Notation)>,String> {
        let ret = self.get_notations.call(uri,()).await;
        ret
    }

    #[cfg(feature="omdoc")]
    pub fn get_notation(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Option<immt_ontology::narration::notations::Notation> {
        #[cfg(any(feature="csr",feature="hydrate"))]
        {
            let lock = self.get_notations.cache.lock();
            lock.values()
                .flat_map(|v| v.iter())
                .find_map(|(u,n)| if u == uri {Some(n.clone())} else {None})
                //.expect("Notation not found; this should not happen")
        }
        #[cfg(not(any(feature="csr",feature="hydrate")))]
        { unreachable!()}
    }

    #[cfg(feature="omdoc")]
    pub async fn present(&self,t:immt_ontology::content::terms::Term) -> Result<String,String> {
        use immt_ontology::content::terms::Term;
        use immt_ontology::narration::notations::{Notation,Presenter,PresentationError};
        use immt_ontology::uris::{ContentURI, NarrativeURI, URIOrRefTrait, URIRef, URIRefTrait};
        use immt_utils::vecmap::VecSet;
        #[cfg(any(feature="csr",feature="hydrate"))]
        {
            let syms : VecSet<_> = t.uri_iter().map(URIRef::owned).collect();
            for s in syms {
                match &s {
                    URI::Content(ContentURI::Symbol(_)) =>
                        self.load_notations(s).await,
                    URI::Narrative(NarrativeURI::Element(_)) =>
                        self.load_notations(s).await,
                    _ => ()
                }
            }

            struct Pres<'p> {
                string:String,
                slf:&'p ServerConfig
            }
            impl std::fmt::Write for Pres<'_> {
                fn write_str(&mut self, s: &str) -> std::fmt::Result {
                    self.string.push_str(s);
                    Ok(())
                }
            }
            impl Presenter for Pres<'_> {
                type N = Notation;
                fn get_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {
                    let lock = self.slf.get_notations.cache.lock();
                    lock.get(&uri.as_uri().owned())
                        .and_then(|v| v.first()
                        .map(|(_,n)| n.clone())
                    )
                }
                fn get_op_notation(&mut self,uri:&SymbolURI) -> Option<Self::N> {
                    let lock = self.slf.get_notations.cache.lock();
                    lock.get(&uri.as_uri().owned())
                        .and_then(|v| 
                            v.iter().find_map(|(_,n)| if n.is_op() {Some(n.clone())} else {None})
                    )
                }
                fn get_variable_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {
                    let lock = self.slf.get_notations.cache.lock();
                    lock.get(&uri.as_uri().owned())
                        .and_then(|v| v.first()
                        .map(|(_,n)| n.clone())
                    )
                }
                fn get_variable_op_notation(&mut self,uri:&DocumentElementURI) -> Option<Self::N> {
                    let lock = self.slf.get_notations.cache.lock();
                    lock.get(&uri.as_uri().owned())
                        .and_then(|v| 
                            v.iter().find_map(|(_,n)| if n.is_op() {Some(n.clone())} else {None})
                    )
                }
                #[inline]
                fn in_text(&self) -> bool {false}
            }
            let mut p = Pres { string:String::new(), slf:self };
            return t.present(&mut p).map(|()| p.string).map_err(|e| e.to_string())
        }
        #[cfg(feature="ssr")]
        {
            todo!()
        }
    }

    #[cfg(all(feature="omdoc",any(feature="csr",feature="hydrate")))]
    #[inline]
    async fn load_notations(&self,uri:URI) {
        if self.get_notations.cache.lock().get(&uri).is_some() { return }
        let _ = self.get_notations.call(uri,()).await;
    }

    #[cfg(any(feature="hydrate",feature="ssr"))]
    pub fn initialize(
      inputref:server_fun!(@URI => (Vec<CSS>,String)),
      full_doc:server_fun!(@DOCURI => (DocumentURI,Vec<CSS>,String)),
      toc:server_fun!(@DOCURI => (Vec<CSS>,Vec<TOCElem>)),
      omdoc:server_fun!(@URI => (Vec<CSS>,AnySpec)),
      los:server_fun!(@SYMURI,bool => Vec<(DocumentElementURI,LOKind)>),
      notations:server_fun!(@URI => Vec<(DocumentElementURI,immt_ontology::narration::notations::Notation)>),
    ) {
      let _ = server_config.get_fragment.getter.set(inputref);
      let _ = server_config.get_omdoc.getter.set(omdoc);
      let _ = server_config.get_full_doc.getter.set(full_doc);
      let _ = server_config.get_toc.getter.set(toc);
      let _ = server_config.get_los.getter.set(los);
      let _ = server_config.get_notations.getter.set(notations);
    }
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
        #[cfg(feature="csr")]
        server_url:immt_utils::parking_lot::Mutex::new(DEFAULT_SERVER_URL.to_string()),
        get_fragment:Cache::new("fragment"),
        get_full_doc:Cache::new("document"),
        get_toc:Cache::new("toc"),
        get_los:Cache::new("los"),
        #[cfg(feature="omdoc")]
        get_omdoc:Cache::new("omdoc"),
        #[cfg(feature="omdoc")]
        get_notations:Cache::new("notations"),
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
    async fn remote<T:for<'a> serde::Deserialize<'a>>(url:String) -> Result<T,String> {
        send_wrapper::SendWrapper::new(Box::pin(async move {
        reqwasm::http::Request::get(&url).send().await.map_err(|e| e.to_string())?
            .json::<T>().await.map_err(|e| e.to_string())
        })).await
    }
    /*
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
     */
}