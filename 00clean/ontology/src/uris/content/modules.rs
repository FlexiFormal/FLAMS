use std::fmt::Display;
use std::str::{FromStr, Split};
use const_format::concatcp;
use crate::languages::Language;
use crate::uris::{ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef, ContentURITrait, Name, PathURI, PathURIRef, PathURITrait, URIOrRefTrait, URIRef, URIWithLanguage};
use crate::uris::errors::URIParseError;
use crate::uris::macros::debugdisplay;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModuleURI {
    pub(in crate::uris) path: PathURI,
    pub(in crate::uris) name:Name,
    pub(in crate::uris) language:Language
}
impl ModuleURI {
    pub const SEPARATOR : char = 'm';
}
impl Display for ModuleURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&{}={}&{}={}", self.path,Self::SEPARATOR, self.name,Language::SEPARATOR, self.language)
    }
}
debugdisplay!(ModuleURI);
impl URIOrRefTrait for ModuleURI {
    #[inline]
    fn base(&self) -> &BaseURI { self.path.base() }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Content(self.as_content())
    }
}
impl URIWithLanguage for ModuleURI {
    #[inline]
    fn language(&self) -> Language { self.language }
}
impl ContentURITrait for ModuleURI {
    #[inline]
    fn as_content(&self) -> ContentURIRef {
        ContentURIRef::Module(self)
    }
    #[inline]
    fn module(&self) -> &ModuleURI { self }
}

impl ModuleURI {
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &Name { &self.name }
    pub(super) fn pre_parse<R>(s:&str,uri_kind:&'static str,f:impl FnOnce(Self,Split<char>) -> Result<R,URIParseError>)
                               -> Result<R,URIParseError>{
        PathURI::pre_parse(s,uri_kind,|path,next,mut split| {
            let Some(m) = next.or_else(|| split.next()) else {
                return Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "module name",
                    original:s.to_string()
                });
            };
            m.strip_prefix(concatcp!(ModuleURI::SEPARATOR,"=")).map_or_else(
                || Err(URIParseError::MissingPartFor {
                    uri_kind,
                    part: "module name",
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
impl FromStr for ModuleURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s,"module uri",|u,mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind:"module uri",
                    original:s.to_string()
                });
            }
            Ok(u)
        })
    }
}
impl ArchiveURITrait for ModuleURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef { self.path.archive_uri() }
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

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::{ModuleURI, serialize};
    serialize!(DE ModuleURI);
}