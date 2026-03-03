use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{
    TermExtSeq,
    rules::{InferenceRule, InhabitableRule, SimplificationRule},
    split::SplitStrategy,
};

/*
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapInferenceRule(pub SymbolUri);
impl SizedSolverRule for MapSimplificationRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ s:T1*, x:T1, f(x):T2 } ⊢ ",&self.0, "([x1,...,xn],f) :=> T2*")
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for MapInferenceRule {

}
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapArgumentSimplificationRule(pub SymbolUri);
impl SizedSolverRule for MapArgumentSimplificationRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "([x1,...,xn],f) ==> [f(x1),..,f(xn)]")
    }
    fn priority(&self) -> isize {
        1000
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for MapArgumentSimplificationRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Application(app) = term {
            app.arguments.iter().any(|a| {
                if let Argument::Sequence(MaybeSequence::One(t)) = a {
                    MapSimplificationRule::applicable_i(&self.0, t)
                } else {
                    false
                }
            })
        } else if let Term::Bound(app) = term {
            app.arguments.iter().any(|a| {
                if let BoundArgument::Sequence(MaybeSequence::One(t)) = a {
                    MapSimplificationRule::applicable_i(&self.0, t)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        if let Term::Application(app) = term {
            Ok(Term::Application(ApplicationTerm::new(
                app.head.clone(),
                app.arguments
                    .iter()
                    .map(|a| match a {
                        Argument::Sequence(MaybeSequence::One(t))
                            if MapSimplificationRule::applicable_i(&self.0, t) =>
                        {
                            MapSimplificationRule::apply_i(&mut checker, t).map_or_else(
                                |_| a.clone(),
                                |v| Argument::Sequence(MaybeSequence::Seq(v.into_boxed_slice())),
                            )
                        }
                        _ => a.clone(),
                    })
                    .collect(),
                app.presentation.clone(),
            )))
        } else if let Term::Bound(app) = term {
            Ok(Term::Bound(BindingTerm::new(
                app.head.clone(),
                app.arguments
                    .iter()
                    .map(|a| match a {
                        BoundArgument::Sequence(MaybeSequence::One(t))
                            if MapSimplificationRule::applicable_i(&self.0, t) =>
                        {
                            MapSimplificationRule::apply_i(&mut checker, t).map_or_else(
                                |_| a.clone(),
                                |v| {
                                    BoundArgument::Sequence(MaybeSequence::Seq(
                                        v.into_boxed_slice(),
                                    ))
                                },
                            )
                        }
                        _ => a.clone(),
                    })
                    .collect(),
                app.presentation.clone(),
            )))
        } else {
            Err(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapSimplificationRule(pub SymbolUri);
impl SizedSolverRule for MapSimplificationRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "([x1,...,xn],f) ==> [f(x1),..,f(xn)]")
    }
}
impl MapSimplificationRule {
    fn apply_i<'t, Split: SplitStrategy>(
        checker: &mut crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Vec<Term>, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Application(app) = term else {
            return Err(None);
        };
        let [a, Argument::Simple(f)] = &*app.arguments else {
            return Err(None);
        };
        match a {
            Argument::Simple(s) | Argument::Sequence(MaybeSequence::One(s)) => {
                let Some(ts) = s.make_concrete_sequence() else {
                    return Err(None);
                };
                Ok(ts
                    .into_iter()
                    .map(|arg| {
                        let t = Term::Application(ApplicationTerm::new(
                            f.clone(),
                            Box::new([Argument::Simple(arg)]),
                            None,
                        ));
                        checker
                            .scoped(|checker| checker.simplify_full(false, &t))
                            .unwrap_or(t)
                    })
                    .collect())
            }
            Argument::Sequence(MaybeSequence::Seq(ts)) => Ok(ts
                .iter()
                .map(|arg| {
                    let t = Term::Application(ApplicationTerm::new(
                        f.clone(),
                        Box::new([Argument::Simple(arg.clone())]),
                        None,
                    ));
                    checker
                        .scoped(|checker| checker.simplify_full(false, &t))
                        .unwrap_or(t)
                })
                .collect()),
        }
    }
    fn applicable_i(sym: &SymbolUri, term: &Term) -> bool {
        if let Term::Application(app) = term
            && let Term::Symbol { uri, .. } = &app.head
            && *uri == *sym
            && let [a, Argument::Simple(_)] = &*app.arguments
        {
            if let Argument::Simple(s) | Argument::Sequence(MaybeSequence::One(s)) = a {
                s.is_concrete_sequence()
            } else {
                true
            }
        } else {
            false
        }
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for MapSimplificationRule {
    fn applicable(&self, term: &Term) -> bool {
        Self::applicable_i(&self.0, term)
    }
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        Self::apply_i::<Split>(&mut checker, term).map(|ts| Term::into_seq(ts.into_iter()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapInhabitableRule(pub SymbolUri);
impl SizedSolverRule for MapInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ x:A, INH f(x), s:A* } ⊢ INH ", &self.0, "(s,f)")
    }
}

impl<Split: SplitStrategy> InhabitableRule<Split> for MapInhabitableRule {
    fn applicable(&self, term: &ftml_ontology::terms::Term) -> bool {
        if let Term::Application(app) = term
            && let Term::Symbol { uri, .. } = &app.head
        {
            *uri == self.0 && app.arguments.len() == 2
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t ftml_ontology::terms::Term,
    ) -> Option<bool> {
        let Term::Application(app) = term else {
            return None;
        };
        let [Argument::Sequence(seq), Argument::Simple(f)] = &*app.arguments else {
            checker.failure("arguments don't match");
            return None;
        };
        let seqtp = match seq {
            MaybeSequence::One(t) if t.as_sequence().is_some() => {
                // SAFETY: pattern match
                let args = unsafe { t.as_sequence().unwrap_unchecked() };
                let mut curr = None;
                for t in args {
                    if !checker.scoped::<Option<bool>>(|checker| {
                        let r = checker.infer_type(t)?;
                        if let Some(c) = &curr {
                            if !checker.scoped(|checker| checker.check_equality(c, &r))? {
                                return None;
                            }
                        } else {
                            curr = Some(r);
                        }
                        Some(true)
                    })? {
                        return None;
                    }
                }
                curr?
            }
            MaybeSequence::One(t) => checker.infer_type(t)?,
            MaybeSequence::Seq(ts) => {
                let mut curr = None;
                for t in ts {
                    if !checker.scoped::<Option<bool>>(|checker| {
                        let r = checker.infer_type(t)?;
                        if let Some(c) = &curr {
                            if !checker.scoped(|checker| checker.check_equality(c, &r))? {
                                return None;
                            }
                        } else {
                            curr = Some(r);
                        }
                        Some(true)
                    })? {
                        return None;
                    }
                }
                curr?
            }
        };
        let seqtp = seqtp.as_sequence_type().cloned().unwrap_or(seqtp);
        let (v, _) = f.fresh_variable(&crate::DUMMY, None);
        checker.extend_context(ComponentVar {
            var: v.clone(),
            tp: Some(seqtp),
            df: None,
        });
        let nt = Term::Application(ApplicationTerm::new(
            f.clone(),
            Box::new([Argument::Simple(Term::Var {
                variable: v,
                presentation: None,
            })]),
            None,
        ));
        checker.scoped(|checker| checker.check_inhabitable(&nt))
    }
}
