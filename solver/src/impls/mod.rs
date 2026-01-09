pub mod backend;
mod equality;
mod preparation;
mod solving;
mod subtyping;
mod typing;

use crate::{
    SolverRef,
    context::Context,
    split::SplitStrategy,
    trace::{CheckingTask, SolverTrace},
};
use ftml_ontology::terms::{ComponentVar, Term, Variable};

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    pub fn infer_var_type(
        self,
        trace: &mut SolverTrace,
        context: Context,
        var: &Variable,
    ) -> Option<Term> {
        let (r, l) = trace.derived(
            CheckingTask::VariableInference(var.name()),
            context,
            |trace, context| self.infer_var_type_i(trace, &context, var),
        );
        trace.add_line(l);
        r
    }

    fn infer_var_type_i(
        self,
        trace: &mut SolverTrace,
        context: &Context,
        var: &Variable,
    ) -> Option<Term> {
        for v in context.iter() {
            match (v, var) {
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        tp,
                        ..
                    },
                    Variable::Name { name: n2, .. },
                ) if *name == *n2 => {
                    if tp.is_some() {
                        trace.comment("Found type in context");
                    } else {
                        trace.failure("variable untyped in context");
                    }
                    return tp.clone().map(|t| self.state.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        tp,
                        ..
                    },
                    Variable::Ref { declaration, .. },
                ) if name.as_ref() == declaration.name().last() && tp.is_some() => {
                    if tp.is_some() {
                        trace.comment("Found type in context");
                    } else {
                        trace.failure("Variable untyped in context");
                    }
                    return tp.clone().map(|t| self.state.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        tp,
                        ..
                    },
                    Variable::Name { name, .. },
                ) if name.as_ref() == declaration.name().last() => {
                    return if tp.is_some() {
                        trace.comment("Found type in context");
                        tp.clone().map(|t| self.state.subst(t))
                    } else {
                        trace.comment("Getting variable globally");
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .tp
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        tp,
                        ..
                    },
                    Variable::Ref {
                        declaration: d2, ..
                    },
                ) if *declaration == *d2 => {
                    return if tp.is_some() {
                        trace.comment("Found type in context");
                        tp.clone().map(|t| self.state.subst(t))
                    } else {
                        trace.comment("Getting variable globally");
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .tp
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                _ => (),
            }
        }
        if let Variable::Ref { declaration, .. } = var {
            trace.comment("Getting variable globally");
            self.get_variable(declaration)
                .ok()?
                .data
                .tp
                .checked_or_parsed()
                .map(|(t, _)| t)
        } else {
            None
        }
    }

    fn get_var_definiens(self, context: &Context, var: &Variable) -> Option<Term> {
        for v in context.iter() {
            match (v, var) {
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        df,
                        ..
                    },
                    Variable::Name { name: n2, .. },
                ) if *name == *n2 => {
                    return df.clone().map(|t| self.state.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        df,
                        ..
                    },
                    Variable::Ref { declaration, .. },
                ) if name.as_ref() == declaration.name().last() && df.is_some() => {
                    return df.clone().map(|t| self.state.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        df,
                        ..
                    },
                    Variable::Name { name, .. },
                ) if name.as_ref() == declaration.name().last() => {
                    return if df.is_some() {
                        df.clone().map(|t| self.state.subst(t))
                    } else {
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .df
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        df,
                        ..
                    },
                    Variable::Ref {
                        declaration: d2, ..
                    },
                ) if *declaration == *d2 => {
                    return if df.is_some() {
                        df.clone().map(|t| self.state.subst(t))
                    } else {
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .df
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                _ => (),
            }
        }
        if let Variable::Ref { declaration, .. } = var {
            self.get_variable(declaration)
                .ok()?
                .data
                .df
                .checked_or_parsed()
                .map(|(t, _)| t)
        } else {
            None
        }
    }

    pub fn infer_type<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<Term> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, line) = trace.derived(CheckingTask::Inference(t), context, |trace, context| {
            self.infer_type_i(trace, context, t)
        });
        trace.add_line(line);
        r
    }
    pub(super) fn infer_type_i<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<Term> {
        match t {
            Term::Symbol { uri, .. } => {
                trace.comment("Looking up symbol");
                let Ok(s) = self.get_symbol(uri) else {
                    trace.failure("Symbol not found");
                    return None;
                };
                let ret = s
                    .data
                    .tp
                    .checked_or_parsed()
                    .map(|(t, _)| self.bind_implicits(t));

                if ret.is_none() {
                    trace.failure("Symbol has no type");
                }
                return ret;
            }
            Term::Var { variable, .. } => {
                return self.infer_var_type_i(trace, &context, variable);
            }
            _ => (),
        }
        let rules = self
            .top
            .rules
            .inference()
            .iter()
            .filter_map(|rl| if rl.applicable(t) { Some(&**rl) } else { None });
        Split::split(self, trace, rules, context, |slf, rl, tk, context| {
            rl.infer(slf, tk, context, t).map(|t| self.state.subst(t))
        })
    }

    pub fn check_inhabitable<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<bool> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, line) = trace.derived(CheckingTask::Inhabitable(t), context, |trace, context| {
            self.check_inhabitable_i(trace, context, t)
        });
        trace.add_line(line);
        r
    }

    pub(super) fn check_inhabitable_i<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        tm: &'t Term,
    ) -> Option<bool> {
        Split::strategies(
            trace,
            context,
            "Using type inference",
            |trace, mut context| {
                let tp = self.infer_type(trace, context.branch(), tm)?;
                context.in_branch(|context| self.check_universe(trace, context, &tp))
            },
            "Using inhabitable rules",
            |trace, context| {
                let rules = self
                    .top
                    .rules
                    .inhabitable()
                    .iter()
                    .filter_map(|rl| if rl.applicable(tm) { Some(&**rl) } else { None });
                Split::split(self, trace, rules, context, |slf, rl, tk, context| {
                    rl.apply(slf, tk, context, tm)
                })
            },
        )
    }

    pub fn check_universe<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<bool> {
        if trace.is_cancelled() {
            return None;
        }
        let (r, line) = trace.derived(CheckingTask::Universe(t), context, |trace, context| {
            self.check_universe_i(trace, context, t)
        });
        trace.add_line(line);
        r
    }
    fn check_universe_i<'t>(
        self,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<bool> {
        let rules = self
            .top
            .rules
            .universe()
            .iter()
            .filter_map(|rl| if rl.applicable(t) { Some(&**rl) } else { None });
        Split::split(self, trace, rules, context, |slf, rl, tk, context| {
            rl.apply(slf, tk, context, t)
        })
    }
}
