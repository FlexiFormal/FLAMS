/*! # URIs
 * 
 * ## Grammar
 * 
 * | Type  |     | Cases/Def | Trait | Reference |
 * |----------- |---- | -----|-------|---------|
 * | [URI]      | ::= | [BaseURI]⏐[ArchiveURI]⏐[PathURI]⏐[ContentURI]⏐[NarrativeURI] | [URITrait] | [URIRef] |
 * | [BaseURI]  | ::= | (URL with no query/fragment) | - | `&`[BaseURI] |
 * | [ArchiveURI] | ::= | [BaseURI]`?a=`[ArchiveId] | [ArchiveURITrait] | [ArchiveURIRef] |
 * | [PathURI]  | ::= | [ArchiveURI]`[&p=`[Name]`]` | [PathURITrait] | [PathURIRef] |
 * | [ContentURI] | ::= | [ModuleURI]⏐[SymbolURI]   | [ContentURITrait] | [ContentURIRef] |
 * | [NarrativeURI] | ::= | [DocumentURI]⏐[DocumentElementURI] | [NarrativeURITrait] | [NarrativeURIRef] |
 * | [ModuleURI] | ::= | [PathURI]`&m=`[Name]`&l=`[Language] | - | `&`[ModuleURI] |
 * | [SymbolURI] | ::= | [ModuleURI]`&s=`[Name] | - | `&`[SymbolURI] |
 * | [DocumentURI] | ::= | [PathURI]`&d=`[Name]`&l=`[Language] | - | `&`[DocumentURI] |
 * | [DocumentElementURI] | ::= | [DocumentURI]`&e=`[Name] | - | `&`[DocumentElementURI] |
*/

#![allow(unused_macros)]
#![allow(unused_imports)]

mod archives;
mod base;
mod content;
mod errors;
mod infix;
mod name;
mod narrative;
mod paths;
pub mod terms;

pub use archives::{ArchiveId, ArchiveURI, ArchiveURIRef, ArchiveURITrait};
pub use base::BaseURI;
pub use content::{
    modules::ModuleURI, symbols::SymbolURI, ContentURI, ContentURIRef, ContentURITrait,
};
pub use errors::URIParseError;
pub use name::{Name, NameStep};
pub use narrative::{
    document_elements::DocumentElementURI, documents::DocumentURI, NarrativeURI, NarrativeURIRef,
    NarrativeURITrait,
};
pub use paths::{PathURI, PathURIRef, PathURITrait};

use const_format::concatcp;
use either::Either;
use immt_utils::parsing::StringOrStr;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::{FromStr, Split};

pub(crate) mod macros {
    macro_rules! debugdisplay {
        ($s:ty) => {
            impl std::fmt::Debug for $s {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(self, f)
                }
            }
        };
    }
    macro_rules! serialize {
        (as+DE $s:ty) => {
            serialize!(as $s);
            impl<'de> serde::Deserialize<'de> for $s {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let s = String::deserialize(deserializer)?;
                    s.parse().map_err(serde::de::Error::custom)
                }
            }
        };
        (as $s:ty) => {
            impl serde::Serialize for $s {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.serialize_str(self.as_ref())
                }
            }
        };
        (DE $s:ty) => {
            serialize!($s);
            impl<'de> serde::Deserialize<'de> for $s {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let s = String::deserialize(deserializer)?;
                    s.parse().map_err(serde::de::Error::custom)
                }
            }
        };
        ($s:ty) => {
            impl serde::Serialize for $s {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.collect_str(self)
                }
            }
        };
    }
    pub(crate) use {debugdisplay, serialize};
}
use crate::languages::Language;
pub(crate) use macros::{debugdisplay, serialize};

mod sealed {
    pub trait Sealed {}
}

macro_rules! common {
    () => {
        #[cfg(feature = "rdf")]
        fn to_iri(&self) -> crate::rdf::NamedNode {
            crate::rdf::NamedNode::new(immt_utils::escaping::IRI_ESCAPE.escape(self).to_string())
                .unwrap_or_else(|_| unreachable!())
        }
        fn base(&self) -> &BaseURI;
        fn as_uri(&self) -> URIRef;
    };
}

#[cfg(not(feature = "serde"))]
pub trait URIOrRefTrait: Display + Debug + Clone + Hash + sealed::Sealed {
    common! {}
}

#[cfg(feature = "serde")]
pub trait URIOrRefTrait:
    Display + Debug + Clone + Hash + serde::Serialize + sealed::Sealed
{
    common! {}
}

#[cfg(not(feature = "serde"))]
pub trait URITrait: URIOrRefTrait + Into<URI> {
    type Ref<'a>: URIRefTrait<'a, Owned = Self>; // where &'a Self:Into<Self::Ref<'a>>;
}

#[cfg(feature = "serde")]
pub trait URITrait: URIOrRefTrait + serde::Deserialize<'static> + Into<URI> {
    type Ref<'a>: URIRefTrait<'a, Owned = Self>; // where &'a Self:Into<Self::Ref<'a>>;
}

pub trait URIRefTrait<'a>: URIOrRefTrait + Copy + Into<URIRef<'a>> {
    type Owned: URITrait<Ref<'a> = Self>;
    fn owned(self) -> Self::Owned;
}

pub trait URIWithLanguage: URIOrRefTrait {
    fn language(&self) -> Language;
}
impl<'a, A: URITrait<Ref<'a> = &'a A> + URIWithLanguage> URIWithLanguage for &'a A {
    #[inline]
    fn language(&self) -> Language {
        A::language(self)
    }
}

impl<'a, U: URITrait> sealed::Sealed for &'a U {}
impl<'a, U: URITrait> URIOrRefTrait for &'a U {
    #[inline]
    fn base(&self) -> &BaseURI {
        (*self).base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        (*self).as_uri()
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum URI {
    Base(BaseURI),
    Archive(ArchiveURI),
    Path(PathURI),
    Content(ContentURI),
    Narrative(NarrativeURI),
}
impl sealed::Sealed for URI {}
impl sealed::Sealed for BaseURI {}
impl sealed::Sealed for ArchiveURI {}
impl sealed::Sealed for PathURI {}
impl sealed::Sealed for ContentURI {}
impl sealed::Sealed for ModuleURI {}
impl sealed::Sealed for SymbolURI {}
impl sealed::Sealed for NarrativeURI {}
impl sealed::Sealed for DocumentURI {}
impl sealed::Sealed for DocumentElementURI {}

impl From<BaseURI> for URI {
    #[inline]
    fn from(b: BaseURI) -> Self {
        Self::Base(b)
    }
}
impl From<ArchiveURI> for URI {
    #[inline]
    fn from(b: ArchiveURI) -> Self {
        Self::Archive(b)
    }
}
impl From<PathURI> for URI {
    #[inline]
    fn from(b: PathURI) -> Self {
        Self::Path(b)
    }
}
impl From<ContentURI> for URI {
    #[inline]
    fn from(b: ContentURI) -> Self {
        Self::Content(b)
    }
}
impl From<NarrativeURI> for URI {
    #[inline]
    fn from(b: NarrativeURI) -> Self {
        Self::Narrative(b)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum URIRef<'a> {
    Base(&'a BaseURI),
    Archive(ArchiveURIRef<'a>),
    Path(PathURIRef<'a>),
    Content(ContentURIRef<'a>),
    Narrative(NarrativeURIRef<'a>),
}
impl sealed::Sealed for ArchiveURIRef<'_> {}
impl sealed::Sealed for PathURIRef<'_> {}
impl sealed::Sealed for URIRef<'_> {}
impl sealed::Sealed for ContentURIRef<'_> {}
impl sealed::Sealed for NarrativeURIRef<'_> {}
impl<'a> From<&'a BaseURI> for URIRef<'a> {
    #[inline]
    fn from(b: &'a BaseURI) -> Self {
        Self::Base(b)
    }
}
impl<'a> From<&'a ArchiveURI> for URIRef<'a> {
    #[inline]
    fn from(b: &'a ArchiveURI) -> Self {
        Self::Archive(b.archive_uri())
    }
}
impl<'a> From<PathURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(b: PathURIRef<'a>) -> Self {
        Self::Path(b)
    }
}
impl<'a> From<ArchiveURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(b: ArchiveURIRef<'a>) -> Self {
        Self::Archive(b)
    }
}
impl<'a> From<ContentURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(b: ContentURIRef<'a>) -> Self {
        Self::Content(b)
    }
}
impl<'a> From<NarrativeURIRef<'a>> for URIRef<'a> {
    #[inline]
    fn from(b: NarrativeURIRef<'a>) -> Self {
        Self::Narrative(b)
    }
}

macro_rules! inherit {
    ($self:ident = $b:ident => $fun:expr) => {
        match $self {
            Self::Base($b) => $fun,
            Self::Archive($b) => $fun,
            Self::Path($b) => $fun,
            Self::Content($b) => $fun,
            Self::Narrative($b) => $fun,
        }
    };
}

impl Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        inherit!(self = b => Display::fmt(b,f))
    }
}
debugdisplay!(URI);
impl Display for URIRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        inherit!(self = b => Display::fmt(b,f))
    }
}
debugdisplay!(URIRef<'_>);

impl URIOrRefTrait for URIRef<'_> {
    #[inline]
    fn base(&self) -> &BaseURI {
        inherit!(self = b => b.base())
    }
    fn as_uri(&self) -> URIRef {
        *self
    }
}

impl URI {
    fn parse_content(
        s: &str,
        module: &str,
        (language, next): (Language, Option<&str>),
        path: impl FnOnce() -> PathURI,
        mut split: Split<char>,
    ) -> Result<ContentURI, URIParseError> {
        let name = move || module.into();
        let module = move || ModuleURI {
            path: path(),
            name: name(),
            language,
        };
        let Some(next) = next else {
            return Ok(ContentURI::Module(module()));
        };
        next.strip_prefix(concatcp!(SymbolURI::SEPARATOR, "="))
            .map_or_else(
                || {
                    Err(URIParseError::UnrecognizedPart {
                        original: s.to_string(),
                    })
                },
                |symbol| {
                    if split.next().is_some() {
                        Err(URIParseError::TooManyPartsFor {
                            uri_kind: "symbol uri",
                            original: s.to_string(),
                        })
                    } else {
                        Ok(ContentURI::Symbol(SymbolURI {
                            module: module(),
                            name: symbol.into(),
                        }))
                    }
                },
            )
    }
    fn parse_narrative(
        s: &str,
        document: &str,
        (language, next): (Language, Option<&str>),
        path: impl FnOnce() -> PathURI,
        mut split: Split<char>,
    ) -> Result<NarrativeURI, URIParseError> {
        let name = move || document.into();
        let document = move || DocumentURI {
            path: path(),
            name: name(),
            language,
        };
        let Some(next) = next else {
            return Ok(NarrativeURI::Document(document()));
        };
        next.strip_prefix(concatcp!(DocumentElementURI::SEPARATOR, "="))
            .map_or_else(
                || {
                    Err(URIParseError::UnrecognizedPart {
                        original: s.to_string(),
                    })
                },
                |element| {
                    if split.next().is_some() {
                        Err(URIParseError::TooManyPartsFor {
                            uri_kind: "document element uri",
                            original: s.to_string(),
                        })
                    } else {
                        Ok(NarrativeURI::Element(DocumentElementURI {
                            document: document(),
                            name: element.into(),
                        }))
                    }
                },
            )
    }
}

impl FromStr for URI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, mut split) = match BaseURI::pre_parse(s)? {
            Either::Left(base) => return Ok(Self::Base(base)),
            Either::Right(c) => c,
        };
        let Some(next) = split.next() else {
            unreachable!()
        };
        next.strip_prefix(concatcp!(ArchiveURI::SEPARATOR, "="))
            .map_or_else(
                || {
                    Err(URIParseError::UnrecognizedPart {
                        original: s.to_string(),
                    })
                },
                |archive| {
                    let archive = move || ArchiveURI {
                        base,
                        archive: archive.parse().unwrap_or_else(|_| unreachable!()),
                    };
                    let Some(next) = split.next() else {
                        return Ok(Self::Archive(archive()));
                    };
                    let (path, next) =
                        if let Some(path) = next.strip_prefix(concatcp!(PathURI::SEPARATOR, "=")) {
                            (
                                Either::Left(|| PathURI {
                                    archive: archive(),
                                    path: Some(path.into()),
                                }),
                                split.next(),
                            )
                        } else {
                            (
                                Either::Right(|| PathURI {
                                    archive: archive(),
                                    path: None,
                                }),
                                Some(next),
                            )
                        };
                    let path = move || match path {
                        Either::Left(p) => p(),
                        Either::Right(p) => p(),
                    };
                    let Some(next) = next else {
                        return Ok(Self::Path(path()));
                    };
                    let mut language = || {
                        split.next().map_or_else(
                            || Ok((Language::default(), None)),
                            |n| {
                                n.strip_prefix(concatcp!(Language::SEPARATOR, "="))
                                    .map_or_else(
                                        || Ok((Language::default(), Some(n))),
                                        |l| l.parse()
                                            .map_err(|()| URIParseError::InvalidLanguage {
                                                uri_kind: "uri",
                                                original: s.to_string(),
                                            })
                                            .map(|l| (l, split.next())),
                                    )
                            },
                        )
                    };
                    if let Some(module) = next.strip_prefix(concatcp!(ModuleURI::SEPARATOR, "=")) {
                        Ok(Self::Content(Self::parse_content(
                            s,
                            module,
                            language()?,
                            path,
                            split,
                        )?))
                    } else if let Some(document) =
                        next.strip_prefix(concatcp!(DocumentURI::SEPARATOR, "="))
                    {
                        Ok(Self::Narrative(Self::parse_narrative(
                            s,
                            document,
                            language()?,
                            path,
                            split,
                        )?))
                    } else {
                        Err(URIParseError::UnrecognizedPart {
                            original: s.to_string(),
                        })
                    }
                },
            )
    }
}
impl URIOrRefTrait for URI {
    #[inline]
    fn base(&self) -> &BaseURI {
        inherit!(self = b => b.base())
    }
    fn as_uri(&self) -> URIRef {
        match self {
            Self::Base(b) => URIRef::Base(b),
            Self::Archive(a) => URIRef::Archive(a.archive_uri()),
            Self::Path(p) => URIRef::Path(p.as_path()),
            Self::Content(c) => URIRef::Content(c.as_content()),
            Self::Narrative(n) => URIRef::Narrative(n.as_narrative()),
        }
    }
}

impl URITrait for URI {
    type Ref<'a> = URIRef<'a>;
}
impl<'a> URIRefTrait<'a> for URIRef<'a> {
    type Owned = URI;
    fn owned(self) -> URI {
        match self {
            Self::Base(b) => URI::Base(b.clone()),
            Self::Archive(a) => URI::Archive(a.owned()),
            Self::Path(p) => URI::Path(p.owned()),
            Self::Content(c) => URI::Content(c.owned()),
            Self::Narrative(n) => URI::Narrative(n.owned()),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{serialize, URIRef, URI};
    serialize!(DE URI);
    serialize!(URIRef<'_>);
}
