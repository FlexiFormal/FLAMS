use std::fmt::{Debug, Display};
use crate::uris::modules::{ModuleURI, ModuleURIRef};
use crate::uris::Name;

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolURI {
    module: ModuleURI,
    name: Name
}
impl Debug for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl SymbolURI {
    pub fn new<S:Into<Name>>(module: ModuleURI, name: S) -> Self {
        Self { module, name:name.into() }
    }
}
impl Display for SymbolURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?{}", self.module, self.name)
    }
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SymbolURIRef<'a> {
    module: ModuleURIRef<'a>,
    name: &'a Name
}