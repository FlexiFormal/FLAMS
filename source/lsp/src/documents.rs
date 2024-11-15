use std::path::Path;

use async_lsp::lsp_types::{Position, Range, Url};
use immt_ontology::uris::{ArchiveURI, DocumentURI, URIRefTrait};
use immt_stex::quickparse::stex::{STeXParseData, STeXParseDataI};
use immt_system::backend::{archives::Archive, AnyBackend, GlobalBackend};
use immt_utils::time::measure;

use crate::LSPStore;


struct DocumentData {
  lsp_uri:Url,
  path:Option<Box<Path>>,
  archive:Option<ArchiveURI>,
  rel_path:Option<Box<str>>,
  doc_uri:Option<DocumentURI>
}

#[derive(Clone)]
pub struct LSPDocument {
  text:triomphe::Arc<parking_lot::Mutex<LSPText>>,
  pub annotations: STeXParseData,
  data:triomphe::Arc<DocumentData>
}


impl LSPDocument {
  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::borrowed_box)]
  #[must_use]
  pub fn new(text:String,lsp_uri:Url) -> Self {
    let path = lsp_uri.to_file_path().ok().map(Into::into);
    let ap = path.as_ref().and_then(|path:&Box<Path>|
      GlobalBackend::get().manager().all_archives().iter().find_map(|a|
      if let Archive::Local(a) = a {
        if path.starts_with(a.path()) {
          let rel_path = path.display().to_string().strip_prefix(&a.source_dir().display().to_string()).map(Into::into);
          Some((a.uri().owned(),rel_path))
        } else {None}
      } else {None}
    ));
    let (archive,rel_path) = ap.map_or((None,None),|(a,p)| (Some(a),p));
    let r = LSPText { text ,up_to_date:false, html_up_to_date: false };
    let doc_uri = archive.as_ref().and_then(|a| rel_path.as_ref().map(|rp:&Box<str>| DocumentURI::from_archive_relpath(a.clone(), rp)));
    let data = DocumentData {
      lsp_uri,path,archive,rel_path,doc_uri
    };
    Self {
      text:triomphe::Arc::new(parking_lot::Mutex::new(r)),
      data:triomphe::Arc::new(data),
      annotations: STeXParseData::default()
    }
  }

  #[inline]#[must_use]
  pub fn lsp_uri(&self) -> &Url {&self.data.lsp_uri}

  #[inline]#[must_use]
  pub fn path(&self) -> Option<&Path> { self.data.path.as_deref()}

  #[inline]#[must_use]
  pub fn archive(&self) -> Option<&ArchiveURI> { self.data.archive.as_ref()}

  #[inline]#[must_use]
  pub fn relative_path(&self) -> Option<&str> { self.data.rel_path.as_deref()}

  #[inline]#[must_use]
  pub fn document_uri(&self) -> Option<&DocumentURI> { self.data.doc_uri.as_ref()}

  #[inline]
  pub fn set_text(&self,s:String) {
    self.text.lock().text = s;
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
  fn load_annotations_and<R>(&self,store:LSPStore<true>,f:impl FnOnce(&STeXParseDataI) -> R) -> Option<R> {
    let mut lock = self.text.lock();
    let uri = self.data.doc_uri.as_ref()?;
    let path = self.data.path.as_ref()?;
    let (data,t) = measure(|| immt_stex::quickparse::stex::quickparse(
      uri,&lock.text, path,
      &AnyBackend::Global(GlobalBackend::get()),
      store
    ));
    tracing::info!("quickparse took {t}");
    data.replace(&self.annotations);
    lock.up_to_date = true;
    drop(lock);
    let lock = self.annotations.lock();
    Some(f(&lock))
  }

  fn is_up_to_date(&self) -> bool {
    self.text.lock().up_to_date
  }

  #[inline]#[must_use]#[allow(clippy::significant_drop_tightening)]
  pub async fn with_annots<R:Send+'static>(self,store:LSPStore<true>,f:impl FnOnce(&STeXParseDataI) -> R + Send + 'static) -> Option<R> {
    if !self.has_annots() {return None}
    if self.is_up_to_date() {
      let lock = self.annotations.lock();
      if lock.is_empty() {return None}
      return Some(f(&lock))
    }
    match tokio::task::spawn_blocking(move || {
      self.load_annotations_and(store,f)
    }).await {
      Ok(r) => r,
      Err(e) => {
        tracing::error!("Error computing annots: {}",e);
        None
      }
    }
  }

  pub fn compute_annots(&self,store:LSPStore<true>) {
    self.load_annotations_and(store,|_| ());
  }
}

struct LSPText {
  text: String,
  up_to_date:bool,
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
    self.up_to_date = false;
    self.html_up_to_date = false;
  }
}
