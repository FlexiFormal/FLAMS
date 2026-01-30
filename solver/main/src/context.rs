use std::borrow::Cow;

use ftml_ontology::terms::ComponentVar;

pub(crate) const CONTEXT_LEN: usize = 4;

pub trait CowLike<'a> {
    fn into_cow(self) -> Cow<'a, ComponentVar>;
}
impl<'a> CowLike<'a> for ComponentVar {
    #[inline]
    fn into_cow(self) -> Cow<'a, ComponentVar> {
        Cow::Owned(self)
    }
}
impl<'a> CowLike<'a> for &'a ComponentVar {
    #[inline]
    fn into_cow(self) -> Cow<'a, ComponentVar> {
        Cow::Borrowed(self)
    }
}
