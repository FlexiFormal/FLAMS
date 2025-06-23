use crate::languages::Language;
use crate::uris::errors::URIParseError;
use crate::uris::macros::debugdisplay;
use crate::uris::{
    ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef, ContentURITrait, Name,
    PathURI, PathURIRef, PathURITrait, SymbolURI, URIOrRefTrait, URIRef, URIRefTrait, URITrait,
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
const TS_URI: &str = "export type ModuleURI = string;";

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModuleURI {
    pub(in crate::uris) path: PathURI,
    pub(in crate::uris) name: Name,
}

impl ModuleURI {
    pub const SEPARATOR: char = 'm';
    #[must_use]
    pub fn into_symbol(mut self) -> Option<SymbolURI> {
        let last = self.name.0.pop()?;
        if self.name.0.is_empty() {
            return None;
        }
        Some(SymbolURI {
            module: self,
            name: last.into(),
        })
    }
}
impl Display for ModuleURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.path, Self::SEPARATOR, self.name)
    }
}
debugdisplay!(ModuleURI);

/*
#[derive(Copy,Clone, PartialEq, Eq, Hash)]
pub struct ModuleURIRef<'a> {
    pub(in crate::uris) path: PathURIRef<'a>,
    pub(in crate::uris) name: &'a Name,
    pub(in crate::uris) language: Language,
}
impl Display for ModuleURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}&{}={}&{}={}",
            self.path,
            ModuleURI::SEPARATOR,
            self.name,
            Language::SEPARATOR,
            self.language
        )
    }
}
debugdisplay!(ModuleURIRef<'_>);
*/

impl URITrait for ModuleURI {
    type Ref<'a> = &'a Self; //ModuleURIRef<'a>;
}

pub type ModuleURIRef<'a> = &'a ModuleURI;

impl<'a> URIRefTrait<'a> for ModuleURIRef<'a> {
    type Owned = ModuleURI;
    #[inline]
    fn owned(self) -> Self::Owned {
        self.clone()
    }
}
/*
impl<'a> URIRefTrait<'a> for ModuleURIRef<'a> {
    type Owned = ModuleURI;
    fn owned(self) -> Self::Owned {
        ModuleURI {
            path: self.path.owned(),
            name: self.name.clone(),
            language: self.language,
        }
    }
}
*/
impl From<ModuleURI> for URI {
    #[inline]
    fn from(value: ModuleURI) -> Self {
        Self::Content(ContentURI::Module(value))
    }
}
impl<'a> From<ModuleURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(value: ModuleURIRef<'a>) -> Self {
        URIRef::Content(ContentURIRef::Module(value))
    }
}
/*
impl<'a> URIOrRefTrait for ModuleURIRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseURI {
        &self.path.archive.base
    }
    #[inline]
    fn as_uri(&self) -> URIRef<'a> {
        URIRef::Content(ContentURIRef::Module(*self))
    }
}

impl<'a> From<&'a ModuleURI> for ModuleURIRef<'a> {
    #[inline]
    fn from(value: &'a ModuleURI) -> Self {
        Self {
            path: value.as_path(),
            name: &value.name,
            language: value.language,
        }
    }
}
*/

impl URIOrRefTrait for ModuleURI {
    #[inline]
    fn base(&self) -> &BaseURI {
        self.path.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}
impl ContentURITrait for ModuleURI {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Module(self)
    }
    #[inline]
    fn module(&self) -> ModuleURIRef {
        self
    }
}
impl<'a> ContentURITrait for ModuleURIRef<'a> {
    #[inline]
    fn as_content(&self) -> ContentURIRef<'a> {
        ContentURIRef::Module(self)
    }
    #[inline]
    fn module(&self) -> Self {
        self
    }
}

impl ModuleURI {
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
            m.strip_prefix(concatcp!(ModuleURI::SEPARATOR, "="))
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
impl FromStr for ModuleURI {
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
impl ArchiveURITrait for ModuleURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.path.archive_uri()
    }
}
impl PathURITrait for ModuleURI {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.path.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.path.path()
    }
}
impl<'a> ArchiveURITrait for ModuleURIRef<'a> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef<'a> {
        self.path.archive_uri()
    }
}
impl<'a> PathURITrait for ModuleURIRef<'a> {
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
    use crate::uris::{serialize, ModuleURI, ModuleURIRef};
    serialize!(DE ModuleURI);
    //serialize!(ModuleURIRef<'_>);
}
