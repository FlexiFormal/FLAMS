use std::{default, ops::Deref, path::PathBuf, sync::atomic::AtomicU64};

use axum::body::Body;
use http::Request;
use immt_ontology::uris::ArchiveId;
use immt_system::{backend::{Backend, GlobalBackend}, settings::Settings};
use immt_utils::time::Timestamp;
use leptos::svg::Image;
use tower::ServiceExt;
use tower_http::services::{fs::ServeFileSystemResponseBody, ServeFile};

use super::ServerState;

#[derive(Clone,Default)]
pub struct ImageStore(immt_utils::triomphe::Arc<ImageStoreI>);

#[derive(Default)]
struct ImageStoreI {
  map:dashmap::DashMap<ImageSpec, ImageData>,
  count:AtomicU64
}

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub enum ImageSpec {
  Kpse(Box<str>),
  ARp(ArchiveId,Box<str>),
  File(Box<str>)
}
impl ImageSpec {
  pub fn path(&self) -> Option<PathBuf> {
    match self {
      Self::Kpse(p) => tex_engine::engine::filesystem::kpathsea::KPATHSEA.which(p),
      Self::ARp(a,p) =>
        GlobalBackend::get().with_local_archive(a, |a| 
          a.map(|a| a.path().join(&**p) )
        ),
      Self::File(p) => Some(std::path::PathBuf::from(p.to_string()))
    }
  }
}

pub struct ImageData {
  img: Box<[u8]>,
  timestamp: AtomicU64
}
impl ImageData {
  pub fn update(&self) {
    let now = Timestamp::now();
    self.timestamp.store(now.0.get() as _,std::sync::atomic::Ordering::SeqCst);
  }
  pub fn new(data: &[u8]) -> Self {
    Self {
      img: data.into(),
      timestamp: AtomicU64::new(Timestamp::now().0.get())
    }
  }
}


pub async fn img_handler(
  mut uri: http::Uri,
  axum::extract::State(ServerState {images,..}): axum::extract::State<ServerState>,
  //request: http::Request<axum::body::Body>,
) -> axum::response::Response<ServeFileSystemResponseBody> {

  let default = || {
    let mut resp = axum::response::Response::new(ServeFileSystemResponseBody::default());
    *resp.status_mut() = http::StatusCode::NOT_FOUND;
    resp
  };

  let Some(s) = uri.query() else { return default(); };

  let spec = if let Some(s) = s.strip_prefix("kpse=") {
    ImageSpec::Kpse(s.into())
  } else if let Some(f) = s.strip_prefix("file=") {
    if Settings::get().lsp {
      ImageSpec::File(f.into())
    } else { return default(); }
  }
  else if let Some(s) = s.strip_prefix("a=") {
    let Some((a,rp)) = s.split_once("&rp=") else { return default(); };
    let a = a.parse().unwrap_or_else(|_| unreachable!());
    let rp = rp.into();
    ImageSpec::ARp(a,rp)
  } else { return default() };

  //tracing::info!("HERE: {spec:?}");
  if let Some(p) = spec.path() {
    let req = Request::builder()
    .uri(uri.clone())
    .body(Body::empty())
    .unwrap();
    ServeFile::new(p).oneshot(req).await.unwrap_or_else(|_| default())
  } else {default() }

}