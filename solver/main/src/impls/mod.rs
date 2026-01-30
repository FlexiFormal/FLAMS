pub mod backend;
mod equality;
mod inference;
mod preparation;
pub mod solving;
mod typing;

use crate::{
    CheckRef, Checker,
    context::CowLike,
    impls::solving::{Ancestor, Solvable},
    split::{CancelToken, SplitStrategy},
    trace::{CheckLogCow, CheckingTask, PreCheckLog, RefCheckLog},
};
use ftml_ontology::terms::ComponentVar;
use smallvec::SmallVec;
use std::borrow::Cow;

impl<'c, 'i, Split: SplitStrategy> CheckRef<'c, 'i, Split> {
    pub fn extend_context<C: CowLike<'c>>(&mut self, var: C) {
        self.added += 1;
        self.context.0.push(var.into_cow());
    }

    pub fn add_msg(&mut self, line: CheckLogCow<'c>) {
        self.messages.push(line);
    }
    pub fn comment(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.messages.push(CheckLogCow::Owned(PreCheckLog::Msg(
            msg.into(),
            crate::trace::MessageLevel::Comment,
        )));
    }
    pub fn counter(&mut self, msg: &'static str, num: usize) {
        self.messages
            .push(CheckLogCow::Owned(PreCheckLog::Count(msg, num)))
    }
    pub fn failure(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.messages.push(CheckLogCow::Owned(PreCheckLog::Msg(
            msg.into(),
            crate::trace::MessageLevel::Failure,
        )));
    }
    #[inline]
    pub(crate) fn split(&mut self) -> (&[Cow<'c, ComponentVar>], Trace<'c, '_>) {
        (self.context.0, Trace(self.messages))
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
        task: CheckingTask<'c>,
        f: impl FnOnce(&mut Self) -> Option<R>,
    ) -> (Option<R>, RefCheckLog<'c>) {
        let old_msg = std::mem::replace(self.messages, SmallVec::new());
        let ret = f(self);
        let msgs = std::mem::replace(self.messages, old_msg);
        let ctx = self.context.0.as_slice();
        let ctx = &ctx[ctx.len() - self.added as usize..ctx.len()];
        let line = task.close(ret.as_ref(), msgs.into_boxed_slice(), ctx);
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

    pub(crate) fn wrap_check<R: Clone>(
        &mut self,
        task: CheckingTask<'c>,
        f: impl FnOnce(&mut Self) -> Option<R>,
    ) -> Option<R> {
        if self.cancel.is_cancelled() {
            return None;
        }
        match self.traced(task, f) {
            Ok(r) => Some(r),
            Err(l) => {
                self.add_msg(l.into());
                None
            }
        }
    }
}

impl<Split: SplitStrategy> Checker<Split> {
    pub(crate) fn wrap_task<'t, R: std::fmt::Debug + Clone + 'static, F>(
        &'t self,
        task: CheckingTask<'t>,
        then: F,
    ) -> (Option<R>, rustc_hash::FxHashSet<Solvable>, PreCheckLog)
    where
        F: FnOnce(CheckRef<'t, '_, Split>) -> Option<R>,
    {
        let mut context = SmallVec::new();
        let mut solutions = rustc_hash::FxHashSet::default();
        let mut messages = SmallVec::new();
        let cancel = CancelToken::default();
        let rf = CheckRef {
            top: self,
            messages: &mut messages,
            cancel: &cancel,
            context: ContextWrap(&mut context),
            added: 0,
            solutions: &mut solutions,
            parent_solutions: None,
            traced: true,
        };
        let r = then(rf);
        let line = task
            .close(r.as_ref(), messages.into_boxed_slice(), &context)
            .into_owned();
        tracing::debug!("Solutions:{solutions:#?}");
        (r, solutions, line)
    }

    pub(crate) fn wrap_none<'t, R: std::fmt::Debug + Clone + 'static, F>(&'t self, then: F) -> R
    where
        F: FnOnce(CheckRef<'t, '_, Split>) -> R,
    {
        let mut context = SmallVec::new();
        let mut solutions = rustc_hash::FxHashSet::default();
        let mut messages = SmallVec::new();
        let cancel = CancelToken::default();
        let rf = CheckRef {
            top: self,
            messages: &mut messages,
            cancel: &cancel,
            context: ContextWrap(&mut context),
            added: 0,
            solutions: &mut solutions,
            parent_solutions: None,
            traced: true,
        };
        then(rf)
    }
}

pub struct CheckRefBranch<'c, 'i, Split: SplitStrategy> {
    top: &'c Checker<Split>,
    context: SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
    solutions: rustc_hash::FxHashSet<Solvable>,
    parent_solutions: Option<Ancestor<'i>>,
    cancel: &'i CancelToken<'i, Split::CancelToken>,
    messages: SmallVec<CheckLogCow<'c>, 2>,
    traced: bool,
}
impl<'c, Split: SplitStrategy> CheckRefBranch<'c, '_, Split> {
    pub const fn get_ref(&mut self) -> CheckRef<'c, '_, Split> {
        CheckRef {
            top: self.top,
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

pub struct ContextWrap<'c, 's>(
    pub(crate) &'s mut SmallVec<Cow<'c, ComponentVar>, { super::context::CONTEXT_LEN }>,
);

pub struct Trace<'c, 'i>(&'i mut SmallVec<CheckLogCow<'c>, 2>);
impl<'c> Trace<'c, '_> {
    pub fn add_msg(&mut self, line: CheckLogCow<'c>) {
        self.0.push(line);
    }
    pub fn comment(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.0.push(CheckLogCow::Owned(PreCheckLog::Msg(
            msg.into(),
            crate::trace::MessageLevel::Comment,
        )));
    }
    pub fn failure(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.0.push(CheckLogCow::Owned(PreCheckLog::Msg(
            msg.into(),
            crate::trace::MessageLevel::Failure,
        )));
    }
}
