pub mod morphisms;
pub mod structures;
pub mod symbols;

use super::{modules::NestedModule, ModuleLike};
use crate::uris::{ModuleURI, SymbolURI};
use morphisms::{Morphism, UncheckedMorphism};
use structures::{Extension, MathStructure, UncheckedExtension, UncheckedMathStructure};
use symbols::Symbol;

pub(super) mod private {
    pub trait Sealed {}
}

pub trait DeclarationTrait: private::Sealed {
    fn from_declaration(decl: &Declaration) -> Option<&Self>;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UncheckedDeclaration {
    NestedModule {
        uri: SymbolURI,
        elements: Vec<UncheckedDeclaration>,
    },
    Import(ModuleURI),
    Symbol(Symbol),
    //Notation(super::constants::NotationRef),
    MathStructure(UncheckedMathStructure),
    Morphism(UncheckedMorphism),
    Extension(UncheckedExtension),
}

#[derive(Debug)]
pub enum Declaration {
    NestedModule(NestedModule),
    Import(Result<ModuleLike, ModuleURI>),
    Symbol(Symbol),
    MathStructure(MathStructure),
    Morphism(Morphism),
    Extension(Extension),
}

impl private::Sealed for Declaration {}
impl DeclarationTrait for Declaration {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        Some(decl)
    }
}
