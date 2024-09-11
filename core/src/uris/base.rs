use std::fmt::Display;
use immt_utils::triomphe::Arc;
pub use url::ParseError;
use immt_ontology::rdf::NamedNode;

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
    pub fn to_iri(&self) -> NamedNode { NamedNode::new(&self.as_str().replace(' ',"%20")).expect("Not a valid iri") }
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