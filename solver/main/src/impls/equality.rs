use std::borrow::Cow;

use crate::{CheckRef, impls::solving::TermExtSolvable, split::SplitStrategy, trace::CheckingTask};
use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
    Variable,
};
use ftml_solver_trace::RefCheckLog;

type Alpha<'t> = smallvec::SmallVec<(&'t str, &'t Variable), 1>;

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    #[allow(clippy::unused_self)]
    pub(crate) fn alpha_equal(&self, lhs: &Term, rhs: &Term) -> bool {
        Self::alpha_i(lhs, rhs, &mut Alpha::default())
    }

    fn alpha_i<'t2>(lhs: &'t2 Term, rhs: &'t2 Term, alpha: &mut Alpha<'t2>) -> bool {
        if lhs == rhs {
            return true;
        }
        match (lhs, rhs) {
            (Term::Var { variable: v1, .. }, Term::Var { variable: v2, .. }) => {
                v1.name() == v2.name()
                    || alpha.iter().any(|(a, b)| {
                        (*a == v1.name() && b.name() == v2.name())
                            || (b.name() == v1.name() && *a == v2.name())
                    })
            }
            (Term::Application(a), Term::Application(b))
                if a.arguments.len() == b.arguments.len() =>
            {
                Self::alpha_i(&a.head, &b.head, alpha)
                    && a.arguments
                        .iter()
                        .zip(b.arguments.iter())
                        .all(|(a, b)| Self::alpha_arg(a, b, alpha))
            }
            (Term::Bound(a), Term::Bound(b)) if a.arguments.len() == b.arguments.len() => {
                let mut pop = 0;
                if !Self::alpha_i(&a.head, &b.head, alpha)
                    || a.arguments.iter().zip(b.arguments.iter()).any(|(a, b)| {
                        Self::alpha_barg(a, b, alpha)
                            .inspect(|i| pop += i)
                            .is_none()
                    })
                {
                    return false;
                }
                for _ in 0..pop {
                    alpha.pop();
                }
                true
            }
            _ => false,
        }
    }

    fn alpha_arg<'t2>(lhs: &'t2 Argument, rhs: &'t2 Argument, alpha: &mut Alpha<'t2>) -> bool {
        match (lhs, rhs) {
            (Argument::Simple(lhs), Argument::Simple(rhs))
            | (
                Argument::Sequence(MaybeSequence::One(lhs)),
                Argument::Sequence(MaybeSequence::One(rhs)),
            ) => Self::alpha_i(lhs, rhs, alpha),
            (
                Argument::Sequence(MaybeSequence::Seq(lhs)),
                Argument::Sequence(MaybeSequence::Seq(rhs)),
            ) if lhs.len() == rhs.len() => lhs
                .iter()
                .zip(rhs.iter())
                .all(|(lhs, rhs)| Self::alpha_i(lhs, rhs, alpha)),
            _ => false,
        }
    }
    fn alpha_barg<'t2>(
        lhs: &'t2 BoundArgument,
        rhs: &'t2 BoundArgument,
        alpha: &mut Alpha<'t2>,
    ) -> Option<usize> {
        macro_rules! ret {
            ($e:expr) => {
                if $e { Some(0) } else { None }
            };
        }
        match (lhs, rhs) {
            (BoundArgument::Simple(lhs), BoundArgument::Simple(rhs))
            | (
                BoundArgument::Sequence(MaybeSequence::One(lhs)),
                BoundArgument::Sequence(MaybeSequence::One(rhs)),
            ) => ret!(Self::alpha_i(lhs, rhs, alpha)),
            (
                BoundArgument::Sequence(MaybeSequence::Seq(lhs)),
                BoundArgument::Sequence(MaybeSequence::Seq(rhs)),
            ) if lhs.len() == rhs.len() => ret!(
                lhs.iter()
                    .zip(rhs.iter())
                    .all(|(lhs, rhs)| Self::alpha_i(lhs, rhs, alpha))
            ),
            (BoundArgument::Bound(lhs), BoundArgument::Bound(rhs))
            | (
                BoundArgument::BoundSeq(MaybeSequence::One(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::One(rhs)),
            ) => {
                if Self::alpha_cv(lhs, rhs, alpha) {
                    Some(1)
                } else {
                    None
                }
            }
            (
                BoundArgument::BoundSeq(MaybeSequence::Seq(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::Seq(rhs)),
            ) if lhs.len() == rhs.len() => {
                if lhs
                    .iter()
                    .zip(rhs.iter())
                    .all(|(a, b)| Self::alpha_cv(a, b, alpha))
                {
                    Some(lhs.len())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn alpha_cv<'t2>(
        lhs: &'t2 ComponentVar,
        rhs: &'t2 ComponentVar,
        alpha: &mut Alpha<'t2>,
    ) -> bool {
        match (lhs.tp.as_ref(), rhs.tp.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                if !Self::alpha_i(lhs, rhs, alpha) {
                    return false;
                }
            }
            (None, None) => (),
            _ => return false,
        }
        match (lhs.df.as_ref(), rhs.df.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                if !Self::alpha_i(lhs, rhs, alpha) {
                    return false;
                }
            }
            (None, None) => (),
            _ => return false,
        }
        alpha.push((lhs.var.name(), &rhs.var));
        true
    }

    pub fn check_equality(&mut self, lhs: &'t Term, rhs: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::Equality(lhs, rhs), |slf| {
            if slf.alpha_equal(lhs, rhs) {
                slf.comment("trivial");
                return Some(true);
            }
            slf.check_equality_i(lhs, rhs)
        })
    }
    pub(crate) fn check_equality_i(&mut self, lhs: &'t Term, rhs: &'t Term) -> Option<bool> {
        if let Some(unk) = lhs.is_solvable() {
            return self.solve_equality(unk, rhs);
        }
        if let Some(unk) = rhs.is_solvable() {
            return self.solve_equality(unk, lhs);
        }

        let rules = self.top.rules.equality().iter().filter_map(|rl| {
            if rl.applicable(lhs, rhs) {
                Some(&**rl)
            } else {
                None
            }
        });
        let prev = match Split::split_i(self, true, rules, |slf, rl| rl.apply(slf, lhs, rhs)) {
            Ok(r) => return Some(r),
            Err(ls) => ls,
        };

        self.congruence(lhs, rhs, prev)
    }

    fn congruence(
        &mut self,
        lhs: &'t Term,
        rhs: &'t Term,
        mut logs: smallvec::SmallVec<RefCheckLog<'t>, 2>,
    ) -> Option<bool> {
        match (lhs, rhs) {
            (Term::Application(l), Term::Application(r))
                if l.arguments.len() == r.arguments.len() =>
            {
                match self.traced(CheckingTask::Strategy("Trying congruence"), |slf| {
                    slf.congruence_app(l, r)
                }) {
                    Ok(r) => Some(r),
                    Err(l) => {
                        logs.push(l);
                        self.congruence_cont(lhs, rhs, logs)
                    }
                }
            }
            (Term::Bound(l), Term::Bound(r)) if l.arguments.len() == r.arguments.len() => {
                match self.traced(CheckingTask::Strategy("Trying congruence"), |slf| {
                    slf.congruence_bind(l, r)
                }) {
                    Ok(r) => Some(r),
                    Err(l) => {
                        logs.push(l);
                        self.congruence_cont(lhs, rhs, logs)
                    }
                }
            }
            _ => self.congruence_cont(lhs, rhs, logs),
        }
    }

    fn congruence_cont(
        &mut self,
        lhs: &'t Term,
        rhs: &'t Term,
        logs: smallvec::SmallVec<RefCheckLog<'t>, 2>,
    ) -> Option<bool> {
        // todo: preserve logs on recursive fail
        if let Some(lhs) = self.simplify_one(true, lhs) {
            if self.alpha_equal(&lhs, rhs) {
                self.comment("trivial");
                return Some(true);
            }
            return self.scoped(|slf| slf.check_equality_i(&lhs, rhs));
        }
        if let Some(rhs) = self.simplify_one(true, rhs) {
            if self.alpha_equal(lhs, &rhs) {
                self.comment("trivial");
                return Some(true);
            }
            return self.scoped(|slf| slf.check_equality_i(lhs, &rhs));
        }
        for l in logs {
            self.add_msg(l.into());
        }
        None
    }

    // invariant: lhs.arguments.len() == rhs.arguments.len()
    fn congruence_app(
        &mut self,
        lhs: &'t ApplicationTerm,
        rhs: &'t ApplicationTerm,
    ) -> Option<bool> {
        self.comment("Comparing operators");
        if !self.check_equality(&lhs.head, &rhs.head)? {
            return None;
        }
        for (i, (a, b)) in lhs.arguments.iter().zip(&rhs.arguments).enumerate() {
            self.counter("Comparing arguments ", i + 1);
            if let (Argument::Simple(a), Argument::Simple(b)) = (a, b) {
                if !self.check_equality(a, b)? {
                    return None;
                }
            } else {
                self.failure("Argument not simple");
                return None;
            }
        }
        Some(true)
    }

    // invariant: lhs.arguments.len() == rhs.arguments.len()
    fn congruence_bind(&mut self, lhs: &'t BindingTerm, rhs: &'t BindingTerm) -> Option<bool> {
        self.comment("Comparing operators");
        if !self.check_equality(&lhs.head, &rhs.head)? {
            return None;
        }
        let mut substs = Alpha::new();
        macro_rules! maybe_subst {
            ($a:expr,$b:expr) => {
                if substs.is_empty()
                    || !$a.has_free_such_that(|av| substs.iter().any(|(v, _)| *v == av.name()))
                {
                    if !self.check_equality($a, $b)? {
                        return None;
                    }
                } else {
                    let subst = substs
                        .iter()
                        .map(|(n, v)| {
                            (
                                n,
                                Term::Var {
                                    variable: (*v).clone(),
                                    presentation: None,
                                },
                            )
                        })
                        .collect::<smallvec::SmallVec<_, 2>>();
                    let r = match $a / &*subst {
                        Cow::Borrowed(_) => self.check_equality($a, $b)?,
                        Cow::Owned(a) => self.scoped(|slf| slf.check_equality(&a, $b))?,
                    };
                    if !r {
                        return None;
                    }
                }
            };
        }
        for (i, (a, b)) in lhs.arguments.iter().zip(&rhs.arguments).enumerate() {
            self.counter("Comparing arguments ", i + 1);
            match (a, b) {
                (BoundArgument::Simple(a), BoundArgument::Simple(b)) => {
                    maybe_subst!(a, b);
                }
                (BoundArgument::Bound(a), BoundArgument::Bound(b)) => {
                    match (a.tp.as_ref(), b.tp.as_ref()) {
                        (Some(a), Some(b)) => {
                            maybe_subst!(a, b);
                        }
                        (None, None) => (),
                        _ => return None,
                    }
                    match (a.df.as_ref(), b.df.as_ref()) {
                        (Some(a), Some(b)) => {
                            maybe_subst!(a, b);
                        }
                        (None, None) => (),
                        _ => return None,
                    }
                    if a.var.name() != b.var.name() {
                        substs.push((a.var.name(), &b.var));
                    }
                }
                _ => {
                    self.failure("Argument not simple");
                    return None;
                }
            }
        }
        Some(true)
    }
}
