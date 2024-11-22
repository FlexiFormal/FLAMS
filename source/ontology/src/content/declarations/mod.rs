pub mod morphisms;
pub mod structures;
pub mod symbols;

use super::modules::NestedModule;
use crate::{ Checked, CheckingState};
use morphisms::Morphism;
use structures::{Extension, MathStructure};
use symbols::Symbol;

pub(super) mod private {
    pub trait Sealed {}
}
pub trait DeclarationTrait: private::Sealed+std::fmt::Debug {
    fn from_declaration(decl: &Declaration) -> Option<&Self>;
}

#[derive(Debug)]
pub enum OpenDeclaration<State:CheckingState> {
    NestedModule(NestedModule<State>),
    Import(State::ModuleLike),
    Symbol(Symbol),
    MathStructure(MathStructure<State>),
    Morphism(Morphism<State>),
    Extension(Extension<State>),
}

pub type Declaration = OpenDeclaration<Checked>;
impl private::Sealed for Declaration {}
impl DeclarationTrait for Declaration {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        Some(decl)
    }
}

crate::serde_impl! {
    enum OpenDeclaration{
        {0 = NestedModule(nm)}
        {1 = Import(ml)}
        {2 = Symbol(s)}
        {3 = MathStructure(s)}
        {4 = Morphism(m)}
        {5 = Extension(e)}
    }
}

/*
crate::serde_impl! {
    OpenDeclaration : slf
    ser => {
        match slf {
            Self::NestedModule(nm) => {
                ser.serialize_newtype_variant(
                    "OpenDeclaration", 
                    0, 
                    "NestedModule", nm
                )
            }
            _ => todo!()
        }
    }
    de => {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = OpenDeclaration<Unchecked>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(stringify!(OpenDeclaration))
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (v,var) = data.variant()?;
                match v {
                    "NestedModule" => {
                        var.newtype_variant().map(|e| OpenDeclaration::NestedModule(e))
                    }
                    _ => todo!()
                }
                
            }
        }
        de.deserialize_enum(
            "OpenDeclaration", 
            &["NestedModule","Import","Symbol","MathStructure","Morphism","Extension"], 
            Visitor
        )
    }
}
*/