mod implementation;
pub mod documents;
pub mod capabilities;
#[cfg(feature="ws")]
pub mod ws;

use std::{collections::hash_map::Entry, path::Path};

use async_lsp::{lsp_types as lsp, ClientSocket, LanguageClient};
pub use async_lsp;
use capabilities::STeXSemanticTokens;
use documents::LSPDocument;
use immt_ontology::uris::{ArchiveURITrait, DocumentURI, PathURITrait, URIRefTrait, URIWithLanguage};
use immt_stex::quickparse::stex::{rules::STeXModuleStore, DiagnosticLevel, STeXAnnot, STeXDiagnostic, STeXParseData};
use immt_system::backend::{archives::LocalArchive, AnyBackend, Backend, GlobalBackend};
use immt_utils::{prelude::HMap, sourcerefs::{LSPLineCol, SourceRange}};
use immt_utils::prelude::TreeChildIter;
use smallvec::SmallVec;

pub trait IMMTLSPServer {
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

  pub fn get_progress(&self,tk: lsp::ProgressToken) -> ProgressCallbackClient {
    ProgressCallbackClient {
      client:self.inner.client().clone(),
      token:tk
    }
  }
}

enum DocOrData {
  Doc(LSPDocument),
  Data(STeXParseData)
}

#[derive(Clone)]
pub struct LSPStore<const FULL:bool>(pub LSPState);
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
        let docs = self.0.documents.read();
        match docs.get(&lsp_uri) {
          Some(DocOrData::Data(d)) => return Some(d.clone()),
          Some(DocOrData::Doc(d)) => return Some(d.annotations.clone()),
          None => ()
        }
        drop(docs);
        let uri = module.uri.as_path().owned() & (module.uri.name().clone(), module.uri.language());
        self.clone().load(p,&uri).inspect(|ret| {
          let mut docs = self.0.documents.write();
          if let Entry::Vacant(e) = docs.entry(lsp_uri) {
            e.insert(DocOrData::Data(ret.clone()));
          }
        })
      })
  }
}

#[derive(Default,Clone)]
pub struct LSPState {
  documents: triomphe::Arc<parking_lot::RwLock<HMap<lsp::Url,DocOrData>>>
}
impl LSPState {
  pub fn load(&self,p:&Path,uri:&DocumentURI,and_then:impl FnOnce(&STeXParseData)) {
    let Some(lsp_uri) = lsp::Url::from_file_path(p).ok() else {return};
    if self.documents.read().get(&lsp_uri).is_some() { return }
    let state = LSPStore::<false>(self.clone());
    if let Some(ret) = state.load(p, uri) {
      and_then(&ret);
      let mut docs = self.documents.write();
      if let Entry::Vacant(e) = docs.entry(lsp_uri) {
        e.insert(DocOrData::Data(ret));
      }
    }
  }

  #[allow(clippy::let_underscore_future)]
  pub fn insert(&self,uri:lsp::Url,doc:String) {
    let doc = LSPDocument::new(doc,uri.clone());
    if doc.has_annots() {
      let d = doc.clone();
      let store = LSPStore(self.clone());
      let _ = tokio::task::spawn_blocking(move || d.compute_annots(store));
    }
    self.documents.write().insert(uri,DocOrData::Doc(doc));
  }

  fn get(&self,uri:&lsp::Url) -> Option<LSPDocument> {
    if let Some(DocOrData::Doc(doc)) = self.documents.read().get(uri) {
      Some(doc.clone())
    } else { None }
  }

  fn get_diagnostics(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=lsp::DocumentDiagnosticReportResult>> {
    fn default() -> lsp::DocumentDiagnosticReportResult { lsp::DocumentDiagnosticReportResult::Report(
      lsp::DocumentDiagnosticReport::Full(
          lsp::RelatedFullDocumentDiagnosticReport::default()
      )
    )}
    let d = self.get(uri)?;
    let store = LSPStore(self.clone());
    Some(async move { 
      d.with_annots(store,|data| {
        let diags = &data.diagnostics;
        let r = lsp::DocumentDiagnosticReportResult::Report(
          lsp::DocumentDiagnosticReport::Full(
            lsp::RelatedFullDocumentDiagnosticReport {
              related_documents:None,
              full_document_diagnostic_report:lsp::FullDocumentDiagnosticReport {
                result_id:None,
                items:diags.iter().map(to_diagnostic).collect()
              }
            }
          )
        );
        tracing::trace!("diagnostics: {:?}",r);
        if let Some(p) = progress { p.finish() }
        r
      }).await.unwrap_or_else(default)
    })
  }

  fn get_symbols(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<lsp::DocumentSymbolResponse>>> {
    let d = self.get(uri)?;
    let store = LSPStore(self.clone());
    Some(d.with_annots(store,|data| {
      let r = lsp::DocumentSymbolResponse::Nested(to_symbols(&data.annotations));
      tracing::trace!("document symbols: {:?}",r);
      if let Some(p) = progress { p.finish() }
      r
    }))
  }

  fn get_links(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<Vec<lsp::DocumentLink>>>> {
    let d = self.get(uri)?;
    let da = d.archive().cloned();
    let store = LSPStore(self.clone());
    Some(d.with_annots(store,move |data| {
      let mut ret = Vec::new();
      for e in <std::slice::Iter<'_,STeXAnnot> as TreeChildIter<STeXAnnot>>::dfs(data.annotations.iter()) {
        match e {
          STeXAnnot::Inputref { archive, filepath, range,.. } => {
            let Some(a) = archive.as_ref().map_or_else(
              || da.as_ref().map(ArchiveURITrait::archive_id),
              |(a,_)| Some(a)
            ) else {continue};
            let Some(path) = GlobalBackend::get().with_local_archive(a, |a| a.map(LocalArchive::source_dir)) else {
              continue
            };
            let path = filepath.0.split('/').fold(path, |p,s| p.join(s));
            let Some(lsp_uri) = lsp::Url::from_file_path(path).ok() else {continue};
            ret.push(lsp::DocumentLink {
              range:to_range(*range),
              target:Some(lsp_uri),
              tooltip:None,
              data:None
            });
          }
          STeXAnnot::SetMetatheory { .. } => (),
          _ => ()
        }
      }
      tracing::info!("document links: {:?}",ret);
      if let Some(p) = progress { p.finish() }
      ret
    }))
  }

  fn get_semantic_tokens(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>,range:Option<lsp::Range>) -> Option<impl std::future::Future<Output=Option<lsp::SemanticTokens>>> {
    let range = range.map(from_range);
    let d = self.get(uri)?;
    let store = LSPStore(self.clone());
    Some(d.with_annots(store, |data| {
      let mut ret = Vec::new();
      let mut curr = (0u32,0u32);
      macro_rules! make{
        ($rng:expr => $tk:ident) => {
          let delta_line = $rng.start.line - curr.0;
          let delta_start = if delta_line == 0 { $rng.start.col - curr.1 } else { $rng.start.col };
          curr = ($rng.start.line,$rng.start.col);
          let length = $rng.end.col - $rng.start.col;
          ret.push(lsp::SemanticToken {
            delta_line,delta_start,length,
            token_type:STeXSemanticTokens::$tk,
            token_modifiers_bitset:0
          });
        };
        ($rng:expr =>> $tk:expr) => {
          let delta_line = $rng.start.line - curr.0;
          let delta_start = if delta_line == 0 { $rng.start.col - curr.1 } else { $rng.start.col };
          curr = ($rng.start.line,$rng.start.col);
          let length = $rng.end.col - $rng.start.col;
          ret.push(lsp::SemanticToken {
            delta_line,delta_start,length,
            token_type:$tk,
            token_modifiers_bitset:0
          });
        }
      }

      for e in <std::slice::Iter<'_,STeXAnnot> as TreeChildIter<STeXAnnot>>::dfs(data.annotations.iter()) {
        match e {
          STeXAnnot::Symdecl { main_name_range, name_ranges, tp, df, token_range, .. } => {
            make!(token_range => DECLARATION);
            make!(main_name_range => NAME);
            let mut props = SmallVec::<_,3>::new();
            if let Some((n,t)) = name_ranges {
              props.push((n,t,Some(STeXSemanticTokens::NAME)));
            }
            if let Some((k,v,_)) = tp {
              props.push((k,v,None));
            }
            if let Some((k,v,_)) = df {
              props.push((k,v,None));
            }
            props.sort_by_key(|p| (p.0.start.line,p.0.start.col));
            for (k,v,t) in props {
              make!(k => KEYWORD);
              if let Some(t) = t { make!(v =>> t); }
            }
          }
          STeXAnnot::Module { uri, name_range, sig, meta_theory, full_range, smodule_range, children } => {
            make!(smodule_range => DECLARATION);
            make!(name_range => NAME);
          }
          _ => ()
        }
      }


      if let Some(p) = progress { p.finish() }
      lsp::SemanticTokens {
        result_id:None,
        data:ret
      }
    }))
  }

}
#[allow(deprecated)]
fn to_symbols(v:&[STeXAnnot]) -> Vec<lsp::DocumentSymbol> {
  let mut curr = v.iter();
  let mut ret = Vec::new();
  let mut stack = Vec::new();
  //tracing::info!("Annotations: {v:?}");
  loop {
    if let Some(e) = curr.next() { match e {
      STeXAnnot::Module { uri, full_range, name_range, children,.. } =>{
        let old = std::mem::replace(&mut curr, children.iter());
        stack.push((old,lsp::DocumentSymbol {
          name: uri.to_string(),
          detail:None,
          kind:lsp::SymbolKind::MODULE,
          tags:None,
          deprecated:None,
          range:to_range(*full_range),
          selection_range:to_range(*name_range),
          children:Some(std::mem::take(&mut ret))
        }));
      }
      STeXAnnot::Symdecl { uri, macroname, main_name_range, name_ranges, full_range, tp, df,.. } => {
        let sym = lsp::DocumentSymbol {
          name: uri.to_string(),
          detail:None,
          kind:lsp::SymbolKind::OBJECT,
          tags:None,
          deprecated:None,
          range:to_range(*full_range),
          selection_range:to_range(*main_name_range),
          children:None
        };
        ret.push(sym);
        /*match (tp,df) {
          (None,None) =>
        }*/
      }
      STeXAnnot::ImportModule { module, full_range,.. } => {
        ret.push(lsp::DocumentSymbol {
          name: format!("import@{}",module.uri),
          detail:Some(module.uri.to_string()),
          kind:lsp::SymbolKind::PACKAGE,
          tags:None,
          deprecated:None,
          range:to_range(*full_range),
          selection_range:to_range(*full_range),
          children:None
        });
      }
      STeXAnnot::UseModule { module, full_range,.. } => {
        ret.push(lsp::DocumentSymbol {
          name: format!("usemodule@{}",module.uri),
          detail:Some(module.uri.to_string()),
          kind:lsp::SymbolKind::PACKAGE,
          tags:None,
          deprecated:None,
          range:to_range(*full_range),
          selection_range:to_range(*full_range),
          children:None
        });
      }
      STeXAnnot::SetMetatheory { module, full_range,.. } => {
        ret.push(lsp::DocumentSymbol {
          name: format!("metatheory@{}",module.uri),
          detail:Some(module.uri.to_string()),
          kind:lsp::SymbolKind::NAMESPACE,
          tags:None,
          deprecated:None,
          range:to_range(*full_range),
          selection_range:to_range(*full_range),
          children:None
        });
      }
      STeXAnnot::Inputref { archive, filepath, range,.. } => {
        ret.push(lsp::DocumentSymbol {
          name: archive.as_ref().map_or_else(
            || format!("inputref@{}",filepath.0),
          |(a,_)| format!("inputref@[{a}]{}",filepath.0)),
          detail:None,
          kind:lsp::SymbolKind::PACKAGE,
          tags:None,
          deprecated:None,
          range:to_range(*range),
          selection_range:to_range(*range),
          children:None
        });
      }
    }} else if let Some((i,mut s)) = stack.pop() {
      curr = i;
      std::mem::swap(&mut ret, s.children.as_mut().unwrap_or_else(|| unreachable!()));
      ret.push(s);
    } else { break }
  }
  //tracing::info!("Returns: {ret:?}");
  ret
}

const fn to_range(range:SourceRange<LSPLineCol>) -> lsp::Range {
  lsp::Range { start: lsp::Position {
    line:range.start.line,
    character:range.start.col
  }, end: lsp::Position {
    line:range.end.line,
    character:range.end.col
  } }
}

const fn from_range(range:lsp::Range) -> SourceRange<LSPLineCol> {
  SourceRange {
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
  pub fn client(&self) -> ClientSocket { self.client.clone() }

  pub fn with<R>(client:ClientSocket,title:String, total:Option<u32>,f:impl FnOnce(&Self) -> R) -> R {
    let slf = Self::new(client,title,total);
    let r = f(&slf);
    drop(slf);
    r
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

#[must_use]
pub fn to_diagnostic(diag:&STeXDiagnostic) -> lsp::Diagnostic {
  lsp::Diagnostic {
    range: to_range(diag.range),
    severity:Some(match diag.level {
      DiagnosticLevel::Error => lsp::DiagnosticSeverity::ERROR,
      DiagnosticLevel::Info => lsp::DiagnosticSeverity::INFORMATION,
      DiagnosticLevel::Warning => lsp::DiagnosticSeverity::WARNING,
      DiagnosticLevel::Hint => lsp::DiagnosticSeverity::HINT
    }),
    code:None,
    code_description:None,
    source:None,
    message:diag.message.clone(),
    related_information:None,
    tags:None,
    data:None
  }
}