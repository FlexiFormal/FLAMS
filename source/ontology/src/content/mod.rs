use std::{borrow::Cow, fmt::Debug};

use declarations::{
    morphisms::Morphism,
    structures::{Extension, MathStructure},
    Declaration, DeclarationTrait,
};
use flams_utils::prelude::InnerArc;
use ftml_uris::{IsDomainUri, UriName};
use modules::{Module, NestedModule};

use crate::{
    uris::{DomainUriRef, ModuleUri, SymbolUri},
    Checked, Resolvable,
};

pub mod checking;
pub mod declarations;
mod macros;
pub mod modules;
pub mod terms;

pub struct ContentReference<T: DeclarationTrait>(InnerArc<Module, T>);
impl<T: DeclarationTrait + Resolvable<From = SymbolUri>> Resolvable for ContentReference<T> {
    type From = SymbolUri;
    fn id(&self) -> Cow<'_, Self::From> {
        self.0.as_ref().id()
    }
}

impl<T: DeclarationTrait> ContentReference<T> {
    #[must_use]
    pub fn new(m: &ModuleLike, name: &UriName) -> Option<Self> {
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
    type From = ModuleUri;
    fn id(&self) -> Cow<'_, Self::From> {
        match self {
            Self::Module(m) => Cow::Borrowed(m.uri()),
            Self::NestedModule(m) => Cow::Owned(m.as_ref().uri.clone().into_module()),
            Self::Structure(s) => Cow::Owned(s.as_ref().uri.clone().into_module()),
            Self::Extension(e) => Cow::Owned(e.as_ref().uri.clone().into_module()),
            Self::Morphism(_) => todo!(), //Cow::Owned(m.0.as_ref().uri.into_module()),
        }
    }
}

impl ModuleLike {
    #[must_use]
    pub fn in_module(m: &Module, name: &UriName) -> Option<Self> {
        let mut steps = name.steps().peekable();
        if steps.next_if_eq(&m.uri().module_name().last()).is_none() {
            return None;
        }
        if steps.peek().is_none() {
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
    fn content_uri(&self) -> DomainUriRef;
    fn find<'s, T: DeclarationTrait>(
        &self,
        steps: impl IntoIterator<Item = &'s str>,
    ) -> Option<&T> {
        let mut steps = steps.into_iter().peekable();
        let mut curr = self.declarations().iter();
        macro_rules! ret {
            ($e:expr;$m:expr) => {{
                if steps.peek().is_none() {
                    return T::from_declaration($e);
                }
                curr = $m.declarations().iter();
            }};
        }
        while let Some(step) = steps.next() {
            while let Some(c) = curr.next() {
                match c {
                    Declaration::NestedModule(m) if m.uri.name().last() == step => ret!(c;m),
                    Declaration::MathStructure(m) if m.uri.name().last() == step => ret!(c;m),
                    Declaration::Morphism(m) if m.uri.name().last() == step => ret!(c;m),
                    Declaration::Extension(m) if m.uri.name().last() == step => ret!(c;m),
                    Declaration::Symbol(s) if s.uri.name().last() == step => {
                        return if steps.peek().is_none() {
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
    fn content_uri(&self) -> DomainUriRef {
        match self {
            Self::Module(m) => m.content_uri(),
            Self::NestedModule(m) => m.as_ref().content_uri(),
            Self::Structure(s) => s.as_ref().content_uri(),
            Self::Extension(s) => s.as_ref().content_uri(),
            Self::Morphism(s) => s.as_ref().content_uri(),
        }
    }
}
