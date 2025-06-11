use super::{
    checking::ModuleChecker,
    declarations::{Declaration, DeclarationTrait, OpenDeclaration},
    ModuleLike, ModuleTrait,
};
use crate::{
    languages::Language,
    uris::{ContentURIRef, ModuleURI, SymbolURI},
    Checked, CheckingState, MaybeResolved, Resolvable, Unchecked,
};
use triomphe::Arc;

#[derive(Debug, Clone)]
pub struct OpenModule<State: CheckingState> {
    pub uri: ModuleURI,
    pub meta: Option<State::Module>,
    pub signature: Option<State::Sig>,
    pub elements: State::Seq<OpenDeclaration<State>>,
}
crate::serde_impl! {mod serde_module =
    struct OpenModule[uri,meta,signature,elements]
}

#[derive(Debug, Clone)]
pub struct Signature(pub Module);
impl Resolvable for Signature {
    type From = Language;
    fn id(&self) -> std::borrow::Cow<'_, Self::From> {
        std::borrow::Cow::Owned(Language::default())
    }
}

impl ModuleTrait for OpenModule<Checked> {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Module(&self.uri)
    }
}

#[derive(Debug, Clone)]
pub struct Module(pub(super) Arc<OpenModule<Checked>>);
impl Resolvable for Module {
    type From = ModuleURI;
    fn id(&self) -> std::borrow::Cow<'_, Self::From> {
        std::borrow::Cow::Borrowed(&self.0.uri)
    }
}
impl Module {
    #[inline]
    #[must_use]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }

    #[inline]
    #[must_use]
    pub fn uri(&self) -> &ModuleURI {
        &self.0.uri
    }

    #[inline]
    #[must_use]
    pub fn meta(&self) -> Option<&MaybeResolved<Self>> {
        self.0.meta.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn signature(&self) -> Option<&MaybeResolved<Signature>> {
        self.0.signature.as_ref()
    }
}

impl ModuleTrait for Module {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.0.elements
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Module(self.uri())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::languages::Language;
    impl serde::Serialize for super::Module {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
    impl serde::Serialize for super::Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Language::default().serialize(serializer)
        }
    }
}

#[derive(Debug, Clone)]
pub struct NestedModule<State: CheckingState> {
    pub uri: SymbolURI,
    pub elements: State::Seq<OpenDeclaration<State>>,
}
impl super::declarations::private::Sealed for NestedModule<Checked> {}
impl DeclarationTrait for NestedModule<Checked> {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::NestedModule(m) => Some(m),
            _ => None,
        }
    }
}
crate::serde_impl! {mod serde_nested_module =
    struct NestedModule[uri,elements]
}
impl ModuleTrait for NestedModule<Checked> {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }

    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Symbol(&self.uri)
    }
}

impl OpenModule<Unchecked> {
    pub fn check(self, checker: &mut impl ModuleChecker) -> Module {
        let meta = self.meta.map(|uri| {
            MaybeResolved::resolve(uri, |m| {
                checker.get_module(m).and_then(|m| {
                    if let ModuleLike::Module(m) = m {
                        Some(m)
                    } else {
                        None
                    }
                })
            })
        });
        /*
        let signature = self.signature.map(|language| {
            let uri = self.uri.clone() % language;
            //println!("Require signature {uri}");
            checker.get_module(&uri).map_or_else(
                || MaybeResolved::unresolved(language),
                |m| match m {
                    ModuleLike::Module(m) => MaybeResolved::resolved(Signature(m)),
                    _ => MaybeResolved::unresolved(language),
                },
            )
        });
         */
        let elements = super::checking::ModuleCheckIter::go(self.elements, checker, &self.uri);
        Module(Arc::new(OpenModule {
            uri: self.uri,
            meta,
            signature: None,
            elements: elements.into_boxed_slice(),
        }))
    }

    /*
    /// #### Errors
    pub fn from_byte_stream(bytes: &mut impl BinaryReader) -> Result<Self, DecodeError> {
        todo!()
    }
     */
}

/*
impl<State:CheckingState> OpenModule<State> {
    /// #### Errors
    pub fn into_byte_stream(&self,bytes:&mut impl BinaryWriter) -> Result<(),std::io::Error> {
        todo!()
    }
}
 */
