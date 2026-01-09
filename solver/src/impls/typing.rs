use crate::{
    SolverRef,
    context::Context,
    split::SplitStrategy,
    trace::{CheckingTask, SolverTrace},
};
use ftml_ontology::terms::Term;

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    pub fn check_type<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        tm: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, l) = trace.derived(CheckingTask::HasType(tm, tp), context, |trace, context| {
            self.check_type_i(trace, context, tm, tp)
        });
        trace.add_line(l);
        r
    }
    pub(crate) fn check_type_i<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        tm: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        Split::strategies(
            trace,
            context,
            "Using type inference",
            |trace, mut context| {
                let subtp = self.infer_type(trace, context.branch(), tm)?;
                context.in_branch(|context| self.check_subtype(trace, context, &subtp, tp))
            },
            "Using checking rules",
            |trace, context| {
                let rules = self.top.rules.checking().iter().filter_map(|rl| {
                    if rl.applicable(tm, tp) {
                        Some(&**rl)
                    } else {
                        None
                    }
                });
                Split::split(self, trace, rules, context, |slf, rl, trace, context| {
                    rl.apply(slf, trace, context, tm, tp)
                })
            },
        )
    }
}
