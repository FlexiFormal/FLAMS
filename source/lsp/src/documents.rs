use std::{path::Path, sync::atomic::AtomicBool};

use async_lsp::{lsp_types::{Position, Range, Url}, ClientSocket};
use immt_ontology::uris::{ArchiveURI, DocumentURI, URIRefTrait};
use immt_stex::quickparse::stex::{STeXParseData, STeXParseDataI};
use immt_system::backend::{AnyBackend, Backend, GlobalBackend};
use immt_utils::time::measure;

use crate::{state::{LSPState, UrlOrFile}, LSPStore};


struct DocumentData {
  path:Option<std::sync::Arc<Path>>,
  archive:Option<ArchiveURI>,
  rel_path:Option<Box<str>>,
  doc_uri:Option<DocumentURI>
}

#[derive(Clone)]
pub struct LSPDocument {
  up_to_date:triomphe::Arc<AtomicBool>,
  text:triomphe::Arc<parking_lot::Mutex<LSPText>>,
  pub(crate) annotations: STeXParseData,
  data:triomphe::Arc<DocumentData>
}


impl LSPDocument {
  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::borrowed_box)]
  #[must_use]
  pub fn new(text:String,lsp_uri:UrlOrFile) -> Self {
    let path = if let UrlOrFile::File(p) = lsp_uri {Some(p)} else {None}; //lsp_uri.to_file_path().ok().map(Into::into);
    let default = || {
      let path = path.as_ref()?.as_os_str().to_str()?.into();
      Some((ArchiveURI::no_archive(),Some(path)))
    };
    let ap = path.as_ref().and_then(|path|
      GlobalBackend::get().archive_of(path,|a,rp| {
        let uri = a.uri().owned();
        let rp = rp.strip_prefix("/source/").map(|r| r.into());
        (uri,rp)
      })
    ).or_else(default);
    let (archive,rel_path) = ap.map_or((None,None),|(a,p)| (Some(a),p));
    let r = LSPText { text , html_up_to_date: false };
    let doc_uri = archive.as_ref().and_then(|a| rel_path.as_ref().map(|rp:&Box<str>| DocumentURI::from_archive_relpath(a.clone(), rp)));
    //tracing::info!("Document: {lsp_uri}\n - {doc_uri:?}\n - [{archive:?}]{{{rel_path:?}}}");
    let data = DocumentData {
      path,archive,rel_path,doc_uri
    };
    Self {
      up_to_date:triomphe::Arc::new(AtomicBool::new(false)),
      text:triomphe::Arc::new(parking_lot::Mutex::new(r)),
      data:triomphe::Arc::new(data),
      annotations: STeXParseData::default()
    }
  }


  #[inline]#[must_use]
  pub fn path(&self) -> Option<&Path> { self.data.path.as_deref()}

  #[inline]#[must_use]
  pub fn archive(&self) -> Option<&ArchiveURI> { self.data.archive.as_ref()}

  #[inline]#[must_use]
  pub fn relative_path(&self) -> Option<&str> { self.data.rel_path.as_deref()}

  #[inline]#[must_use]
  pub fn document_uri(&self) -> Option<&DocumentURI> { self.data.doc_uri.as_ref()}

  #[inline]
  pub fn set_text(&self,s:String) -> bool {
    let mut txt = self.text.lock();
    if txt.text == s { return false }
    txt.text = s;
    self.up_to_date.store(false,std::sync::atomic::Ordering::SeqCst);
    true
  }

  #[inline]
  pub fn with_text<R>(&self,f:impl FnOnce(&str) -> R) -> R {
    f(&self.text.lock().text)
  }

  #[inline]
  pub fn html_up_to_date(&self) -> bool {
    self.text.lock().html_up_to_date
  }

  pub fn set_html_up_to_date(&self) {
    self.text.lock().html_up_to_date = true
  }

  #[inline]
  pub fn delta(&self,text:String,range:Option<Range>) {
    self.up_to_date.store(false,std::sync::atomic::Ordering::SeqCst);
    self.text.lock().delta(text, range);
  }
  #[inline]
  #[must_use]
  pub fn get_range(&self,range:Range) -> (usize,usize) {
    self.text.lock().get_range(range)
  }
  #[inline]
  #[must_use]
  pub fn get_position(&self,pos:Position) -> usize {
    self.text.lock().get_position(pos)
  }

  #[inline]#[must_use]
  pub fn has_annots(&self) -> bool {
    self.data.doc_uri.is_some() && self.data.path.is_some()
  }

  #[allow(clippy::significant_drop_tightening)]
  fn load_annotations_and<R>(&self,state:LSPState,f:impl FnOnce(&STeXParseDataI) -> R) -> Option<R> {
    let mut lock = self.text.lock();
    let uri = self.data.doc_uri.as_ref()?;
    let path = self.data.path.as_ref()?;

    let mut docs = state.documents.write();
    let mut store = LSPStore::<true>::new(&mut *docs);
    let data =
    //let (data,t) = measure(|| 
      immt_stex::quickparse::stex::quickparse(
      uri,&lock.text, path,
      &AnyBackend::Global(GlobalBackend::get()),
      &mut store);
    //);
    data.replace(&self.annotations);
    self.up_to_date.store(true, std::sync::atomic::Ordering::SeqCst);
    drop(store);
    drop(docs);
    //tracing::info!("quickparse took {t}");
    drop(lock);
    /*let path = path.clone();
    let _ = tokio::task::spawn_blocking(move || {
      state.relint_dependents(path);
    });*/
    let lock = self.annotations.lock();
    Some(f(&lock))
  }

  pub fn is_up_to_date(&self) -> bool {
    self.up_to_date.load(std::sync::atomic::Ordering::SeqCst)
  }

  #[inline]#[must_use]#[allow(clippy::significant_drop_tightening)]
  pub async fn with_annots<R:Send+'static>(self,state:LSPState,f:impl FnOnce(&STeXParseDataI) -> R + Send + 'static) -> Option<R> {
    if !self.has_annots() {return None}
    if self.is_up_to_date() {
      let lock = self.annotations.lock();
      if lock.is_empty() {return None}
      return Some(f(&lock))
    }
    match tokio::task::spawn_blocking(move || {
      self.load_annotations_and(state,f)
    }).await {
      Ok(r) => r,
      Err(e) => {
        tracing::error!("Error computing annots: {}",e);
        None
      }
    }
  }

  #[inline]
  pub fn compute_annots(&self,state:LSPState) {
    self.load_annotations_and(state,|_| ());
  }
}

struct LSPText {
  text: String,
  html_up_to_date:bool
}

impl LSPText {
  fn get_position(&self,Position{mut line,character}:Position) -> usize {
    let mut rest = self.text.as_str();
    let mut off = 0;
    while line > 0 {
      if let Some(i) = rest.find(['\n','\r']) {
        off += i + 1;
        if rest.as_bytes()[i] == b'\r' && rest.as_bytes().get(i + 1) == Some(&b'\n') {
          off += 1;
          rest = &rest[i + 2..];
        } else {
          rest = &rest[i + 1..];
        }
        line -= 1;
      } else {
        off = self.text.len(); 
        rest = "";
        break
      }
    }
    let next = rest.chars().take(character as usize).map(char::len_utf8).sum::<usize>();
    off += next;
    off
  }

  fn get_range(&self,range:Range) -> (usize,usize) {
    let Range{
      start:Position{line:mut start_line,character:startc}, 
      end:Position{line:mut end_line,character:mut endc}
    } = range;
    if start_line == end_line { endc -= startc; }
    end_line -= start_line;

    let mut start = 0;
    let mut rest = self.text.as_str();
    while start_line > 0 {
      if let Some(i) = rest.find(['\n','\r']) {
        start += i + 1;
        if rest.as_bytes()[i] == b'\r' && rest.as_bytes().get(i + 1) == Some(&b'\n') {
          start += 1;
          rest = &rest[i + 2..];
        } else {
          rest = &rest[i + 1..];
        }
        start_line -= 1;
      } else {
        start = self.text.len(); 
        rest = "";
        end_line = 0;
        break
      }
    }
    let next = rest.chars().take(startc as usize).map(char::len_utf8).sum::<usize>();
    start += next;
    rest = &rest[next..];

    let mut end = start;
    while end_line > 0 {
      if let Some(i) = rest.find(['\n','\r']) {
        end += i + 1;
        if rest.as_bytes()[i] == b'\r' && rest.as_bytes().get(i + 1) == Some(&b'\n') {
          end += 1;
          rest = &rest[i + 2..];
        } else {
          rest = &rest[i + 1..];
        }
        end_line -= 1;
      } else {
        end = self.text.len();
        rest = "";
        break
      }
    }
    end += rest.chars().take(endc as usize).map(char::len_utf8).sum::<usize>();
    (start,end)
  }

  #[allow(clippy::cast_possible_truncation)]
  fn delta(&mut self, text: String, range: Option<Range>) {
    let Some(range) = range else {
      self.text = text;
      return
    };
    let (start,end) = self.get_range(range);
    self.text.replace_range(start..end, &text);
    self.html_up_to_date = false;
  }
}
