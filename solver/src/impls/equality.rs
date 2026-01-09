use ftml_ontology::terms::{ApplicationTerm, Argument, Term};

use crate::{
    SolverRef, TermExtSolvable,
    context::Context,
    split::SplitStrategy,
    trace::{CheckingTask, SolverTrace},
};

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    #[allow(clippy::unused_self)]
    pub(crate) fn trivially_equal(self, tma: &Term, tmb: &Term) -> bool {
        if tma == tmb {
            return true;
        }
        match (tma, tmb) {
            (Term::Var { variable: v1, .. }, Term::Var { variable: v2, .. }) => {
                v1.name() == v2.name()
            }
            _ => false,
        }
    }

    pub fn check_equality<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        lhs: &'t Term,
        rhs: &'t Term,
    ) -> Option<bool> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, l) = trace.derived(
            CheckingTask::Equality(lhs, rhs),
            context,
            |trace, context| {
                if self.trivially_equal(lhs, rhs) {
                    trace.comment("trivial");
                    return Some(true);
                }
                self.check_equality_i(trace, context, lhs, rhs)
            },
        );
        trace.add_line(l);
        r
    }
    pub(crate) fn check_equality_i<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        lhs: &'t Term,
        rhs: &'t Term,
    ) -> Option<bool> {
        if let Some(unk) = lhs.is_solvable() {
            return self.solve_equality(trace, context, unk, rhs);
        }
        if let Some(unk) = rhs.is_solvable() {
            return self.solve_equality(trace, context, unk, lhs);
        }

        let rules = self.top.rules.equality().iter().filter_map(|rl| {
            if rl.applicable(lhs, rhs) {
                Some(&**rl)
            } else {
                None
            }
        });
        let prev = match Split::split_i(
            self,
            trace,
            rules,
            context.branch(),
            |slf, rl, tk, context| rl.apply(slf, tk, context, lhs, rhs),
        ) {
            Ok(r) => return Some(r),
            Err(ls) => ls,
        };
        match (lhs, rhs) {
            (Term::Application(lhs), Term::Application(rhs))
                if lhs.arguments.len() == rhs.arguments.len() =>
            {
                let (r, l) = trace.derived(
                    CheckingTask::Strategy("Trying congruence"),
                    context,
                    |trace, context| self.congruence(trace, context, lhs, rhs),
                );
                if let Some(r) = r {
                    trace.add_line(l);
                    Some(r)
                } else {
                    for l in prev {
                        trace.add_line(l);
                    }
                    trace.add_line(l);
                    None
                }
            }
            _ => {
                for l in prev {
                    trace.add_line(l);
                }
                None
            }
        }
    }

    // invariant: lhs.arguments.len() == rhs.arguments.len()
    fn congruence<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        lhs: &'t ApplicationTerm,
        rhs: &'t ApplicationTerm,
    ) -> Option<bool> {
        trace.comment("Comparing operators");
        if !self.check_equality(trace, context.branch(), &lhs.head, &rhs.head)? {
            return None;
        }
        for (i, (a, b)) in lhs.arguments.iter().zip(&rhs.arguments).enumerate() {
            trace.comment(format!("Comparing arguments {}", i + 1));
            match (a, b) {
                (Argument::Simple(a), Argument::Simple(b)) => {
                    if !self.check_equality(trace, context.branch(), a, b)? {
                        return None;
                    }
                }
                _ => {
                    trace.failure("Argument not simple");
                    return None;
                }
            }
        }
        Some(true)
    }
}
