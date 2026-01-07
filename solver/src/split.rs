use rayon::iter::{IntoParallelIterator, ParallelIterator};
use smallvec::SmallVec;

use crate::{
    SolverRef,
    context::Context,
    rules::{
        SolverRule,
        extractors::{RuleExtractor, SymbolRuleExtractor},
    },
    trace::{SolverTask, SolverTrace, TraceLine},
};

pub trait Cancellation: Default {
    fn is_cancelled(&self) -> bool;
    fn cancel(&self);
}
#[derive(Default)]
pub struct CancelToken<'a, C: Cancellation> {
    cancelled: C,
    parent: Option<&'a Self>,
}
impl Cancellation for std::sync::atomic::AtomicBool {
    #[inline]
    fn is_cancelled(&self) -> bool {
        self.load(std::sync::atomic::Ordering::Acquire)
    }
    #[inline]
    fn cancel(&self) {
        self.store(true, std::sync::atomic::Ordering::Release);
    }
}
impl<'a, C: Cancellation> CancelToken<'a, C> {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
            || self.parent.as_ref().is_some_and(|tk| {
                tk.is_cancelled() && {
                    self.cancelled.cancel();
                    true
                }
            })
    }
    #[inline]
    pub fn cancel(&self) {
        self.cancelled.cancel();
    }
    pub fn derive(&self) -> CancelToken<'_, C> {
        CancelToken {
            cancelled: C::default(),
            parent: Some(self),
        }
    }
}
impl Cancellation for () {
    #[inline]
    fn is_cancelled(&self) -> bool {
        false
    }
    #[inline(always)]
    fn cancel(&self) {}
}

pub trait SplitStrategy:
    Send + Sync + Sized + 'static + Copy + Clone + Default + std::fmt::Debug
{
    type CancelToken: Cancellation;
    const SYMBOL_EXTRACTORS: &[SymbolRuleExtractor<Self>] =
        { super::rules::extractors::all_symbol_extractors() };

    const RULE_EXTRACTORS: &[RuleExtractor<Self>] =
        { super::rules::extractors::all_rule_extractors() };

    fn strategies<'t, A, B, R>(
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        strategy_a: &'static str,
        oper_a: A,
        strategy_b: &'static str,
        oper_b: B,
    ) -> Option<R>
    where
        A: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        B: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        R: Send + std::fmt::Debug;

    /// ### Errors
    fn split_i<'t, 'r, Rl: SolverRule + ?Sized, R: Send + std::fmt::Debug + 'static>(
        slf: SolverRef<Self>,
        trace: &mut SolverTrace,
        rules: impl Iterator<Item = &'r Rl>, //smallvec::SmallVec<&Rl, 2>,
        context: Context<'t, '_>,
        then: impl Fn(SolverRef<Self>, &Rl, &mut SolverTrace, Context<'t, '_>) -> Option<R>
        + Send
        + Sync,
    ) -> Result<R, smallvec::SmallVec<TraceLine, 2>>;

    fn split<'t, 'r, Rl: SolverRule + ?Sized, R: Send + std::fmt::Debug + 'static>(
        slf: SolverRef<Self>,
        trace: &mut SolverTrace,
        rules: impl Iterator<Item = &'r Rl>, //smallvec::SmallVec<&Rl, 2>,
        context: Context<'t, '_>,
        then: impl Fn(SolverRef<Self>, &Rl, &mut SolverTrace, Context<'t, '_>) -> Option<R>
        + Send
        + Sync,
    ) -> Option<R> {
        match Self::split_i(slf, trace, rules, context, then) {
            Ok(r) => Some(r),
            Err(ls) => {
                for e in ls {
                    trace.add_line(e);
                }
                None
            }
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct SingleThreadedSplit;
impl SplitStrategy for SingleThreadedSplit {
    type CancelToken = ();
    #[inline]
    fn strategies<'t, A, B, R>(
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        strategy_a: &'static str,
        oper_a: A,
        strategy_b: &'static str,
        oper_b: B,
    ) -> Option<R>
    where
        A: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        B: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        R: Send + std::fmt::Debug,
    {
        let mut nt = trace.derive(SolverTask::Strategy(strategy_a));
        let r = oper_a(&mut nt, context.branch());
        //let (r, l1) = trace.derived(SolverTask::Strategy(strategy_a), context.branch(), oper_a);
        if r.is_some() {
            trace.add_line(nt.destroy(r.as_ref(), &context));
            return r;
        }
        let mut nt2 = trace.derive(SolverTask::Strategy(strategy_b));
        let r2 = oper_b(&mut nt2, context.branch());
        //let (r, l2) = trace.derived(SolverTask::Strategy(strategy_b), context.branch(), oper_b);
        if r2.is_none() {
            let l2 = nt2.destroy(r2.as_ref(), &context);
            trace.add_line(nt.destroy(r.as_ref(), &context));
            trace.add_line(l2);
            None
        } else {
            trace.add_line(nt2.destroy(r2.as_ref(), &context));
            r2
        }
    }
    fn split_i<'t, 'r, Rl: SolverRule + ?Sized, R: Send + std::fmt::Debug + 'static>(
        slf: SolverRef<Self>,
        trace: &mut SolverTrace,
        rules: impl Iterator<Item = &'r Rl>, //smallvec::SmallVec<&Rl, 2>,
        mut context: Context<'t, '_>,
        then: impl Fn(SolverRef<Self>, &Rl, &mut SolverTrace, Context<'t, '_>) -> Option<R>
        + Send
        + Sync,
    ) -> Result<R, smallvec::SmallVec<TraceLine, 2>> {
        let mut rules = rules.peekable();
        if rules.peek().is_none() {
            return Err(smallvec::smallvec![TraceLine::NoRuleApplicable]);
        }
        let mut failures = SmallVec::<_, 2>::new();
        for rule in rules {
            let mut nt = trace.derive(SolverTask::Rule(rule.as_dyn()));
            let r = then(slf, rule, &mut nt, context.branch());
            /*
            let (r, inner) = trace.derived(
                SolverTask::Rule(rule.as_dyn()),
                context.branch(),
                |t, context| then(slf, rule, t, context),
            );
            */

            if let Some(r) = r {
                drop(failures);
                trace.add_line(nt.destroy(Some(&r), &context));
                return Ok(r);
            }
            failures.push(nt);
        }
        // double iteration to get rid of *immutable* borrows of trace first
        // (add_line requires mutable!)
        let lines = failures
            .into_iter()
            .map(|t| t.destroy(None::<&R>, &context))
            .collect();
        Err(lines)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct RayonStrategiesOnly;
impl SplitStrategy for RayonStrategiesOnly {
    type CancelToken = std::sync::atomic::AtomicBool;
    #[inline]
    fn strategies<'t, A, B, R>(
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        strategy_a: &'static str,
        oper_a: A,
        strategy_b: &'static str,
        oper_b: B,
    ) -> Option<R>
    where
        A: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        B: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        R: Send + std::fmt::Debug,
    {
        RayonSplit::strategies(trace, context, strategy_a, oper_a, strategy_b, oper_b)
    }
    fn split_i<'t, 'r, Rl: SolverRule + ?Sized, R: Send + std::fmt::Debug + 'static>(
        slf: SolverRef<Self>,
        trace: &mut SolverTrace,
        rules: impl Iterator<Item = &'r Rl>, //smallvec::SmallVec<&Rl, 2>,
        mut context: Context<'t, '_>,
        then: impl Fn(SolverRef<Self>, &Rl, &mut SolverTrace, Context<'t, '_>) -> Option<R>
        + Send
        + Sync,
    ) -> Result<R, smallvec::SmallVec<TraceLine, 2>> {
        let mut rules = rules.peekable();
        if rules.peek().is_none() {
            return Err(smallvec::smallvec![TraceLine::NoRuleApplicable]);
        }
        let mut failures = SmallVec::<_, 2>::new();
        for rule in rules {
            let mut nt = trace.derive(SolverTask::Rule(rule.as_dyn()));
            let r = then(slf, rule, &mut nt, context.branch());
            /*
            let (r, inner) = trace.derived(
                SolverTask::Rule(rule.as_dyn()),
                context.branch(),
                |t, context| then(slf, rule, t, context),
            );
            */

            if let Some(r) = r {
                drop(failures);
                trace.add_line(nt.destroy(Some(&r), &context));
                return Ok(r);
            }
            failures.push(nt);
        }
        // double iteration to get rid of *immutable* borrows of trace first
        // (add_line requires mutable!)
        let lines = failures
            .into_iter()
            .map(|t| t.destroy(None::<&R>, &context))
            .collect();
        Err(lines)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct RayonSplit;
impl SplitStrategy for RayonSplit {
    type CancelToken = std::sync::atomic::AtomicBool;
    #[inline]
    fn strategies<'t, A, B, R>(
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        strategy_a: &'static str,
        oper_a: A,
        strategy_b: &'static str,
        oper_b: B,
    ) -> Option<R>
    where
        A: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        B: FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R> + Send,
        R: Send + std::fmt::Debug,
    {
        let mut top = context.clone_top();
        let (a, b) = rayon::join(
            || {
                let context = top.build();
                let (r, l) = trace.derived(SolverTask::Strategy(strategy_a), context, oper_a);
                if r.is_some() {
                    trace.cancel();
                }
                (r, l)
            },
            || {
                let (r, l) = trace.derived(SolverTask::Strategy(strategy_b), context, oper_b);
                if r.is_some() {
                    trace.cancel();
                }
                (r, l)
            },
        );
        match (a, b) {
            ((Some(t), inner), _) | (_, (Some(t), inner)) => {
                trace.add_line(inner);
                Some(t)
            }
            ((None, i1), (None, i2)) => {
                trace.add_line(i1);
                trace.add_line(i2);
                None
            }
        }
    }

    fn split_i<'t, 'r, Rl: SolverRule + ?Sized, R: Send + std::fmt::Debug + 'static>(
        slf: SolverRef<Self>,
        trace: &mut SolverTrace,
        rules: impl Iterator<Item = &'r Rl>, //smallvec::SmallVec<&Rl, 2>,
        context: Context<'t, '_>,
        then: impl Fn(SolverRef<Self>, &Rl, &mut SolverTrace, Context<'t, '_>) -> Option<R>
        + Send
        + Sync,
    ) -> Result<R, smallvec::SmallVec<TraceLine, 2>> {
        macro_rules! then {
            ($rl:ident !) => {{
                let mut top = context.clone_top();
                let context = top.build();
                let then = &then;
                let r = trace.derived(
                    SolverTask::Rule($rl.as_dyn()),
                    context,
                    move |t, context| then(slf, $rl, t, context),
                );
                if r.0.is_some() {
                    trace.cancel();
                }
                r
            }};
            ($rl:expr) => {{
                trace.derived(SolverTask::Rule($rl.as_dyn()), context, |t, context| {
                    then(slf, $rl, t, context)
                })
            }};
        }
        let mut rules: smallvec::SmallVec<_, 2> = rules.collect();
        match rules.len() {
            0 => return Err(smallvec::smallvec![TraceLine::NoRuleApplicable]),
            1 => {
                // SAFETY: len == 1
                let rule = unsafe { rules.pop().unwrap_unchecked() };
                let (res, inner) = then!(rule);
                return if let Some(r) = res {
                    trace.add_line(inner);
                    Ok(r)
                } else {
                    Err(smallvec::smallvec![inner])
                };
            }
            2 => {
                // SAFETY: len == 2
                let rule_a = unsafe { rules.pop().unwrap_unchecked() };
                let rule_b = unsafe { rules.pop().unwrap_unchecked() };
                return match rayon::join(|| then!(rule_a!), || then!(rule_b!)) {
                    ((Some(t), inner), _) | (_, (Some(t), inner)) => {
                        trace.add_line(inner);
                        Ok(t)
                    }
                    ((None, i1), (None, i2)) => Err(smallvec::smallvec_inline![i1, i2]),
                };
            }
            _ => (),
        }
        let result = parking_lot::Mutex::new(None);
        let failures = parking_lot::Mutex::new(SmallVec::<_, 2>::new());
        rules.into_vec().into_par_iter().for_each(|rule| {
            if trace.is_cancelled() {
                return;
            }
            let (r, inner) = then!(rule!);
            if let Some(r) = r {
                *result.lock() = Some((r, inner));
            } else {
                failures.lock().push(inner);
            }
        });
        if let Some((r, l)) = result.into_inner() {
            trace.add_line(l);
            Ok(r)
        } else {
            Err(failures.into_inner())
        }
    }
}
