use std::{default, ops::Deref, path::PathBuf, sync::atomic::AtomicU64};

use axum::body::Body;
use flams_ontology::{
    languages::Language,
    uris::{ArchiveId, DocumentURI},
};
use flams_router_base::uris::DocURIComponents;
use flams_system::{
    backend::{Backend, GlobalBackend},
    settings::Settings,
};
use flams_utils::time::Timestamp;
use http::Request;
use leptos_router::location::LocationProvider;
use tower::ServiceExt;
use tower_http::services::{fs::ServeFileSystemResponseBody, ServeFile};

use super::ServerState;

#[derive(Clone, Default)]
pub struct ImageStore(flams_utils::triomphe::Arc<ImageStoreI>);

#[derive(Default)]
struct ImageStoreI {
    map: dashmap::DashMap<ImageSpec, ImageData>,
    count: AtomicU64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ImageSpec {
    Kpse(Box<str>),
    ARp(ArchiveId, Box<str>),
    File(Box<str>),
}
impl ImageSpec {
    pub fn path(&self) -> Option<PathBuf> {
        match self {
            Self::Kpse(p) => tex_engine::engine::filesystem::kpathsea::KPATHSEA.which(p),
            Self::ARp(a, p) => {
                GlobalBackend::get().with_local_archive(a, |a| a.map(|a| a.path().join(&**p)))
            }
            Self::File(p) => Some(std::path::PathBuf::from(p.to_string())),
        }
    }
}

pub struct ImageData {
    img: Box<[u8]>,
    timestamp: AtomicU64,
}
impl ImageData {
    pub fn update(&self) {
        let now = Timestamp::now();
        self.timestamp
            .store(now.0.get() as _, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn new(data: &[u8]) -> Self {
        Self {
            img: data.into(),
            timestamp: AtomicU64::new(Timestamp::now().0.get()),
        }
    }
}

pub(crate) async fn img_handler(
    uri: http::Uri,
    axum::extract::State(ServerState { images: _, .. }): axum::extract::State<ServerState>,
    //request: http::Request<axum::body::Body>,
) -> axum::response::Response<ServeFileSystemResponseBody> {
    let default = || {
        let mut resp = axum::response::Response::new(ServeFileSystemResponseBody::default());
        *resp.status_mut() = http::StatusCode::NOT_FOUND;
        resp
    };

    let Some(s) = uri.query() else {
        return default();
    };

    let spec = if let Some(s) = s.strip_prefix("kpse=") {
        ImageSpec::Kpse(s.into())
    } else if let Some(f) = s.strip_prefix("file=") {
        if Settings::get().lsp {
            ImageSpec::File(f.into())
        } else {
            return default();
        }
    } else if let Some(s) = s.strip_prefix("a=") {
        let Some((a, rp)) = s.split_once("&rp=") else {
            return default();
        };
        let a = a.parse().unwrap_or_else(|_| unreachable!());
        let rp = rp.into();
        ImageSpec::ARp(a, rp)
    } else {
        return default();
    };

    //tracing::info!("HERE: {spec:?}");
    if let Some(p) = spec.path() {
        let req = Request::builder()
            .uri(uri.clone())
            .body(Body::empty())
            .unwrap();
        ServeFile::new(p)
            .oneshot(req)
            .await
            .unwrap_or_else(|_| default())
    } else {
        default()
    }
}

pub(crate) async fn doc_handler(
    uri: http::Uri,
) -> axum::response::Response<ServeFileSystemResponseBody> {
    let req_uri = uri;
    let default = || {
        let mut resp = axum::response::Response::new(ServeFileSystemResponseBody::default());
        *resp.status_mut() = http::StatusCode::NOT_FOUND;
        resp
    };
    let err = |s: &str| {
        let mut resp = axum::response::Response::new(ServeFileSystemResponseBody::default());
        *resp.status_mut() = http::StatusCode::BAD_REQUEST;
        resp
    };

    let Some(params) = Params::new(&req_uri) else {
        return err("Invalid URI");
    };

    macro_rules! parse {
        ($id:literal) => {
            if let Some(s) = params.get_str($id) {
                let Ok(r) = s.parse() else {
                    return err("malformed uri");
                };
                Some(r)
            } else {
                None
            }
        };
    }
    let Some(format) = params.get_str("format") else {
        return err("Missing format");
    };

    let uri: Option<DocumentURI> = parse!("uri");
    let rp = params.get("rp");
    let a: Option<ArchiveId> = parse!("a");
    let p = params.get("p");
    let l: Option<Language> = parse!("l");
    let d = params.get("d");

    let comps: Result<DocURIComponents, _> = (uri, rp, a, p, l, d).try_into();
    let uri = if let Ok(comps) = comps {
        let Some(uri) = comps.parse() else {
            return err("Malformed URI components");
        };
        uri
    } else {
        return err("Malformed URI components");
    };
    let Some(path) = GlobalBackend::get().artifact_path(&uri, format) else {
        return default();
    };

    let pandq = format!("/{}.{format}", uri.name().first_name());
    let mime = mime_guess::from_ext(&format).first_or_octet_stream();
    let req_uri = http::Uri::builder()
        .path_and_query(pandq)
        .build()
        .unwrap_or(req_uri);
    let req = Request::builder().uri(req_uri).body(Body::empty()).unwrap();
    ServeFile::new_with_mime(path, &mime)
        .oneshot(req)
        .await
        .unwrap_or_else(|_| default())
}

struct Params<'a>(&'a str);
impl<'a> Params<'a> {
    fn new(uri: &'a http::Uri) -> Option<Self> {
        uri.query().map(Self)
    }
    fn get_str(&self, name: &str) -> Option<&str> {
        self.0
            .split('&')
            .find(|s| s.starts_with(name) && s.as_bytes().get(name.len()) == Some(&b'='))?
            .split('=')
            .nth(1)
    }
    fn get(&self, name: &str) -> Option<String> {
        self.get_str(name).map(|s| s.to_string())
    }
}
