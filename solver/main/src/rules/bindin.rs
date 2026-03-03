use std::{borrow::Cow, hint::unreachable_unchecked};

use ftml_ontology::terms::{BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{
    CheckRef, TermExtSeq,
    rules::{InferenceRule, InhabitableRule},
    split::SplitStrategy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInInhabitableRule {
    pub bindin: SymbolUri,
    pub bind: SymbolUri,
}
impl SizedSolverRule for BindInInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ INH T, INH B, f: (",
            &self.bind,
            " x:T. B), INH t } ⊢ INH ",
            &self.bindin,
            " f: (",
            &self.bind,
            " x:T. B). t"
        )
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for BindInInhabitableRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Bound(b) = term
            && let Term::Symbol { uri, .. } = &b.head
            && *uri == self.bindin
            && let [BoundArgument::BoundSeq(_), BoundArgument::Simple(_)] = &*b.arguments
        {
            true
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<bool> {
        let Term::Bound(b) = term else { return None };
        let [BoundArgument::BoundSeq(f), BoundArgument::Simple(ret)] = &*b.arguments else {
            return None;
        };
        match f {
            MaybeSequence::Seq(seq) if seq.len() == 1 => {
                let [
                    ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    },
                ] = &**seq
                else {
                    // technically unreachable
                    return None;
                };
                let Some(bind) = checker.simplify_until(tp, |t| {
                    if let Term::Bound(b) = t
                        && let Term::Symbol { uri, .. } = &b.head
                        && *uri == self.bind
                    {
                        true
                    } else {
                        false
                    }
                }) else {
                    checker.failure("Type of bound variable is not a binder");
                    return None;
                };
                // subsumes INH T, INH B
                if !checker.scoped(|checker| checker.check_inhabitable(&bind))? {
                    return None;
                }
                checker.extend_context(ComponentVar {
                    var: var.clone(),
                    tp: Some(bind.into_owned()),
                    df: None,
                });
                checker.check_inhabitable(ret)
            }
            MaybeSequence::Seq(_) => {
                checker.comment("TODO: sequence variable in BindIn");
                None
            }
            MaybeSequence::One(_) => {
                checker.failure("Untyped bound variable");
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInInferenceRule {
    pub bindin: SymbolUri,
    pub bind: SymbolUri,
}
impl SizedSolverRule for BindInInferenceRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ INH A, INH B, f: (",
            &self.bind,
            " x:A. B), t: T } ⊢ (",
            &self.bindin,
            " f: (",
            &self.bind,
            " x:A. B). t) :=> ",
            &self.bindin,
            " f: (",
            &self.bind,
            " x:A. B). T"
        )
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for BindInInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Bound(b) = term
            && let Term::Symbol { uri, .. } = &b.head
            && *uri == self.bindin
            && let [BoundArgument::BoundSeq(_), BoundArgument::Simple(_)] = &*b.arguments
        {
            true
        } else {
            false
        }
    }

    fn infer<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<Term> {
        let Term::Bound(b) = term else { return None };
        let [BoundArgument::BoundSeq(f), BoundArgument::Simple(ret)] = &*b.arguments else {
            return None;
        };
        match f {
            MaybeSequence::Seq(seq) if seq.len() == 1 => {
                let [
                    cv @ ComponentVar {
                        var,
                        tp: Some(tp),
                        df: None,
                    },
                ] = &**seq
                else {
                    // technically unreachable
                    return None;
                };
                checker.extend_context(cv);

                // first infer type of body, *then* check type of variable
                // to potentially solve variables
                let body = checker.infer_type(ret)?;

                let Some(bind) = checker.simplify_until(tp, |t| {
                    if let Term::Bound(b) = t
                        && let Term::Symbol { uri, .. } = &b.head
                        && *uri == self.bind
                    {
                        true
                    } else {
                        false
                    }
                }) else {
                    checker.failure("Type of bound variable is not a binder");
                    return None;
                };
                // subsumes INH T, INH B
                if !checker.scoped(|checker| checker.check_inhabitable(&bind))? {
                    return None;
                }

                Some(Term::Bound(BindingTerm::new(
                    b.head.clone(),
                    Box::new([
                        BoundArgument::BoundSeq(MaybeSequence::Seq(Box::new([ComponentVar {
                            var: var.clone(),
                            tp: Some(bind.into_owned()),
                            df: None,
                        }]))),
                        BoundArgument::Simple(body),
                    ]),
                    None,
                )))
            }
            MaybeSequence::Seq(_) => {
                checker.comment("TODO: sequence variable in BindIn");
                None
            }
            MaybeSequence::One(_) => {
                checker.failure("Untyped bound variable");
                None
            }
        }
    }
    /*
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<bool> {

    }
     */
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInApplyRule {
    pub bindin: SymbolUri,
    pub bind: SymbolUri,
}
impl SizedSolverRule for BindInApplyRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            "{ F: (",
            &self.bindin,
            " f: (",
            &self.bind,
            " x:A. B). T(f)) } ⊢ F y:A. t :=> T(",
            &self.bind,
            " y:A. t)"
        )
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for BindInApplyRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Bound(_))
    }
    fn infer<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<Term> {
        let Term::Bound(app) = term else { return None };
        let mut ret = checker.infer_type(&app.head)?;
        let args = &*app.arguments;
        let mut i = 0;
        loop {
            if args.get(i).is_none() {
                return Some(ret);
            }
            let (b, is_bindin) = self.deconstruct_tp(&mut checker, ret)?;

            let [first, BoundArgument::Simple(body)] = &*b.arguments else {
                // SAFETY: invariant of deconstruct_tp
                unsafe { unreachable_unchecked() }
            };
            // SAFETY: !args.get(i).is_none() above
            let arg = unsafe { args.get(i).unwrap_unchecked() };
            if is_bindin {
                let Some(BoundArgument::Simple(next_arg)) = args.get(i + 1) else {
                    checker.failure("Need normal argument after bound variable");
                    return None;
                };
                checker.counter("Binding argument", i + 1);
                i += 2;
                let BoundArgument::BoundSeq(MaybeSequence::Seq(seq)) = first else {
                    checker.failure("not a bound variable sequence");
                    return None;
                };
                let [
                    ComponentVar {
                        var: tpvar,
                        tp: Some(expected),
                        df: None,
                    },
                ] = &**seq
                else {
                    checker.failure("TODO: multiple bound variables / type inference 1");
                    return None;
                };
                let BoundArgument::Bound(cv @ ComponentVar { df: None, .. }) = arg else {
                    checker.failure("TODO: multiple bound variables / type inference 2");
                    return None;
                };
                let f = Term::Bound(BindingTerm::new(
                    Term::Symbol {
                        uri: self.bind.clone(),
                        presentation: None,
                    },
                    Box::new([
                        BoundArgument::Bound(cv.clone()),
                        BoundArgument::Simple(next_arg.clone()),
                    ]),
                    None,
                ));
                if !checker.scoped(|checker| checker.check_type(&f, expected))? {
                    return None;
                }
                ret = (body / (tpvar.name(), &f)).into_owned();
            } else {
                ret = self.do_app(&mut checker, first, arg, body, &b, &mut i)?;
            }
        }
    }
}

impl BindInApplyRule {
    // INVARIANT: return has 2 arguments, the second one being simple
    fn deconstruct_tp<Split: SplitStrategy>(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        tp: Term,
    ) -> Option<(BindingTerm, bool)> {
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
        if b.arguments.len() != 2 || !matches!(b.arguments.get(1), Some(BoundArgument::Simple(_))) {
            checker.failure("Type is not a Π anymore");
            return None;
        }
        match &b.head {
            Term::Symbol { uri, .. } if *uri == self.bind => Some((b, false)),
            Term::Symbol { uri, .. } if *uri == self.bindin => Some((b, true)),
            _ => None,
        }
    }

    fn do_app<'t, Split: SplitStrategy>(
        &self,
        checker: &mut crate::CheckRef<'t, '_, Split>,
        first: &BoundArgument,
        arg: &'t BoundArgument,
        body: &Term,
        b: &BindingTerm,
        i: &mut usize,
    ) -> Option<Term> {
        match (first, arg) {
            (
                BoundArgument::Bound(ComponentVar {
                    var,
                    tp: Some(tp),
                    df: None,
                }),
                BoundArgument::Sequence(seq),
            ) if !body.has_free_such_that(|v| v.name() == var.name())
                && tp.is_concrete_sequence() =>
            {
                Some(super::pi::PiInferenceRule::flatten_sequence(
                    checker,
                    tp,
                    b,
                    body.clone(),
                ))
            }
            (
                BoundArgument::Bound(ComponentVar {
                    var,
                    tp: Some(tp),
                    df: None,
                }),
                BoundArgument::Sequence(MaybeSequence::Seq(seq)),
            ) if !seq.is_empty() => {
                *i += 1;
                checker.counter("Checking Argument", *i);
                super::pi::PiInferenceRule::recurse_seq_args(&self.bind, checker, b, seq, body)
            }
            (_, BoundArgument::Simple(arg)) => {
                *i += 1;
                checker.counter("Checking Argument", *i);
                super::pi::PiInferenceRule::simple_apply(checker, b, arg, body)
            }
            (_, BoundArgument::Sequence(arg)) => {
                *i += 1;
                checker.counter("Checking Argument", *i);
                checker
                    .scoped(|checker| super::pi::PiInferenceRule::seq_apply(checker, b, arg, body))
            }
            _ => {
                checker.failure("argument modes don't match");
                None
            }
        }
    }
}

/*
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithBoundComputationRule(pub SymbolUri);
impl SizedSolverRule for WithBoundComputationRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " replaces bound variables")
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for WithBoundComputationRule {
    fn applicable(&self, term: &ftml_ontology::terms::Term) -> bool {
        let Term::Bound(app) = term else { return false };
        let Term::Bound(head) = &app.head else {
            return false;
        };
        if let Term::Symbol { uri, .. } = &head.head
            && *uri == self.0
            && let [
                BoundArgument::BoundSeq(MaybeSequence::Seq(_)),
                BoundArgument::Simple(Term::Bound(ret)),
            ] = &*head.arguments
        {
            matches!(
                &app.arguments.first(),
                Some(BoundArgument::Bound(_) | BoundArgument::BoundSeq(_))
            ) && matches!(
                &ret.arguments.first(),
                Some(BoundArgument::Bound(_) | BoundArgument::BoundSeq(_))
            )
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        println!("HERE: {:?}", term.debug_short());
        let Term::Bound(app) = term else {
            return Err(None);
        };
        let Term::Bound(withbound) = &app.head else {
            return Err(None);
        };
        let [
            BoundArgument::BoundSeq(MaybeSequence::Seq(vars)),
            BoundArgument::Simple(Term::Bound(ret)),
        ] = &*withbound.arguments
        else {
            return Err(None);
        };
    }
}
 */
