use crate::languages::Language;
use crate::uris::narrative::document_elements::DocumentElementURI;
use crate::uris::narrative::documents::DocumentURI;
use crate::uris::{
    debugdisplay, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURI, ContentURIRef,
    ContentURITrait, ModuleURI, PathURITrait, SymbolURI, URIOrRefTrait, URIParseError, URIRef,
    URIRefTrait, URITrait, URIWithLanguage,
};
use const_format::concatcp;
use std::fmt::Display;
use std::str::FromStr;

pub(super) mod document_elements;
pub(super) mod documents;

pub trait NarrativeURITrait: URIWithLanguage {
    fn as_narrative(&self) -> NarrativeURIRef;
    fn document(&self) -> &DocumentURI;
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum NarrativeURI {
    Document(DocumentURI),
    Element(DocumentElementURI),
}
impl From<DocumentURI> for NarrativeURI {
    #[inline]
    fn from(value: DocumentURI) -> Self {
        Self::Document(value)
    }
}
impl From<DocumentElementURI> for NarrativeURI {
    #[inline]
    fn from(value: DocumentElementURI) -> Self {
        Self::Element(value)
    }
}
impl URIOrRefTrait for NarrativeURI {
    #[inline]
    fn base(&self) -> &BaseURI {
        match self {
            Self::Document(m) => m.base(),
            Self::Element(s) => s.base(),
        }
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}
impl URITrait for NarrativeURI {
    type Ref<'a> = NarrativeURIRef<'a>;
}
impl URIWithLanguage for NarrativeURI {
    #[inline]
    fn language(&self) -> Language {
        match self {
            Self::Document(m) => m.language,
            Self::Element(s) => s.language(),
        }
    }
}
impl NarrativeURITrait for NarrativeURI {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        match self {
            Self::Document(m) => NarrativeURIRef::Document(m),
            Self::Element(s) => NarrativeURIRef::Element(s),
        }
    }
    #[inline]
    fn document(&self) -> &DocumentURI {
        match self {
            Self::Document(m) => m,
            Self::Element(s) => s.document(),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum NarrativeURIRef<'a> {
    Document(&'a DocumentURI),
    Element(&'a DocumentElementURI),
}

impl URIWithLanguage for NarrativeURIRef<'_> {
    #[inline]
    fn language(&self) -> Language {
        match self {
            Self::Document(m) => m.language,
            Self::Element(s) => s.language(),
        }
    }
}
impl<'a> From<&'a NarrativeURI> for NarrativeURIRef<'a> {
    #[inline]
    fn from(value: &'a NarrativeURI) -> Self {
        match value {
            NarrativeURI::Document(m) => Self::Document(m),
            NarrativeURI::Element(s) => Self::Element(s),
        }
    }
}
impl URIOrRefTrait for NarrativeURIRef<'_> {
    #[inline]
    fn base(&self) -> &BaseURI {
        match self {
            Self::Document(m) => m.base(),
            Self::Element(s) => s.base(),
        }
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}

impl<'a> URIRefTrait<'a> for NarrativeURIRef<'a> {
    type Owned = NarrativeURI;
    #[inline]
    fn owned(self) -> NarrativeURI {
        match self {
            Self::Document(m) => NarrativeURI::Document(m.clone()),
            Self::Element(s) => NarrativeURI::Element(s.clone()),
        }
    }
}
impl NarrativeURITrait for NarrativeURIRef<'_> {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        *self
    }
    #[inline]
    fn document(&self) -> &DocumentURI {
        match self {
            Self::Document(m) => m,
            Self::Element(s) => s.document(),
        }
    }
}
impl Display for NarrativeURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(m) => Display::fmt(m, f),
            Self::Element(s) => Display::fmt(s, f),
        }
    }
}
debugdisplay!(NarrativeURI);
impl Display for NarrativeURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(m) => Display::fmt(m, f),
            Self::Element(s) => Display::fmt(s, f),
        }
    }
}
debugdisplay!(NarrativeURIRef<'_>);

impl ArchiveURITrait for NarrativeURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.document().archive_uri()
    }
}
impl ArchiveURITrait for NarrativeURIRef<'_> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.document().archive_uri()
    }
}

impl PathURITrait for NarrativeURI {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef {
        self.document().as_path()
    }
    #[inline]
    fn path(&self) -> Option<&crate::uris::Name> {
        self.document().path()
    }
}
impl PathURITrait for NarrativeURIRef<'_> {
    #[inline]
    fn as_path(&self) -> crate::uris::PathURIRef {
        self.document().as_path()
    }
    #[inline]
    fn path(&self) -> Option<&crate::uris::Name> {
        self.document().path()
    }
}

impl FromStr for NarrativeURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DocumentURI::pre_parse(s, "narrative uri", |document, mut split| {
            let Some(c) = split.next() else {
                return Ok(Self::Document(document));
            };
            c.strip_prefix(concatcp!(DocumentElementURI::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::TooManyPartsFor {
                            uri_kind: "narrative uri",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        Ok(Self::Element(DocumentElementURI {
                            document,
                            name: name.into(),
                        }))
                    },
                )
        })
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{serialize, NarrativeURI, NarrativeURIRef};
    serialize!(DE NarrativeURI);
    serialize!(NarrativeURIRef<'_>);
}
