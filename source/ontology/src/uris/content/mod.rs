use crate::languages::Language;
use crate::uris::content::symbols::SymbolURI;
use crate::uris::{
    debugdisplay, ArchiveURIRef, ArchiveURITrait, BaseURI, ModuleURI, Name, PathURITrait,
    URIOrRefTrait, URIParseError, URIRef, URIRefTrait, URITrait, URIWithLanguage, URI,
};
use const_format::concatcp;
use modules::ModuleURIRef;
use symbols::SymbolURIRef;
use std::fmt::Display;
use std::str::FromStr;

pub(super) mod modules;
pub(super) mod symbols;

pub trait ContentURITrait: URIWithLanguage {
    fn as_content(&self) -> ContentURIRef;
    fn module(&self) -> ModuleURIRef;
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ContentURI {
    Module(ModuleURI),
    Symbol(SymbolURI),
}
impl ContentURI {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &Name {
        match self {
            Self::Module(m) => m.name(),
            Self::Symbol(s) => s.name(),
        }
    }
}
impl From<ModuleURI> for ContentURI {
    #[inline]
    fn from(value: ModuleURI) -> Self {
        Self::Module(value)
    }
}
impl From<SymbolURI> for ContentURI {
    #[inline]
    fn from(value: SymbolURI) -> Self {
        Self::Symbol(value)
    }
}
impl URIOrRefTrait for ContentURI {
    #[inline]
    fn base(&self) -> &BaseURI {
        match self {
            Self::Module(m) => m.base(),
            Self::Symbol(s) => s.base(),
        }
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}
impl URITrait for ContentURI {
    type Ref<'a> = ContentURIRef<'a>;
}
impl URIWithLanguage for ContentURI {
    #[inline]
    fn language(&self) -> Language {
        match self {
            Self::Module(m) => m.language,
            Self::Symbol(s) => s.language(),
        }
    }
}
impl ContentURITrait for ContentURI {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        match self {
            Self::Module(m) => ContentURIRef::Module(m.module()),
            Self::Symbol(s) => ContentURIRef::Symbol(s),
        }
    }
    #[inline]
    fn module(&self) -> ModuleURIRef {
        match self {
            Self::Module(m) => m.module(),
            Self::Symbol(s) => s.module(),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ContentURIRef<'a> {
    Module(ModuleURIRef<'a>),
    Symbol(SymbolURIRef<'a>),
}
impl URIWithLanguage for ContentURIRef<'_> {
    #[inline]
    fn language(&self) -> Language {
        match self {
            Self::Module(m) => m.language,
            Self::Symbol(s) => s.language(),
        }
    }
}
impl<'a> From<&'a ContentURI> for ContentURIRef<'a> {
    #[inline]
    fn from(value: &'a ContentURI) -> Self {
        match value {
            ContentURI::Module(m) => Self::Module(m.module()),
            ContentURI::Symbol(s) => Self::Symbol(s),
        }
    }
}
impl<'a> URIOrRefTrait for ContentURIRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseURI {
        match self {
            Self::Module(m) => &m.path.archive.base,
            Self::Symbol(s) => &s.module.path.archive.base,
        }
    }
    #[inline]
    fn as_uri(&self) -> URIRef<'a> {
        URIRef::Content(*self)
    }
}
impl<'a> URIRefTrait<'a> for ContentURIRef<'a> {
    type Owned = ContentURI;
    #[inline]
    fn owned(self) -> ContentURI {
        match self {
            Self::Module(m) => ContentURI::Module(m.owned()),
            Self::Symbol(s) => ContentURI::Symbol(s.owned()),
        }
    }
}
impl<'a> ContentURITrait for ContentURIRef<'a> {
    #[inline]
    fn as_content(&self) -> Self {
        *self
    }
    #[inline]
    fn module(&self) -> ModuleURIRef<'a> {
        match self {
            Self::Module(m) => m,
            Self::Symbol(s) => &s.module,
        }
    }
}

impl Display for ContentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => Display::fmt(m, f),
            Self::Symbol(s) => Display::fmt(s, f),
        }
    }
}
debugdisplay!(ContentURI);

impl Display for ContentURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => Display::fmt(m, f),
            Self::Symbol(s) => Display::fmt(s, f),
        }
    }
}
debugdisplay!(ContentURIRef<'_>);
impl ArchiveURITrait for ContentURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        match self {
            Self::Module(m) => m.archive_uri(),
            Self::Symbol(s) => s.module.path.archive_uri(),
        }
    }
}
impl<'a> ArchiveURITrait for ContentURIRef<'a> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef<'a> {
        match self {
            Self::Module(m) => m.path.archive_uri(),
            Self::Symbol(s) => s.module.path.archive_uri(),
        }
    }
}

impl PathURITrait for ContentURI {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef {
        match self {
            Self::Module(m) => m.as_path(),
            Self::Symbol(s) => s.as_path(),
        }
    }
    #[inline]
    fn path(&self) -> Option<&crate::uris::Name> {
        match self {
            Self::Module(m) => m.path(),
            Self::Symbol(s) => s.path(),
        }
    }
}
impl<'a> PathURITrait for ContentURIRef<'a> {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef<'a> {
        match self {
            Self::Module(m) => (**m).as_path(),
            Self::Symbol(s) => s.module.as_path(),
        }
    }
    #[inline]
    fn path(&self) -> Option<&'a crate::uris::Name> {
        match self {
            Self::Module(m) => m.path.path.as_ref(),
            Self::Symbol(s) => s.module.path.path.as_ref(),
        }
    }
}

impl FromStr for ContentURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ModuleURI::pre_parse(s, "content uri", |module, mut split| {
            let Some(c) = split.next() else {
                return Ok(Self::Module(module));
            };
            c.strip_prefix(concatcp!(SymbolURI::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::TooManyPartsFor {
                            uri_kind: "content uri",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        Ok(Self::Symbol(SymbolURI {
                            module,
                            name: name.into(),
                        }))
                    },
                )
        })
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, ContentURI, ContentURIRef};
    serialize!(DE ContentURI);
    serialize!(ContentURIRef<'_>);
}
