use crate::{
    content::ModuleTrait, uris::{ContentURIRef, SymbolURI}, Checked, CheckingState, Resolvable
};

use super::{Declaration, DeclarationTrait, OpenDeclaration};

#[derive(Debug)]
pub struct Morphism<State:CheckingState> {
    pub uri: Option<SymbolURI>,
    pub domain: State::ModuleLike,
    pub total: bool,
    pub elements: State::Seq<OpenDeclaration<State>>,
}
impl Resolvable for Morphism<Checked> {
    type From = SymbolURI;
    fn id(&self) -> std::borrow::Cow<'_,Self::From> {
        todo!()
    }
}
impl super::private::Sealed for Morphism<Checked> {}
impl DeclarationTrait for Morphism<Checked> {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::Morphism(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for Morphism<Checked> {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Symbol(self.uri.as_ref().unwrap_or_else(|| unreachable!()))
    }
}
crate::serde_impl!{
    struct Morphism[uri,domain,total,elements]
}
