use crate::languages::Language;
use crate::uris::errors::URIParseError;
use crate::uris::macros::debugdisplay;
use crate::uris::{
    ArchiveUri, ArchiveUriRef, ArchiveUriTrait, BaseUri, ContentURIRef, ContentURITrait, Name,
    PathURI, PathURIRef, PathURITrait, SymbolUri, URIOrRefTrait, URIRef, URIRefTrait, URITrait,
    URIWithLanguage, URI,
};
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

use super::ContentURI;

#[cfg(feature = "wasm")]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
const TS_URI: &str = "export type ModuleUri = string;";

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModuleUri {
    pub(in crate::uris) path: PathURI,
    pub(in crate::uris) name: Name,
}

impl ModuleUri {
    pub const SEPARATOR: char = 'm';
    #[must_use]
    pub fn into_symbol(mut self) -> Option<SymbolUri> {
        let last = self.name.0.pop()?;
        if self.name.0.is_empty() {
            return None;
        }
        Some(SymbolUri {
            module: self,
            name: last.into(),
        })
    }
}
impl Display for ModuleUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.path, Self::SEPARATOR, self.name)
    }
}
debugdisplay!(ModuleUri);

/*
#[derive(Copy,Clone, PartialEq, Eq, Hash)]
pub struct ModuleUriRef<'a> {
    pub(in crate::uris) path: PathURIRef<'a>,
    pub(in crate::uris) name: &'a Name,
    pub(in crate::uris) language: Language,
}
impl Display for ModuleUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}&{}={}&{}={}",
            self.path,
            ModuleUri::SEPARATOR,
            self.name,
            Language::SEPARATOR,
            self.language
        )
    }
}
debugdisplay!(ModuleUriRef<'_>);
*/

impl URITrait for ModuleUri {
    type Ref<'a> = &'a Self; //ModuleUriRef<'a>;
}

pub type ModuleUriRef<'a> = &'a ModuleUri;

impl<'a> URIRefTrait<'a> for ModuleUriRef<'a> {
    type Owned = ModuleUri;
    #[inline]
    fn owned(self) -> Self::Owned {
        self.clone()
    }
}
/*
impl<'a> URIRefTrait<'a> for ModuleUriRef<'a> {
    type Owned = ModuleUri;
    fn owned(self) -> Self::Owned {
        ModuleUri {
            path: self.path.owned(),
            name: self.name.clone(),
            language: self.language,
        }
    }
}
*/
impl From<ModuleUri> for URI {
    #[inline]
    fn from(value: ModuleUri) -> Self {
        Self::Content(ContentURI::Module(value))
    }
}
impl<'a> From<ModuleUriRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(value: ModuleUriRef<'a>) -> Self {
        URIRef::Content(ContentURIRef::Module(value))
    }
}
/*
impl<'a> URIOrRefTrait for ModuleUriRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseUri {
        &self.path.archive.base
    }
    #[inline]
    fn as_uri(&self) -> URIRef<'a> {
        URIRef::Content(ContentURIRef::Module(*self))
    }
}

impl<'a> From<&'a ModuleUri> for ModuleUriRef<'a> {
    #[inline]
    fn from(value: &'a ModuleUri) -> Self {
        Self {
            path: value.as_path(),
            name: &value.name,
            language: value.language,
        }
    }
}
*/

impl URIOrRefTrait for ModuleUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        self.path.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}
impl ContentURITrait for ModuleUri {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Module(self)
    }
    #[inline]
    fn module(&self) -> ModuleUriRef {
        self
    }
}
impl<'a> ContentURITrait for ModuleUriRef<'a> {
    #[inline]
    fn as_content(&self) -> ContentURIRef<'a> {
        ContentURIRef::Module(self)
    }
    #[inline]
    fn module(&self) -> Self {
        self
    }
}

impl ModuleUri {
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
        PathURI::pre_parse(s, uri_kind, |path, next, mut split| {
            let Some(m) = next.or_else(|| split.next()) else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "module name",
                    original: s.to_string(),
                });
            };
            m.strip_prefix(concatcp!(ModuleUri::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "module name",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        f(
                            Self {
                                path,
                                name: name.parse()?,
                            },
                            split,
                        )
                    },
                )
        })
    }
}
impl FromStr for ModuleUri {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, "module uri", |u, mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind: "module uri",
                    original: s.to_string(),
                });
            }
            Ok(u)
        })
    }
}
impl ArchiveUriTrait for ModuleUri {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef {
        self.path.archive_uri()
    }
}
impl PathURITrait for ModuleUri {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.path.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.path.path()
    }
}
impl<'a> ArchiveUriTrait for ModuleUriRef<'a> {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef<'a> {
        self.path.archive_uri()
    }
}
impl<'a> PathURITrait for ModuleUriRef<'a> {
    #[inline]
    fn as_path(&self) -> PathURIRef<'a> {
        (*self).as_path()
    }
    #[inline]
    fn path(&self) -> Option<&'a Name> {
        self.path.path.as_ref()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, ModuleUri, ModuleUriRef};
    serialize!(DE ModuleUri);
    //serialize!(ModuleUriRef<'_>);
}
