use std::fmt::Display;
use std::ops::Div;
use std::str::FromStr;
use triomphe::Arc;
use crate::uris::base::BaseURI;
use crate::ontology::rdf::terms::NamedNode;
use crate::uris::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchiveId(Name);

impl ArchiveId {
    #[inline]
    pub fn as_str(&self) -> &str { &self.0.0 }
    #[inline]
    pub fn is_empty(&self) -> bool { self.0.0.is_empty() }
    pub fn last_name(&self) -> &str {
        self.0.0.rsplit_once('/').map(|(_, s)| s).unwrap_or(&self.0.0)
    }
    #[inline]
    pub fn steps(&self) -> std::str::Split<'_,char> {
        self.0.0.split('/')
    }
    #[inline]
    pub fn new<S: Into<Arc<str>>>(s: S) -> Self { Self(Name(s.into())) }
    pub fn is_meta(&self) -> bool {
        self.last_name().eq_ignore_ascii_case("meta-inf")
    }
}
impl FromStr for ArchiveId {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Name(s.into())))
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
        Self(Name(inner.into()))
    }
}
impl Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.0)
    }
}

impl<'a> Div<&'a ArchiveId> for &'a BaseURI {
    type Output = ArchiveURIRef<'a>;
    fn div(self, rhs: &'a ArchiveId) -> Self::Output {
        ArchiveURIRef { dom: self, archive: rhs }
    }
}
impl<S:Into<ArchiveId>> Div<S> for BaseURI {
    type Output = ArchiveURI;
    fn div(self, rhs: S) -> Self::Output {
        ArchiveURI { dom: self, archive: rhs.into() }
    }
}
impl<S:Into<ArchiveId>> Div<S> for &BaseURI {
    type Output = ArchiveURI;
    fn div(self, rhs: S) -> Self::Output {
        ArchiveURI { dom: self.clone(), archive: rhs.into() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchiveURI {
    dom: BaseURI,
    archive: ArchiveId,
}
impl ArchiveURI {
    pub fn new(dom: BaseURI, archive: ArchiveId) -> Self {
        Self { dom, archive }
    }
    #[inline]
    pub fn dom(&self) -> &BaseURI {
        &self.dom
    }
    #[inline]
    pub fn id(&self) -> &ArchiveId {
        &self.archive
    }

    #[inline]
    pub fn as_ref(&self) -> ArchiveURIRef<'_> {
        ArchiveURIRef { dom: &self.dom, archive: &self.archive }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}?a={}", self.dom(), self.id())).unwrap()
    }
}
impl Display for ArchiveURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?a={}", self.dom, self.archive)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArchiveURIRef<'a> {
    dom: &'a BaseURI,
    archive: &'a ArchiveId,
}
impl<'a> ArchiveURIRef<'a> {
    #[inline]
    pub fn base(&self) -> &'a BaseURI {
        self.dom
    }
    #[inline]
    pub fn id(&self) -> &'a ArchiveId {
        self.archive
    }
    pub fn to_owned(&self) -> ArchiveURI {
        ArchiveURI {
            dom: self.dom.clone(),
            archive: self.archive.clone(),
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}?a={}", self.base(), self.id())).unwrap()
    }
}
impl<'a> Display for ArchiveURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?a={}", self.dom, self.archive)
    }
}

impl PartialEq<ArchiveURI> for ArchiveURIRef<'_> {
    fn eq(&self, other: &ArchiveURI) -> bool {
        self.dom == other.dom && self.archive == &other.archive
    }
}
impl<'a> PartialEq<ArchiveURIRef<'a>> for ArchiveURI {
    fn eq(&self, other: &ArchiveURIRef<'a>) -> bool {
        self.dom == other.dom && &self.archive == other.archive
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use crate::uris::base::BaseURI;

    #[rstest]
    fn archive_uris(setup:()) {
        let dom = BaseURI::new("http://mathhub.info/:sTeX").unwrap();
        let id = super::ArchiveId::new("test/general");
        let borrowed = &dom / &id;
        info!("Borrowed: {borrowed}");
        let owned = dom.clone() / borrowed.id().clone();
        info!("Owned: {owned}");
        assert_eq!(borrowed, owned);
        info!("Scheme: {}, host: {}, authority: {}, path: {}",
            dom.scheme(),
            dom.host().unwrap_or(""),
            dom.authority(),
            dom.path()
        );
    }
}
