use crate::languages::Language;
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

use crate::uris::{
    debugdisplay, ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef, ContentURITrait, ModuleURI, Name, PathURIRef, PathURITrait, URIOrRefTrait, URIParseError, URIRef, URIRefTrait, URITrait, URIWithLanguage, URI
};

use super::modules::ModuleURIRef;
use super::ContentURI;

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section))]
const TS_URI: &str = "export type SymbolURI = string;";


#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolURI {
    pub(in crate::uris) module: ModuleURI,
    pub(in crate::uris) name: Name,
}
impl SymbolURI {
    pub const SEPARATOR: char = 's';
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(module: ModuleURI, name: Name) -> Self {
        Self { module, name }
    }
    #[must_use]
    pub fn into_module(self) -> ModuleURI {
        self.module / self.name
    }
}
impl Display for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.module, Self::SEPARATOR, self.name)
    }
}
debugdisplay!(SymbolURI);

/*
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SymbolURIRef<'a> {
    pub(in crate::uris) module: ModuleURIRef<'a>,
    pub(in crate::uris) name: &'a Name,
}
impl Display for SymbolURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.module, SymbolURI::SEPARATOR, self.name)
    }
}
debugdisplay!(SymbolURIRef<'_>);
 */

 pub type SymbolURIRef<'a> = &'a SymbolURI;

impl URITrait for SymbolURI {
    type Ref<'a> = SymbolURIRef<'a>;
}
impl<'a> URIRefTrait<'a> for SymbolURIRef<'a> {
    type Owned = SymbolURI;
    #[inline]
    fn owned(self) -> SymbolURI {
        self.clone()
    }
}
impl From<SymbolURI> for URI {
    #[inline]
    fn from(u: SymbolURI) -> Self {
        Self::Content(ContentURI::Symbol(u))
    }
}
impl<'a> From<SymbolURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(u: SymbolURIRef<'a>) -> Self {
        URIRef::Content(ContentURIRef::Symbol(u))
    }
}
/*
impl<'a> URIOrRefTrait for SymbolURIRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseURI {
        &self.module.path.archive.base
    }
    #[inline]
    fn as_uri(&self) -> URIRef<'a> {
        URIRef::Content(ContentURIRef::Symbol(*self))
    }
}

impl<'a> From<&'a SymbolURI> for SymbolURIRef<'a> {
    #[inline]
    fn from(u: &'a SymbolURI) -> Self {
        SymbolURIRef {
            module: (&u.module).into(),
            name: &u.name,
        }
    }
}
*/

impl URIOrRefTrait for SymbolURI {
    #[inline]
    fn base(&self) -> &BaseURI {
        self.module.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}

impl ContentURITrait for SymbolURI {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Symbol(self)
    }
    #[inline]
    fn module(&self) -> ModuleURIRef {
        &self.module
    }
}
impl<'a> ContentURITrait for SymbolURIRef<'a> {
    #[inline]
    fn as_content(&self) -> ContentURIRef<'a> {
        ContentURIRef::Symbol(self)
    }
    #[inline]
    fn module(&self) -> ModuleURIRef<'a> {
        &self.module
    }
}

impl SymbolURI {
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
        ModuleURI::pre_parse(s, uri_kind, |module, mut split| {
            let Some(s) = split.next() else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "symbol name",
                    original: s.to_string(),
                });
            };
            s.strip_prefix(concatcp!(SymbolURI::SEPARATOR, "="))
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
impl FromStr for SymbolURI {
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
impl ArchiveURITrait for SymbolURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.module.archive_uri()
    }
}
impl PathURITrait for SymbolURI {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.module.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.module.path()
    }
}
impl<'a> ArchiveURITrait for SymbolURIRef<'a> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef<'a> {
        self.module.path.archive_uri()
    }
}
impl<'a> PathURITrait for SymbolURIRef<'a> {
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
    use crate::uris::{serialize, SymbolURI,SymbolURIRef};
    serialize!(DE SymbolURI);
    //serialize!(SymbolURIRef<'_>);
}

#[cfg(feature="tantivy")]
impl tantivy::schema::document::ValueDeserialize for SymbolURI {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, tantivy::schema::document::DeserializeError>
        where D: tantivy::schema::document::ValueDeserializer<'de> {
        deserializer.deserialize_string()?.parse()
          .map_err(|_| tantivy::schema::document::DeserializeError::custom(""))
    }
  }