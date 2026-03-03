use crate::{
    CheckRef,
    rules::{PreparationRule, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{ApplicationTerm, Argument, MaybeSequence, Term};
use ftml_uris::SymbolUri;
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsConjunctionRule(pub SymbolUri);
impl SizedSolverRule for IsConjunctionRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "is a conjunction")
    }
}
impl std::fmt::Display for IsConjunctionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a conjunction", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for IsConjunctionRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn apply(
        &self,
        _: &mut CheckRef<'_, '_, Split>,
        t: Term,
        _: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }

    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConjunctiveRule(pub SymbolUri);
impl SizedSolverRule for ConjunctiveRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is conjunctive")
    }
}
impl std::fmt::Display for ConjunctiveRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is conjunctive", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for ConjunctiveRule {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        let Some(head) = checker.get_head(t) else {
            return false;
        };
        let head = head.as_ref().map_either(|e| &**e, |e| &**e);
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term> {
        let Some(head) = checker.get_head(&t) else {
            return ControlFlow::Continue(t);
        };
        let head = head.as_ref().map_either(|e| &**e, |e| &**e);
        let Some((app, args, index)) = super::is_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let Some(conj): Option<&IsConjunctionRule> = checker
            .rules()
            .preparation()
            .iter()
            .find_map(|rl| rl.as_any().downcast_ref())
        else {
            return ControlFlow::Continue(t);
        };
        let pre = &app.arguments[..index];
        let post = &app.arguments[index + 1..];
        let args: &[Term] = match args {
            MaybeSequence::Seq(s) => s,
            MaybeSequence::One(o) => std::slice::from_ref(o),
        };
        if args.is_empty() {
            return ControlFlow::Continue(t);
        }
        let conjuncts = args.iter().map(|a| {
            Term::Application(ApplicationTerm::new(
                app.head.clone(),
                {
                    let mut args = pre.to_vec();
                    args.push(Argument::Simple(a.clone()));
                    args.extend_from_slice(post);
                    args.into_boxed_slice()
                },
                app.presentation.clone(),
            ))
        });
        // SAFETY: !args.is_empty()
        let t = unsafe {
            conjuncts
                .reduce(|a, b| {
                    Term::Application(ApplicationTerm::new(
                        Term::Symbol {
                            uri: conj.0.clone(),
                            presentation: None,
                        },
                        Box::new([Argument::Simple(a), Argument::Simple(b)]),
                        None,
                    ))
                })
                .unwrap_unchecked()
        };
        ControlFlow::Continue(t)
    }

    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairwiseConjunctiveRule(pub SymbolUri);
impl SizedSolverRule for PairwiseConjunctiveRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is pairwise conjunctive")
    }
}
impl std::fmt::Display for PairwiseConjunctiveRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is pairwise conjunctive", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for PairwiseConjunctiveRule {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        let Some(head) = checker.get_head(t) else {
            return false;
        };
        let head = head.as_ref().map_either(|e| &**e, |e| &**e);
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term> {
        let Some(head) = checker.get_head(&t) else {
            return ControlFlow::Continue(t);
        };
        let head = head.as_ref().map_either(|e| &**e, |e| &**e);
        let Some((app, args, index)) = super::is_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let Some(conj): Option<&IsConjunctionRule> = checker
            .rules()
            .preparation()
            .iter()
            .find_map(|rl| rl.as_any().downcast_ref())
        else {
            return ControlFlow::Continue(t);
        };
        let pre = &app.arguments[..index];
        let post = &app.arguments[index + 1..];
        let args: &[Term] = match args {
            MaybeSequence::Seq(s) => s,
            MaybeSequence::One(o) => std::slice::from_ref(o),
        };
        if args.len() < 2 {
            return ControlFlow::Continue(t);
        }
        let mut conjuncts = (0..args.len() - 1).map(|i| {
            let (a, b) = (&args[i], &args[i + 1]);
            Term::Application(ApplicationTerm::new(
                app.head.clone(),
                {
                    let mut args = pre.to_vec();
                    args.push(Argument::Simple(a.clone()));
                    args.push(Argument::Simple(b.clone()));
                    args.extend_from_slice(post);
                    args.into_boxed_slice()
                },
                app.presentation.clone(),
            ))
        });
        let t = if args.len() == 2 {
            // Safety: args.len() == 2 => conjuncts.len() == 1
            unsafe { conjuncts.next().unwrap_unchecked() }
        } else {
            // SAFETY: args.len() > 2 => conjuncts.len() >= 2
            unsafe {
                conjuncts
                    .reduce(|a, b| {
                        Term::Application(ApplicationTerm::new(
                            Term::Symbol {
                                uri: conj.0.clone(),
                                presentation: None,
                            },
                            Box::new([Argument::Simple(a), Argument::Simple(b)]),
                            None,
                        ))
                    })
                    .unwrap_unchecked()
            }
        };
        ControlFlow::Continue(t)
    }
    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }
}
