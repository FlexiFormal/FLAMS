use std::fmt::Display;
use std::num::NonZeroU16;
use std::ops::{BitAnd, Div};
use std::str::FromStr;
use immt_utils::triomphe::Arc;
use crate::uris::base::BaseURI;
use immt_ontology::rdf::NamedNode;
use crate::uris::Name;

lazy_static::lazy_static! {
    static ref IDS:Arc<lasso::ThreadedRodeo<lasso::MiniSpur,rustc_hash::FxBuildHasher>> = Arc::new(lasso::ThreadedRodeo::with_hasher(rustc_hash::FxBuildHasher::default()));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArchiveId(lasso::MiniSpur);
impl ArchiveId {
    pub fn num(self) -> NonZeroU16 {
        self.0.into_inner()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ArchiveId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(IDS.resolve(&self.0))
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ArchiveId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let r = Self::new(&s);
        Ok(r)
    }
}

impl ArchiveId {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        IDS.resolve(&self.0)
    }
    #[inline]
    pub fn is_empty(&self) -> bool { self.as_str().is_empty() }
    pub fn last_name(&self) -> &'static str {
        let s = self.as_str();
        s.rsplit_once('/').map(|(_, s)| s).unwrap_or(s)
    }
    #[inline]
    pub fn steps(&self) -> std::str::Split<'static,char> {
        self.as_str().split('/')
    }
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        let r = IDS.get_or_intern(s.as_ref());
        Self(r)
    }
    pub fn is_meta(&self) -> bool {
        self.last_name().eq_ignore_ascii_case("meta-inf")
    }
}
impl FromStr for ArchiveId {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Self::new(s);
        Ok(r)
    }
}
impl<S: AsRef<str>,I:IntoIterator<Item=S>> From<I> for ArchiveId {
    fn from(v: I) -> Self {
        let mut inner = String::new();
        for s in v {
            inner.push_str(s.as_ref());
            inner.push('/');
        }
        inner.pop();
        let r = Self::new(&inner);
        r
    }
}
impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl BitAnd<ArchiveId> for BaseURI {
    type Output = ArchiveURI;
    fn bitand(self, rhs: ArchiveId) -> Self::Output {
        ArchiveURI { base: self, archive: rhs }
    }
}

/*
impl<'a> Div<&'a ArchiveId> for &'a BaseURI {
    type Output = ArchiveURI;
    fn div(self, rhs: &'a ArchiveId) -> Self::Output {
        ArchiveURI { base: *self, archive: *rhs }
    }
}
impl<S:Into<ArchiveId>> Div<S> for BaseURI {
    type Output = ArchiveURI;
    fn div(self, rhs: S) -> Self::Output {
        ArchiveURI { base: self, archive: rhs.into() }
    }
}
impl<S:Into<ArchiveId>> Div<S> for &BaseURI {
    type Output = ArchiveURI;
    fn div(self, rhs: S) -> Self::Output {
        ArchiveURI { base: *self, archive: rhs.into() }
    }
}

 */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArchiveURI {
    base: BaseURI,
    archive: ArchiveId,
}
impl FromStr for ArchiveURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('?');
        let base = parts.next().ok_or("Not a valid archive URI")?;
        let archive = parts.next().ok_or("Not a valid archive URI")?;
        if !archive.starts_with("a=") {
            return Err("Not a valid archive URI");
        }
        let archive = ArchiveId::new(&archive[2..]);
        Ok(ArchiveURI {
            base: BaseURI::new(base).map_err(|_| "Not a valid URI")?,
            archive,
        })
    }

}

#[cfg(feature = "serde")]
impl serde::Serialize for ArchiveURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
#[cfg(feature = "serde")]
impl<'d> serde::Deserialize<'d> for ArchiveURI {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("Invalid ArchiveURI"))
    }
}

impl ArchiveURI {
    pub fn new(dom: BaseURI, archive: ArchiveId) -> Self {
        Self { base: dom, archive }
    }
    #[inline]
    pub fn base(&self) -> BaseURI {
        self.base
    }
    #[inline]
    pub const fn id(&self) -> ArchiveId {
        self.archive
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}?a={}", self.base(), self.id()).replace(' ',"%20")).unwrap()
    }
}
impl Display for ArchiveURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?a={}", self.base, self.archive)
    }
}