use crate::languages::Language;
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

use crate::uris::{
    debugdisplay, ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef,
    ContentURITrait, ModuleURI, Name, PathURIRef, PathURITrait, URIOrRefTrait, URIParseError,
    URIRef, URIWithLanguage,
};

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
impl URIWithLanguage for SymbolURI {
    #[inline]
    fn language(&self) -> Language {
        self.module.language
    }
}
impl ContentURITrait for SymbolURI {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Symbol(self)
    }
    #[inline]
    fn module(&self) -> &ModuleURI {
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
                                name: name.into(),
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

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, SymbolURI};
    serialize!(DE SymbolURI);
}
