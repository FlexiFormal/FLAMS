use super::{
    checking::ModuleChecker,
    declarations::{Declaration, DeclarationTrait, UncheckedDeclaration},
    ModuleLike, ModuleTrait,
};
use crate::{
    languages::Language,
    uris::{ModuleURI, SymbolURI},
    DecodeError,
};
use immt_utils::binary::BinaryReader;
use triomphe::Arc;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedModule {
    pub uri: ModuleURI,
    pub meta: Option<ModuleURI>,
    pub signature: Option<Language>,
    pub elements: Vec<UncheckedDeclaration>,
}

#[derive(Debug)]
pub(super) struct ModuleI {
    pub uri: ModuleURI,
    pub meta: Option<Result<Module, ModuleURI>>,
    pub signature: Option<Result<Module, Language>>,
    pub elements: Box<[Declaration]>,
}
impl ModuleTrait for ModuleI {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
}

#[derive(Debug, Clone)]
pub struct Module(pub(super) Arc<ModuleI>);
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
    pub fn meta(&self) -> Option<Result<&Self, &ModuleURI>> {
        self.0.meta.as_ref().map(|r| match r {
            Ok(r) => Ok(r),
            Err(e) => Err(e),
        })
    }

    #[inline]
    #[must_use]
    pub fn signature(&self) -> Option<Result<&Self, Language>> {
        self.0.signature.as_ref().map(|r| match r {
            Ok(r) => Ok(r),
            Err(e) => Err(*e),
        })
    }
}

impl ModuleTrait for Module {
    #[inline]
    #[must_use]
    fn declarations(&self) -> &[Declaration] {
        &self.0.elements
    }
}

#[derive(Debug)]
pub struct NestedModule {
    pub uri: SymbolURI,
    pub elements: Box<[Declaration]>,
}
impl super::declarations::private::Sealed for NestedModule {}
impl DeclarationTrait for NestedModule {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::NestedModule(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for NestedModule {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
}

impl UncheckedModule {
    pub fn check(self, checker: &mut impl ModuleChecker) -> Module {
        let meta = self.meta.map(|uri| {
            //println!("Require meta {uri}");
            match checker.get_module(&uri) {
                Some(ModuleLike::Module(m)) => Ok(m),
                _ => Err(uri),
            }
        });
        let signature = self.signature.map(|language| {
            let uri = self.uri.clone() % language;
            //println!("Require signature {uri}");
            checker.get_module(&uri).map_or_else(
                || Err(language),
                |m| match m {
                    ModuleLike::Module(m) => Ok(m),
                    _ => Err(language),
                },
            )
        });
        let elements = super::checking::ModuleCheckIter::go(self.elements, checker, &self.uri);
        Module(Arc::new(ModuleI {
            uri: self.uri,
            meta,
            signature,
            elements: elements.into_boxed_slice(),
        }))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn from_byte_stream(bytes: &mut impl BinaryReader) -> Result<Self, DecodeError> {
        todo!()
    }
}
