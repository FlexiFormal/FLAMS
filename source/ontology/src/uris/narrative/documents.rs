use crate::languages::Language;
use crate::uris::{
    debugdisplay, ArchiveURI, ArchiveURIRef, ArchiveURITrait, BaseURI, ContentURIRef,
    ContentURITrait, ModuleURI, Name, NarrativeURIRef, NarrativeURITrait, PathURI, PathURIRef,
    PathURITrait, URIOrRefTrait, URIParseError, URIRef, URIWithLanguage,
};
use const_format::concatcp;
use std::fmt::Display;
use std::str::{FromStr, Split};

lazy_static::lazy_static! {
    static ref NO_DOCUMENT:DocumentURI = "http://unknown.source?a=no/archive&d=unknown_document&l=en".parse().unwrap_or_else(|_| unreachable!());
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DocumentURI {
    pub(in crate::uris) path: PathURI,
    pub(in crate::uris) name: Name,
    pub(in crate::uris) language: Language,
}
impl DocumentURI {
    pub const SEPARATOR: char = 'd';
    #[must_use]
    pub fn no_doc() -> Self { NO_DOCUMENT.clone()}

    #[must_use]
    pub fn from_archive_relpath(a:ArchiveURI,rel_path:&str) -> Self {
        let (path,mut name) = rel_path.rsplit_once('/')
            .unwrap_or(("",rel_path));
        name = name.rsplit_once('.').map_or_else(|| name,|(name,_)| name);
        let lang = Language::from_rel_path(name);
        name = name.strip_suffix(&format!(".{lang}")).unwrap_or(name);
        (a % path) & (name,lang)
    }
}
impl Display for DocumentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}&{}={}&{}={}",
            self.path,
            Self::SEPARATOR,
            self.name,
            Language::SEPARATOR,
            self.language
        )
    }
}
debugdisplay!(DocumentURI);
impl URIOrRefTrait for DocumentURI {
    #[inline]
    fn base(&self) -> &BaseURI {
        self.path.base()
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Narrative(self.as_narrative())
    }
}
impl URIWithLanguage for DocumentURI {
    #[inline]
    fn language(&self) -> Language {
        self.language
    }
}
impl NarrativeURITrait for DocumentURI {
    #[inline]
    fn as_narrative(&self) -> NarrativeURIRef {
        NarrativeURIRef::Document(self)
    }
    #[inline]
    fn document(&self) -> &DocumentURI {
        self
    }
}

impl DocumentURI {
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
                    part: "document name",
                    original: s.to_string(),
                });
            };
            m.strip_prefix(concatcp!(DocumentURI::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(URIParseError::MissingPartFor {
                            uri_kind,
                            part: "document name",
                            original: s.to_string(),
                        })
                    },
                    |name| {
                        let Some(l) = split.next() else {
                            return Err(URIParseError::MissingPartFor {
                                uri_kind,
                                part: "language",
                                original: s.to_string(),
                            });
                        };
                        l.strip_prefix(concatcp!(Language::SEPARATOR, "="))
                            .map_or_else(
                                || {
                                    Err(URIParseError::MissingPartFor {
                                        uri_kind,
                                        part: "language",
                                        original: s.to_string(),
                                    })
                                },
                                |lang| {
                                    let language = lang.parse().map_or_else(
                                        |()| {
                                            Err(URIParseError::InvalidLanguage {
                                                uri_kind,
                                                original: s.to_string(),
                                            })
                                        },
                                        Ok,
                                    )?;
                                    f(
                                        Self {
                                            path,
                                            name: name.into(),
                                            language,
                                        },
                                        split,
                                    )
                                },
                            )
                    },
                )
        })
    }
}

impl FromStr for DocumentURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, "document uri", |u, mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind: "document uri",
                    original: s.to_string(),
                });
            }
            Ok(u)
        })
    }
}
impl ArchiveURITrait for DocumentURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef {
        self.path.archive_uri()
    }
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
    use crate::uris::{serialize, DocumentURI};
    serialize!(DE DocumentURI);
}
