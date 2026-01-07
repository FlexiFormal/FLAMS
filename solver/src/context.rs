use std::borrow::Cow;

use ftml_ontology::terms::ComponentVar;
use smallvec::SmallVec;

pub(crate) const CONTEXT_LEN: usize = 4;

pub struct ContextTop<'c>(SmallVec<Cow<'c, ComponentVar>, CONTEXT_LEN>);
impl<'c> ContextTop<'c> {
    pub const fn build(&mut self) -> Context<'c, '_> {
        Context {
            vars: &mut self.0,
            added: 0,
        }
    }
}

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

pub struct Context<'c, 'a> {
    vars: &'a mut SmallVec<Cow<'c, ComponentVar>, CONTEXT_LEN>,
    added: usize,
}
impl Drop for Context<'_, '_> {
    fn drop(&mut self) {
        for _ in 0..self.added {
            let _ = self.vars.pop();
        }
    }
}
impl<'c> Context<'c, '_> {
    #[must_use]
    pub const fn new_top() -> ContextTop<'c> {
        ContextTop(SmallVec::new())
    }
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ComponentVar> {
        self.vars.iter().rev().map(|c| &**c)
    }

    pub fn in_branch<'a, R>(&'a mut self, then: impl FnOnce(Context<'a, 'a>) -> R) -> R
    where
        'c: 'a,
    {
        let nc = Context {
            vars: self.vars,
            added: 0,
        };
        // SAFETY: all variables added in `then` with lifetime 'a are popped again when nc is
        // dropped, which happens at the end of `then`.
        let nc = unsafe { std::mem::transmute::<Context<'c, '_>, Context<'a, '_>>(nc) };
        then(nc)
    }

    pub fn extend<C: CowLike<'c>>(&mut self, var: C) {
        self.added += 1;
        self.vars.push(var.into_cow());
    }
    /*
    pub fn extend_owned(&mut self, var: ComponentVar) {
        self.added += 1;
        self.vars.push(Cow::Owned(var));
    }
     */
    pub const fn branch(&mut self) -> Context<'c, '_> {
        //crate::update_stack();
        Context {
            vars: self.vars,
            added: 0,
        }
    }
    #[must_use]
    pub fn clone_top(&self) -> ContextTop<'c> {
        ContextTop(self.vars.clone())
    }
    pub(crate) fn to_boxed(&self) -> Box<[ComponentVar]> {
        self.vars
            .iter()
            .map(|c| (*c).clone().into_owned())
            .collect()
    }
}
