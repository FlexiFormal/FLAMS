use crate::{
    rules::{PreparationRule, RuleSet, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::{
    domain::declarations::symbols::Symbol,
    narrative::elements::VariableDeclaration,
    terms::{
        ApplicationTerm, Argument, ArgumentMode, BindingTerm, BoundArgument, MaybeSequence, Term,
    },
};
use ftml_uris::SymbolUri;
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrenexRule(pub SymbolUri);
impl SizedSolverRule for PrenexRule {
    fn priority(&self) -> isize {
        10_000
    }
}
impl PrenexRule {
    fn do_app(seq_index: usize, app: ApplicationTerm) -> std::ops::ControlFlow<Term, Term> {
        let pre = &app.arguments[..seq_index];
        let post = &app.arguments[seq_index + 1..];
        let Some(Argument::Sequence(seq)) = app.arguments.get(seq_index) else {
            return ControlFlow::Continue(Term::Application(app));
        };
        match seq {
            MaybeSequence::One(v) => {
                let mut joined = pre.to_vec();
                joined.push(Argument::Simple(v.clone()));
                joined.extend_from_slice(post);
                ControlFlow::Continue(Term::Application(ApplicationTerm::new(
                    app.head.clone(),
                    joined.into_boxed_slice(),
                    app.presentation.clone(),
                )))
            }
            MaybeSequence::Seq(m) => {
                let Some(Argument::Simple(last)) = post.last() else {
                    return ControlFlow::Continue(Term::Application(app));
                };
                let post = &post[..post.len() - 1];
                let t = m.iter().rfold(last.clone(), |t, v| {
                    let mut joined = pre.to_vec();
                    joined.push(Argument::Simple(v.clone()));
                    joined.extend_from_slice(post);
                    joined.push(Argument::Simple(t));
                    Term::Application(ApplicationTerm::new(
                        app.head.clone(),
                        joined.into_boxed_slice(),
                        app.presentation.clone(),
                    ))
                });
                ControlFlow::Continue(t)
            }
        }
    }

    fn do_bound(bound_index: usize, b: BindingTerm) -> std::ops::ControlFlow<Term, Term> {
        let pre = &b.arguments[..bound_index];
        let post = &b.arguments[bound_index + 1..];
        let Some(BoundArgument::BoundSeq(bound)) = b.arguments.get(bound_index) else {
            return ControlFlow::Continue(Term::Bound(b));
        };
        match bound {
            MaybeSequence::One(v) => {
                let mut joined = pre.to_vec();
                joined.push(BoundArgument::Bound(v.clone()));
                joined.extend_from_slice(post);
                ControlFlow::Continue(Term::Bound(BindingTerm::new(
                    b.head.clone(),
                    joined.into_boxed_slice(),
                    b.presentation.clone(),
                )))
            }
            MaybeSequence::Seq(m) => {
                let Some(BoundArgument::Simple(last)) = post.last() else {
                    return ControlFlow::Continue(Term::Bound(b));
                };
                let post = &post[..post.len() - 1];
                let t = m.iter().rfold(last.clone(), |t, v| {
                    let mut joined = pre.to_vec();
                    joined.push(BoundArgument::Bound(v.clone()));
                    joined.extend_from_slice(post);
                    joined.push(BoundArgument::Simple(t));
                    Term::Bound(BindingTerm::new(
                        b.head.clone(),
                        joined.into_boxed_slice(),
                        b.presentation.clone(),
                    ))
                });
                ControlFlow::Continue(t)
            }
        }
    }
}

impl<Split: SplitStrategy> PreparationRule<Split> for PrenexRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        let either::Left(sym) = head else {
            return false;
        };
        if sym.uri != self.0 {
            return false;
        }
        if let Term::Bound(b) = t {
            let Some(bound_index) = sym
                .data
                .arity
                .iter()
                .position(|m| matches!(m, ArgumentMode::BoundVariableSequence))
            else {
                tracing::trace!("No bound index");
                return false;
            };
            matches!(
                b.arguments.get(bound_index),
                Some(BoundArgument::BoundSeq(_))
            )
        } else if let Term::Application(a) = t {
            let Some(seq_index) = sym
                .data
                .arity
                .iter()
                .position(|m| matches!(m, ArgumentMode::Sequence))
            else {
                tracing::trace!("No sequence index");
                return false;
            };
            matches!(a.arguments.get(seq_index), Some(Argument::Sequence(_)))
        } else {
            tracing::trace!("Not a binder or application");
            false
        }
    }

    fn apply(
        &self,
        _: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> std::ops::ControlFlow<Term, Term> {
        tracing::trace!("Prenexing");
        let either::Left(sym) = head else {
            return ControlFlow::Continue(t);
        };
        match t {
            Term::Bound(b) => {
                if let Some(bound_index) = sym
                    .data
                    .arity
                    .iter()
                    .position(|m| matches!(m, ArgumentMode::BoundVariableSequence))
                {
                    Self::do_bound(bound_index, b)
                } else {
                    ControlFlow::Continue(Term::Bound(b))
                }
            }
            Term::Application(a) => {
                if let Some(seq_index) = sym
                    .data
                    .arity
                    .iter()
                    .position(|m| matches!(m, ArgumentMode::Sequence))
                {
                    Self::do_app(seq_index, a)
                } else {
                    ControlFlow::Continue(Term::Application(a))
                }
            }
            o => ControlFlow::Continue(o),
        }
    }
}

impl std::fmt::Display for PrenexRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a prenex binder", self.0)
    }
}
