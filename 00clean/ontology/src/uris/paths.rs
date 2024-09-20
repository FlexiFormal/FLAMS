use std::fmt::Display;
use std::str::{FromStr, Split};
use const_format::concatcp;
use either::Either;
use immt_utils::parsing::StringOrStr;
use crate::uris::{ArchiveURI, ArchiveURITrait, BaseURI, Name, URI, URIRef, URIOrRefTrait, URITrait, URIRefTrait};
use crate::uris::archives::ArchiveURIRef;
use crate::uris::errors::URIParseError;
use crate::uris::macros::debugdisplay;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PathURI {
    pub(super) archive: ArchiveURI,
    pub(super) path: Option<Name>
}
impl From<ArchiveURI> for PathURI {
    #[inline]
    fn from(archive: ArchiveURI) -> Self {
        Self { archive, path: None }
    }
}
impl PathURI {
    pub const SEPARATOR : char = 'p';
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn path(&self) -> Option<&Name> { self.path.as_ref() }
}
impl Display for PathURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_path(),f)
    }
}
debugdisplay!(PathURI);

impl URIOrRefTrait for PathURI {
    #[inline]
    fn base(&self) -> &BaseURI { &self.archive.base }
    #[inline]
    fn as_uri(&self) -> URIRef {
        URIRef::Path(self.as_path())
    }
}
impl URITrait for PathURI {
    type Ref<'a> = PathURIRef<'a>;
}
impl ArchiveURITrait for PathURI {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef { self.archive.archive_uri() }
}

#[derive(Clone, Copy,PartialEq, Eq, Hash)]
pub struct PathURIRef<'a> {
    pub(super) archive: ArchiveURIRef<'a>,
    pub(super) path: Option<&'a Name>
}
impl<'a> From<&'a PathURI> for PathURIRef<'a> {
    #[inline]
    fn from(value: &'a PathURI) -> Self {
        Self {
            archive: value.archive.archive_uri(),
            path: value.path.as_ref()
        }
    }
}
impl URIOrRefTrait for PathURIRef<'_> {
    #[inline]
    fn base(&self) -> &BaseURI { self.archive.base }
    #[inline]
    fn as_uri(&self) -> URIRef { URIRef::Path(*self) }
}
impl<'a> URIRefTrait<'a> for PathURIRef<'a> {
    type Owned = PathURI;
    #[inline]
    fn to_owned(self) -> PathURI {
        PathURI {
            archive: self.archive.to_owned(),
            path: self.path.cloned()
        }
    }
}
impl Display for PathURIRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = self.path {
            write!(f, "{}&{}={}", self.archive,PathURI::SEPARATOR, path)
        } else {
            Display::fmt(&self.archive,f)
        }
    }
}
debugdisplay!(PathURIRef<'_>);

pub trait PathURITrait : ArchiveURITrait {
    fn as_path(&self) -> PathURIRef;
    #[inline]
    fn path(&self) -> Option<&Name> { self.as_path().path }
}
impl PathURITrait for PathURI {
    fn as_path(&self) -> PathURIRef {
        PathURIRef {
            archive: self.archive.archive_uri(),
            path: self.path.as_ref()
        }
    }
}
impl PathURITrait for PathURIRef<'_> {
    #[inline]
    fn as_path(&self) -> Self { *self }
}
impl ArchiveURITrait for PathURIRef<'_> {
    #[inline]
    fn archive_uri(&self) -> ArchiveURIRef { self.archive }
}
impl PathURI {
    pub(super) fn pre_parse<R>(s:&str,uri_kind:&'static str,f:impl FnOnce(Self,Option<&str>,Split<char>) -> Result<R,URIParseError>)
        -> Result<R,URIParseError>{
        ArchiveURI::pre_parse(s,uri_kind,|archive,mut split| {
            let (p,n) = if let Some(p) = split.next() {
                if let Some(p) = p.strip_prefix(concatcp!(PathURI::SEPARATOR,"=")) {
                    (Self { archive, path:Some(p.into()) },None)
                } else {
                    (Self { archive, path:None },Some(p))
                }
            } else {
                (Self { archive, path:None },None)
            };
            f(p,n,split)
        })
    }

}
impl FromStr for PathURI {
    type Err = URIParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s,"path uri",|u,next,mut split| {
            if next.is_some() || split.next().is_some() {
                return Err(URIParseError::TooManyPartsFor {
                    uri_kind:"path uri",
                    original:s.to_string()
                });
            }
            Ok(u)
        })
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{PathURI,PathURIRef};
    use crate::uris::serialize;
    serialize!(DE PathURI);
    serialize!(PathURIRef<'_>);
}