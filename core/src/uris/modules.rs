use std::fmt::Display;
use crate::uris::archives::{ArchiveURI, ArchiveURIRef};
use crate::uris::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModuleURI {
    archive: ArchiveURI,
    path: Name,
    name: Name
}
impl ModuleURI {
    pub fn new<S1:Into<Name>,S2:Into<Name>>(archive:ArchiveURI,path:S1,name:S2) -> Self {
        Self { archive, path:path.into(), name:name.into() }
    }
    pub fn name(&self) -> &str {
        &*self.name.0
    }
}
impl Display for ModuleURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.0.is_empty() {
            write!(f, "{}?{}", self.archive, self.name)
        } else {
            write!(f, "{}/{}?{}", self.archive, self.path, self.name)
        }
    }
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ModuleURIRef<'a> {
    archive: ArchiveURIRef<'a>,
    path: &'a Name,
    name: &'a Name
}