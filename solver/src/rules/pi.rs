use std::borrow::Cow;

use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};
use ftml_uris::SymbolUri;
use smallvec::SmallVec;

macro_rules! ret_i {
    ($(;)?) => {};
    (& $e:expr; $($tk:tt)*) => {
        if !$e {
            return None;
        }
        ret_i!($($tk)*)
    };
    ($p:pat = $e:expr; $($tk:tt)*) => {
        let $p = $e else {
            return None;
        };
        ret_i!($($tk)*)
    };

}
macro_rules! ret {
    ( $($tk:tt)* ) => {
        ret_i!($($tk)*;)
    };
}

pub(crate) fn destruct_binder<'t>(
    t: &'t Term,
    head: &SymbolUri,
) -> Option<(&'t ComponentVar, &'t Term)> {
    ret!(
        Term::Bound(b) = t;
        & b.arguments.len() == 2;
        & matches!(&b.head,Term::Symbol { uri, .. } if *uri == *head);
        Some(BoundArgument::Bound(v)) = b.arguments.first();
        Some(BoundArgument::Simple(body)) = b.arguments.get(1);
    );
    Some((v, body))
}
fn construct_binder(var: ComponentVar, body: Term, head: &SymbolUri) -> Term {
    Term::Bound(BindingTerm::new(
        Term::Symbol {
            uri: head.clone(),
            presentation: None,
        },
        Box::new([BoundArgument::Bound(var), BoundArgument::Simple(body)]),
        None,
    ))
}

use crate::{
    SolverRef,
    context::Context,
    rules::{CheckingRule, InferenceRule, InhabitableRule, SizedSolverRule},
    split::SplitStrategy,
    trace::SolverTrace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaPiRule {
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for LambdaPiRule {
    //fn display(&self) -> crate::trace::RefCheckLog<'static> {
    //    crate::traceline!()
    //}
}
impl std::fmt::Display for LambdaPiRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is a λ-operator for Π-operator {}",
            self.lambda, self.pi
        )
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for LambdaPiRule {
    fn applicable(&self, term: &Term) -> bool {
        destruct_binder(term, &self.lambda).is_some()
    }
    fn infer<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<Term> {
        let (var, body) = destruct_binder(term, &self.lambda)?;
        let btp = match &var.tp {
            None => {
                if solver
                    .infer_var_type(trace, context.branch(), &var.var)
                    .is_some()
                {
                    context.in_branch(|mut context| {
                        context.extend(var);
                        solver.infer_type(trace, context, body)
                    })?
                } else {
                    let nvar = ComponentVar {
                        var: var.var.clone(),
                        tp: Some(Term::Var {
                            variable: solver.new_solvable(),
                            presentation: None,
                        }),
                        df: var.df.clone(),
                    };
                    context.in_branch(|mut context| {
                        context.extend(nvar);
                        solver.infer_type(trace, context, body)
                    })?
                }
            }
            Some(tp) => {
                ret!(&solver.check_inhabitable(trace, context.branch(), tp) == Some(true));
                context.in_branch(|mut context| {
                    context.extend(var);
                    solver.infer_type(trace, context, body)
                })?
            }
        };
        Some(construct_binder(var.clone(), btp, &self.pi))
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for LambdaPiRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool {
        destruct_binder(term, &self.lambda).is_some() && destruct_binder(tp, &self.pi).is_some()
    }
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        let (var, lambda_body) = destruct_binder(term, &self.lambda)?;
        let (pivar, pi_body) = destruct_binder(tp, &self.pi)?;
        let pi_tp = match &pivar.tp {
            None => Cow::Owned(solver.infer_var_type(trace, context.branch(), &var.var)?),
            Some(tp) => {
                //ret!(&solver.check_inhabitable(trace, context.branch(), tp) == Some(true));
                Cow::Borrowed(tp)
            }
        };
        let lam_tp = match &var.tp {
            None => Cow::Owned(solver.infer_var_type(trace, context.branch(), &var.var)?),
            Some(tp) => {
                //ret!(&solver.check_inhabitable(trace, context.branch(), tp) == Some(true));
                Cow::Borrowed(tp)
            }
        };
        ret!(
            &context.in_branch(|context| { solver.check_subtype(trace, context, &lam_tp, &pi_tp) })
                == Some(true)
        );
        let ntp = pi_body
            / (
                pivar.var.name(),
                &Term::Var {
                    variable: var.var.clone(),
                    presentation: None,
                },
            );
        context.in_branch(|mut context| {
            context.extend(var);
            solver.check_type(trace, context, lambda_body, &ntp)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiRule(pub SymbolUri);
impl SizedSolverRule for PiRule {}
impl std::fmt::Display for PiRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a Π-binding operator", self.0)
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for PiRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Bound(b) if matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0))
    }
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<bool> {
        let Term::Bound(b) = term else { return None };
        let Some(BoundArgument::Simple(body)) = b.arguments.last() else {
            return None;
        };
        let previous = &b.arguments[..&b.arguments.len() - 1];
        for arg in previous {
            match arg {
                BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                    let _ = solver.infer_type(trace, context.branch(), t)?;
                }
                BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                    for t in ts {
                        let _ = solver.infer_type(trace, context.branch(), t)?;
                    }
                }
                BoundArgument::Bound(cv @ ComponentVar { var, tp, .. })
                | BoundArgument::BoundSeq(MaybeSequence::One(cv @ ComponentVar { var, tp, .. })) => {
                    if let Some(tp) = tp {
                        if !solver.check_inhabitable(trace, context.branch(), tp)? {
                            return Some(false);
                        }
                    } else {
                        let _ = solver.infer_var_type(trace, context.branch(), var)?;
                        /*context.extend_owned(ComponentVar {
                            var: var.clone(),
                            tp: Some(tp),
                            df: None,
                        });*/
                    }
                    context.extend(cv);
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vars)) => {
                    for cv @ ComponentVar { var, tp, .. } in vars {
                        if let Some(tp) = tp {
                            if !solver.check_inhabitable(trace, context.branch(), tp)? {
                                return Some(false);
                            }
                        } else {
                            let _ = solver.infer_var_type(trace, context.branch(), var)?;
                            /*context.extend_owned(ComponentVar {
                                var: var.clone(),
                                tp: Some(tp),
                                df: None,
                            });*/
                        }
                        context.extend(cv);
                    }
                }
            }
        }
        solver.check_inhabitable(trace, context, body)
    }
}

impl PiRule {
    fn type_apply<Split: SplitStrategy>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context,
        tp: Term,
        args: &[Argument],
    ) -> Option<Term> {
        args.iter().enumerate().try_fold(tp, |tp, (i, arg)| {
            trace.comment(format!("Checking Argument {}", i + 1));
            let Argument::Simple(arg) = arg else {
                trace.failure("Argument is not simple");
                return None;
            };
            let Term::Bound(b) = tp else {
                trace.failure("Type is not a binder");
                return None;
            };
            if !matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0)
                || b.arguments.len() != 2
            {
                trace.failure("Type is not a Π anymore");
                return None;
            }
            let Some(BoundArgument::Bound(headvar)) = b.arguments.first() else {
                trace.failure("First argument is not a bound variable");
                return None;
            };
            let Some(BoundArgument::Simple(body)) = b.arguments.get(1) else {
                trace.failure("Second argument is not simple");
                return None;
            };
            let (varname, vartp) = match headvar {
                ComponentVar {
                    var, tp: Some(tp), ..
                } => (var.name(), tp.clone()),
                ComponentVar { var, .. } => (
                    var.name(),
                    solver
                        .infer_var_type(trace, context.branch(), var)
                        .unwrap_or_else(|| Term::Var {
                            variable: solver.new_solvable(),
                            presentation: None,
                        }),
                ),
            };
            if context
                .in_branch(|context| solver.check_type(trace, context, arg, &vartp))
                .is_none_or(|b| !b)
            {
                return None;
            }
            Some((body / (varname, arg)).into_owned())
        })
    }

    fn type_bound<'t, Split: SplitStrategy>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        tp: Term,
        args: &'t [BoundArgument],
    ) -> Option<Term> {
        let mut names = SmallVec::<_, 2>::new();
        let r = args.iter().enumerate().try_fold(tp, |tp, (i, arg)| {
            trace.comment(format!("Checking Argument {}", i + 1));
            let Term::Bound(b) = tp else {
                trace.failure("Type is not a binder");
                return None;
            };
            if !matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0)
                || b.arguments.len() != 2
            {
                trace.failure("Type is not a Π anymore");
                return None;
            }
            let Some(BoundArgument::Bound(headvar)) = b.arguments.first() else {
                trace.failure("Argument is not a bound variable");
                return None;
            };
            let Some(BoundArgument::Simple(body)) = b.arguments.get(1) else {
                trace.failure("Argument is not simple");
                return None;
            };
            let (varname, vartp) = match headvar {
                ComponentVar {
                    var, tp: Some(tp), ..
                } => (var.name(), tp.clone()),
                ComponentVar { var, .. } => (
                    var.name(),
                    solver.infer_var_type(trace, context.branch(), var)?,
                ),
            };
            match arg {
                BoundArgument::Simple(arg) => {
                    if context
                        .in_branch(|context| solver.check_type(trace, context, arg, &vartp))
                        .is_none_or(|b| !b)
                    {
                        return None;
                    }
                    Some((body / (varname, arg)).into_owned())
                }
                BoundArgument::Bound(
                    cv @ ComponentVar {
                        var: argvar,
                        tp,
                        df,
                    },
                ) => {
                    names.push(argvar.name());
                    if let Some(tp) = tp {
                        if context
                            .in_branch(|context| solver.check_subtype(trace, context, tp, &vartp))
                            != Some(true)
                        {
                            return None;
                        }
                        context.extend(cv);
                        Some(
                            (body
                                / (
                                    varname,
                                    &Term::Var {
                                        variable: cv.var.clone(),
                                        presentation: None,
                                    },
                                ))
                                .into_owned(),
                        )
                    } else {
                        trace.failure("Untyped argument");
                        None
                    }
                }
                _ => {
                    trace.failure("Argument is not simple");
                    None
                }
            }
        })?;
        if r.free_variables()
            .into_iter()
            .any(|v| names.contains(&v.name()))
        {
            trace.failure("Resulting type depends on eliminated variables");
            None
        } else {
            Some(r)
        }
    }

    fn infer_app<'t, Split: SplitStrategy>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        app: &'t ApplicationTerm,
    ) -> Option<Term> {
        let Some(Argument::Simple(first_arg)) = app.arguments.first() else {
            trace.failure("Argument is not simple");
            return None;
        };
        let tp = solver.infer_type(trace, context.branch(), &app.head)?;
        self.type_apply(solver, trace, context, tp, &app.arguments)
    }
    fn infer_bind<'t, Split: SplitStrategy>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        mut context: Context<'t, '_>,
        app: &'t BindingTerm,
    ) -> Option<Term> {
        let tp = solver.infer_type(trace, context.branch(), &app.head)?;
        self.type_bound(solver, trace, context, tp, &app.arguments)
    }
}

impl<Split: SplitStrategy> InferenceRule<Split> for PiRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Application(_) | Term::Bound(_))
    }
    fn infer<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<Term> {
        match term {
            Term::Application(app) => self.infer_app(solver, trace, context, app),
            Term::Bound(b) => self.infer_bind(solver, trace, context, b),
            _ => {
                trace.failure("Not an application");
                None
            }
        }
    }
}
