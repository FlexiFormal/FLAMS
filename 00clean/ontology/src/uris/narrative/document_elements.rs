use std::fmt::Display;
use std::str::{FromStr, Split};
use const_format::concatcp;
use crate::languages::Language;
use crate::uris::{ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef, ContentURITrait, debugdisplay, DocumentURI, ModuleURI, Name, NarrativeURIRef, NarrativeURITrait, PathURIRef, PathURITrait, SymbolURI, URIOrRefTrait, URIParseError, URIRef, URIWithLanguage};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DocumentElementURI {
    pub(in crate::uris) document:DocumentURI,
    pub(in crate::uris) name:Name,
}
impl DocumentElementURI {
    pub const SEPARATOR : char = 'e';
}
impl Display for DocumentElementURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}", self.document,Self::SEPARATOR, self.name)
    }
}
debugdisplay!(DocumentElementURI);
impl URIOrRefTrait for DocumentElementURI {
    #[inline]
    fn base(&self) -> &BaseURI { self.document.base() }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}
impl URIWithLanguage for DocumentElementURI {
    #[inline]
    fn language(&self) -> Language { self.document.language }
}
impl NarrativeURITrait for DocumentElementURI {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        NarrativeURIRef::Element(self)
    }
    #[inline]
    fn document(&self) -> &DocumentURI { &self.document }
}
impl DocumentElementURI {

    #[inline]
    #[must_use]
    pub const fn name(&self) -> &Name { &self.name }

    pub(super) fn pre_parse<R>(s:&str,uri_kind:&'static str,f:impl FnOnce(Self,Split<char>) -> Result<R,URIParseError>)
                               -> Result<R,URIParseError>{
        DocumentURI::pre_parse(s,uri_kind,|document,mut split| {
            let Some(s) = split.next() else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "narrative element name",
                    original:s.to_string()
                });
            };
            s.strip_prefix(concatcp!(DocumentElementURI::SEPARATOR,"=")).map_or_else(
                || Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "narrative element name",
                    original:s.to_string()
                }),
                |name| f(Self { document, name: name.into() }, split)
            )
        })
    }
}

impl FromStr for DocumentElementURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s,"document element uri",|u,mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind:"document element uri",
                    original:s.to_string()
                });
            }
            Ok(u)
        })
    }
}

impl ArchiveURITrait for DocumentElementURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef { self.document.archive_uri() }
}
impl PathURITrait for DocumentElementURI {
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
    use crate::uris::{DocumentElementURI, serialize};
    serialize!(DE DocumentElementURI);
}
