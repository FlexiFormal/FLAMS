use std::borrow::Cow;

use ftml_ontology::terms::{ComponentVar, Term};
use smallvec::SmallVec;

use crate::{
    Checker,
    context::CowLike,
    rules::SolverRule,
    split::{CancelToken, SplitStrategy},
    state::Solvable,
    trace::{SolverTask, TraceLineB, TraceLineCow},
};

pub struct CheckRef<'c, 'm, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    task: Option<SolverTask<'c>>,
    //
    context: ContextWrap<'c, 'm>,
    solutions: &'m mut rustc_hash::FxHashSet<Solvable>,
    messages: &'m mut SmallVec<TraceLineCow<'c>, 2>,
    //
    pub(crate) cancel: &'m CancelToken<'m, Split::CancelToken>,
    parent_solutions: Option<&'m rustc_hash::FxHashSet<Solvable>>,
    added: u8,
    traced: bool,
}

impl<'c, 'm, Split: SplitStrategy> CheckRef<'c, 'm, Split> {
    pub fn extend_context<C: CowLike<'c>>(&mut self, var: C) {
        self.added += 1;
        self.context.0.push(var.into_cow());
    }

    pub fn add_msg(&mut self, line: TraceLineCow<'c>) {
        self.messages.push(line);
    }

    #[must_use]
    pub fn iter_context(&self) -> impl ExactSizeIterator<Item = &ComponentVar> {
        self.context.0.iter().rev().map(|c| &**c)
    }

    pub(crate) fn traced<R>(
        &mut self,
        tsk: SolverTask<'c>,
        f: impl FnOnce(&mut Self) -> Option<R>,
    ) -> Result<R, TraceLineB<'c>> {
        let (r, l) = self.traced_inner(tsk, f);
        if let Some(r) = r {
            self.messages.push(TraceLineCow::Borrowed(l));
            Ok(r)
        } else {
            Err(l)
        }
    }

    pub(crate) fn traced_inner<R>(
        &mut self,
        tsk: SolverTask<'c>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> (R, TraceLineB<'c>) {
        let old_msg = std::mem::replace(self.messages, SmallVec::new());
        let old_task = self.task.replace(tsk);
        let ret = f(self);
        let msgs = std::mem::replace(self.messages, old_msg);
        // SAFETY: we just set it
        let old_task = unsafe { std::mem::replace(&mut self.task, old_task).unwrap_unchecked() };
        (ret, TraceLineB::from_task(old_task, msgs))
    }

    pub fn branch<R>(&mut self, f: impl FnOnce(CheckRef<'c, '_, Split>) -> R) -> R {
        let mut messages = SmallVec::<TraceLineCow<'c>, _>::new();
        let mut solutions = rustc_hash::FxHashSet::default();
        let inner = CheckRef {
            messages: &mut messages,
            context: ContextWrap(self.context.0),
            solutions: &mut solutions,
            parent_solutions: Some(self.solutions),
            top: self.top,
            task: None,
            cancel: self.cancel,
            added: 0,
            traced: self.traced,
        };
        let r = f(inner);

        drop(messages);
        drop(solutions);

        todo!();
        r
    }

    pub(crate) fn branch_scoped<'nt, R: Send + Sync + 'static>(
        &'nt mut self,
        f: impl FnOnce(CheckRef<'nt, '_, Split>) -> R,
    ) -> R {
        let nc = ContextWrap(self.context.0);
        let mut messages = SmallVec::<TraceLineCow<'c>, _>::new();
        // SAFETY: all variables added in `f` with lifetime 'nt are popped again when nc is
        // dropped, which happens at the end of `f`.
        let nc = unsafe { std::mem::transmute::<ContextWrap<'c, '_>, ContextWrap<'_, '_>>(nc) };
        let mut solutions = rustc_hash::FxHashSet::default();

        let inner = CheckRef {
            messages: &mut messages,
            solutions: &mut solutions,
            context: nc,
            top: self.top,
            task: None,
            cancel: self.cancel,
            added: 0,
            parent_solutions: Some(self.solutions),
            traced: self.traced,
        };
        let r = f(inner);

        drop(messages);
        drop(solutions);

        todo!();
        r
    }

    pub(crate) fn new_top(checker: &Checker<Split>) -> CheckRefTop<'_, Split> {
        CheckRefTop {
            context: SmallVec::new(),
            solutions: rustc_hash::FxHashSet::default(),
            messages: SmallVec::new(),
            cancel: CancelToken::default(),
            // -----------------
            top: checker,
        }
    }

    pub(crate) fn copied(&self) -> CheckRefBranch<'c, 'm, Split> {
        CheckRefBranch {
            context: SmallVec::default(),
            solutions: rustc_hash::FxHashSet::default(),
            messages: SmallVec::new(),
            // --------------
            top: self.top,
            parent_solutions: self.parent_solutions,
            cancel: self.cancel,
            traced: self.traced,
        }
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

pub(crate) struct CheckRefTop<'c, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    context: SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
    solutions: rustc_hash::FxHashSet<Solvable>,
    messages: SmallVec<TraceLineCow<'c>, 2>,
    cancel: CancelToken<'c, Split::CancelToken>,
}
impl<'c, Split: SplitStrategy> CheckRefTop<'c, Split> {
    pub fn get_ref(&'c mut self) -> CheckRef<'c, '_, Split> {
        CheckRef {
            top: self.top,
            task: None,
            messages: &mut self.messages,
            cancel: &self.cancel,
            context: ContextWrap(&mut self.context),
            added: 0,
            solutions: &mut self.solutions,
            parent_solutions: None,
            traced: true,
        }
    }
}

pub(crate) struct CheckRefBranch<'c, 'm, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    context: SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
    solutions: rustc_hash::FxHashSet<Solvable>,
    parent_solutions: Option<&'m rustc_hash::FxHashSet<Solvable>>,
    cancel: &'m CancelToken<'m, Split::CancelToken>,
    messages: SmallVec<TraceLineCow<'c>, 2>,
    traced: bool,
}
impl<'c, Split: SplitStrategy> CheckRefBranch<'c, '_, Split> {
    pub fn get_ref(&mut self) -> CheckRef<'c, '_, Split> {
        CheckRef {
            top: self.top,
            task: None,
            cancel: self.cancel,
            messages: &mut self.messages,
            context: ContextWrap(&mut self.context),
            added: 0,
            solutions: &mut self.solutions,
            parent_solutions: self.parent_solutions,
            traced: self.traced,
        }
    }
}

impl<Split: SplitStrategy> Drop for CheckRef<'_, '_, Split> {
    fn drop(&mut self) {
        for _ in 0..self.added {
            self.context.0.pop();
        }
    }
}

struct ContextWrap<'c, 's>(
    &'s mut SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
);

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub fn check_type(&mut self, tm: &'t Term, tp: &'t Term) -> Option<bool> {
        if self.cancel.is_cancelled() {
            return None;
        }
        match self.traced(SolverTask::HasType(tm, tp), |slf| slf.check_type_i(tm, tp)) {
            Ok(r) => Some(r),
            Err(l) => {
                self.add_msg(l.into());
                None
            }
        }
    }
    pub(crate) fn check_type_i(&mut self, tm: &'t Term, tp: &'t Term) -> Option<bool> {
        self.cancellable(|slf| {
            Split::strategies_test(
                slf,
                "Using type inference",
                |slf| {
                    let subtp = slf.infer_type(tm)?;
                    slf.branch_scoped(|mut slf| slf.check_subtype(&subtp, tp))
                },
                "Using checking rules",
                |slf| {
                    let rules: Vec<&'t dyn CheckingRuleB<Split>> = todo!();
                    Split::split_test(slf, rules.into_iter(), |slf, rl| {
                        slf.branch(|slf| rl.apply(slf, tm, tp))
                    })
                },
            )
        })
    }
    pub fn infer_type(&mut self, t: &'t Term) -> Option<Term> {
        todo!()
    }
    pub fn check_subtype(&mut self, sub: &'t Term, sup: &'t Term) -> Option<bool> {
        todo!()
    }
}

pub trait CheckingRuleB<Split: SplitStrategy>: SolverRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool>;
}
