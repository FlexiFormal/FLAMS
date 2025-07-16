use crate::uris::errors::URIParseError;
use crate::uris::macros::debugdisplay;
use crate::uris::{
    BaseUri, Name, PathURIRef, PathURITrait, URIOrRefTrait, URIRef, URIRefTrait, URITrait, URI,
};
use const_format::concatcp;
use either::Either;
use flams_utils::gc::{TArcInterned, TArcInterner};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::{FromStr, Split};
use triomphe::Arc;

#[cfg(feature = "wasm")]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
const TS_ARCHIVE_ID: &str = "export type ArchiveId = string;";

lazy_static! {
    static ref ARCHIVE_IDS: Arc<Mutex<TArcInterner<str, 4, 100>>> =
        Arc::new(Mutex::new(TArcInterner::default()));
    static ref NO_ARCHIVE_ID: ArchiveId = ArchiveId::new("no/archive");
    static ref NO_ARCHIVE_URI: ArchiveUri = "http://unknown.source?a=no/archive"
        .parse()
        .unwrap_or_else(|_| unreachable!());
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArchiveId(TArcInterned<str>);
impl ArchiveId {
    #[must_use]
    pub fn last_name(&self) -> &str {
        let s = self.as_ref();
        s.rsplit_once('/').map_or(s, |(_, s)| s)
    }

    #[inline]
    #[must_use]
    pub fn no_archive() -> Self {
        NO_ARCHIVE_ID.clone()
    }

    #[must_use]
    pub fn steps(&self) -> std::str::Split<char> {
        self.as_ref().split('/')
    }

    #[must_use]
    pub fn is_meta(&self) -> bool {
        self.last_name().eq_ignore_ascii_case("meta-inf")
    }

    #[must_use]
    pub fn new(s: &str) -> Self {
        Self(ARCHIVE_IDS.lock().get_or_intern(s))
    }
}

impl Ord for ArchiveId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        let mut left = self.steps();
        let mut right = other.steps();
        loop {
            match (left.next(), right.next()) {
                (None, None) => return Equal,
                (None, _) => return Less,
                (_, None) => return Greater,
                (Some(l), Some(r)) => match l.cmp(r) {
                    Equal => (),
                    _ if self.is_meta() && left.next().is_none() && right.next().is_none() => {
                        return Less
                    }
                    _ if other.is_meta() && left.next().is_none() && right.next().is_none() => {
                        return Greater
                    }
                    o => return o,
                },
            }
        }
    }
}
impl PartialOrd for ArchiveId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for ArchiveId {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(ARCHIVE_IDS.lock().get_or_intern(s)))
    }
}
impl AsRef<str> for ArchiveId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
debugdisplay!(ArchiveId);
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArchiveUri {
    pub(super) base: super::BaseUri,
    pub(super) archive: ArchiveId,
}
impl Display for ArchiveUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?{}={}", self.base, Self::SEPARATOR, self.archive)
    }
}
debugdisplay!(ArchiveUri);

impl ArchiveUri {
    #[must_use]
    pub fn no_archive() -> Self {
        NO_ARCHIVE_URI.clone()
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(base: BaseUri, archive: ArchiveId) -> Self {
        Self { base, archive }
    }
    pub const SEPARATOR: char = 'a';
    pub(super) fn pre_parse<R>(
        s: &str,
        uri_kind: &'static str,
        f: impl FnOnce(Self, Split<char>) -> Result<R, URIParseError>,
    ) -> Result<R, URIParseError> {
        let Either::Right((base, mut split)) = BaseUri::pre_parse(s)? else {
            return Err(URIParseError::MissingPartFor {
                uri_kind,
                part: "archive id",
                original: s.to_string(),
            });
        };
        let Some(archive) = split.next() else {
            unreachable!()
        };
        if !archive.starts_with(concatcp!(ArchiveUri::SEPARATOR, "=")) {
            return Err(URIParseError::MissingPartFor {
                uri_kind,
                part: "archive id",
                original: s.to_string(),
            });
        }
        let archive = Self {
            base,
            archive: ArchiveId::new(&archive[2..]),
        };
        f(archive, split)
    }
}
impl FromStr for ArchiveUri {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, "archive uri", |a, mut split| {
            if split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind: "archive uri",
                    original: s.to_string(),
                });
            }
            Ok(a)
        })
    }
}
impl URIOrRefTrait for ArchiveUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        &self.base
    }
    fn as_uri(&self) -> URIRef {
        URIRef::Archive(self.archive_uri())
    }
}
impl URITrait for ArchiveUri {
    type Ref<'a> = ArchiveUriRef<'a>;
}

pub trait ArchiveUriTrait: URIOrRefTrait {
    fn archive_uri(&self) -> ArchiveUriRef;

    #[inline]
    fn archive_id(&self) -> &ArchiveId {
        self.archive_uri().archive
    }
}
impl ArchiveUriTrait for ArchiveUri {
    #[inline]
    fn archive_uri(&self) -> ArchiveUriRef {
        ArchiveUriRef {
            base: &self.base,
            archive: &self.archive,
        }
    }
    #[inline]
    fn archive_id(&self) -> &ArchiveId {
        &self.archive
    }
}

impl PathURITrait for ArchiveUri {
    fn as_path(&self) -> PathURIRef {
        PathURIRef {
            archive: self.archive_uri(),
            path: None,
        }
    }
    #[inline]
    fn path(&self) -> Option<&Name> {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchiveUriRef<'a> {
    pub(super) base: &'a super::BaseUri,
    pub(super) archive: &'a ArchiveId,
}
impl<'a> ArchiveUriRef<'a> {
    #[inline]
    #[must_use]
    pub const fn new(base: &'a BaseUri, archive: &'a ArchiveId) -> Self {
        Self { base, archive }
    }
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &ArchiveId {
        self.archive
    }
}

impl<'a> URIOrRefTrait for ArchiveUriRef<'a> {
    #[inline]
    fn base(&self) -> &'a BaseUri {
        self.base
    }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::<'a>::Archive(*self)
    }
}
impl<'a> URIRefTrait<'a> for ArchiveUriRef<'a> {
    type Owned = ArchiveUri;
    #[inline]
    fn owned(self) -> ArchiveUri {
        ArchiveUri {
            base: self.base.clone(),
            archive: self.archive.clone(),
        }
    }
}

impl Display for ArchiveUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}?{}={}",
            self.base,
            ArchiveUri::SEPARATOR,
            self.archive
        )
    }
}
debugdisplay!(ArchiveUriRef<'_>);
impl<'a> PathURITrait for ArchiveUriRef<'a> {
    #[inline]
    fn as_path(&self) -> PathURIRef<'a> {
        PathURIRef {
            archive: *self,
            path: Option::<&'a Name>::None,
        }
    }
}

impl ArchiveUriTrait for ArchiveUriRef<'_> {
    #[inline]
    fn archive_uri(&self) -> Self {
        *self
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{ArchiveId, ArchiveUri};
    use crate::uris::{serialize, ArchiveUriRef};

    serialize!(as+DE ArchiveId);
    serialize!(DE ArchiveUri);
    serialize!(ArchiveUriRef<'_>);
}
