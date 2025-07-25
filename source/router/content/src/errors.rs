use leptos::{
    prelude::{FromServerFnError, ServerFnErrorErr},
    server_fn::{Bytes, ContentType, Decodes, Encodes, FormatType, error::ServerFnErrorEncoding},
};
use std::fmt::Write;

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum BackendError {
    /*
    #[cfg(feature = "server_fn")]
    #[error("error serializing error")]
    ErrorSerializing,
    #[cfg(feature = "server_fn")]
    #[error("error deserializing error: {0}")]
    ErrorDeserializing(String),
     */
    #[error("server error: {0}")]
    ServerFn(#[from] leptos::server_fn::error::ServerFnErrorErr),
    #[error("invalid uri components: {0}")]
    InvalidUriComponent(#[from] ftml_uris::components::ComponentError),
    #[error("{0} not found")]
    NotFound(ftml_uris::UriKind),
    #[error("no html for document")]
    HtmlNotFound,
    #[error("element does not have a fragment")]
    NoFragment,
    #[error("no definition for element found")]
    NoDefinition,
    #[error("not yet implemented")]
    ToDo(String),
}

impl FromServerFnError for BackendError {
    type Encoder = Encoder;
    #[inline]
    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        value.into()
    }
}

pub struct Encoder;

pub fn encode_server_fn(e: &ServerFnErrorErr) -> Result<Bytes, std::fmt::Error> {
    let mut buf = String::new();
    let result = match e {
        ServerFnErrorErr::Registration(e) => {
            write!(&mut buf, "Registration|{e}")
        }
        ServerFnErrorErr::Request(e) => write!(&mut buf, "Request|{e}"),
        ServerFnErrorErr::Response(e) => write!(&mut buf, "Response|{e}"),
        ServerFnErrorErr::ServerError(e) => {
            write!(&mut buf, "ServerError|{e}")
        }
        ServerFnErrorErr::MiddlewareError(e) => {
            write!(&mut buf, "MiddlewareError|{e}")
        }
        ServerFnErrorErr::Deserialization(e) => {
            write!(&mut buf, "Deserialization|{e}")
        }
        ServerFnErrorErr::Serialization(e) => {
            write!(&mut buf, "Serialization|{e}")
        }
        ServerFnErrorErr::Args(e) => write!(&mut buf, "Args|{e}"),
        ServerFnErrorErr::MissingArg(e) => {
            write!(&mut buf, "MissingArg|{e}")
        }
        ServerFnErrorErr::UnsupportedRequestMethod(req) => {
            write!(&mut buf, "UnsupportedRequestMethod|{req}")
        }
    };

    match result {
        Ok(()) => Ok(Bytes::from(buf)),
        Err(e) => Err(e),
    }
}

impl FormatType for Encoder {
    const FORMAT_TYPE: leptos::server_fn::Format = ServerFnErrorEncoding::FORMAT_TYPE;
}
impl ContentType for Encoder {
    const CONTENT_TYPE: &'static str = ServerFnErrorEncoding::CONTENT_TYPE;
}

impl Encodes<BackendError> for Encoder {
    type Error = String;

    fn encode(output: &BackendError) -> Result<Bytes, Self::Error> {
        let mut buf = String::new();
        let result = match output {
            BackendError::ServerFn(e) => {
                return encode_server_fn(e).map_err(|_| format!("error serializing"));
            }
            BackendError::InvalidUriComponent(u) => write!(
                &mut buf,
                "InvalidUri|{}",
                serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
            ),
            BackendError::NotFound(u) => write!(
                &mut buf,
                "NotFound|{}",
                serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
            ),
            BackendError::ToDo(u) => write!(
                &mut buf,
                "NotYetImplemented|{}",
                serde_json::to_string(u).map_err(|e| format!("error serializing: {e}"))?
            ),
            BackendError::HtmlNotFound => {
                buf.push_str("HtmlNotFound|");
                Ok(())
            }
            BackendError::NoFragment => {
                buf.push_str("NoFragment|");
                Ok(())
            }
            BackendError::NoDefinition => {
                buf.push_str("NoDefinition|");
                Ok(())
            }
        }
        .map_err(|_| format!("Error deserializing"))?;
        Ok(Bytes::from(buf))
    }
}

pub fn decode_server_fn(ty: &str, data: String) -> Result<ServerFnErrorErr, String> {
    match ty {
        "Registration" => Ok(ServerFnErrorErr::Registration(data)),
        "Request" => Ok(ServerFnErrorErr::Request(data)),
        "Response" => Ok(ServerFnErrorErr::Response(data)),
        "ServerError" => Ok(ServerFnErrorErr::ServerError(data)),
        "MiddlewareError" => Ok(ServerFnErrorErr::MiddlewareError(data)),
        "Deserialization" => Ok(ServerFnErrorErr::Deserialization(data)),
        "Serialization" => Ok(ServerFnErrorErr::Serialization(data)),
        "Args" => Ok(ServerFnErrorErr::Args(data)),
        "MissingArg" => Ok(ServerFnErrorErr::MissingArg(data)),
        "UnsupportedRequestMethod" => Ok(ServerFnErrorErr::UnsupportedRequestMethod(data)),
        _ => Err(data),
    }
}

impl Decodes<BackendError> for Encoder {
    type Error = String;

    fn decode(bytes: Bytes) -> Result<BackendError, Self::Error> {
        let mut prefix = String::from_utf8(bytes.to_vec())
            .map_err(|err| format!("UTF-8 conversion error: {err}"))?;
        let Some(j) = prefix.find('|') else {
            return Err(format!("Invalid format: missing delimiter in {prefix:?}"));
        };
        if j == 0 {
            return Err(format!("Invalid format: missing delimiter in {prefix:?}"));
        }
        let data = prefix.split_off(j + 1);
        let prefix = &prefix[..prefix.len() - 1];
        let data = match decode_server_fn(prefix, data) {
            Ok(e) => return Ok(e.into()),
            Err(e) => e,
        };
        match prefix {
            "InvalidUri" => Ok(BackendError::InvalidUriComponent(
                serde_json::from_str(&data).map_err(|e| e.to_string())?,
            )),
            "NotFound" => Ok(BackendError::NotFound(
                serde_json::from_str(&data).map_err(|e| e.to_string())?,
            )),
            "HtmlNotFound" => Ok(BackendError::HtmlNotFound),
            "NoFragment" => Ok(BackendError::NoFragment),
            "NoDefinition" => Ok(BackendError::NoDefinition),
            "NotYetImplemented" => Ok(BackendError::ToDo(data)),
            _ => Err(format!("unknown error: {data}")),
        }
    }
}
