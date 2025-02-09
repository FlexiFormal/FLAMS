use std::{borrow::Cow, fmt::Debug};

use declarations::{
    morphisms::Morphism,
    structures::{Extension, MathStructure},
    Declaration, DeclarationTrait,
};
use flams_utils::prelude::InnerArc;
use modules::{Module, NestedModule};

use crate::{uris::{ContentURIRef, ModuleURI, Name, NameStep, SymbolURI}, Checked, Resolvable};

pub mod checking;
pub mod declarations;
mod macros;
pub mod modules;
pub mod terms;

pub struct ContentReference<T: DeclarationTrait>(InnerArc<Module, T>);
impl<T:DeclarationTrait+Resolvable<From=SymbolURI>> Resolvable for ContentReference<T> {
    type From = SymbolURI;
    fn id(&self) -> Cow<'_,Self::From> {
        self.0.as_ref().id()
    }
}

impl<T: DeclarationTrait> ContentReference<T> {
    #[must_use]
    pub fn new(m: &ModuleLike, name: &Name) -> Option<Self> {
        macro_rules! get {
            () => {
                |m| {
                    if let Some(d) = m.find(name.steps()) {
                        Ok(d)
                    } else {
                        Err(())
                    }
                }
            };
        }
        let r = unsafe {
            match m {
                ModuleLike::Module(m) => InnerArc::new(m, |m| &m.0, get!()).ok()?,
                ModuleLike::NestedModule(m) => m.0.inherit(get!()).ok()?,
                ModuleLike::Structure(s) => s.0.inherit(get!()).ok()?,
                ModuleLike::Extension(e) => e.0.inherit(get!()).ok()?,
                ModuleLike::Morphism(m) => m.0.inherit(get!()).ok()?,
            }
        };
        Some(Self(r))
    }
}

impl<T: DeclarationTrait> AsRef<T> for ContentReference<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: DeclarationTrait + Debug> Debug for ContentReference<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_ref(), f)
    }
}

#[derive(Debug)]
pub enum ModuleLike {
    Module(Module),
    NestedModule(ContentReference<NestedModule<Checked>>),
    Structure(ContentReference<MathStructure<Checked>>),
    Extension(ContentReference<Extension<Checked>>),
    Morphism(ContentReference<Morphism<Checked>>),
}
impl Resolvable for ModuleLike {
    type From = ModuleURI;
    fn id(&self) -> Cow<'_,Self::From> {
        match self {
            Self::Module(m) => Cow::Borrowed(m.uri()),
            Self::NestedModule(m) => Cow::Owned(m.as_ref().uri.clone().into_module()),
            Self::Structure(s) => Cow::Owned(s.as_ref().uri.clone().into_module()),
            Self::Extension(e) => Cow::Owned(e.as_ref().uri.clone().into_module()),
            Self::Morphism(_) => todo!()//Cow::Owned(m.0.as_ref().uri.into_module()),
        }
    }
}

impl ModuleLike {
    #[must_use]
    pub fn in_module(m: &Module, name: &Name) -> Option<Self> {
        let steps = name.steps();
        if steps.is_empty() || &steps[0] != m.uri().name().last_name() {
            return None;
        }
        let steps = &steps[1..];
        if steps.is_empty() {
            return Some(Self::Module(m.clone()));
        }
        let d: &Declaration = m.find(steps)?;
        match d {
            Declaration::NestedModule(nm) => Some(Self::NestedModule(ContentReference(unsafe {
                InnerArc::new_owned_infallible(m.clone(), |m| &m.0, |_| nm)
            }))),
            Declaration::MathStructure(s) => Some(Self::Structure(ContentReference(unsafe {
                InnerArc::new_owned_infallible(m.clone(), |m| &m.0, |_| s)
            }))),
            Declaration::Extension(s) => Some(Self::Extension(ContentReference(unsafe {
                InnerArc::new_owned_infallible(m.clone(), |m| &m.0, |_| s)
            }))),
            Declaration::Morphism(s) => Some(Self::Morphism(ContentReference(unsafe {
                InnerArc::new_owned_infallible(m.clone(), |m| &m.0, |_| s)
            }))),
            _ => None,
        }
    }
}

pub trait ModuleTrait {
    fn declarations(&self) -> &[Declaration];
    fn content_uri(&self) -> ContentURIRef;
    fn find<T: DeclarationTrait>(&self, steps: &[NameStep]) -> Option<&T> {
        //println!("Trying to find {steps:?} in {}",self.content_uri());
        let mut steps = steps;
        let mut curr = self.declarations().iter();
        while !steps.is_empty() {
            let step = &steps[0];
            steps = &steps[1..];
            while let Some(c) = curr.next() {
                match c {
                    Declaration::NestedModule(m) if m.uri.name().last_name() == step => {
                        if steps.is_empty() {
                            return T::from_declaration(c);
                        }
                        curr = m.declarations().iter();
                    }
                    Declaration::MathStructure(m) if m.uri.name().last_name() == step => {
                        if steps.is_empty() {
                            return T::from_declaration(c);
                        }
                        curr = m.declarations().iter();
                    }
                    Declaration::Morphism(m)
                        if m.uri.name().last_name() == step =>
                    {
                        if steps.is_empty() {
                            return T::from_declaration(c);
                        }
                        curr = m.declarations().iter();
                    }
                    Declaration::Extension(m) if m.uri.name().last_name() == step => {
                        if steps.is_empty() {
                            return T::from_declaration(c);
                        }
                        curr = m.declarations().iter();
                    }
                    Declaration::Symbol(s) if s.uri.name().last_name() == step => {
                        return if steps.is_empty() {
                            T::from_declaration(c)
                        } else {
                            None
                        }
                    }
                    _ => (),
                }
            }
        }
        None
    }
}

impl ModuleTrait for ModuleLike {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        match self {
            Self::Module(m) => m.declarations(),
            Self::NestedModule(m) => m.as_ref().declarations(),
            Self::Structure(s) => s.as_ref().declarations(),
            Self::Extension(s) => s.as_ref().declarations(),
            Self::Morphism(s) => s.as_ref().declarations(),
        }
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        match self {
            Self::Module(m) => m.content_uri(),
            Self::NestedModule(m) => m.as_ref().content_uri(),
            Self::Structure(s) => s.as_ref().content_uri(),
            Self::Extension(s) => s.as_ref().content_uri(),
            Self::Morphism(s) => s.as_ref().content_uri(),
        }
    }
}
