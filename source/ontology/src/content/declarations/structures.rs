use crate::{
    content::{ContentReference, ModuleTrait},
    uris::SymbolURI,
};

use super::{Declaration, DeclarationTrait, UncheckedDeclaration};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedMathStructure {
    pub uri: SymbolURI,
    pub elements: Vec<UncheckedDeclaration>,
    pub macroname: Option<Box<str>>,
}

#[derive(Debug)]
pub struct MathStructure {
    pub uri: SymbolURI,
    pub elements: Box<[Declaration]>,
    pub macroname: Option<Box<str>>,
}
impl super::private::Sealed for MathStructure {}
impl DeclarationTrait for MathStructure {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::MathStructure(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for MathStructure {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedExtension {
    pub uri: SymbolURI,
    pub target: SymbolURI,
    pub elements: Vec<UncheckedDeclaration>,
}

#[derive(Debug)]
pub struct Extension {
    pub uri: SymbolURI,
    pub target: Result<ContentReference<MathStructure>, SymbolURI>,
    pub elements: Box<[Declaration]>,
}
impl super::private::Sealed for Extension {}
impl DeclarationTrait for Extension {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::Extension(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for Extension {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
}
