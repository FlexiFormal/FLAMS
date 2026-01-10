use crate::{
    CheckRef,
    rules::{CheckingRule, InferenceRule, InhabitableRule, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};
use ftml_uris::{FtmlUri, SymbolUri};
use smallvec::SmallVec;
use std::borrow::Cow;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaPiRule {
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for LambdaPiRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(
            displayer,
            f,
            self.lambda.as_uri(),
            " is a λ-operator for Π-operator ",
            self.pi.as_uri()
        )
    }
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
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let (var, body) = destruct_binder(term, &self.lambda)?;
        let btp = match &var.tp {
            None => {
                if checker.infer_var_type(&var.var).is_some() {
                    checker.scoped(|checker| {
                        checker.extend_context(var);
                        checker.infer_type(body)
                    })?
                } else {
                    let nvar = ComponentVar {
                        var: var.var.clone(),
                        tp: Some(Term::Var {
                            variable: checker.new_solvable(),
                            presentation: None,
                        }),
                        df: var.df.clone(),
                    };
                    checker.scoped(|checker| {
                        checker.extend_context(nvar);
                        checker.infer_type(body)
                    })?
                }
            }
            Some(tp) => {
                ret!(&checker.check_inhabitable(tp) == Some(true));
                checker.scoped(|checker| {
                    checker.extend_context(var);
                    checker.infer_type(body)
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
        mut checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        let (var, lambda_body) = destruct_binder(term, &self.lambda)?;
        let (pivar, pi_body) = destruct_binder(tp, &self.pi)?;
        let pi_tp = match &pivar.tp {
            None => Cow::Owned(checker.infer_var_type(&var.var)?),
            Some(tp) => {
                //ret!(&checker.check_inhabitable(trace, context.branch(), tp) == Some(true));
                Cow::Borrowed(tp)
            }
        };
        let lam_tp = match &var.tp {
            None => Cow::Owned(checker.infer_var_type(&var.var)?),
            Some(tp) => {
                //ret!(&checker.check_inhabitable(trace, context.branch(), tp) == Some(true));
                Cow::Borrowed(tp)
            }
        };
        ret!(&checker.scoped(|checker| { checker.check_subtype(&lam_tp, &pi_tp) }) == Some(true));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiRule(pub SymbolUri);
impl SizedSolverRule for PiRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), " is a Π-binding operator")
    }
}
impl std::fmt::Display for PiRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a Π-binding operator", self.0)
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for PiRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Bound(b) if matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0))
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        let Term::Bound(b) = term else { return None };
        let Some(BoundArgument::Simple(body)) = b.arguments.last() else {
            return None;
        };
        let previous = &b.arguments[..&b.arguments.len() - 1];
        for arg in previous {
            match arg {
                BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                    let _ = checker.infer_type(t)?;
                }
                BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                    for t in ts {
                        let _ = checker.infer_type(t)?;
                    }
                }
                BoundArgument::Bound(cv @ ComponentVar { var, tp, .. })
                | BoundArgument::BoundSeq(MaybeSequence::One(cv @ ComponentVar { var, tp, .. })) => {
                    if let Some(tp) = tp {
                        if !checker.check_inhabitable(tp)? {
                            return Some(false);
                        }
                    } else {
                        let _ = checker.infer_var_type(var)?;
                    }
                    checker.extend_context(cv);
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vars)) => {
                    for cv @ ComponentVar { var, tp, .. } in vars {
                        if let Some(tp) = tp {
                            if !checker.check_inhabitable(tp)? {
                                return Some(false);
                            }
                        } else {
                            let _ = checker.infer_var_type(var)?;
                        }
                        checker.extend_context(cv);
                    }
                }
            }
        }
        checker.check_inhabitable(body)
    }
}

impl PiRule {
    fn type_apply<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        tp: Term,
        args: &'t [Argument],
    ) -> Option<Term> {
        args.iter().enumerate().try_fold(tp, |tp, (i, arg)| {
            checker.counter("Checking Argument ", i + 1);
            let Argument::Simple(arg) = arg else {
                checker.failure("Argument is not simple");
                return None;
            };
            let Term::Bound(b) = tp else {
                checker.failure("Type is not a binder");
                return None;
            };
            if !matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0)
                || b.arguments.len() != 2
            {
                checker.failure("Type is not a Π anymore");
                return None;
            }
            let Some(BoundArgument::Bound(headvar)) = b.arguments.first() else {
                checker.failure("First argument is not a bound variable");
                return None;
            };
            let Some(BoundArgument::Simple(body)) = b.arguments.get(1) else {
                checker.failure("Second argument is not simple");
                return None;
            };
            let (varname, vartp) = match headvar {
                ComponentVar {
                    var, tp: Some(tp), ..
                } => (var.name(), tp.clone()),
                ComponentVar { var, .. } => (
                    var.name(),
                    checker.scoped(|checker| {
                        checker.infer_var_type(var).unwrap_or_else(|| Term::Var {
                            variable: checker.new_solvable(),
                            presentation: None,
                        })
                    }),
                ),
            };
            if checker
                .scoped(|checker| checker.check_type(arg, &vartp))
                .is_none_or(|b| !b)
            {
                return None;
            }
            Some((body / (varname, arg)).into_owned())
        })
    }

    fn type_bound<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        tp: Term,
        args: &'t [BoundArgument],
    ) -> Option<Term> {
        let mut names = SmallVec::<_, 2>::new();
        let r = args.iter().enumerate().try_fold(tp, |tp, (i, arg)| {
            checker.counter("Checking Argument ", i + 1);
            let Term::Bound(b) = tp else {
                checker.failure("Type is not a binder");
                return None;
            };
            if !matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0)
                || b.arguments.len() != 2
            {
                checker.failure("Type is not a Π anymore");
                return None;
            }
            let Some(BoundArgument::Bound(headvar)) = b.arguments.first() else {
                checker.failure("Argument is not a bound variable");
                return None;
            };
            let Some(BoundArgument::Simple(body)) = b.arguments.get(1) else {
                checker.failure("Argument is not simple");
                return None;
            };
            let (varname, vartp) = match headvar {
                ComponentVar {
                    var, tp: Some(tp), ..
                } => (var.name(), tp.clone()),
                ComponentVar { var, .. } => (
                    var.name(),
                    checker.scoped(|checker| checker.infer_var_type(var))?,
                ),
            };
            match arg {
                BoundArgument::Simple(arg) => {
                    if checker
                        .scoped(|checker| checker.check_type(arg, &vartp))
                        .is_none_or(|b| !b)
                    {
                        return None;
                    }
                    Some((body / (varname, arg)).into_owned())
                }
                BoundArgument::Bound(
                    cv @ ComponentVar {
                        var: argvar, tp, ..
                    },
                ) => {
                    names.push(argvar.name());
                    if let Some(tp) = tp {
                        if checker.scoped(|checker| checker.check_subtype(tp, &vartp)) != Some(true)
                        {
                            return None;
                        }
                        checker.extend_context(cv);
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
                        checker.failure("Untyped argument");
                        None
                    }
                }
                _ => {
                    checker.failure("Argument is not simple");
                    None
                }
            }
        })?;
        if r.free_variables()
            .into_iter()
            .any(|v| names.contains(&v.name()))
        {
            checker.failure("Resulting type depends on eliminated variables");
            None
        } else {
            Some(r)
        }
    }

    fn infer_app<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        app: &'t ApplicationTerm,
    ) -> Option<Term> {
        let Some(Argument::Simple(_)) = app.arguments.first() else {
            checker.failure("Argument is not simple");
            return None;
        };
        let tp = checker.infer_type(&app.head)?;
        self.type_apply(checker, tp, &app.arguments)
    }
    fn infer_bind<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        app: &'t BindingTerm,
    ) -> Option<Term> {
        let tp = checker.infer_type(&app.head)?;
        self.type_bound(checker, tp, &app.arguments)
    }
}

impl<Split: SplitStrategy> InferenceRule<Split> for PiRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Application(_) | Term::Bound(_))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        match term {
            Term::Application(app) => self.infer_app(checker, app),
            Term::Bound(b) => self.infer_bind(checker, b),
            _ => {
                checker.failure("Not an application");
                None
            }
        }
    }
}
