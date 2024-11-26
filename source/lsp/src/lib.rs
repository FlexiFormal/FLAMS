mod implementation;
pub mod annotations;
pub mod documents;
pub mod capabilities;
pub mod state;
#[cfg(feature="ws")]
pub mod ws;

use std::{collections::hash_map::Entry, path::Path};

use async_lsp::{lsp_types as lsp, ClientSocket, LanguageClient};
pub use async_lsp;
use immt_ontology::uris::{DocumentURI, PathURITrait, URIRefTrait, URIWithLanguage};
use immt_stex::quickparse::stex::{rules::STeXModuleStore, STeXParseData};
use immt_system::backend::{AnyBackend, GlobalBackend};
use immt_utils::sourcerefs::{LSPLineCol, SourceRange};
use implementation::HTMLRequest;
use state::{DocOrData, LSPState};

pub trait ClientExt {
  fn html_result(&self,uri:&DocumentURI);
}

struct HTMLResult;
impl lsp::notification::Notification for HTMLResult {
    type Params = String;
    const METHOD : &str = "immt/htmlResult";
}

impl ClientExt for ClientSocket {
  #[inline]
  fn html_result(&self,uri:&DocumentURI) {
    let _ = self.notify::<HTMLResult>(uri.to_string());
  }
}


pub trait IMMTLSPServer:'static {
  fn client_mut(&mut self) -> &mut ClientSocket;
  fn client(& self) -> &ClientSocket;
  fn state(&self) -> &LSPState;
  #[inline]
  fn initialized(&mut self) {}
  #[inline]
  fn initialize<I:Iterator<Item=(String,lsp::Url)> + Send + 'static>(&mut self,_workspaces:I) {}
}


pub struct ServerWrapper<T:IMMTLSPServer> {
  pub inner:T
}
impl <T:IMMTLSPServer> ServerWrapper<T> {
  #[inline]
  pub const fn new(inner:T) -> Self {
    Self { inner }
  }

  pub fn router(self) -> async_lsp::router::Router<Self> {
    let mut r = async_lsp::router::Router::from_language_server(self);
    r.request::<HTMLRequest,_>(Self::html_request);
    //r.request(handler)
    r
  }

  pub fn get_progress(&self,tk: lsp::ProgressToken) -> ProgressCallbackClient {
    ProgressCallbackClient {
      client:self.inner.client().clone(),
      token:tk
    }
  }
}


#[derive(Clone)]
pub struct LSPStore<const FULL:bool> {
  state:LSPState,
  cycles:Vec<DocumentURI>
}
impl<const FULL:bool> LSPStore<FULL> {
  #[inline]
  pub fn new(state:LSPState) -> Self {
    Self {
      state,
      cycles:Vec::new()
    }
  }
}

impl<const FULL:bool> LSPStore<FULL> {
  fn load(self,p:&Path,uri:&DocumentURI) -> Option<STeXParseData> {
    let text = std::fs::read_to_string(p).ok()?;
    Some(immt_stex::quickparse::stex::quickparse(
      uri,&text, p,
      &AnyBackend::Global(GlobalBackend::get()),
      self
    ).lock())
  }
}
impl<const FULL:bool> STeXModuleStore for LSPStore<FULL> {
  const FULL:bool = FULL;
  fn get_module(&mut self,module:&immt_stex::quickparse::stex::rules::ModuleReference) -> Option<STeXParseData> {
      module.full_path.as_ref().and_then(|p| {
        let lsp_uri = lsp::Url::from_file_path(p).ok()?;
        let docs = self.state.documents.read();
        match docs.get(&lsp_uri) {
          Some(DocOrData::Data(d)) => return Some(d.clone()),
          Some(DocOrData::Doc(d)) => return Some(d.annotations.clone()),
          None => ()
        }
        drop(docs);
        let uri = module.doc_uri()?;
        
        if self.cycles.contains(&uri) { 
          let mut str = String::new();
          for c in &self.cycles {
            str.push_str(&format!("{c}\n => "));
          }
          str.push_str(&format!("{uri}"));
          tracing::error!("Importmodule cycle:\n{str}");
          return None
        }
        self.cycles.push(uri.clone());
        let r = self.clone().load(p,&uri).inspect(|ret| {
          let mut docs = self.state.documents.write();
          if let Entry::Vacant(e) = docs.entry(lsp_uri) {
            e.insert(DocOrData::Data(ret.clone()));
          }
        });
        self.cycles.pop();
        r
        /*
        let slf = self.clone();
        let p = p.clone();
        // avoid stack overflows
        std::thread::spawn(move || {
          slf.clone().load(&p,&uri).inspect(|ret| {
            let mut docs = slf.0.documents.write();
            if let Entry::Vacant(e) = docs.entry(lsp_uri) {
              e.insert(DocOrData::Data(ret.clone()));
            }
          })
        }).join().unwrap_or_else(|_| unreachable!())
         */
      })
  }
}

pub trait IsLSPRange {
  fn into_range(self) -> lsp::Range;
  fn from_range(range:lsp::Range) -> Self;
}

impl IsLSPRange for SourceRange<LSPLineCol> {
  fn into_range(self) -> lsp::Range {
    lsp::Range { start: lsp::Position {
      line:self.start.line,
      character:self.start.col
    }, end: lsp::Position {
      line:self.end.line,
      character:self.end.col
    } }
  }
  fn from_range(range:lsp::Range) -> Self {
    Self {
      start:LSPLineCol {
        line:range.start.line,
        col:range.start.character
      },
      end:LSPLineCol {
        line:range.end.line,
        col:range.end.character
      }
    }
  }
}

pub struct ProgressCallbackClient {
  client:ClientSocket,
  token: async_lsp::lsp_types::ProgressToken
}

pub struct ProgressCallbackServer {
  client:ClientSocket,
  token:String,
  progress:Option<parking_lot::Mutex<(u32,u32)>>
}
impl ProgressCallbackServer {

  #[inline]
  pub fn client_mut(&mut self) -> &mut ClientSocket { &mut self.client }

  #[inline]
  pub fn client(&self) -> ClientSocket { self.client.clone() }

  pub fn with<R>(client:ClientSocket,title:String, total:Option<u32>,f:impl FnOnce(Self) -> R) -> R {
    let slf = Self::new(client,title,total);
    f(slf)
  }

  #[must_use]#[allow(clippy::let_underscore_future)]
  pub fn new(mut client:ClientSocket,title:String, total:Option<u32>) -> Self {
    let tk = immt_utils::hashstr("progress_", &title);//TOKEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let token = async_lsp::lsp_types::ProgressToken::String(tk.clone());
    let f = client.work_done_progress_create(
      lsp::WorkDoneProgressCreateParams {
          token
        }
      );
    let mut c = client.clone();
    let tk2 = tk.clone();
    let _ = tokio::spawn(async move {
      let r = f.await;
      if let Err(e) = r {
        tracing::error!("Error: {}",e);
      } else {
        let _ = c.progress(async_lsp::lsp_types::ProgressParams {
          token:async_lsp::lsp_types::ProgressToken::String(tk2),
          value:async_lsp::lsp_types::ProgressParamsValue::WorkDone(
            async_lsp::lsp_types::WorkDoneProgress::Begin(
              async_lsp::lsp_types::WorkDoneProgressBegin {
                message:None,
                title,
                percentage:total.map(|_| 0),
                cancellable:None
              }
            )
          )
        });
      }
  });
    Self { client, token:tk, progress:total.map(|i| parking_lot::Mutex::new((0,i))) }
  }

  pub fn update(&self,message:String,add_step:Option<u32>) {
    let (message,percentage) = if let Some(i) = add_step {
      if let Some(lock) = self.progress.as_ref() {
        let mut lock = lock.lock();
        lock.0 += i;
        (format!("{}/{}:{message}",lock.0,lock.1),Some(100 * lock.0 / lock.1))
      } else {(message,None)}
    } else if let Some(lock) = self.progress.as_ref() {
      let lock = lock.lock();
      (format!("{}/{}:{message}",lock.0,lock.1),Some(100 * lock.0 / lock.1))
    } else {(message,None)};
    let token = async_lsp::lsp_types::ProgressToken::String(self.token.clone());
    //tracing::info!("updating progress {}",self.token);
    let _ = self.client.clone().progress(async_lsp::lsp_types::ProgressParams {
      token,
      value:async_lsp::lsp_types::ProgressParamsValue::WorkDone(
        async_lsp::lsp_types::WorkDoneProgress::Report(
          async_lsp::lsp_types::WorkDoneProgressReport {
            message:Some(message),
            percentage,
            cancellable:None
          }
        )
      )
    });
  }
}

impl Drop for ProgressCallbackServer {
  fn drop(&mut self) {
    let token = async_lsp::lsp_types::ProgressToken::String(std::mem::take(&mut self.token));
    let _ = self.client.progress(async_lsp::lsp_types::ProgressParams {
      token,
      value:async_lsp::lsp_types::ProgressParamsValue::WorkDone(
        async_lsp::lsp_types::WorkDoneProgress::End(
          async_lsp::lsp_types::WorkDoneProgressEnd {
            message:Some("done".to_string())
          }
        )
      )
    });
  }
}

impl ProgressCallbackClient {

  pub fn finish(mut self) {
    let _ = self.client.progress(async_lsp::lsp_types::ProgressParams {
      token:self.token,
      value:async_lsp::lsp_types::ProgressParamsValue::WorkDone(
        async_lsp::lsp_types::WorkDoneProgress::End(
          async_lsp::lsp_types::WorkDoneProgressEnd {
            message:Some("done".to_string())
          }
        )
      )
    });
  }

  #[allow(clippy::let_underscore_future)]
  pub fn finish_delay(self) {
    let _ = tokio::spawn(async move {
      tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.5)).await;
      self.finish();
    });
  }
}
