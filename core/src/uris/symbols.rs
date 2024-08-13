use std::fmt::{Debug, Display};
use std::str::FromStr;
use oxrdf::NamedNode;
use crate::uris::archives::ArchiveURI;
use crate::uris::modules::{ModuleURI};
use crate::uris::Name;

#[derive(Clone, Copy,PartialEq, Eq, Hash)]
pub struct SymbolURI {
    module: ModuleURI,
    name: Name
}

impl FromStr for SymbolURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut f = s.split("&c=");
        let module = f.next().ok_or_else(|| "No module")?;
        let module : ModuleURI = module.parse().map_err(|_| "Invalid module")?;
        let name = f.next().ok_or_else(|| "No name")?;
        if f.next().is_some() {
            return Err("Too many '?'-parts");
        }
        Ok(Self::new(module, name))
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for SymbolURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for SymbolURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}

impl Debug for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl SymbolURI {
    #[inline]
    pub fn new<S:Into<Name>>(module: ModuleURI, name: S) -> Self {
        Self { module, name:name.into() }
    }
    #[inline]
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string()
            .replace(' ',"%20")
            .replace('\\',"%5C")
            .replace('^',"%5E")
            .replace('[',"%5B")
            .replace(']',"%5D")
        ).unwrap()
    }
    #[inline]
    pub fn name(&self) -> Name { self.name }
    #[inline]
    pub fn module(&self) -> ModuleURI { self.module }
}
impl Display for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&c={}", self.module, self.name)
    }
}