use ftml_ontology::terms::{Term, Variable};
use ftml_uris::Id;

use crate::{
    SolverRef, TermExtSolvable, context::Context, split::SplitStrategy, state::BoundedValue,
    trace::SolverTrace,
};

const PREFIX: &str = "SOLVE!";

impl TermExtSolvable for Term {
    fn is_solvable(&self) -> Option<&Id> {
        if let Self::Var {
            variable: Variable::Name { name, .. },
            ..
        } = self
            && name.as_ref().starts_with(PREFIX)
            && name.as_ref().as_bytes()[PREFIX.len()..]
                .iter()
                .all(u8::is_ascii_digit)
        {
            Some(name)
        } else {
            None
        }
    }
}

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    #[must_use]
    pub fn new_solvable(self) -> Variable {
        let i = self
            .top
            .implicits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let name: Id =
            // SAFETY: is a valid Id
            unsafe { format!("{PREFIX}{i}").parse().unwrap_unchecked() };
        self.state.add(name.clone());
        Variable::Name {
            name,
            notated: None,
        }
    }

    pub(crate) fn solve_equality<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        unk: &Id,
        solution: &'t Term,
    ) -> Option<bool> {
        trace.comment(format!("solving unknown {unk}"));
        let Some(unks) = self.state.get(unk) else {
            trace.failure("Unknown unknown!");
            return Some(false);
        };
        if let BoundedValue::Solved(tm) = &unks.solution {
            trace.comment("already solved");
            let tm = tm.clone();
            drop(unks);
            return context.in_branch(|context| self.check_equality(trace, context, &tm, solution));
        }
        drop(unks);
        trace.comment("Solved");
        self.state.solve(unk.clone(), solution.clone());
        Some(true)
    }

    pub(crate) fn solve_upper_bound<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        unk: &Id,
        bound: &'t Term,
    ) -> Option<bool> {
        trace.comment(format!("solving boundaries of unknown {unk}"));
        let Some(unks) = self.state.get(unk) else {
            trace.failure("Unknown unknown!");
            return Some(false);
        };
        if let BoundedValue::Solved(tm) = &unks.solution {
            trace.comment("already solved");
            let tm = tm.clone();
            drop(unks);
            return context.in_branch(|context| self.check_subtype(trace, context, &tm, bound));
        }
        /*
        if let BoundedValue::Bounded(lower, upper) = &unks.solution {
            let lower = lower.clone();
            let upper = upper.clone();
            drop(unks);
            return context.in_branch(|context| {
                if let Some(lower) = lower {
                    trace.comment("Checking against previous lower bound");
                    let r = self.check_subtype(trace, context, &lower, sup);
                    if r != Some(true) {
                        return r;
                    }
                }
                if let Some(upper) = upper {
                    trace.comment("Checking against previous upper bound");
                    let r = self.check_subtype(trace, context, &upper, sup);
                    if r != Some(true) {
                        return r;
                    }
                }
            });
        }

        trace.comment(format!("Solving upper type bound of {unk}"));
         */
        drop(unks);
        trace.comment("Solved");
        self.state.solve(unk.clone(), bound.clone());
        Some(true)
    }

    pub(crate) fn solve_lower_bound<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        unk: &Id,
        bound: &'t Term,
    ) -> Option<bool> {
        trace.comment(format!("solving boundaries of unknown {unk}"));
        let Some(unks) = self.state.get(unk) else {
            trace.failure("Unknown unknown!");
            return Some(false);
        };

        // todo boundary checks
        //
        if let BoundedValue::Solved(tm) = &unks.solution {
            trace.comment("already solved");
            let tm = tm.clone();
            drop(unks);
            return context.in_branch(|context| self.check_subtype(trace, context, bound, &tm));
        }
        drop(unks);
        trace.comment("Solved");
        self.state.solve(unk.clone(), bound.clone());
        Some(true)
    }
}
