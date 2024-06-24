use std::fmt::{Debug, Display};
use std::hash::Hash;
use triomphe::Arc;
use crate::uris::archives::{ArchiveURI, ArchiveURIRef};
use crate::uris::base::BaseURI;
use crate::ontology::rdf::terms::NamedNode;

pub mod base;
pub mod archives;
pub mod documents;
pub mod modules;
pub mod symbols;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Name(pub(crate) Arc<str>);
impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    impl serde::Serialize for super::Name {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.0)
        }
    }
    impl<'de> serde::Deserialize<'de> for super::Name {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Ok(Self(s.into()))
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
impl URITrait for MMTURI {
    type Ref<'u> = MMTURIRef<'u>;
    fn to_iri(&self) -> NamedNode {
        match self {
            MMTURI::Base(b) => b.to_iri(),
            MMTURI::Archive(a) => a.to_iri(),
        }
    }
}
impl Display for MMTURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MMTURI::Base(b) => Display::fmt(b,f),
            MMTURI::Archive(a) => Display::fmt(a,f),
        }
    }
}
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
    Archive(ArchiveURIRef<'u>),
}
impl<'u> URIRefTrait<'u> for MMTURIRef<'u> {
    type Owned = MMTURI;
    fn to_iri(&self) -> NamedNode {
        match self {
            MMTURIRef::Base(b) => b.to_iri(),
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

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub enum ContentURI {
    Module(modules::ModuleURI),
    Symbol(symbols::SymbolURI),
}