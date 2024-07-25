use std::fmt::Display;
use crate::uris::archives::{ArchiveURI, ArchiveURIRef};
use crate::uris::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentURI {
    archive: ArchiveURI,
    path: Name,
    name: Name
}
impl DocumentURI {
    pub fn new<S1:Into<Name>,S2:Into<Name>>(archive: ArchiveURI, path: S1, name: S2) -> Self {
        Self { archive, path:path.into(), name:name.into() }
    }
}
impl Display for DocumentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.0.is_empty() {
            write!(f, "{}/{}", self.archive, self.name)
        } else {
            write!(f, "{}/{}/{}", self.archive, self.path, self.name)
        }
    }
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DocumentURIRef<'a> {
    archive: ArchiveURIRef<'a>,
    path: &'a Name,
    name: &'a Name
}