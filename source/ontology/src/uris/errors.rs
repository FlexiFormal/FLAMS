use std::error::Error;
use std::fmt::Display;

use super::name::InvalidURICharacter;

#[derive(Debug)]
pub enum URIParseError {
    TooManyPartsFor {
        uri_kind: &'static str,
        original: String,
    },
    InvalidLanguage {
        uri_kind: &'static str,
        original: String,
    },
    MissingPartFor {
        uri_kind: &'static str,
        part: &'static str,
        original: String,
    },
    UnrecognizedPart {
        original: String,
    },
    URLParseError(url::ParseError),
    InvalidCharacter(InvalidURICharacter)
}
impl From<InvalidURICharacter> for URIParseError {
    #[inline]
    fn from(e: InvalidURICharacter) -> Self {
        Self::InvalidCharacter(e)
    }
}
impl From<url::ParseError> for URIParseError {
    #[inline]
    fn from(e: url::ParseError) -> Self {
        Self::URLParseError(e)
    }
}
impl Display for URIParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyPartsFor { uri_kind, original } => {
                write!(f, "too many parts for {uri_kind}: {original}")
            }
            Self::MissingPartFor {
                uri_kind,
                part,
                original,
            } => {
                write!(
                    f,
                    "missing query fragment ({part}) for {uri_kind}: {original}"
                )
            }
            Self::InvalidLanguage { uri_kind, original } => {
                write!(f, "invalid language for {uri_kind}: {original}")
            }
            Self::UnrecognizedPart { original } => {
                write!(f, "unrecognized query fragment in uri: {original}")
            }
            Self::URLParseError(_) => {
                write!(f, "invalid URL")
            }
            Self::InvalidCharacter(e) => {
                Display::fmt(e, f)
            }
        }
    }
}
impl Error for URIParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::URLParseError(e) => Some(e),
            _ => None,
        }
    }
}
