use std::fmt::Display;
use std::str::FromStr;
use const_format::concatcp;
use crate::languages::Language;
use crate::uris::{ArchiveURIRef, ArchiveURITrait, BaseURI, debugdisplay, ModuleURI, Name, PathURITrait, URI, URIOrRefTrait, URIParseError, URIRef, URIRefTrait, URITrait, URIWithLanguage};
use crate::uris::content::symbols::SymbolURI;

pub(super) mod modules;
pub(super) mod symbols;

#[allow(clippy::module_name_repetitions)]
pub trait ContentURITrait:URIWithLanguage {
    fn as_content(&self) -> ContentURIRef;
    fn module(&self) -> &ModuleURI;
}

#[derive(Clone, Hash, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub enum ContentURI {
    Module(ModuleURI),
    Symbol(SymbolURI)
}
impl ContentURI {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &Name {
        match self {
            Self::Module(m) => m.name(),
            Self::Symbol(s) => s.name()
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
            Self::Module(m) => ContentURIRef::Module(m),
            Self::Symbol(s) => ContentURIRef::Symbol(s),
        }
    }
    #[inline]
    fn module(&self) -> &ModuleURI {
        match self {
            Self::Module(m) => m,
            Self::Symbol(s) => s.module(),
        }
    }
}

impl<'a,A:ContentURITrait+URITrait<Ref<'a>=&'a A>> ContentURITrait for &'a A {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        A::as_content(*self)
    }
    #[inline]
    fn module(&self) -> &ModuleURI {
        A::module(*self)
    }
}

#[derive(Clone, Copy,Hash, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub enum ContentURIRef<'a> {
    Module(&'a ModuleURI),
    Symbol(&'a SymbolURI)
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
            ContentURI::Module(m) => Self::Module(m),
            ContentURI::Symbol(s) => Self::Symbol(s),
        }
    }
}
impl<'a> URIOrRefTrait for ContentURIRef<'a> {
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
impl<'a> URIRefTrait<'a> for ContentURIRef<'a> {
    type Owned = ContentURI;
    #[inline]
    fn to_owned(self) -> ContentURI {
        match self {
            Self::Module(m) => ContentURI::Module(m.clone()),
            Self::Symbol(s) => ContentURI::Symbol(s.clone()),
        }
    }
}
impl ContentURITrait for ContentURIRef<'_> {
    #[inline]
    fn as_content(&self) -> ContentURIRef { *self }
    #[inline]
    fn module(&self) -> &ModuleURI {
        match self {
            Self::Module(m) => m,
            Self::Symbol(s) => s.module(),
        }
    }
}

impl Display for ContentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => Display::fmt(m,f),
            Self::Symbol(s) => Display::fmt(s,f),
        }
    }
}
debugdisplay!(ContentURI);

impl Display for ContentURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => Display::fmt(m,f),
            Self::Symbol(s) => Display::fmt(s,f),
        }
    }
}
debugdisplay!(ContentURIRef<'_>);
impl ArchiveURITrait for ContentURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.module().archive_uri()
    }
}
impl ArchiveURITrait for ContentURIRef<'_> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.module().archive_uri()
    }
}

impl PathURITrait for ContentURI {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef {
        self.module().as_path()
    }
    #[inline]
    fn path(&self) -> Option<&crate::uris::Name> {
        self.module().path()
    }
}
impl PathURITrait for ContentURIRef<'_> {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef {
        self.module().as_path()
    }
    #[inline]
    fn path(&self) -> Option<&crate::uris::Name> {
        self.module().path()
    }
}

impl FromStr for ContentURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ModuleURI::pre_parse(s, "content uri", |module, mut split| {
            let Some(c) = split.next() else { return Ok(Self::Module(module)) };
            c.strip_prefix(concatcp!(SymbolURI::SEPARATOR,"=")).map_or_else(
                || Err(URIParseError::TooManyPartsFor {
                    uri_kind: "content uri",
                    original: s.to_string()
                }),
                |name| Ok(Self::Symbol(SymbolURI { module, name: name.into() })
                )
            )
        })
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{ContentURI, ContentURIRef, serialize};
    serialize!(DE ContentURI);
    serialize!(ContentURIRef<'_>);
}