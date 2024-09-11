use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Div;
use std::str::FromStr;
use immt_utils::triomphe::Arc;
use crate::narration::Language;
pub use crate::uris::archives::{ArchiveURI,ArchiveId};
pub use crate::uris::base::BaseURI;
use immt_ontology::rdf::NamedNode;
pub use crate::uris::documents::{DocumentURI, NarrativeURI, NarrDeclURI};
pub use crate::uris::modules::ModuleURI;
pub use crate::uris::symbols::SymbolURI;

pub mod base;
pub mod archives;
pub mod documents;
pub mod modules;
pub mod symbols;

lazy_static::lazy_static! {
    static ref NAMES:Arc<lasso::ThreadedRodeo<lasso::Spur,rustc_hash::FxBuildHasher>> = Arc::new(lasso::ThreadedRodeo::with_hasher(rustc_hash::FxBuildHasher::default()));
    static ref EMPTY_NAME:lasso::Spur = NAMES.get_or_intern("");
}

/*
#[derive(Clone, Copy,Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SafeURI<A>{pub get:A}
impl From<Name> for SafeURI<Name> {
    fn from(value: Name) -> Self {
        Self{get:value}
    }
}

 */

#[derive(Clone, Copy,Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(lasso::Spur);
impl Name {
    #[inline]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(NAMES.get_or_intern(s))
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

impl<A:Into<Name>> Div<A> for Name {
    type Output = Self;
    fn div(self, rhs: A) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.as_ref().to_string() + "/" + rhs.as_ref())
    }
}

#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq)]
pub enum ContentURI {
    Module(ModuleURI),
    Symbol(SymbolURI),
}
impl FromStr for ContentURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("&c=") {
            SymbolURI::from_str(s).map(ContentURI::Symbol)
        } else {
            ModuleURI::from_str(s).map(ContentURI::Module)
        }
    }
}
impl ContentURI {
    pub fn to_iri(&self) -> NamedNode {
        match self {
            ContentURI::Module(m) => m.to_iri(),
            ContentURI::Symbol(s) => s.to_iri(),
        }
    }
    pub fn name(&self) -> Name {
        match self {
            ContentURI::Module(m) => m.name(),
            ContentURI::Symbol(s) => s.name(),
        }
    }
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

#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq)]
pub enum URI {
    Archive(ArchiveURI),
    Narrative(NarrativeURI),
    Content(ContentURI),
}
impl From<ArchiveURI> for URI {
    fn from(value: ArchiveURI) -> Self {
        Self::Archive(value)
    }
}
impl<A:Into<NarrativeURI>> From<A> for URI {
    fn from(value: A) -> Self {
        Self::Narrative(value.into())
    }
}
impl From<ContentURI> for URI {
    fn from(value: ContentURI) -> Self {
        Self::Content(value)
    }
}
impl FromStr for URI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("&d=") {
            NarrativeURI::from_str(s).map(URI::Narrative)
        } else if s.contains("&m=") {
            ContentURI::from_str(s).map(URI::Content)
        } else {
            ArchiveURI::from_str(s).map(URI::Archive)
        }
    }
}
impl Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            URI::Archive(a) => Display::fmt(a, f),
            URI::Narrative(n) => Display::fmt(n, f),
            URI::Content(c) => Display::fmt(c, f),
        }
    }
}


#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq)]
pub enum NamedURI {
    Narrative(NarrativeURI),
    Content(ContentURI),
}
impl NamedURI {
    pub fn name(&self) -> Name {
        match self {
            NamedURI::Narrative(n) => n.name(),
            NamedURI::Content(c) => c.name(),
        }
    }
}
impl<A:Into<NarrativeURI>> From<A> for NamedURI {
    fn from(value: A) -> Self {
        Self::Narrative(value.into())
    }
}
impl From<ContentURI> for NamedURI {
    fn from(value: ContentURI) -> Self {
        Self::Content(value)
    }
}
impl FromStr for NamedURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("&d=") {
            NarrativeURI::from_str(s).map(NamedURI::Narrative)
        } else if s.contains("&m=") {
            ContentURI::from_str(s).map(NamedURI::Content)
        } else {
            Err("Missing name")
        }
    }
}
impl Display for NamedURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamedURI::Narrative(n) => Display::fmt(n, f),
            NamedURI::Content(c) => Display::fmt(c, f),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    impl serde::Serialize for Name {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_ref())
        }
    }
    impl<'de> serde::Deserialize<'de> for Name {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Ok(Self::new(s))
        }
    }
    impl serde::Serialize for URI {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }
    impl<'de> serde::Deserialize<'de> for URI {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(|s| serde::de::Error::custom(s))
        }
    }
    impl serde::Serialize for NamedURI {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }
    impl<'de> serde::Deserialize<'de> for NamedURI {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(|s| serde::de::Error::custom(s))
        }
    }
    /*
    impl<A:serde::Serialize> serde::Serialize for SafeURI<A> {
        #[inline]
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.get.serialize(serializer)
        }
    }
    impl<'de> serde::Deserialize<'de> for SafeURI<Name> {
        #[inline]
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            if let Some(n) = super::NAMES.get(&s) {
                Ok(Self{get:super::Name(n)})
            } else {
                Err(serde::de::Error::custom("Unknown URI-Name"))
            }
        }
    }
    
     */

    impl serde::Serialize for ContentURI {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> serde::Deserialize<'de> for ContentURI {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            if s.contains("&c=") {
                Ok(ContentURI::Symbol(s.parse().map_err(serde::de::Error::custom)?))
            } else {
                Ok(ContentURI::Module(s.parse().map_err(serde::de::Error::custom)?))
            }
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathURI {
    archive: ArchiveURI,
    path: Option<Name>,
    language:Option<Language>
}
impl PathURI {
    #[inline]
    pub fn archive(&self) -> ArchiveURI { self.archive}
    #[inline]
    pub fn path(&self) -> Option<Name> { self.path }
    #[inline]
    pub fn language(&self) -> Option<Language> { self.language }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string().replace(' ',"%20")).unwrap()
    }
    pub fn new(archive: ArchiveURI, path: Option<impl Into<Name>>,lang:Option<Language>) -> Self {
        Self { archive, path:path.map(|n| n.into()), language: lang }
    }
}
impl Display for PathURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.path,self.language) {
            (Some(p),Some(l)) => write!(f, "{}&p={}&l={}", self.archive, p,l),
            (Some(p),None) => write!(f, "{}&p={}", self.archive, p),
            (None,Some(l)) => write!(f, "{}&l={}", self.archive, l),
            _ => Display::fmt(&self.archive,f)
        }
    }
}
impl FromStr for PathURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for PathURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for PathURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}