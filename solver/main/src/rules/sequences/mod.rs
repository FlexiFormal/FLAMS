use ftml_ontology::terms::{ApplicationTerm, Argument, MaybeSequence, Numeric, Term, Variable};
use ftml_solver_trace::SizedSolverRule;

use crate::{CheckRef, rules::InferenceRule, split::SplitStrategy};

pub mod map;

pub trait TermExtSeq: Sized {
    fn as_sequence_type(&self) -> Option<&Self>;
    fn as_sequence(&self) -> Option<&[Self]>;
    #[must_use]
    fn into_seq_type(self) -> Self;
    fn is_sequence_variable(&self) -> bool;
    fn is_concrete_sequence(&self) -> bool;
    fn make_concrete_sequence(&self) -> Option<Vec<Self>>;
    fn into_seq(seqs: impl Iterator<Item = Self>) -> Self;
}
impl TermExtSeq for Term {
    fn into_seq(seqs: impl Iterator<Item = Self>) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_EXPRESSION.clone(),
                presentation: None,
            },
            Box::new([Argument::Sequence(MaybeSequence::Seq(seqs.collect()))]),
            None,
        ))
    }

    fn is_sequence_variable(&self) -> bool {
        matches!(
            self,
            Self::Var {
                variable: Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
                ..
            }
        )
    }
    fn is_concrete_sequence(&self) -> bool {
        let Self::Application(app) = self else {
            return false;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return false;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION {
            return matches!(&*app.arguments, [Argument::Sequence(MaybeSequence::Seq(_))]);
        }
        if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            let [
                Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                Argument::Simple(_),
            ] = &*app.arguments
            else {
                return false;
            };
            seq.is_concrete_sequence()
        } else {
            false
        }
    }
    fn make_concrete_sequence(&self) -> Option<Vec<Self>> {
        let Self::Application(app) = self else {
            return None;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return None;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION {
            let [Argument::Sequence(MaybeSequence::Seq(ts))] = &*app.arguments else {
                return None;
            };
            return Some(ts.to_vec());
        }
        if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            let [
                Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                Argument::Simple(f),
            ] = &*app.arguments
            else {
                return None;
            };
            Some(
                seq.make_concrete_sequence()?
                    .into_iter()
                    .map(|a| {
                        Self::Application(ApplicationTerm::new(
                            f.clone(),
                            Box::new([Argument::Simple(a)]),
                            None,
                        ))
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    fn as_sequence(&self) -> Option<&[Self]> {
        let Self::Application(app) = self else {
            return None;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return None;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
            && let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
        {
            Some(seq)
        } else {
            None
        }
    }
    fn as_sequence_type(&self) -> Option<&Self> {
        if let Self::Application(app) = self
            && matches!(&app.head,
                Self::Symbol { uri, .. } if *uri == *ftml_uris::metatheory::SEQUENCE_TYPE
            )
            && app.arguments.len() == 1
            && let Some(Argument::Simple(t)) = app.arguments.first()
        {
            Some(t)
        } else {
            None
        }
    }
    fn into_seq_type(self) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_TYPE.clone(),
                presentation: None,
            },
            Box::new([Argument::Simple(self)]),
            None,
        ))
    }
}

#[allow(clippy::match_like_matches_macro)]
const fn is_index(t: &Argument) -> bool {
    match t {
        Argument::Simple(Term::Number(Numeric::Int(_))) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqIndexRule;
impl SizedSolverRule for SeqIndexRule {
    fn priority(&self) -> isize {
        1000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("sequence index")
    }
}
impl std::fmt::Display for SeqIndexRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("sequence index")
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for SeqIndexRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Application(app) if app.head.is_sequence_variable() && app.arguments.len() == 1
            && app.arguments.first().is_some_and(is_index))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let Term::Application(app) = term else {
            return None;
        };
        checker.infer_type(&app.head)?.as_sequence_type().cloned()
    }
}
