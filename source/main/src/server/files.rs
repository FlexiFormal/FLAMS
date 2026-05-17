use super::ServerState;
use axum::body::Body;
use flams_math_archives::{
    LocallyBuilt, MathArchive,
    backend::{GlobalBackend, LocalBackend},
};
use flams_system::settings::Settings;
use ftml_ontology::utils::time::Timestamp;
use ftml_uris::{
    ArchiveId, DocumentUri, IsNarrativeUri, Language, UriWithArchive, UriWithPath,
    components::{DocumentUriComponentTuple, DocumentUriComponents},
};
use http::{Request, Response};
use leptos::server_fn::{codec::IntoRes, response::Res};
use std::{borrow::Cow, ops::DerefMut, path::PathBuf, sync::atomic::AtomicU64};
use tower::ServiceExt;
use tower_http::services::{ServeFile, fs::ServeFileSystemResponseBody};

#[derive(Clone, Default)]
pub struct ImageStore(/*flams_utils::triomphe::Arc<ImageStoreI>*/ ImageStoreI);

#[derive(Default, Copy, Clone)]
struct ImageStoreI {
    //map: dashmap::DashMap<ImageSpec, ImageData>,
    //count: AtomicU64,
}
impl ImageStoreI {
    // may cache stuff at some point
    async fn get(&self, spec: ImageSpec) -> Option<Box<[u8]>> {
        let path = match spec {
            ImageSpec::Kpse(p) => tex_engine::engine::filesystem::kpathsea::KPATHSEA.which(p)?,
            ImageSpec::ARp(a, p) => {
                GlobalBackend.with_local_archive(&a, |a| a.map(|a| a.path().join(&*p)))?
            }
            ImageSpec::File(p) => std::path::PathBuf::from(p.to_string()),
        };
        let img =
            tokio::task::spawn_blocking(|| image::ImageReader::open(path).ok()?.decode().ok())
                .await
                .ok()??;
        let mut v = Vec::<u8>::new();
        img.write_with_encoder(image::codecs::webp::WebPEncoder::new_lossless(&mut v))
            .ok()?;
        Some(v.into_boxed_slice())
    }
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
                GlobalBackend.with_local_archive(a, |a| a.map(|a| a.path().join(&**p)))
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
    #[must_use]
    pub fn new(data: &[u8]) -> Self {
        Self {
            img: data.into(),
            timestamp: AtomicU64::new(Timestamp::now().0.get()),
        }
    }
}

pub(crate) struct Img(Box<[u8]>);
impl axum::response::IntoResponse for Img {
    fn into_response(self) -> axum::response::Response {
        ([(axum::http::header::CONTENT_TYPE, "image/webp")], self.0).into_response()
    }
}

#[axum::debug_handler]
pub(crate) async fn img_handler(
    uri: http::Uri,
    axum::extract::State(ServerState { images, .. }): axum::extract::State<ServerState>,
    //request: http::Request<axum::body::Body>,
) -> Result<Img, axum::http::StatusCode> /*axum::response::Response<ServeFileSystemResponseBody>*/ {
    let Some(s) = uri.query() else {
        return Err(http::StatusCode::NOT_FOUND);
    };

    let spec = if let Some(s) = s.strip_prefix("kpse=") {
        ImageSpec::Kpse(s.into())
    } else if let Some(f) = s.strip_prefix("file=")
        && Settings::get().lsp
    {
        ImageSpec::File(f.into())
    } else if let Some(s) = s.strip_prefix("a=")
        && let Some((a, rp)) = s.split_once("&rp=")
    {
        let a = a.parse().unwrap_or_else(|_| unreachable!());
        let rp = rp.into();
        ImageSpec::ARp(a, rp)
    } else {
        return Err(http::StatusCode::NOT_FOUND);
    };

    if let Some(img) = images.0.get(spec).await {
        Ok(Img(img))
    }
    /*
    if let Some(p) = spec.path() {
        let req = Request::builder()
            .uri(uri.clone())
            .body(Body::empty())
            .unwrap();
        ServeFile::new(p)
            .oneshot(req)
            .await
            .unwrap_or_else(|_| default())
    }*/
    else {
        Err(http::StatusCode::NOT_FOUND)
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
        tracing::info!("pdf download error: {s}");
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

    let uri: Option<DocumentUri> = parse!("uri");
    let rp = params.get("rp");
    let a: Option<ArchiveId> = parse!("a");
    let p = params.get("p");
    let l: Option<Language> = parse!("l");
    let d = params.get("d");

    let comps = DocumentUriComponentTuple {
        uri,
        rp,
        a,
        p,
        d,
        l,
    };

    let comps: Result<DocumentUriComponents, _> = comps.try_into();
    let uri = if let Ok(comps) = comps {
        let Ok(uri) =
            comps.parse(|a| GlobalBackend.with_archive(a, |a| a.map(|a| a.uri().clone())))
        else {
            return err("Malformed URI components");
        };
        uri
    } else {
        return err("Malformed URI components");
    };
    let uri2 = uri.clone();
    let formatstr = format.to_string();
    let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
        GlobalBackend.with_local_archive(uri.archive_id(), |a| {
            a.map(|a| {
                a.out_path_of(uri.path(), uri.document_name(), None, uri.language)
                    .join(&formatstr)
            })
        })
    })
    .await
    else {
        return default();
    };

    let pandq = format!("/{}.{format}", uri2.document_name());
    let mime = mime_guess::from_ext(&format).first_or_octet_stream();
    let req_uri = http::Uri::builder()
        .path_and_query(pandq)
        .build()
        .unwrap_or(req_uri);
    let req = Request::builder()
        .uri(req_uri)
        .body(Body::empty())
        .expect("this is a bug");
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
    fn get_str(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0
            .split('&')
            .find(|s| s.starts_with(name) && s.as_bytes().get(name.len()) == Some(&b'='))?
            .split('=')
            .nth(1)
            .and_then(|s| urlencoding::decode(s).ok())
    }
    fn get(&self, name: &str) -> Option<String> {
        self.get_str(name).map(Cow::into_owned)
    }
}
