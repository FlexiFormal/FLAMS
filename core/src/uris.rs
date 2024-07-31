use std::fmt::{Debug, Display};
use std::hash::Hash;
use triomphe::Arc;
use crate::uris::archives::ArchiveURI;
use crate::uris::base::BaseURI;
use crate::ontology::rdf::terms::NamedNode;

pub mod base;
pub mod archives;
pub mod documents;
pub mod modules;
pub mod symbols;

lazy_static::lazy_static! {
    static ref NAMES:Arc<lasso::ThreadedRodeo<lasso::Spur,rustc_hash::FxBuildHasher>> = Arc::new(lasso::ThreadedRodeo::with_hasher(rustc_hash::FxBuildHasher::default()));
    static ref EMPTY_NAME:lasso::Spur = NAMES.get_or_intern("");
}

#[derive(Clone, Copy,Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(lasso::Spur);
impl Name {
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(NAMES.get_or_intern(s))
    }
    #[inline]
    pub fn empty() -> Self {
        Self(*EMPTY_NAME)
    }
}
impl Display for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}
impl From<&str> for Name {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        NAMES.resolve(&self.0)
    }
}


#[cfg(feature = "serde")]
mod serde_impl {
    impl serde::Serialize for super::Name {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_ref())
        }
    }
    impl<'de> serde::Deserialize<'de> for super::Name {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Ok(Self::new(s))
        }
    }
}

pub(crate) trait URITrait:Debug+Display+Clone+Eq+Hash+PartialEq {
    type Ref<'u>: URIRefTrait<'u,Owned=Self>;
    fn to_iri(&self) -> NamedNode;
}

pub(crate) trait URIRefTrait<'u>:Debug+Display+Clone+Copy+Eq+Hash+PartialEq
    where Self:PartialEq<Self::Owned> {
    type Owned: URITrait<Ref<'u>=Self>+PartialEq<Self>;
    fn to_iri(&self) -> NamedNode;
    fn to_owned(&self) -> Self::Owned;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MMTURI {
    Base(BaseURI),
    Archive(ArchiveURI),
}
/*
impl URITrait for MMTURI {
    type Ref<'u> = Self;
    fn to_iri(&self) -> NamedNode {
        match self {
            MMTURI::Base(b) => b.to_iri().into_owned(),
            MMTURI::Archive(a) => a.to_iri(),
        }
    }
}

 */
impl Display for MMTURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MMTURI::Base(b) => Display::fmt(b,f),
            MMTURI::Archive(a) => Display::fmt(a,f),
        }
    }
}
/*
impl PartialEq<MMTURIRef<'_>> for MMTURI {
    fn eq(&self, other: &MMTURIRef<'_>) -> bool {
        match (self,other) {
            (MMTURI::Base(a),MMTURIRef::Base(b)) => a == *b,
            (MMTURI::Archive(a), MMTURIRef::Archive(b)) => a == b,
            _ => false,
        }
    }
}
impl PartialEq<MMTURI> for MMTURIRef<'_> {
    #[inline]
    fn eq(&self, other: &MMTURI) -> bool {
        other == self
    }
}

#[derive(Copy,Clone, Debug, Hash, PartialEq, Eq)]
pub enum MMTURIRef<'u> {
    Base(&'u BaseURI),
    Archive(ArchiveURI),
}
impl<'u> URIRefTrait<'u> for MMTURIRef<'u> {
    type Owned = MMTURI;
    fn to_iri(&self) -> NamedNode {
        match self {
            MMTURIRef::Base(b) => b.to_iri().into_owned(),
            MMTURIRef::Archive(a) => a.to_iri(),
        }
    }
    fn to_owned(&self) -> MMTURI {
        match *self {
            MMTURIRef::Base(b) => MMTURI::Base(b.clone()),
            MMTURIRef::Archive(a) => MMTURI::Archive(a.to_owned()),
        }
    }
}
impl Display for MMTURIRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MMTURIRef::Base(b) => Display::fmt(b,f),
            MMTURIRef::Archive(a) => Display::fmt(a,f),
        }
    }
}

 */

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContentURI {
    Module(modules::ModuleURI),
    Symbol(symbols::SymbolURI),
}
impl From<modules::ModuleURI> for ContentURI {
    fn from(value: modules::ModuleURI) -> Self {
        Self::Module(value)
    }
}
impl From<symbols::SymbolURI> for ContentURI {
    fn from(value: symbols::SymbolURI) -> Self {
        Self::Symbol(value)
    }
}
impl Display for ContentURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentURI::Module(m) => Display::fmt(m,f),
            ContentURI::Symbol(s) => Display::fmt(s,f),
        }
    }
}