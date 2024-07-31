use std::fmt::Display;
use triomphe::Arc;
pub use url::ParseError;
use crate::ontology::rdf::terms::{NamedNode,NamedNodeRef};
use crate::uris::{URIRefTrait, URITrait};

lazy_static::lazy_static! {
    static ref URIS:Arc<lasso::ThreadedRodeo<lasso::MicroSpur,rustc_hash::FxBuildHasher>> = Arc::new(lasso::ThreadedRodeo::with_hasher(rustc_hash::FxBuildHasher::default()));
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BaseURI(lasso::MicroSpur);
impl BaseURI {
    pub fn new_unchecked(s:impl AsRef<str>) -> Self {
        Self(URIS.get_or_intern(s.as_ref()))
    }
    pub fn new(s:impl AsRef<str>) -> Result<Self,ParseError> {
        let url = url::Url::parse(s.as_ref())?;
        if url.cannot_be_a_base() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        Ok(Self::new_unchecked(s))
    }
    #[inline]
    pub fn as_str(&self) -> &'static str { URIS.resolve(&self.0) }
    #[inline]
    pub fn to_iri(&self) -> NamedNodeRef<'_> { NamedNodeRef::new(self.as_str()).expect("Not a valid iri") }
}
impl serde::Serialize for BaseURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(URIS.resolve(&self.0))
    }
}
impl<'de> serde::Deserialize<'de> for BaseURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(|e| serde::de::Error::custom(e))
    }
}

impl Display for BaseURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
/*
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BaseURI(Arc<url::Url>);
impl BaseURI {
    pub fn new_unchecked<S:AsRef<str>>(s:S) -> Result<Self,ParseError> {
        let url = url::Url::parse(s.as_ref())?;
        Ok(Self(Arc::new(url)))
    }
    pub fn new<S:AsRef<str>>(s:S) -> Result<Self,ParseError> {
        let url = url::Url::parse(s.as_ref())?;
        if url.cannot_be_a_base() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(ParseError::RelativeUrlWithoutBase);
        }
        Ok(Self(Arc::new(url)))
    }

    #[inline]
    pub fn as_str(&self) -> &str { self.0.as_str() }
    pub fn to_iri_ref(&self) -> NamedNodeRef<'_> { NamedNodeRef::new(self.as_str()).unwrap() }

    #[inline]
    pub fn scheme(&self) -> &str { self.0.scheme() }
    #[inline]
    pub fn host(&self) -> Option<&str> { self.0.host_str() }
    #[inline]
    pub fn path(&self) -> &str { &self.0.path()[1..] }
    #[inline]
    pub fn authority(&self) -> &str { self.0.authority() }
}
impl URITrait for BaseURI {
    type Ref<'u> = &'u BaseURI;
    #[inline]
    fn to_iri(&self) -> NamedNode {
        self.to_iri_ref().into_owned()
    }
}
impl<'u> URIRefTrait<'u> for &'u BaseURI {
    type Owned = BaseURI;
    #[inline]
    fn to_iri(&self) -> NamedNode { URITrait::to_iri(*self) }
    #[inline]
    fn to_owned(&self) -> Self::Owned { (*self).clone() }
}
impl PartialEq<BaseURI> for &BaseURI {
    #[inline]
    fn eq(&self, other: &BaseURI) -> bool {
        self.0 == other.0
    }
}
impl PartialEq<&BaseURI> for BaseURI {
    #[inline]
    fn eq(&self, other: &&BaseURI) -> bool {
        self.0 == other.0
    }
}

impl Display for BaseURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

 */