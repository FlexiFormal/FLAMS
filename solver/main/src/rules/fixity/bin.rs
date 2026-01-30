use crate::{
    rules::{PreparationRule, RuleSet, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::{
    domain::declarations::symbols::Symbol,
    narrative::elements::VariableDeclaration,
    terms::{ApplicationTerm, Argument, IsTerm, MaybeSequence, Term},
};
use ftml_uris::SymbolUri;
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinLRule(pub SymbolUri);

impl SizedSolverRule for BinLRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "is a left-associative binary operator")
    }
}
impl std::fmt::Display for BinLRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a left-associative binary operator", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for BinLRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        _: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        tracing::trace!("binl!");
        let Some((app, MaybeSequence::Seq(seq), idx)) =
            super::is_sequence_binary(&self.0, &t, head)
        else {
            return ControlFlow::Continue(t);
        };
        if seq.len() < 2 {
            return ControlFlow::Continue(t);
        }
        let preargs = &app.arguments[..idx];
        let postargs = &app.arguments[idx + 1..];
        //SAFETY: len() >= 2
        unsafe {
            ControlFlow::Continue(
                seq.iter()
                    .cloned()
                    .reduce(|a, b| {
                        Term::Application(ApplicationTerm::new(
                            app.head.clone(),
                            {
                                let mut args = preargs.to_vec();
                                args.extend([Argument::Simple(a), Argument::Simple(b)]);
                                args.extend_from_slice(postargs);
                                args.into_boxed_slice()
                            },
                            app.presentation.clone(),
                        ))
                    })
                    .unwrap_unchecked(),
            )
        }
    }
    fn applicable_revert(
        &self,
        t: &Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> bool {
        super::was_sequence_binary(&self.0, t, head).is_some()
    }

    fn revert(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        let Some((app, first, second, idx)) = super::was_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let pre = &app.arguments[..idx];
        let post = &app.arguments[idx + 2..];
        let mut nargs = vec![first.clone()];
        //nargs.push(Argument::Simple(second.clone()));
        let mut to_check = second;
        while super::match_head(head, to_check.head()) {
            let Some((napp, first, second, nidx)) =
                super::was_sequence_binary(&self.0, to_check, head)
            else {
                break;
            };
            if nidx != idx {
                break;
            }
            let npre = &napp.arguments[..idx];
            let npost = &napp.arguments[idx + 2..];
            if npre != pre || npost != post {
                break;
            }
            nargs.push(first.clone());
            //nargs.push(Argument::Simple(second.clone()));
            to_check = second;
        }
        nargs.push(to_check.clone());
        let nargs = pre
            .iter()
            .cloned()
            .chain(std::iter::once(Argument::Sequence(MaybeSequence::Seq(
                nargs.into_boxed_slice(),
            ))))
            .chain(post.iter().cloned())
            .collect();
        ControlFlow::Continue(Term::Application(ApplicationTerm::new(
            app.head.clone(),
            nargs,
            app.presentation.clone(),
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinRRule(pub SymbolUri);

impl SizedSolverRule for BinRRule {
    fn priority(&self) -> isize {
        10_000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "is a right-associative binary operator")
    }
}
impl std::fmt::Display for BinRRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a right-associative binary operator", self.0)
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for BinRRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        super::is_sequence_binary(&self.0, t, head).is_some()
    }
    fn apply(
        &self,
        _: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        {
            let Some((app, MaybeSequence::Seq(seq), idx)) =
                super::is_sequence_binary(&self.0, &t, head)
            else {
                return ControlFlow::Continue(t);
            };
            if seq.len() < 2 {
                return ControlFlow::Continue(t);
            }
            let preargs = &app.arguments[..idx];
            let postargs = &app.arguments[idx + 1..];
            //SAFETY: len() >= 2
            unsafe {
                ControlFlow::Continue(seq[..seq.len() - 1].iter().cloned().rfold(
                    seq.last().unwrap_unchecked().clone(),
                    |a, b| {
                        Term::Application(ApplicationTerm::new(
                            app.head.clone(),
                            {
                                let mut args = preargs.to_vec();
                                args.extend([Argument::Simple(a), Argument::Simple(b)]);
                                args.extend_from_slice(postargs);
                                args.into_boxed_slice()
                            },
                            app.presentation.clone(),
                        ))
                    },
                ))
            }
        }
    }

    fn applicable_revert(
        &self,
        t: &Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> bool {
        super::was_sequence_binary(&self.0, t, head).is_some()
    }
    fn revert(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        let Some((app, first, second, idx)) = super::was_sequence_binary(&self.0, &t, head) else {
            return ControlFlow::Continue(t);
        };
        let pre = &app.arguments[..idx];
        let post = &app.arguments[idx + 2..];
        let mut nargs = vec![second.clone()];

        let mut to_check = first;
        while super::match_head(head, to_check.head()) {
            let Some((napp, first, second, nidx)) =
                super::was_sequence_binary(&self.0, to_check, head)
            else {
                break;
            };
            if nidx != idx {
                break;
            }
            let npre = &napp.arguments[..idx];
            let npost = &napp.arguments[idx + 2..];
            if npre != pre || npost != post {
                break;
            }
            //nargs.push(Argument::Simple(first.clone()));
            nargs.push(second.clone());
            to_check = first;
        }
        nargs.push(to_check.clone());
        nargs.reverse();
        let nargs = pre
            .iter()
            .cloned()
            .chain(std::iter::once(Argument::Sequence(MaybeSequence::Seq(
                nargs.into_boxed_slice(),
            ))))
            .chain(post.iter().cloned())
            .collect();
        ControlFlow::Continue(Term::Application(ApplicationTerm::new(
            app.head.clone(),
            nargs,
            app.presentation.clone(),
        )))
    }
}
