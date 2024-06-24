use std::fmt::Display;
use crate::uris::modules::{ModuleURI, ModuleURIRef};
use crate::uris::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolURI {
    module: ModuleURI,
    name: Name
}
impl Display for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.module, self.name)
    }
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SymbolURIRef<'a> {
    module: ModuleURIRef<'a>,
    name: &'a Name
}