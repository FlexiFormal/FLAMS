use ftml_uris::DomainUriRef;

use crate::{content::ModuleTrait, uris::SymbolUri, Checked, CheckingState, Resolvable};

use super::{Declaration, DeclarationTrait, OpenDeclaration};

#[derive(Debug)]
pub struct Morphism<State: CheckingState> {
    pub uri: SymbolUri,
    pub domain: State::ModuleLike,
    pub total: bool,
    pub elements: State::Seq<OpenDeclaration<State>>,
}
impl Resolvable for Morphism<Checked> {
    type From = SymbolUri;
    fn id(&self) -> std::borrow::Cow<'_, Self::From> {
        std::borrow::Cow::Borrowed(&self.uri)
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
    fn content_uri(&self) -> DomainUriRef {
        DomainUriRef::Symbol(&self.uri)
    }
}
crate::serde_impl! {
    struct Morphism[uri,domain,total,elements]
}
