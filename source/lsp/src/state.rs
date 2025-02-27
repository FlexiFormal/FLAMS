use std::{collections::hash_map::Entry, path::{Path, PathBuf}};

use async_lsp::{lsp_types as lsp, ClientSocket, LanguageClient};
use flams_ontology::uris::{DocumentURI, URIRefTrait};
use flams_stex::{quickparse::stex::{DiagnosticLevel, STeXAnnot, STeXDiagnostic, STeXParseData, STeXParseDataI}, OutputCont, RusTeX};
use flams_system::{backend::{archives::{source_files::{SourceDir, SourceEntry}, Archive, LocalArchive}, AnyBackend, Backend, GlobalBackend, TemporaryBackend}, formats::OMDocResult};
use flams_utils::{prelude::{HMap, TreeChildIter}, sourcerefs::{LSPLineCol, SourceRange}, vecmap::OrdSet};
use smallvec::SmallVec;

use crate::{annotations::to_diagnostic, capabilities::STeXSemanticTokens, documents::LSPDocument, ClientExt, IsLSPRange, LSPStore, ProgressCallbackClient, ProgressCallbackServer};

#[derive(Clone)]
pub enum DocData {
  Doc(LSPDocument),
  Data(STeXParseData,bool)
}
impl DocData {
  pub fn merge(&mut self,other:Self) {
    fn merge_a(from:&mut STeXParseDataI,to:&mut STeXParseDataI) {
      to.dependencies = std::mem::take(&mut from.dependencies);
      /*for dep in std::mem::take(&mut from.dependents) {
        to.dependents.insert(dep);
      }*/
      for d in std::mem::take(&mut from.diagnostics) {
        to.diagnostics.insert(d);
      }
    }
    match (self,other) {
      (Self::Doc(d1),Self::Doc(mut d2)) => {
        //merge_a(&mut d1.annotations.lock(),&mut d2.annotations.lock());
        *d1 = d2;
      }
      (Self::Doc(d1),Self::Data(d2,_)) => {
        merge_a(&mut d2.lock(),&mut d1.annotations.lock());
      }
      (d2@Self::Data(_,_),Self::Doc(d1)) => {
        {
          let Self::Data(ref mut d2,_) = d2 else { unreachable!() };
          merge_a(&mut d2.lock(),&mut d1.annotations.lock());
        }
        *d2 = Self::Doc(d1)
      }
      (d1@Self::Data(_,false),Self::Data(d2,true)) => {
        {
          let Self::Data(ref mut d1,_) = d1 else { unreachable!() };
          //merge_a(&mut d1.lock(),&mut d2.lock());
        }
        *d1 = Self::Data(d2,true)
      }
      (Self::Data(d1,_),Self::Data(d2,_)) => {
        merge_a(&mut d2.lock(),&mut d1.lock());
      }
    }
  }
}

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub enum UrlOrFile {
  Url(lsp::Url),
  File(std::sync::Arc<Path>)
}
impl UrlOrFile {
  pub fn name(&self) -> &str {
    match self {
      Self::Url(u) => u.path().split('/').last().unwrap_or(""),
      Self::File(p) => p.file_name().and_then(|s| s.to_str()).unwrap_or("")
    }
  }
}
impl From<lsp::Url> for UrlOrFile {
  fn from(value: lsp::Url) -> Self {
    match value.to_file_path() {
      Ok(p) => Self::File(p.into()),
      Err(_) => Self::Url(value)
    }
  }
}
impl Into<lsp::Url> for UrlOrFile {
  fn into(self) -> lsp::Url {
    match self {
      Self::Url(u) => u,
      Self::File(p) => lsp::Url::from_file_path(p).unwrap()
    }
  }
}
impl std::fmt::Display for UrlOrFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Url(u) => u.fmt(f),
      Self::File(p) => p.display().fmt(f)
    }
  }
}


#[derive(Default,Clone)]
pub struct LSPState {
  pub(crate) documents: triomphe::Arc<parking_lot::RwLock<HMap<UrlOrFile,DocData>>>,
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
    self.rustex.get_or_init(|| RusTeX::get().unwrap_or_else(|()| {
      tracing::error!("Could not initialize RusTeX");
      panic!("Could not initialize RusTeX")
    }))
  }

  pub fn build_html(&self,uri:&UrlOrFile,client:&mut ClientSocket) -> Option<DocumentURI> {
    let Some(DocData::Doc(doc)) = self.documents.read().get(uri).cloned() else {return None };
    let UrlOrFile::File(path) = uri else {return None};//.to_file_path().ok()?;
    let doc_uri = doc.document_uri().cloned()?;
    if doc.html_up_to_date() {return Some(doc_uri)};
    if doc.relative_path().is_none() {return None };
    let engine = self.rustex().builder()
      .set_sourcerefs(true);
    let engine = doc.with_text(|text| engine.set_string(path,text))?;
    ProgressCallbackServer::with(client.clone(), format!("Building {}",uri.name()), None, move |progress| {
      let out = ClientOutput(std::cell::RefCell::new(progress));
      let(mut res,mut old) = engine.set_output(out).run();
      doc.set_html_up_to_date();
      {
        let mut lock = doc.annotations.lock();
        for (fnt,dt) in &res.font_data {
          if dt.web.is_none() {
            lock.diagnostics.insert(STeXDiagnostic {
              level: DiagnosticLevel::Warning,
              message: format!("Unknown web font for {fnt}"),
              range: SourceRange::default()
            });
            for (glyph,char) in &dt.missing.inner {
              lock.diagnostics.insert(STeXDiagnostic {
                level: DiagnosticLevel::Warning,
                message: format!("unknown unicode character for glyph {char} ({glyph}) in font {fnt}"),
                range: SourceRange::default()
              });
            }
          }
        }
      }
      //let progress: ClientOutput = old.take_output().unwrap_or_else(|| unreachable!());
      if let Some((ref e,ft)) = &mut res.error {
        let mut done = None;
        for ft in std::mem::take(ft) {
          let url = UrlOrFile::File(ft.file.into());
          if url == *uri { 
            done = Some(SourceRange { 
              start:LSPLineCol{line:ft.line,col:0},
              end:LSPLineCol{line:ft.line,col:ft.col} 
            });
          } else if let Some(dc) = self.documents.read().get(&url) {
            let data = match dc {
              DocData::Data(d,_) => d,
              DocData::Doc(d) => &d.annotations
            };
            let mut lock = data.lock();
            lock.diagnostics.insert(STeXDiagnostic {
              level: DiagnosticLevel::Error,
              message: format!("RusTeX Error: {e}"),
              range: SourceRange { 
                start:LSPLineCol{line:ft.line,col:0},
                end:LSPLineCol{line:ft.line,col:ft.col} 
              }
            });
            let _ = client.publish_diagnostics(lsp::PublishDiagnosticsParams {
              uri:url.clone().into(),version:None,diagnostics:lock.diagnostics.iter().map(to_diagnostic).collect()
            });
          }
        }
        let mut lock = doc.annotations.lock();
        lock.diagnostics.insert(STeXDiagnostic {
          level: DiagnosticLevel::Error,
          message: format!("RusTeX Error: {e}"),
          range: done.unwrap_or_default()
        });
        let _ = client.publish_diagnostics(lsp::PublishDiagnosticsParams {
          uri:uri.clone().into(),version:None,diagnostics:lock.diagnostics.iter().map(to_diagnostic).collect()
        });
        drop(lock);
        None
      } else {
        let html = res.to_string();
        let rel_path = doc.relative_path().unwrap_or_else(|| unreachable!());
        match flams_ftml::build_ftml(&AnyBackend::Temp(self.backend.clone()), &html, doc_uri.clone(), rel_path) {
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
            let mut lock = doc.annotations.lock();
            lock.diagnostics.insert(STeXDiagnostic {
              level: DiagnosticLevel::Error,
              message: format!("FTML Error: {e}"),
              range: SourceRange::default()
            });
            let _ = client.publish_diagnostics(lsp::PublishDiagnosticsParams {
              uri:uri.clone().into(),version:None,diagnostics:lock.diagnostics.iter().map(to_diagnostic).collect()
            });
            drop(lock);
            None
          }
        }
      }
    })
  }

  #[inline]
  pub fn build_html_and_notify(&self,uri:&UrlOrFile,mut client:ClientSocket) {
    if let Some(uri) = self.build_html(uri, &mut client) {
      client.html_result(&uri)
    }
  }

  pub fn relint_dependents(mut self,path:std::sync::Arc<Path>) { /*
    let docs = self.documents.read();
    let mut deps = vec![UrlOrFile::File(path.clone())];
    for (k,v) in docs.iter() {
      if matches!(k,UrlOrFile::File(p) if p == path) { continue }
      let d = match v {
        DocData::Doc(d) => &d.annotations,
        DocData::Data(d,_) => d
      };
      let lock = d.lock();
      if lock.dependencies.contains(&path) {
        deps.push(k.clone());
      }
    } */
  }
  /*
  fn relint_dependents_i(&mut self,path:std::sync::Arc<Path>,&mut v:Vec<UrlOrFile>) {
    let docs = self.documents.read();
    let mut deps = vec![UrlOrFile::File(path.clone())];
    for (k,v) in docs.iter() {
      if matches!(k,UrlOrFile::File(p) if p == path) { continue }
      let d = match v {
        DocData::Doc(d) => &d.annotations,
        DocData::Data(d,_) => d
      };
      let lock = d.lock();
      if lock.dependencies.contains(&path) {
        deps.push(k.clone());
      }
    }
  } */

  pub fn load_mathhubs(&self,client:ClientSocket) {
    let (_,t) = flams_utils::time::measure(move || {
      let mut files = Vec::new();

      for a in GlobalBackend::get().all_archives().iter() {
        if let Archive::Local(a) = a { 
          let mut v = Vec::new();
          a.with_sources(|d| for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
            match e {
              SourceEntry::File(f) => v.push((
                f.relative_path.split('/').fold(a.source_dir(),|p,s| p.join(s)).into(),
                DocumentURI::from_archive_relpath(a.uri().owned(), &f.relative_path)
            )),
              _ => {}
            }
          });
          files.push((a.id().clone(),v))
        }
      }

      ProgressCallbackServer::with(client, "Linting MathHub".to_string(), Some(files.len() as _), move |progress| {
        self.load_all(files.into_iter().map(|(id,v)| {
          progress.update(id.to_string(),Some(1));
          v
        }).flatten(),|file,data| {
          let lock = data.lock();
          if !lock.diagnostics.is_empty() {
            if let Ok(uri) = lsp::Url::from_file_path(&file) { 
              let _ = progress.client().clone().publish_diagnostics(lsp::PublishDiagnosticsParams {
                uri,version:None,diagnostics:lock.diagnostics.iter().map(to_diagnostic).collect()
              });
            }
          }
        });
      });
    });
    tracing::info!("Linting mathhubs finished after {t}");
  }

  pub fn load_all<I:IntoIterator<Item=(std::sync::Arc<Path>,DocumentURI)>>(&self,iter:I,mut and_then:impl FnMut(&std::sync::Arc<Path>,&STeXParseData)) {
    let mut ndocs = HMap::default();
    let mut state = LSPStore::<true>::new(&mut ndocs);
    for (p,uri) in iter {
      if let Some(ret) = state.load(p.as_ref(), &uri) {
        and_then(&p,&ret);
        let p = UrlOrFile::File(p);
        match state.map.entry(p) {
          Entry::Vacant(e) => {e.insert(DocData::Data(ret,true));}
          Entry::Occupied(mut e) => {
            e.get_mut().merge(DocData::Data(ret,true));
          }
        }
      }
    }
    let mut docs = self.documents.write();
    for (k,v) in ndocs {
      match docs.entry(k) {
        Entry::Vacant(e) => {e.insert(v);}
        Entry::Occupied(mut e) => {
          e.get_mut().merge(v);
        }
      }
    }
  }

  pub fn load<const FULL:bool>(&self,p:std::sync::Arc<Path>,uri:&DocumentURI,and_then:impl FnOnce(&STeXParseData)) {
    //let Some(lsp_uri) = lsp::Url::from_file_path(p).ok() else {return};
    let lsp_uri = UrlOrFile::File(p);
    let UrlOrFile::File(path) = &lsp_uri else { unreachable!()}; 
    if self.documents.read().get(&lsp_uri).is_some() { return }
    let mut docs = self.documents.write();
    let mut state = LSPStore::<'_,FULL>::new(&mut *docs);
    if let Some(ret) = state.load(path, uri) {
      and_then(&ret);
      drop(state);
      match docs.entry(lsp_uri) {
        Entry::Vacant(e) => {e.insert(DocData::Data(ret,FULL));}
        Entry::Occupied(mut e) => {
          e.get_mut().merge(DocData::Data(ret,FULL));
        }
      }
    }
  }

  #[allow(clippy::let_underscore_future)]
  pub fn insert(&self,uri:UrlOrFile,doctext:String) {
    let doc = self.documents.read().get(&uri).cloned();
    match doc {
      Some(DocData::Doc(doc)) => {
        if doc.set_text(doctext) {doc.compute_annots(self.clone()); }
      },
      _ => {
        let doc = LSPDocument::new(doctext,uri.clone());
        if doc.has_annots() {
          doc.compute_annots(self.clone());
        }
        match self.documents.write().entry(uri) {
          Entry::Vacant(e) => {e.insert(DocData::Doc(doc));}
          Entry::Occupied(mut e) => {
            e.get_mut().merge(DocData::Doc(doc));
          }
        }
      }
    }
  }

  #[must_use]
  pub fn get(&self,uri:&UrlOrFile) -> Option<LSPDocument> {
    if let Some(DocData::Doc(doc)) = self.documents.read().get(uri) {
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

