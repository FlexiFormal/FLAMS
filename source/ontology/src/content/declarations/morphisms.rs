use crate::{
    content::{ModuleLike, ModuleTrait},
    uris::{ModuleURI, SymbolURI},
};

use super::{Declaration, DeclarationTrait, UncheckedDeclaration};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedMorphism {
    pub uri: Option<SymbolURI>,
    pub domain: ModuleURI,
    pub total: bool,
    pub elements: Vec<UncheckedDeclaration>,
}

#[derive(Debug)]
pub struct Morphism {
    pub uri: Option<SymbolURI>,
    pub domain: Result<ModuleLike, ModuleURI>,
    pub total: bool,
    pub elements: Box<[Declaration]>,
}
impl super::private::Sealed for Morphism {}
impl DeclarationTrait for Morphism {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::Morphism(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for Morphism {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
}
