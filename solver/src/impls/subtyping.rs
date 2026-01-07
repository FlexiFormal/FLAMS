use crate::{
    SolverRef, TermExtSolvable,
    context::Context,
    split::SplitStrategy,
    trace::{SolverTask, SolverTrace},
};
use ftml_ontology::terms::Term;

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    pub fn check_subtype<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        sub: &'t Term,
        sup: &'t Term,
    ) -> Option<bool> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, line) = trace.derived(SolverTask::Subtype(sub, sup), context, |trace, context| {
            self.check_subtype_i(trace, context, sub, sup)
        });
        trace.add_line(line);
        r
    }
    pub(crate) fn check_subtype_i<'t>(
        self,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        sub: &'t Term,
        sup: &'t Term,
    ) -> Option<bool> {
        if self.trivially_equal(sub, sup) {
            trace.comment("trivial");
            return Some(true);
        }
        if let Some(unk) = sub.is_solvable() {
            return self.solve_upper_bound(trace, context, unk, sup);
        }
        if let Some(unk) = sup.is_solvable() {
            return self.solve_lower_bound(trace, context, unk, sub);
        }
        let rules = self.top.rules.subtyping().iter().filter_map(|rl| {
            if rl.applicable(sub, sup) {
                Some(&**rl)
            } else {
                None
            }
        });
        let lines = match Split::split_i(
            self,
            trace,
            rules,
            context.branch(),
            |slf, rl, tk, context| rl.apply(slf, tk, context, sub, sup),
        ) {
            Ok(r) => return Some(r),
            Err(ls) => ls,
        };
        let (r, l) = trace.derived(
            SolverTask::Strategy("Proving subtyping failed; Falling back to checking equality"),
            context,
            |trace, context| self.check_equality_i(trace, context, sub, sup),
        );
        if let Some(r) = r {
            trace.add_line(l);
            Some(r)
        } else {
            for l in lines {
                trace.add_line(l);
            }
            trace.add_line(l);
            None
        }
    }
}
