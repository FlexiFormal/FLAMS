use crate::{
    CheckRef,
    rules::{CheckingRule, InferenceRule, InhabitableRule, PreparationRule, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};
use ftml_uris::SymbolUri;
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
) -> Option<(either::Either<&'t ComponentVar, &'t ComponentVar>, &'t Term)> {
    ret!(
        Term::Bound(b) = t;
        & b.arguments.len() == 2;
        & matches!(&b.head,Term::Symbol { uri, .. } if *uri == *head);
        //Some(BoundArgument::Bound(v)) = b.arguments.first();
        Some(BoundArgument::Simple(body)) = b.arguments.get(1);
    );
    let v = if let Some(BoundArgument::Bound(v)) = b.arguments.first() {
        either::Left(v)
    } else if let Some(BoundArgument::BoundSeq(MaybeSequence::One(s))) = b.arguments.first() {
        either::Right(s)
    } else {
        return None;
    };
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
pub struct ArrowRule {
    pub arrow: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for ArrowRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.arrow, " is simple version of ", &self.pi)
    }
    fn priority(&self) -> isize {
        100_000_000 // SimpleTypeOperatorRule::priority() * 100
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for ArrowRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        if let Term::Application(app) = t {
            matches!(&app.head,Term::Symbol{uri,..} if *uri == self.arrow)
        } else {
            false
        }
    }
    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        if let Term::Bound(bind) = t {
            matches!(&bind.head,Term::Symbol { uri, .. } if *uri == self.pi)
        } else {
            false
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    fn apply(
        &self,
        _: &CheckRef<'_, '_, Split>,
        t: Term,
        mut path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        let Term::Application(app) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let Some(Argument::Simple(ret)) = app.arguments.last() else {
            return std::ops::ControlFlow::Continue(Term::Application(app));
        };
        let args = &app.arguments[..app.arguments.len() - 1];
        if args.is_empty() {
            return std::ops::ControlFlow::Continue(ret.clone());
        }
        let mut count = 0u16;
        let args = args.iter().map(|a| match a {
            Argument::Simple(t) => {
                if let Some((v, i)) = path.as_mut()
                    && v.get(*i).copied() == Some(count as u8)
                {
                    v.insert(*i + 1, 0);
                }
                let (v, idx) = ret.fresh_variable(&crate::DUMMY, Some(count));
                if let Some(idx) = idx {
                    count = idx + 1;
                } else {
                    count += 1;
                }
                BoundArgument::Bound(ComponentVar {
                    var: v,
                    tp: Some(t.clone()),
                    df: None,
                })
            }
            Argument::Sequence(MaybeSequence::One(t)) => {
                if let Some((v, i)) = path.as_mut()
                    && v.get(*i).copied() == Some(count as u8)
                {
                    v.insert(*i + 1, 0);
                }
                let (v, idx) = ret.fresh_variable(&crate::DUMMY, Some(count));
                if let Some(idx) = idx {
                    count = idx + 1;
                } else {
                    count += 1;
                }
                BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                    var: v,
                    tp: Some(t.clone()),
                    df: None,
                }))
            }
            Argument::Sequence(MaybeSequence::Seq(ts)) => {
                if let Some((v, i)) = path.as_mut()
                    && v.get(*i).copied() == Some(count as u8)
                {
                    v.insert(*i + 2, 0);
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(
                    ts.iter()
                        .map(|t| {
                            let (v, idx) = ret.fresh_variable(&crate::DUMMY, Some(count));
                            if let Some(idx) = idx {
                                count = idx + 1;
                            } else {
                                count += 1;
                            }
                            ComponentVar {
                                var: v,
                                tp: Some(t.clone()),
                                df: None,
                            }
                        })
                        .collect(),
                ))
            }
        });
        let ret = Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri: self.pi.clone(),
                presentation: None,
            },
            args.chain([BoundArgument::Simple(ret.clone())]).collect(),
            None,
        ));
        std::ops::ControlFlow::Continue(ret)
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> std::ops::ControlFlow<Term, Term> {
        let Term::Bound(app) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let Some(BoundArgument::Simple(ret)) = app.arguments.last() else {
            return std::ops::ControlFlow::Continue(Term::Bound(app));
        };
        let args = &app.arguments[..app.arguments.len() - 1];
        let mut free_vars = args.iter().map(|a| a.free_variables()).collect::<Vec<_>>();
        free_vars.push(ret.free_variables());

        let args: Result<Vec<Argument>, ()> = args
            .iter()
            .enumerate()
            .map(|(i, a)| match a {
                BoundArgument::Bound(ComponentVar {
                    var,
                    tp: Some(tp),
                    df: None,
                }) if !free_vars[i + 1..]
                    .iter()
                    .flatten()
                    .any(|v| v.name() == var.name()) =>
                {
                    Ok(Argument::Simple(tp.clone()))
                }
                BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                    var,
                    tp: Some(tp),
                    df: None,
                })) if !free_vars[i + 1..]
                    .iter()
                    .flatten()
                    .any(|v| v.name() == var.name()) =>
                {
                    Ok(Argument::Sequence(MaybeSequence::One(tp.clone())))
                }
                BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                    let seq: Result<Box<[Term]>, ()> = vs
                        .iter()
                        .map(|v| {
                            if let ComponentVar {
                                var,
                                tp: Some(tp),
                                df: None,
                            } = v
                                && !free_vars[i + 1..]
                                    .iter()
                                    .flatten()
                                    .any(|v| v.name() == var.name())
                            {
                                Ok(tp.clone())
                            } else {
                                Err(())
                            }
                        })
                        .collect();
                    Ok(Argument::Sequence(MaybeSequence::Seq(seq?)))
                }
                _ => Err(()),
            })
            .collect();
        drop(free_vars);
        std::ops::ControlFlow::Continue(match args {
            Err(()) => Term::Bound(app),
            Ok(mut args) => {
                args.push(Argument::Simple(ret.clone()));
                Term::Application(ApplicationTerm::new(
                    Term::Symbol {
                        uri: self.arrow.clone(),
                        presentation: None,
                    },
                    args.into_boxed_slice(),
                    None,
                ))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaPiInferenceRule {
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for LambdaPiInferenceRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ x:A, B(x):T } ⊢ (",
            &self.lambda,
            " x:A. B) :=> ",
            &self.pi,
            " x:A. T"
        )
    }
}
impl LambdaPiInferenceRule {
    fn infer_simple<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        var: &'t ComponentVar,
        body: &'t Term,
    ) -> Option<Term> {
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

    fn infer_sequence<'t, Split: SplitStrategy>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        var: &'t ComponentVar,
        body: &'t Term,
    ) -> Option<Term> {
        let btp = match &var.tp {
            None => {
                if let Some(tp) = checker.infer_var_type(&var.var) {
                    checker.comment(format!("Sequence type (inferred): {:?}", tp.debug_short()));
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
                checker.comment(format!("Sequence type (provided): {:?}", tp.debug_short()));
                ret!(&checker.check_inhabitable(tp) == Some(true));
                checker.scoped(|checker| {
                    checker.extend_context(var);
                    checker.infer_type(body)
                })?
            }
        };
        let r = Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri: self.pi.clone(),
                presentation: None,
            },
            Box::new([
                BoundArgument::BoundSeq(MaybeSequence::One(var.clone())),
                BoundArgument::Simple(btp),
            ]),
            None,
        ));
        Some(r)
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for LambdaPiInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        destruct_binder(term, &self.lambda).is_some()
    }
    fn infer<'t>(&self, checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let (var, body) = destruct_binder(term, &self.lambda)?;
        match var {
            either::Left(var) => self.infer_simple(checker, var, body),
            either::Right(var) => self.infer_sequence(checker, var, body),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaPiCheckingRule {
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for LambdaPiCheckingRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ x:A, B(x):T } ⊢ (",
            &self.lambda,
            " x:A. B) : ",
            &self.pi,
            " y:A. T"
        )
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for LambdaPiCheckingRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool {
        destruct_binder(term, &self.lambda).is_some_and(|(v, _)| v.is_left())
            && destruct_binder(tp, &self.pi).is_some_and(|(v, _)| v.is_left())
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        let (var, lambda_body) = destruct_binder(term, &self.lambda)?;
        let var = var.expect_left("applicable");
        let (pivar, pi_body) = destruct_binder(tp, &self.pi)?;
        let pivar = pivar.expect_left("applicable");
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
pub struct PiInhabitableRule(pub SymbolUri);
impl SizedSolverRule for PiInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ INH A, x:A, INH B(x) } ⊢ INH ", &self.0, " x:A. B")
    }
}

impl<Split: SplitStrategy> InhabitableRule<Split> for PiInhabitableRule {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiInferenceRule(pub SymbolUri);
impl SizedSolverRule for PiInferenceRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ f: ", &self.0, " x:A.B(x), t:A } ⊢ f(t) :=> B(t)")
    }
}

impl PiInferenceRule {
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
                //println!("Here: {:?}", b.arguments.first());
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

impl<Split: SplitStrategy> InferenceRule<Split> for PiInferenceRule {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInInhabitableRule(pub SymbolUri);
impl SizedSolverRule for BindInInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ INH A, x:A, INH B(x), INH T } ⊢ INH ",
            &self.0,
            "(x:A. B(x)) T"
        )
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for BindInInhabitableRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Bound(b) if matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.0))
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        let Term::Bound(b) = term else { return None };
        let Some(BoundArgument::Simple(body)) = b.arguments.last() else {
            return None;
        };
        let Some(BoundArgument::Simple(inarg)) = &b.arguments.get(b.arguments.len() - 2) else {
            return None;
        };
        let previous = &b.arguments[..&b.arguments.len() - 2];
        if !checker.scoped(|checker| {
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
                    | BoundArgument::BoundSeq(MaybeSequence::One(
                        cv @ ComponentVar { var, tp, .. },
                    )) => {
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
            checker.check_inhabitable(inarg)
        })? {
            return Some(false);
        }

        checker.check_inhabitable(body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInInferenceRule(pub SymbolUri);
impl SizedSolverRule for BindInInferenceRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ f: ",
            &self.0,
            " (x:A.B(x)) t, x:A, b:B(x), t:T } ⊢ f(x,b) :=> T"
        )
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for BindInInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Bound(_))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let Term::Bound(app) = term else {
            return None;
        };
        let [args, BoundArgument::Simple(boundin)] = &*app.arguments else {
            checker.failure("number of arguments doesn't match");
            return None;
        };
        let Some(Term::Bound(tp)) = checker.infer_type(&app.head) else {
            return None;
        };
        if !matches!(&tp.head,Term::Symbol { uri, .. } if *uri == self.0) {
            checker.failure("type doesn't match");
            return None;
        }
        let [
            argtps,
            BoundArgument::Simple(boundintp),
            BoundArgument::Simple(rettp),
        ] = &*tp.arguments
        else {
            checker.failure("number of arguments in type doesn't match");
            return None;
        };
        let scoped = checker.scoped(|checker| {
            let (v, tp, boundintp) = match (args, argtps) {
                (
                    BoundArgument::Bound(ComponentVar { var, tp, df: None }),
                    BoundArgument::BoundSeq(MaybeSequence::Seq(s)),
                ) => {
                    let [
                        ComponentVar {
                            var: subv,
                            tp: Some(tptp),
                            df: None,
                        },
                    ] = &**s
                    else {
                        checker.failure("number of bound variables don't match");
                        return None;
                    };
                    let tp = if let Some(tp) = tp {
                        if !checker.check_subtype(tp, tptp)? {
                            return None;
                        }
                        tp
                    } else {
                        tptp
                    };
                    (
                        var,
                        tp,
                        boundintp
                            / (
                                subv.name(),
                                &Term::Var {
                                    variable: var.clone(),
                                    presentation: None,
                                },
                            ),
                    )
                }
                (BoundArgument::BoundSeq(_), BoundArgument::BoundSeq(_)) => {
                    checker.failure("TODO: bound sequence");
                    // TODO
                    return None;
                }
                _ => {
                    checker.failure("argument modes don't match");
                    return None;
                }
            };
            checker.extend_context(ComponentVar {
                var: v.clone(),
                tp: Some(tp.clone()),
                df: None,
            });
            checker.scoped(|checker| checker.check_type(boundin, &boundintp))
        })?;
        if scoped { Some(rettp.clone()) } else { None }
    }
}

/*

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInRule {
    pub bind_in: SymbolUri,
    pub pi: SymbolUri,
}
impl SizedSolverRule for BindInRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.bind_in, " binds in ", &self.pi)
    }
    fn priority(&self) -> isize {
        100_000_000 // SimpleTypeOperatorRule::priority() * 100
    }
}

impl<Split: SplitStrategy> PreparationRule<Split> for BindInRule {
    fn applicable(
        &self,
        t: &Term,
        _: either::Either<
            &ftml_ontology::domain::declarations::symbols::Symbol,
            &ftml_ontology::narrative::elements::VariableDeclaration,
        >,
    ) -> bool {
        if let Term::Bound(app) = t {
            matches!(&app.head,Term::Symbol { uri, .. } if *uri == self.bind_in)
        } else {
            false
        }
    }
    fn applicable_revert(
        &self,
        t: &Term,
        _: either::Either<
            &ftml_ontology::domain::declarations::symbols::Symbol,
            &ftml_ontology::narrative::elements::VariableDeclaration,
        >,
    ) -> bool {
        false
        /*
        if let Term::Bound(app) = t
            && matches!(&app.head,Term::Symbol { uri, .. } if *uri == self.pi)
            && let Some(BoundArgument::Bound(ComponentVar {
                tp: Some(Term::Bound(app2)),
                df: None,
                ..
            })) = app.arguments.first()
        {
            matches!(&app2.head,Term::Symbol { uri, .. } if *uri == self.pi)
        } else {
            false
        } */
    }
    fn apply(
        &self,
        checker: &CheckRef<'_, '_, Split>,
        t: Term,
        head: either::Either<
            &ftml_ontology::domain::declarations::symbols::Symbol,
            &ftml_ontology::narrative::elements::VariableDeclaration,
        >,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        let Term::Bound(app) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let Some(BoundArgument::Simple(ret)) = app.arguments.last() else {
            return std::ops::ControlFlow::Continue(Term::Bound(app));
        };
        let prev = &app.arguments[..app.arguments.len() - 1];
        if let Some((p, i)) = path
            && let Some(j) = p.get_mut(i)
        {
            if *j as usize == app.arguments.len() - 1 {
                *j = 1;
            } else {
                p.insert(i, 0);
                p.insert(i, 0);
            }
            //p.insert(*i, value);
        }
        std::ops::ControlFlow::Continue(Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri: self.pi.clone(),
                presentation: None,
            },
            Box::new([
                BoundArgument::Bound(ComponentVar {
                    var: ret.fresh_variable(&crate::DUMMY, None).0,
                    tp: Some(Term::Bound(BindingTerm::new(
                        Term::Symbol {
                            uri: self.pi.clone(),
                            presentation: None,
                        },
                        prev.iter().cloned().collect(),
                        None,
                    ))),
                    df: None,
                }),
                BoundArgument::Simple(ret.clone()),
            ]),
            None,
        )))
    }
    fn revert(
        &self,
        _: &CheckRef<'_, '_, Split>,
        t: Term,
        _: either::Either<
            &ftml_ontology::domain::declarations::symbols::Symbol,
            &ftml_ontology::narrative::elements::VariableDeclaration,
        >,
    ) -> std::ops::ControlFlow<Term, Term> {
        std::ops::ControlFlow::Continue(t)
    }
}
 */
