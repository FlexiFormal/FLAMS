use crate::{
    CheckRef, TermExtSeq,
    impls::solving::TermExtSolvable,
    rules::{
        CheckingRule, InferenceRule, InhabitableRule, PreparationRule, SimplificationRule,
        SizedSolverRule,
    },
    split::SplitStrategy,
};
use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
    Variable,
};
use ftml_uris::SymbolUri;
use smallvec::SmallVec;
use std::{borrow::Cow, hint::unreachable_unchecked};

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
impl ArrowRule {
    fn go<Split: SplitStrategy>(
        &self,
        t: &Term,
        mut path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> Option<Term> {
        let Term::Application(app) = t else {
            return None;
        };
        let Some(Argument::Simple(ret)) = app.arguments.last() else {
            return None;
        };
        let args = &app.arguments[..app.arguments.len() - 1];
        if args.is_empty() {
            return Some(ret.clone());
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
        Some(ret)
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for ArrowRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Application(app) = term {
            matches!(&app.head,Term::Symbol{uri,..} if *uri == self.arrow)
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        _: CheckRef<'t, '_, Split>,
        t: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        self.go::<Split>(t, None).ok_or(None)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for ArrowRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        <Self as SimplificationRule<Split>>::applicable(self, t)
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
        _: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        std::ops::ControlFlow::Continue(self.go::<Split>(&t, path).unwrap_or(t))
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
pub struct NeedsTypeRule(pub SymbolUri);
impl SizedSolverRule for NeedsTypeRule {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "needs typed variables")
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for NeedsTypeRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        if let Term::Bound(b) = t
            && let Term::Symbol { uri, .. } = &b.head
            && *uri == self.0
        {
            b.arguments.iter().any(|a| matches!(a,BoundArgument::Bound(ComponentVar { tp:None, .. })|BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar { tp:None, .. })))
                || matches!(a,BoundArgument::BoundSeq(MaybeSequence::Seq(seq)) if seq.iter().any(|a| matches!(a,ComponentVar { tp:None, .. }))))
        } else {
            false
        }
    }
    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn apply(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        let Term::Bound(b) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let arguments = &b.arguments;
        let ret = checker.scoped(|checker| {
            let mut changed = false;
            let nargs = arguments
                .iter()
                .map(|a| match a {
                    BoundArgument::Bound(ComponentVar { var, tp: None, df }) => {
                        if checker.infer_var_type(var).is_none() {
                            changed = true;
                            let v = checker.new_solvable();
                            Cow::Owned(BoundArgument::Bound(ComponentVar {
                                var: var.clone(),
                                tp: Some(Term::Var {
                                    variable: v,
                                    presentation: None,
                                }),
                                df: df.clone(),
                            }))
                        } else {
                            Cow::Borrowed(a)
                        }
                    }
                    BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                        var,
                        tp: None,
                        df,
                    })) => {
                        if checker.infer_var_type(var).is_none() {
                            changed = true;
                            let v = checker.new_solvable();
                            Cow::Owned(BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                                var: var.clone(),
                                tp: Some(Term::Var {
                                    variable: v,
                                    presentation: None,
                                }),
                                df: df.clone(),
                            })))
                        } else {
                            Cow::Borrowed(a)
                        }
                    }
                    BoundArgument::BoundSeq(MaybeSequence::Seq(vs)) => {
                        let nvs = vs
                            .iter()
                            .map(|cv @ ComponentVar { var, tp, df }| {
                                if tp.is_none() && checker.infer_var_type(var).is_none() {
                                    changed = true;
                                    let v = checker.new_solvable();
                                    Cow::Owned(ComponentVar {
                                        var: var.clone(),
                                        tp: Some(Term::Var {
                                            variable: v,
                                            presentation: None,
                                        }),
                                        df: df.clone(),
                                    })
                                } else {
                                    Cow::Borrowed(cv)
                                }
                            })
                            .collect::<Vec<_>>();
                        if changed {
                            Cow::Owned(BoundArgument::BoundSeq(MaybeSequence::Seq(
                                nvs.into_iter().map(Cow::into_owned).collect(),
                            )))
                        } else {
                            Cow::Borrowed(a)
                        }
                    }
                    _ => Cow::Borrowed(a),
                })
                .collect::<Vec<_>>();
            if changed {
                Some(Term::Bound(BindingTerm::new(
                    b.head.clone(),
                    nargs.into_iter().map(Cow::into_owned).collect(),
                    b.presentation.clone(),
                )))
            } else {
                None
            }
        });
        std::ops::ControlFlow::Continue(ret.unwrap_or(Term::Bound(b)))
    }

    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> std::ops::ControlFlow<Term, Term> {
        std::ops::ControlFlow::Continue(t)
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
            Some(tp) if tp.is_solvable().is_some() => {
                let inf = checker.scoped(|checker| {
                    checker.extend_context(var);
                    checker.infer_type(body)
                })?;
                ret!(&checker.check_inhabitable(tp) == Some(true));
                inf
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
            Some(tp) if tp.is_solvable().is_some() => {
                let inf = checker.scoped(|checker| {
                    checker.extend_context(var);
                    checker.infer_type(body)
                })?;
                ret!(&checker.check_inhabitable(tp) == Some(true));
                inf
            }
            Some(tp) => {
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

impl<Split: SplitStrategy> InferenceRule<Split> for PiInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Application(_)) // | Term::Bound(_))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        match term {
            Term::Application(app) => {
                let tp = checker.infer_type(&app.head)?;
                if app.arguments.is_empty() {
                    return None;
                }
                self.type_apply(&mut checker, tp, &app.arguments)
            }
            /*
            Term::Bound(app) => {
                //todo!();
                let tp = checker.infer_type(&app.head)?;
                self.type_bound(checker, tp, &app.arguments)
            } */
            _ => {
                checker.failure("Not an application");
                None
            }
        }
    }
}

impl PiInferenceRule {
    // INVARIANT: return has 2 arguments, the second one being simple
    fn deconstruct_tp<Split: SplitStrategy>(
        bind_uri: &SymbolUri,
        checker: &mut CheckRef<'_, '_, Split>,
        tp: Term,
    ) -> Option<BindingTerm> {
        let Some(nret) = checker.scoped(|checker| {
            match checker.simplify_until(&tp, |t| matches!(t, Term::Bound(_)))? {
                Cow::Borrowed(_) => Some(None),
                Cow::Owned(tp) => Some(Some(tp)),
            }
        }) else {
            checker.failure(format!("type is not a binder: {:?}", tp.debug_short()));
            return None;
        };
        let Term::Bound(b) = nret.unwrap_or(tp) else {
            // SAFETY: simplify_until above would have returned None otherwise
            unsafe { unreachable_unchecked() }
        };
        if !matches!(&b.head,Term::Symbol { uri, .. } if *uri == *bind_uri)
            || b.arguments.len() != 2
            || !matches!(b.arguments.get(1), Some(BoundArgument::Simple(_)))
        {
            checker.failure("Type is not a Π anymore");
            return None;
        }
        Some(b)
    }

    // INVARIANT: tp.is_concrete_sequence()
    pub(super) fn flatten_sequence<Split: SplitStrategy>(
        checker: &mut CheckRef<'_, '_, Split>,
        tp: &Term,
        b: &BindingTerm,
        body: Term,
    ) -> Term {
        let Some(new_args) = tp.make_concrete_sequence() else {
            // SAFETY: tp.is_concrete_sequence()
            unsafe { unreachable_unchecked() }
        };
        checker.comment("Flattening concrete sequence argument");
        new_args.into_iter().rfold(body.clone(), |body, arg| {
            Term::Bound(BindingTerm::new(
                b.head.clone(),
                Box::new([
                    BoundArgument::Bound(ComponentVar {
                        var: Variable::Name {
                            name: crate::DUMMY.clone(),
                            notated: None,
                        },
                        tp: Some(arg),
                        df: None,
                    }),
                    BoundArgument::Simple(body),
                ]),
                b.presentation.clone(),
            ))
        })
    }

    // INVARIANT: !seq.is_empty()
    pub(super) fn recurse_seq_args<'t, Split: SplitStrategy>(
        uri: &SymbolUri,
        checker: &mut CheckRef<'t, '_, Split>,
        b: &BindingTerm,
        seq: &'t [Term],
        body: &Term,
    ) -> Option<Term> {
        // SAFETY: !seq.is_empty()
        let first = unsafe { seq.first().unwrap_unchecked() };
        let rest = &seq[1..];
        let mut ret = Self::simple_apply(checker, &b, first, body)?;
        for arg in rest {
            let b = Self::deconstruct_tp(uri, checker, ret)?;
            let [_, BoundArgument::Simple(body)] = &*b.arguments else {
                // SAFETY: invariant of deconstruct_tp
                unsafe { unreachable_unchecked() }
            };
            ret = Self::simple_apply(checker, &b, arg, body)?;
        }
        Some(ret)
    }

    fn type_apply<'t, Split: SplitStrategy>(
        &self,
        checker: &mut CheckRef<'t, '_, Split>,
        tp: Term,
        args: &'t [Argument],
    ) -> Option<Term> {
        let mut ret = tp;
        let mut i = 0;
        loop {
            if args.get(i).is_none() {
                return Some(ret);
            }
            let b = Self::deconstruct_tp(&self.0, checker, ret)?;
            let [first, BoundArgument::Simple(body)] = &*b.arguments else {
                // SAFETY: invariant of deconstruct_tp
                unsafe { unreachable_unchecked() }
            };
            // SAFETY: !args.get(i).is_none() above
            let arg = unsafe { args.get(i).unwrap_unchecked() };
            match (first, arg) {
                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    }),
                    Argument::Sequence(seq),
                ) if !body.has_free_such_that(|v| v.name() == var.name())
                    && tp.is_concrete_sequence() =>
                {
                    ret = Self::flatten_sequence(checker, tp, &b, body.clone());
                }
                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    }),
                    Argument::Sequence(MaybeSequence::Seq(seq)),
                ) if !seq.is_empty() => {
                    i += 1;
                    checker.counter("Checking Argument", i);
                    ret = Self::recurse_seq_args(&self.0, checker, &b, seq, body)?;
                }
                (_, Argument::Simple(arg)) => {
                    i += 1;
                    checker.counter("Checking Argument", i);
                    ret = Self::simple_apply(checker, &b, arg, body)?;
                }
                (_, Argument::Sequence(arg)) => {
                    i += 1;
                    checker.counter("Checking Argument", i);
                    ret = checker.scoped(|checker| Self::seq_apply(checker, &b, arg, body))?;
                }
            }
        }
    }

    pub(super) fn simple_apply<'t, Split: SplitStrategy>(
        checker: &mut CheckRef<'t, '_, Split>,
        b: &BindingTerm,
        arg: &'t Term,
        body: &Term,
    ) -> Option<Term> {
        let headvar = if let Some(BoundArgument::Bound(headvar)) = b.arguments.first() {
            headvar
        } else if let Some(BoundArgument::BoundSeq(MaybeSequence::Seq(vs))) = b.arguments.first()
            && let [headvar] = &**vs
        {
            headvar
        } else {
            checker.failure("First argument is not a bound variable");
            //checker.comment(format!("Here: {:?}{:?}", b.head.debug_short(), b.arguments));
            //println!("Here: {:?}", b.arguments.first());
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
    }

    pub(super) fn seq_apply<'t, Split: SplitStrategy>(
        checker: &mut CheckRef<'t, '_, Split>,
        b: &'t BindingTerm,
        arg: &'t MaybeSequence<Term>,
        body: &Term,
    ) -> Option<Term> {
        if let Some(BoundArgument::Bound(ComponentVar {
            var,
            tp: Some(tp),
            df: None,
        })) = b.arguments.first()
            && let Some(tpargs) = tp.as_sequence()
            && !body
                .free_variables()
                .into_iter()
                .any(|v| v.name() == var.name())
            && let MaybeSequence::Seq(args) = arg
            && args.len() == tpargs.len()
        {
            for (a, t) in args.iter().zip(tpargs.iter()) {
                if !checker.check_type(a, t)? {
                    return None;
                }
            }
            return Some(body.clone());
        }

        let Some(BoundArgument::BoundSeq(MaybeSequence::One(headvar))) = b.arguments.first() else {
            checker.failure("First argument is not a bound variable sequence");
            checker.comment(format!(
                "Here: {:?}{:?}  <-- {arg:?}",
                b.head.debug_short(),
                b.arguments
            ));
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

        match arg {
            MaybeSequence::One(arg) => {
                if checker
                    .scoped(|checker| checker.check_type(arg, &vartp))
                    .is_none_or(|b| !b)
                {
                    return None;
                }
                Some((body / (varname, arg)).into_owned())
            }
            MaybeSequence::Seq(arg) => {
                let narg = Term::into_seq(arg.iter().cloned());
                if let Some(vartp) = vartp.as_sequence_type() {
                    if !checker.scoped(|checker| {
                        for a in arg {
                            if !checker.check_type(a, vartp)? {
                                return None;
                            }
                        }
                        Some(true)
                    })? {
                        return None;
                    }
                } else if checker
                    .scoped(|checker| checker.check_type(&narg, &vartp))
                    .is_none_or(|b| !b)
                {
                    return None;
                }

                Some((body / (varname, &narg)).into_owned())
            }
        }
    }

    /*
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
                checker.failure("Argument is not simple 2");
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
                    checker.failure("Argument is not simple 3");
                    None
                }
            }
        })?;
        Some(r)
        /*
        let mut fails = r
            .free_variables()
            .into_iter()
            .filter(|v| names.contains(&v.name()));
        #[allow(clippy::option_if_let_else)]
        if let Some(fail) = fails.next() {
            checker.failure(format!(
                "Resulting type depends on eliminated variables: {}",
                fail.name()
            ));
            None
        } else {
            drop(fails);
            Some(r)
        } */
    } */
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BetaRule(pub SymbolUri);
impl SizedSolverRule for BetaRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ a:A } (", &self.0, " x:A. t)(a) ==> t[x/a]")
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for BetaRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Application(app) = term
            && let Term::Bound(op) = &app.head
            && let Term::Symbol { uri, .. } = &op.head
        {
            *uri == self.0 && app.arguments.len() >= op.arguments.len() - 1
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Application(app) = term else {
            return Err(None);
        };
        let Term::Bound(fun) = &app.head else {
            return Err(None);
        };
        let Some(BoundArgument::Simple(ret)) = fun.arguments.last() else {
            return Err(None);
        };
        let funargs = &fun.arguments[..fun.arguments.len() - 1];
        let actual_args = &app.arguments[..funargs.len()];
        let rest_args = &app.arguments[funargs.len()..];
        let mut ret = Cow::Borrowed(ret);
        for (i, (v, a)) in funargs.iter().zip(actual_args).enumerate() {
            checker.counter("Checking argument ", i + 1);
            match (v, a) {
                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    }),
                    Argument::Simple(a),
                ) => {
                    if !checker.check_type(a, tp).ok_or(None)? {
                        return Err(None);
                    }
                    if let Cow::Owned(nr) = &*ret / (var.name(), a) {
                        ret = Cow::Owned(nr);
                    }
                }
                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: None,
                        df: None,
                    }),
                    Argument::Simple(a),
                ) => {
                    let tp = checker.infer_var_type(var).ok_or(None)?;
                    if !checker
                        .scoped(|checker| checker.check_type(a, &tp))
                        .ok_or(None)?
                    {
                        return Err(None);
                    }
                    if let Cow::Owned(nr) = &*ret / (var.name(), a) {
                        ret = Cow::Owned(nr);
                    }
                }
                (
                    BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    })),
                    Argument::Sequence(MaybeSequence::Seq(ts)),
                ) => {
                    let Some(tp) = tp.as_sequence_type() else {
                        checker.failure("Type of sequence variable is not a sequence type");
                        checker.comment(format!("Here: {:?} <-> {ts:?}", tp.debug_short()));
                        return Err(None);
                    };
                    for t in ts {
                        if !checker.check_type(t, tp).ok_or(None)? {
                            return Err(None);
                        }
                    }
                    if let Cow::Owned(nr) =
                        &*ret / (var.name(), &Term::into_seq(ts.iter().cloned()))
                    {
                        ret = Cow::Owned(nr);
                    }
                }

                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: None,
                        df: None,
                    }),
                    Argument::Sequence(MaybeSequence::Seq(args)),
                ) => {
                    let tp = checker.infer_var_type(var).ok_or(None)?;
                    let tp = checker
                        .scoped(|checker| {
                            checker
                                .simplify_until(&tp, Term::is_concrete_sequence)
                                .map(Cow::into_owned)
                        })
                        .ok_or(None)?;
                    let vartps = tp.make_concrete_sequence().ok_or(None)?;
                    if vartps.len() != args.len() {
                        checker.failure("sequence lengths don't match");
                        return Err(None);
                    }
                    for (a, tp) in args.iter().zip(vartps) {
                        if !checker
                            .scoped(|checker| checker.check_type(a, &tp))
                            .ok_or(None)?
                        {
                            return Err(None);
                        }
                    }
                }
                (
                    BoundArgument::Bound(ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    }),
                    Argument::Sequence(MaybeSequence::Seq(args)),
                ) => {
                    let ntp = checker
                        .scoped(|checker| {
                            checker
                                .simplify_until(tp, Term::is_concrete_sequence)
                                .map(Cow::into_owned)
                        })
                        .ok_or(None)?;
                    let vartps = ntp.make_concrete_sequence().ok_or(None)?;
                    if vartps.len() != args.len() {
                        checker.failure("sequence lengths don't match");
                        return Err(None);
                    }
                    for (a, tp) in args.iter().zip(vartps) {
                        if !checker
                            .scoped(|checker| checker.check_type(a, &tp))
                            .ok_or(None)?
                        {
                            return Err(None);
                        }
                    }
                }

                (a, b) => {
                    checker.failure(format!("TODO: {a:?} <-> {b:?}"));
                    return Err(None);
                }
            }
        }
        if rest_args.is_empty() {
            Ok(ret.into_owned())
        } else {
            Ok(Term::Application(ApplicationTerm::new(
                ret.into_owned(),
                rest_args.iter().cloned().collect(),
                None,
            )))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyRule(pub SymbolUri);
impl SizedSolverRule for ApplyRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ a:A } ", &self.0, "(f,a*) ==> f(a*)")
    }
    fn priority(&self) -> isize {
        isize::MAX
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for ApplyRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        if let Term::Application(app) = t
            && let [Argument::Simple(_), Argument::Sequence(_)] = &*app.arguments
            && let Term::Symbol { uri, .. } = &app.head
        {
            *uri == self.0
        } else {
            false
        }
    }
    fn apply(
        &self,
        _: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        let Term::Application(app) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let [Argument::Simple(f), Argument::Sequence(a)] = &*app.arguments else {
            return std::ops::ControlFlow::Continue(Term::Application(app));
        };
        let ret = Term::Application(ApplicationTerm::new(
            f.clone(),
            Box::new([Argument::Sequence(match a {
                MaybeSequence::One(o) => MaybeSequence::One(o.clone()),
                MaybeSequence::Seq(ts) => MaybeSequence::Seq(ts.clone()),
            })]),
            app.presentation.clone(),
        ));
        if let Some((p, i)) = path
            && let Some(j) = p.get_mut(i)
        {
            *j = j.saturating_sub(1);
        }
        std::ops::ControlFlow::Continue(ret)
    }
    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> std::ops::ControlFlow<Term, Term> {
        std::ops::ControlFlow::Continue(t)
    }
}

/*
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
