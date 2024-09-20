use std::fmt::Display;
use std::str::{FromStr, Split};
use const_format::concatcp;
use crate::languages::Language;
use crate::uris::{ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef, ContentURITrait, debugdisplay, ModuleURI, Name, NarrativeURIRef, NarrativeURITrait, PathURI, PathURIRef, PathURITrait, URIOrRefTrait, URIParseError, URIRef, URIWithLanguage};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DocumentURI {
    pub(in crate::uris) path: PathURI,
    pub(in crate::uris) name:Name,
    pub(in crate::uris) language:Language
}
impl DocumentURI {
    pub const SEPARATOR : char = 'd';
}
impl Display for DocumentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}&{}={}", self.path,Self::SEPARATOR, self.name,Language::SEPARATOR, self.language)
    }
}
debugdisplay!(DocumentURI);
impl URIOrRefTrait for DocumentURI {
    #[inline]
    fn base(&self) -> &BaseURI { self.path.base() }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}
impl URIWithLanguage for DocumentURI {
    #[inline]
    fn language(&self) -> Language { self.language }
}
impl NarrativeURITrait for DocumentURI {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        NarrativeURIRef::Document(self)
    }
    #[inline]
    fn document(&self) -> &DocumentURI { self }
}

impl DocumentURI {
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &Name { &self.name }
    pub(super) fn pre_parse<R>(s:&str,uri_kind:&'static str,f:impl FnOnce(Self,Split<char>) -> Result<R,URIParseError>)
                               -> Result<R,URIParseError>{
        PathURI::pre_parse(s,uri_kind,|path,next,mut split| {
            let Some(m) = next.or_else(|| split.next()) else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "document name",
                    original:s.to_string()
                });
            };
            m.strip_prefix(concatcp!(DocumentURI::SEPARATOR,"=")).map_or_else(
                || Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "document name",
                    original:s.to_string()
                }),
                |name| {
                    let Some(l) = split.next() else {
                        return Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "language",
                            original:s.to_string()
                        });
                    };
                    l.strip_prefix(concatcp!(Language::SEPARATOR,"=")).map_or_else(
                        || Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "language",
                            original:s.to_string()
                        }),
                        |lang| {
                            let language = lang.parse().map_or_else(
                                |()| Err(URIParseError::InvalidLanguage {
                                    uri_kind,
                                    original:s.to_string(),
                                }),
                                Ok
                            )?;
                            f(Self { path, name: name.into(),language },split)
                        }
                    )
                }
            )
        })
    }
}

impl FromStr for DocumentURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s,"document uri",|u,mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind:"document uri",
                    original:s.to_string()
                });
            }
            Ok(u)
        })
    }
}
impl ArchiveURITrait for DocumentURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef { self.path.archive_uri() }
}
impl PathURITrait for DocumentURI {
    #[inline]
    fn as_path(&self) -> PathURIRef {
        self.path.as_path()
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        self.path.path()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{DocumentURI, serialize};
    serialize!(DE DocumentURI);
}