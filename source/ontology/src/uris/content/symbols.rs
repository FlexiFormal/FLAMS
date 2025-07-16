use crate::languages::Language;
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

use crate::uris::{
    debugdisplay, ArchiveUri, ArchiveUriRef, ArchiveUriTrait, BaseUri, ContentURIRef,
    ContentURITrait, ModuleUri, Name, PathURIRef, PathURITrait, URIOrRefTrait, URIParseError,
    URIRef, URIRefTrait, URITrait, URIWithLanguage, URI,
};

use super::modules::ModuleUriRef;
use super::ContentURI;

#[cfg(feature = "wasm")]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
const TS_URI: &str = "export type SymbolUri = string;";

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolUri {
    pub(in crate::uris) module: ModuleUri,
    pub(in crate::uris) name: Name,
}
impl SymbolUri {
    pub const SEPARATOR: char = 's';
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(module: ModuleUri, name: Name) -> Self {
        Self { module, name }
    }
    #[must_use]
    pub fn into_module(self) -> ModuleUri {
        self.module / self.name
    }
}
impl Display for SymbolUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.module, Self::SEPARATOR, self.name)
    }
}
debugdisplay!(SymbolUri);

/*
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SymbolUriRef<'a> {
    pub(in crate::uris) module: ModuleUriRef<'a>,
    pub(in crate::uris) name: &'a Name,
}
impl Display for SymbolUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.module, SymbolUri::SEPARATOR, self.name)
    }
}
debugdisplay!(SymbolUriRef<'_>);
 */

pub type SymbolUriRef<'a> = &'a SymbolUri;

impl URITrait for SymbolUri {
    type Ref<'a> = SymbolUriRef<'a>;
}
impl<'a> URIRefTrait<'a> for SymbolUriRef<'a> {
    type Owned = SymbolUri;
    #[inline]
    fn owned(self) -> SymbolUri {
        self.clone()
    }
}
impl From<SymbolUri> for URI {
    #[inline]
    fn from(u: SymbolUri) -> Self {
        Self::Content(ContentURI::Symbol(u))
    }
}
impl<'a> From<SymbolUriRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(u: SymbolUriRef<'a>) -> Self {
        URIRef::Content(ContentURIRef::Symbol(u))
    }
}
/*
impl<'a> URIOrRefTrait for SymbolUriRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseUri {
        &self.module.path.archive.base
    }
    #[inline]
    fn as_uri(&self) -> URIRef<'a> {
        URIRef::Content(ContentURIRef::Symbol(*self))
    }
}

impl<'a> From<&'a SymbolUri> for SymbolUriRef<'a> {
    #[inline]
    fn from(u: &'a SymbolUri) -> Self {
        SymbolUriRef {
            module: (&u.module).into(),
            name: &u.name,
        }
    }
}
*/

impl URIOrRefTrait for SymbolUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        self.module.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}

impl ContentURITrait for SymbolUri {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Symbol(self)
    }
    #[inline]
    fn module(&self) -> ModuleUriRef {
        &self.module
    }
}
impl<'a> ContentURITrait for SymbolUriRef<'a> {
    #[inline]
    fn as_content(&self) -> ContentURIRef<'a> {
        ContentURIRef::Symbol(self)
    }
    #[inline]
    fn module(&self) -> ModuleUriRef<'a> {
        &self.module
    }
}

impl SymbolUri {
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &Name {
        &self.name
    }
    pub(super) fn pre_parse<R>(
        s: &str,
        uri_kind: &'static str,
        f: impl FnOnce(Self, Split<char>) -> Result<R, URIParseError>,
    ) -> Result<R, URIParseError> {
        ModuleUri::pre_parse(s, uri_kind, |module, mut split| {
            let Some(s) = split.next() else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "symbol name",
                    original: s.to_string(),
                });
            };
            s.strip_prefix(concatcp!(SymbolUri::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "symbol name",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        f(
                            Self {
                                module,
                                name: name.parse()?,
                            },
                            split,
                        )
                    },
                )
        })
    }
}
impl FromStr for SymbolUri {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, "symbol uri", |u, mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind: "symbol uri",
                    original: s.to_string(),
                });
            }
            Ok(u)
        })
    }
}
impl ArchiveUriTrait for SymbolUri {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef {
        self.module.archive_uri()
    }
}
impl PathURITrait for SymbolUri {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.module.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.module.path()
    }
}
impl<'a> ArchiveUriTrait for SymbolUriRef<'a> {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef<'a> {
        self.module.path.archive_uri()
    }
}
impl<'a> PathURITrait for SymbolUriRef<'a> {
    #[inline]
    fn as_path(&self) -> PathURIRef<'a> {
        self.module.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.module.path.path.as_ref()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, SymbolUri, SymbolUriRef};
    serialize!(DE SymbolUri);
    //serialize!(SymbolUriRef<'_>);
}

#[cfg(feature = "tantivy")]
impl tantivy::schema::document::ValueDeserialize for SymbolUri {
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
