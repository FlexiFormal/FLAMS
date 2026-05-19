use axum::body::Body;
use flams_math_archives::{
    LocallyBuilt, MathArchive,
    backend::{GlobalBackend, LocalBackend},
};
use flams_system::settings::Settings;
use ftml_uris::{
    ArchiveId, DocumentUri, IsNarrativeUri, Language, UriWithArchive, UriWithPath,
    components::{DocumentUriComponentTuple, DocumentUriComponents},
};
use http::Request;
use std::borrow::Cow;
use tower::ServiceExt;
use tower_http::services::{ServeFile, fs::ServeFileSystemResponseBody};

// May cache images at some point
#[derive(Clone, Default)]
pub struct ImageStore(/*flams_utils::triomphe::Arc<ImageStoreI>*/ ImageStoreI);

#[derive(Default, Copy, Clone)]
struct ImageStoreI {
    //map: dashmap::DashMap<ImageSpec, ImageData>,
    //count: AtomicU64,
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
    // axum::extract::State(ServerState { images, .. }): axum::extract::State<ServerState>,
    //request: http::Request<axum::body::Body>,
) -> Result<Img, axum::http::StatusCode> /*axum::response::Response<ServeFileSystemResponseBody>*/ {
    let Ok(Some(img)) = tokio::task::spawn_blocking(move || {
        let query = uri.query()?;
        let path = flams_math_archives::images::ImagePath::from_query(query, Settings::get().lsp)?;
        path.get()
    })
    .await
    else {
        return Err(http::StatusCode::NOT_FOUND);
    };
    Ok(Img(img))
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
