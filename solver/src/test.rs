use crate::{
    Checker,
    context::CowLike,
    rules::CheckerRule,
    split::{CancelToken, SplitStrategy},
    state::Solvable,
    trace::{CheckLogCow, CheckingTask, RefCheckLog},
};
use ftml_ontology::terms::{ComponentVar, Term, Variable};
use smallvec::SmallVec;
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub(crate) struct Ancestor<'i> {
    pub(crate) p: &'i rustc_hash::FxHashSet<Solvable>,
    pub(crate) gp: Option<&'i Self>,
}

pub struct CheckRef<'c, 'i, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    task: Option<CheckingTask<'c>>,
    context: ContextWrap<'c, 'i>,
    pub(crate) solutions: &'i mut rustc_hash::FxHashSet<Solvable>,
    pub(crate) parent_solutions: Option<Ancestor<'i>>,
    messages: &'i mut SmallVec<CheckLogCow<'c>, 2>,
    pub(crate) cancel: &'i CancelToken<'i, Split::CancelToken>,
    added: u8,
    traced: bool,
}

impl<'c, 'i, Split: SplitStrategy> CheckRef<'c, 'i, Split> {
    pub fn extend_context<C: CowLike<'c>>(&mut self, var: C) {
        self.added += 1;
        self.context.0.push(var.into_cow());
    }

    pub fn add_msg(&mut self, line: CheckLogCow<'c>) {
        self.messages.push(line);
    }

    #[must_use]
    pub fn iter_context(&self) -> impl ExactSizeIterator<Item = &ComponentVar> {
        self.context.0.iter().rev().map(|c| &**c)
    }

    pub(crate) fn traced<R: Clone>(
        &mut self,
        tsk: CheckingTask<'c>,
        f: impl FnOnce(&mut Self) -> Option<R>,
    ) -> Result<R, RefCheckLog<'c>> {
        let (r, l) = self.traced_inner(tsk, f);
        if let Some(r) = r {
            self.messages.push(CheckLogCow::Borrowed(l));
            Ok(r)
        } else {
            Err(l)
        }
    }

    pub(crate) fn traced_inner<R: Clone>(
        &mut self,
        tsk: CheckingTask<'c>,
        f: impl FnOnce(&mut Self) -> Option<R>,
    ) -> (Option<R>, RefCheckLog<'c>) {
        let old_msg = std::mem::replace(self.messages, SmallVec::new());
        let old_task = self.task.replace(tsk);
        let ret = f(self);
        let msgs = std::mem::replace(self.messages, old_msg);
        // SAFETY: we just set it
        let old_task = unsafe { std::mem::replace(&mut self.task, old_task).unwrap_unchecked() };
        let ctx = self.context.0.as_slice();
        let ctx = &ctx[ctx.len() - self.added as usize..ctx.len()];
        let line = old_task.close(ret.as_ref(), msgs.into_boxed_slice(), ctx);
        (ret, line)
    }

    pub(crate) fn branch_traced<R: Clone>(
        &mut self,
        task: CheckingTask<'c>,
        f: impl FnOnce(CheckRef<'c, '_, Split>) -> Option<R>,
    ) -> Result<R, RefCheckLog<'c>> {
        let mut messages = SmallVec::<CheckLogCow<'c>, _>::new();
        let mut solutions = rustc_hash::FxHashSet::default();
        let inner = CheckRef {
            messages: &mut messages,
            context: ContextWrap(self.context.0),
            solutions: &mut solutions,
            parent_solutions: Some(Ancestor {
                p: self.solutions,
                gp: self.parent_solutions.as_ref(),
            }),
            top: self.top,
            task: None,
            cancel: self.cancel,
            added: 0,
            traced: self.traced,
        };
        let ret = f(inner);
        let ctx = self.context.0.as_slice();
        let ctx = &ctx[ctx.len() - self.added as usize..ctx.len()];
        let line = task.close(ret.as_ref(), messages.into_boxed_slice(), ctx);
        if let Some(r) = ret {
            self.merge_solutions(solutions);
            self.messages.push(line.into());
            Ok(r)
        } else {
            Err(line)
        }
    }

    pub fn scoped<'nt, R: Send + Sync + 'static>(
        &'nt mut self,
        f: impl FnOnce(&mut CheckRef<'nt, '_, Split>) -> R,
    ) -> R {
        let old_added = std::mem::take(&mut self.added);
        let old_msgs = std::mem::take(self.messages);

        // SAFETY:
        // - all variables added in `f` with lifetime 'nt are popped again before we return
        // - all messages added in `f` with lifetime 'nt are turned into owned ones
        //   before readding
        let muted = unsafe {
            std::mem::transmute::<&mut CheckRef<'c, '_, Split>, &mut CheckRef<'nt, '_, Split>>(self)
        };
        let r = f(muted);
        for _ in 0..std::mem::replace(&mut self.added, old_added) {
            self.context.0.pop();
        }
        for m in std::mem::replace(self.messages, old_msgs) {
            self.messages.push(m.into_owned().into());
        }
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

    pub(crate) fn copied(&self) -> CheckRefBranch<'c, 'i, Split> {
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
    messages: SmallVec<CheckLogCow<'c>, 2>,
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

pub(crate) struct CheckRefBranch<'c, 'i, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    context: SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
    solutions: rustc_hash::FxHashSet<Solvable>,
    parent_solutions: Option<Ancestor<'i>>,
    cancel: &'i CancelToken<'i, Split::CancelToken>,
    messages: SmallVec<CheckLogCow<'c>, 2>,
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
        match self.traced(CheckingTask::HasType(tm, tp), |slf| {
            slf.check_type_i(tm, tp)
        }) {
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
                    slf.scoped(|slf| slf.check_subtype(&subtp, tp))
                },
                "Using checking rules",
                |slf| {
                    let rules: Vec<&'t dyn CheckingRuleB<Split>> = todo!();
                    Split::split_test(slf, rules.into_iter(), |slf, rl| rl.apply(slf, tm, tp))
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
    pub fn infer_var_type(&mut self, var: &Variable) -> Option<Term> {
        todo!()
    }
}

pub trait CheckingRuleB<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool>;
}

impl<Split: SplitStrategy> CheckingRuleB<Split> for super::rules::pi::LambdaPiRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool {
        super::rules::pi::destruct_binder(term, &self.lambda).is_some()
            && super::rules::pi::destruct_binder(tp, &self.pi).is_some()
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        let (var, lambda_body) = super::rules::pi::destruct_binder(term, &self.lambda)?;
        let (pivar, pi_body) = super::rules::pi::destruct_binder(tp, &self.pi)?;
        let pi_tp = match &pivar.tp {
            None => Cow::Owned(checker.infer_var_type(&var.var)?),
            Some(tp) => Cow::Borrowed(tp),
        };
        let lam_tp = match &var.tp {
            None => Cow::Owned(checker.infer_var_type(&var.var)?),
            Some(tp) => Cow::Borrowed(tp),
        };
        if !checker.scoped(|checker| checker.check_subtype(&lam_tp, &pi_tp))? {
            return None;
        }
        let ntp = pi_body
            / (
                pivar.var.name(),
                &Term::Var {
                    variable: var.var.clone(),
                    presentation: None,
                },
            );
        checker.scoped(|checker| {
            checker.extend_context(var);
            checker.check_type(lambda_body, &ntp)
        })
    }
}
