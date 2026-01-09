use std::ops::ControlFlow;

use ftml_ontology::{
    domain::declarations::symbols::Symbol,
    narrative::elements::VariableDeclaration,
    terms::{ApplicationTerm, Argument, MaybeSequence, Term},
};
use ftml_uris::{FtmlUri, SymbolUri};

use crate::{
    rules::{PreparationRule, RuleSet, SizedSolverRule},
    split::SplitStrategy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsConjunctionRule(pub SymbolUri);
impl SizedSolverRule for IsConjunctionRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), "is a conjunction")
    }
}
impl std::fmt::Display for IsConjunctionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a conjunction", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for IsConjunctionRule {
    fn applicable(&self, _: &Term, _: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        false
    }
    fn apply(
        &self,
        _: &RuleSet<Split>,
        t: Term,
        _: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConjunctiveRule(pub SymbolUri);
impl SizedSolverRule for ConjunctiveRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), "is conjunctive")
    }
}
impl std::fmt::Display for ConjunctiveRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is conjunctive", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for ConjunctiveRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        let Some((app, args, index)) = super::is_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let Some(conj): Option<&IsConjunctionRule> = rules
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairwiseConjunctiveRule(pub SymbolUri);
impl SizedSolverRule for PairwiseConjunctiveRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), "is pairwise conjunctive")
    }
}
impl std::fmt::Display for PairwiseConjunctiveRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is pairwise conjunctive", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for PairwiseConjunctiveRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        let Some((app, args, index)) = super::is_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let Some(conj): Option<&IsConjunctionRule> = rules
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
}
