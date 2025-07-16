use crate::languages::Language;
use crate::uris::{
    debugdisplay, ArchiveUri, ArchiveUriRef, ArchiveUriTrait, BaseUri, ContentURIRef,
    ContentURITrait, DocumentUri, ModuleUri, Name, NarrativeURIRef, NarrativeURITrait, PathURIRef,
    PathURITrait, SymbolUri, URIOrRefTrait, URIParseError, URIRef, URIRefTrait, URITrait,
    URIWithLanguage, URI,
};
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

use super::NarrativeURI;

#[cfg(feature = "wasm")]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
const TS_URI: &str = "export type DocumentElementUri = string;";

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DocumentElementUri {
    pub(in crate::uris) document: DocumentUri,
    pub(in crate::uris) name: Name,
}
impl DocumentElementUri {
    pub const SEPARATOR: char = 'e';
    #[inline]
    #[must_use]
    pub const fn document(&self) -> &DocumentUri {
        &self.document
    }

    #[inline]
    #[must_use]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[must_use]
    pub fn parent(&self) -> Self {
        if self.name.is_simple() {
            return self.clone();
        }
        let steps = self.name.steps();
        let steps = &steps[0..steps.len() - 1];
        let name = Name(steps.into());
        Self {
            document: self.document.clone(),
            name,
        }
    }
}
impl Display for DocumentElementUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.document, Self::SEPARATOR, self.name)
    }
}
debugdisplay!(DocumentElementUri);
impl URIOrRefTrait for DocumentElementUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        self.document.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}
impl URIWithLanguage for DocumentElementUri {
    #[inline]
    fn language(&self) -> Language {
        self.document.language
    }
}
impl NarrativeURITrait for DocumentElementUri {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        NarrativeURIRef::Element(self)
    }
    #[inline]
    fn document(&self) -> &DocumentUri {
        &self.document
    }
}
impl URITrait for DocumentElementUri {
    type Ref<'a> = &'a Self;
}
impl From<DocumentElementUri> for URI {
    #[inline]
    fn from(value: DocumentElementUri) -> Self {
        Self::Narrative(NarrativeURI::Element(value))
    }
}
impl<'a> From<&'a DocumentElementUri> for URIRef<'a> {
    #[inline]
    fn from(value: &'a DocumentElementUri) -> Self {
        URIRef::Narrative(NarrativeURIRef::Element(value))
    }
}
impl<'a> URIRefTrait<'a> for &'a DocumentElementUri {
    type Owned = DocumentElementUri;
    #[inline]
    fn owned(self) -> DocumentElementUri {
        self.clone()
    }
}

impl DocumentElementUri {
    pub(super) fn pre_parse<R>(
        s: &str,
        uri_kind: &'static str,
        f: impl FnOnce(Self, Split<char>) -> Result<R, URIParseError>,
    ) -> Result<R, URIParseError> {
        DocumentUri::pre_parse(s, uri_kind, |document, mut split| {
            let Some(s) = split.next() else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "narrative element name",
                    original: s.to_string(),
                });
            };
            s.strip_prefix(concatcp!(DocumentElementUri::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "narrative element name",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        f(
                            Self {
                                document,
                                name: name.parse()?,
                            },
                            split,
                        )
                    },
                )
        })
    }
}

impl FromStr for DocumentElementUri {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, "document element uri", |u, mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind: "document element uri",
                    original: s.to_string(),
                });
            }
            Ok(u)
        })
    }
}

impl ArchiveUriTrait for DocumentElementUri {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef {
        self.document.archive_uri()
    }
}
impl PathURITrait for DocumentElementUri {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.document.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.document.path()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, DocumentElementUri};
    serialize!(DE DocumentElementUri);
}

#[cfg(feature = "tantivy")]
impl tantivy::schema::document::ValueDeserialize for DocumentElementUri {
    fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::ValueDeserializer<'de>,
    {
        deserializer
            .deserialize_string()?
            .parse()
            .map_err(|_| tantivy::schema::document::DeserializeError::custom(""))
    }
}
