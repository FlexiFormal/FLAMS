use std::borrow::Cow;

use ftml_ontology::terms::ComponentVar;
use smallvec::SmallVec;

use crate::{
    Checker, SmallSet,
    split::{CancelToken, Cancellation, SplitStrategy},
    state::{Solvable, SolverState},
    trace::SolverTask,
};

pub struct CheckRef<'c, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    task: Option<SolverTask<'c>>,
    //
    context: ContextWrap<'c, 'c>,
    solutions: &'c mut rustc_hash::FxHashSet<Solvable>,
    //
    cancel: &'c CancelToken<'c, Split::CancelToken>,
    parent_solutions: Option<&'c rustc_hash::FxHashSet<Solvable>>,
    added: u8,
    traced: bool,
}
impl<Split: SplitStrategy> Drop for CheckRef<'_, Split> {
    fn drop(&mut self) {
        for _ in 0..self.added {
            self.context.0.pop();
        }
    }
}

impl<'c, Split: SplitStrategy> CheckRef<'c, Split> {
    pub fn branch<R: Send + Sync>(&mut self, f: impl FnOnce(CheckRef<'_, Split>) -> R) -> R {
        let nc = ContextWrap(self.context.0);
        // SAFETY: all variables added in `f` with lifetime 'b are popped again when nc is
        // dropped, which happens at the end of `f`.
        let nc = unsafe { std::mem::transmute::<ContextWrap<'c, '_>, ContextWrap<'_, '_>>(nc) };
        let mut solutions = rustc_hash::FxHashSet::default();
        let slf = CheckRef {
            top: self.top,
            task: None,
            cancel: self.cancel,
            context: nc,
            added: 0,
            solutions: &mut solutions,
            parent_solutions: Some(self.solutions),
            traced: self.traced,
        };
        let r = f(slf);

        todo!()
    }
    pub(crate) fn cancellable<R: Send + Sync>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old = self.cancel;
        let cancel = old.derive();
        let rf = &cancel;
        let rf: &'c CancelToken<'c, Split::CancelToken> = unsafe { std::mem::transmute(rf) };
        self.cancel = rf;
        let r = f(self);
        self.cancel = old;
        drop(cancel);
        r
    }
}
struct ContextWrap<'c, 's>(
    &'s mut SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
);
