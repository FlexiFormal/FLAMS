use std::{collections::hash_map::Entry, path::Path};

use async_lsp::{lsp_types as lsp, ClientSocket, LanguageClient};
use immt_ontology::uris::DocumentURI;
use immt_stex::{quickparse::stex::{DiagnosticLevel, STeXAnnot, STeXDiagnostic, STeXParseData, STeXParseDataI}, OutputCont, RusTeX};
use immt_system::{backend::{archives::LocalArchive, AnyBackend, Backend, GlobalBackend, TemporaryBackend}, formats::OMDocResult};
use immt_utils::{prelude::{HMap, TreeChildIter}, sourcerefs::{LSPLineCol, SourceRange}};
use smallvec::SmallVec;

use crate::{annotations::to_diagnostic, capabilities::STeXSemanticTokens, documents::LSPDocument, ClientExt, IsLSPRange, LSPStore, ProgressCallbackClient, ProgressCallbackServer};

#[derive(Clone)]
pub enum DocOrData {
  Doc(LSPDocument),
  Data(STeXParseData)
}


#[derive(Default,Clone)]
pub struct LSPState {
  pub(crate) documents: triomphe::Arc<parking_lot::RwLock<HMap<lsp::Url,DocOrData>>>,
  rustex: triomphe::Arc<std::sync::OnceLock<RusTeX>>,
  backend:TemporaryBackend
}
impl LSPState {

  #[inline]#[must_use]
  pub const fn backend(&self) -> &TemporaryBackend {
    &self.backend
  }

  #[must_use]#[inline]
  pub fn rustex(&self) -> &RusTeX {
    self.rustex.get_or_init(RusTeX::get)
  }

  pub fn build_html(&self,uri:&lsp::Url,client:&mut ClientSocket) -> Option<DocumentURI> {
    let Some(DocOrData::Doc(doc)) = self.documents.read().get(uri).cloned() else {return None };
    let path = uri.to_file_path().ok()?;
    let doc_uri = doc.document_uri().cloned()?;
    if doc.html_up_to_date() {return Some(doc_uri)};
    if doc.relative_path().is_none() {return None };
    let engine = self.rustex().builder()
      .set_sourcerefs(true);
    let engine = doc.with_text(|text| engine.set_string(&path,text))?;
    ProgressCallbackServer::with(client.clone(), format!("Building {}",uri.as_str().rsplit_once('/').unwrap_or_else(|| unreachable!()).1), None, move |progress| {
      let out = ClientOutput(std::cell::RefCell::new(progress));
      let(res,mut old) = engine.set_output(out).run();
      doc.set_html_up_to_date();
      //let progress: ClientOutput = old.take_output().unwrap_or_else(|| unreachable!());
      if let Some(e) = &res.error {
        let _ = client.log_message(lsp::LogMessageParams {
          typ: lsp::MessageType::ERROR,
          message: format!("RusTeX Error: {e}")
        });
        let mut lock = doc.annotations.lock();
        lock.diagnostics.insert(STeXDiagnostic {
          level: DiagnosticLevel::Error,
          message: format!("RusTeX Error: {e}"),
          range: SourceRange::default()
        });
        let _ = client.publish_diagnostics(lsp::PublishDiagnosticsParams {
          uri:uri.clone(),version:None,diagnostics:lock.diagnostics.iter().map(to_diagnostic).collect()
        });
        drop(lock);
        None
      } else {
        let html = res.to_string();
        let rel_path = doc.relative_path().unwrap_or_else(|| unreachable!());
        match immt_shtml::build_shtml(&AnyBackend::Temp(self.backend.clone()), &html, doc_uri.clone(), rel_path) {
          Ok((OMDocResult{document,html,modules},_)) => {
            self.backend.add_html(document.uri.clone(), html);
            for m in modules {
              let m = m.check(&mut self.backend.as_checker());
              self.backend.add_module(m);
            }
            let document = document.check(&mut self.backend.as_checker());
            self.backend.add_document(document);
            old.memorize(self.rustex());
            Some(doc_uri)
          }
          Err(e) => {
            //let progress: ClientOutput = old.take_output().unwrap_or_else(|| unreachable!());
            let _ = client.log_message(lsp::LogMessageParams {
              typ: lsp::MessageType::ERROR,
              message: format!("SHTML Error: {e}")
            });
            None
          }
        }
      }
    })
  }

  #[inline]
  pub fn build_html_and_notify(&self,uri:&lsp::Url,mut client:ClientSocket) {
    if let Some(uri) = self.build_html(uri, &mut client) {
      client.html_result(&uri)
    }
  }

  pub fn load(&self,p:&Path,uri:&DocumentURI,and_then:impl FnOnce(&STeXParseData)) {
    let Some(lsp_uri) = lsp::Url::from_file_path(p).ok() else {return};
    if self.documents.read().get(&lsp_uri).is_some() { return }
    let state = LSPStore::<false>::new(self.clone());
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
      let store = LSPStore::new(self.clone());
      let _ = tokio::task::spawn_blocking(move || d.compute_annots(store));
    }
    self.documents.write().insert(uri,DocOrData::Doc(doc));
  }

  #[must_use]
  pub fn get(&self,uri:&lsp::Url) -> Option<LSPDocument> {
    if let Some(DocOrData::Doc(doc)) = self.documents.read().get(uri) {
      Some(doc.clone())
    } else { None }
  }

}

struct ClientOutput(std::cell::RefCell<ProgressCallbackServer>);
impl OutputCont for ClientOutput {
  fn message(&self,_:String) {}
  fn errmessage(&self,text:String) {
      let _ = self.0.borrow_mut().client_mut().show_message(lsp::ShowMessageParams {
        typ:lsp::MessageType::ERROR,
        message:text
      });
  }
  fn file_open(&self,text:String) {
      self.0.borrow().update(text, None);
  }
  fn file_close(&self,_text:String) {}
  fn write_16(&self,_text:String) {}
  fn write_17(&self,_text:String) {}
  fn write_18(&self,_text:String) {}
  fn write_neg1(&self,_text:String) {}
  fn write_other(&self,_text:String) {}

  #[inline]
  fn as_any(self:Box<Self>) -> Box<dyn std::any::Any> {
      self
  }
}

