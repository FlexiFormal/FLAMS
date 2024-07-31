use std::fmt::Display;
use oxrdf::NamedNode;
use crate::uris::archives::{ArchiveURI};
use crate::uris::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DocumentURI {
    archive: ArchiveURI,
    path: Name,
    name: Name
}
impl DocumentURI {
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string()).unwrap()
    }
    pub fn new(archive: ArchiveURI, path: impl Into<Name>, name: impl Into<Name>) -> Self {
        Self { archive, path:path.into(), name:name.into() }
    }
}
impl Display for DocumentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.as_ref().is_empty() {
            write!(f, "{}&n={}", self.archive, self.name)
        } else {
            write!(f, "{}&p={}&n={}", self.archive, self.path, self.name)
        }
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for DocumentURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for DocumentURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let mut f = s.split('&');
        let archive = f.next().ok_or_else(|| serde::de::Error::custom("No archive"))?;
        let archive : ArchiveURI = archive.parse().map_err(|_| serde::de::Error::custom("Invalid archive"))?;
        let mut maybe_path = f.next().ok_or_else(|| serde::de::Error::custom("No name"))?;
        if maybe_path.starts_with("p=") {
            let path = Name::new(&maybe_path[2..]);
            let name = f.next().ok_or_else(|| serde::de::Error::custom("No name"))?;
            if !name.starts_with("n=") {
                return Err(serde::de::Error::custom("Invalid name"));
            }
            if f.next().is_some() {
                return Err(serde::de::Error::custom("Too many '?'-parts"));
            }
            let name = Name::new(&name[2..]);
            Ok(Self::new(archive, path, name))
        } else {
            if !maybe_path.starts_with("n=") {
                return Err(serde::de::Error::custom("Invalid name"));
            }
            if f.next().is_some() {
                return Err(serde::de::Error::custom("Too many '?'-parts"));
            }
            let name = Name::new(&maybe_path[2..]);
            Ok(Self::new(archive, Name::empty(), name))
        }
    }
}

/*
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DocumentURIRef<'a> {
    archive: ArchiveURI,
    path: &'a Name,
    name: &'a Name
}
 */