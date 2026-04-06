use ftml_ontology::terms::{
    ApplicationTerm, Argument, MaybeSequence, Numeric, Term, Variable,
    sequences::{Sequence, SequenceType},
};
use ftml_solver_trace::SizedSolverRule;

use crate::{
    CheckRef,
    rules::{
        EqualityRule, InferenceRule, InhabitableRule, UniverseRule,
        operators::numbers::{NumberRule, NumberType},
    },
    split::SplitStrategy,
};

pub mod fold;
pub mod map;

/*
pub enum Sequence<'t> {
    Var(&'t Variable),
    SequenceExpression(&'t [Term]),
    Map(Box<Self>, &'t Term),
    Concatenation(Vec<Self>),
}
impl Sequence<'_> {
    pub fn to_term(&self) -> Term {
        match self {
            Self::Var(v) => Term::Var {
                variable: (*v).clone(),
                presentation: None,
            },
            Self::SequenceExpression(ts) => Term::into_seq(ts.iter().cloned()),
            Self::Map(s, f) => Term::Application(ApplicationTerm::new(
                ftml_uris::metatheory::SEQUENCE_MAP.clone().into(),
                Box::new([
                    Argument::Simple(s.to_term()),
                    Argument::Simple((*f).clone()),
                ]),
                None,
            )),
            Self::Concatenation(ts) => Term::Application(ApplicationTerm::new(
                ftml_uris::metatheory::SEQUENCE_CONC.clone().into(),
                Box::new([Argument::Sequence(MaybeSequence::Seq(
                    ts.iter().map(Self::to_term).collect(),
                ))]),
                None,
            )),
        }
    }
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        match self {
            Self::Var(_) => false,
            Self::SequenceExpression(_) => true,
            Self::Map(s, _) => s.is_concrete(),
            Self::Concatenation(v) => v.iter().all(Self::is_concrete),
        }
    }
    #[must_use]
    pub fn to_concrete(&self) -> Option<Vec<Term>> {
        match self {
            Self::Var(_) => None,
            Self::SequenceExpression(es) => Some(es.to_vec()),
            Self::Map(seq, f) => seq.to_concrete().map(|seq| {
                seq.into_iter()
                    .map(|a| {
                        Term::Application(ApplicationTerm::new(
                            (*f).clone(),
                            Box::new([Argument::Simple(a)]),
                            None,
                        ))
                    })
                    .collect()
            }),
            Self::Concatenation(v) => {
                let mut ret = Vec::new();
                for v in v {
                    ret.extend(v.to_concrete()?);
                }
                Some(ret)
            }
        }
    }
}

pub enum SequenceType<'t> {
    Var(&'t Variable),
    SequenceExpression(&'t [Term]),
    Map(Sequence<'t>, &'t Term),
    SeqType(&'t Term, Option<&'t [Term]>),
}

pub trait TermExtSeq: Sized {
    fn is_sequence_variable(&self) -> bool;
    fn is_sequence(&self) -> bool;
    fn as_sequence(&self) -> Option<Sequence<'_>>;
    fn into_seq(seqs: impl Iterator<Item = Self>) -> Self;
    fn as_sequence_type(&self) -> Option<SequenceType<'_>>;
    #[must_use]
    fn into_seq_type(self) -> Self;
    #[must_use]
    fn into_ranged_seq_type(self, range: impl IntoIterator<Item = Term>) -> Self;
    /*
    fn as_sequence(&self) -> Option<&[Self]>;
    fn is_concrete_sequence(&self) -> bool;
    fn make_concrete_sequence(&self) -> Option<Vec<Self>>;
     */
}
impl TermExtSeq for Term {
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
    fn is_sequence(&self) -> bool {
        if self.is_sequence_variable() {
            return true;
        }
        let Self::Application(app) = self else {
            return false;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return false;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
            && let [Argument::Sequence(MaybeSequence::Seq(_))] = &*app.arguments
        {
            true
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            if let [
                Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                Argument::Simple(_),
            ] = &*app.arguments
                && seq.is_sequence()
            {
                true
            } else {
                false
            }
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_CONC {
            if let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
                && seq.iter().all(Self::is_sequence)
            {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn as_sequence(&self) -> Option<Sequence<'_>> {
        if let Self::Var {
            variable:
                v @ Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
            ..
        } = self
        {
            return Some(Sequence::Var(v));
        }
        let Self::Application(app) = self else {
            return None;
        };
        let Self::Symbol { uri, .. } = &app.head else {
            return None;
        };
        if *uri == *ftml_uris::metatheory::SEQUENCE_EXPRESSION
            && let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
        {
            Some(Sequence::SequenceExpression(seq))
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP {
            if let [
                Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                Argument::Simple(f),
            ] = &*app.arguments
                && let Some(seq) = seq.as_sequence()
            {
                Some(Sequence::Map(Box::new(seq), f))
            } else {
                None
            }
        } else if *uri == *ftml_uris::metatheory::SEQUENCE_CONC {
            if let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
                && seq.iter().all(Self::is_sequence)
            {
                Some(Sequence::Concatenation(
                    seq.iter().filter_map(Term::as_sequence).collect(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
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

    fn as_sequence_type(&self) -> Option<SequenceType<'_>> {
        if let Self::Application(app) = self
            && let Self::Symbol { uri, .. } = &app.head
        {
            if *uri == *ftml_uris::metatheory::SEQUENCE_TYPE
                && let [Argument::Simple(t)] = &*app.arguments
            {
                Some(SequenceType::SeqType(t, None))
            } else if *uri == *ftml_uris::metatheory::RANGED_SEQUENCE_TYPE
                && let [
                    Argument::Simple(t),
                    Argument::Sequence(MaybeSequence::Seq(range)),
                ] = &*app.arguments
            {
                Some(SequenceType::SeqType(t, Some(&**range)))
            } else if *uri == *ftml_uris::metatheory::SEQUENCE_MAP
                && let [
                    Argument::Simple(seq) | Argument::Sequence(MaybeSequence::One(seq)),
                    Argument::Simple(f),
                ] = &*app.arguments
            {
                Some(SequenceType::Map(seq.as_sequence()?, f))
            } else {
                None
            }
        } else if let Self::Var {
            variable:
                v @ Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
            ..
        } = self
        {
            // TODO check that variable is inhabitable?
            Some(SequenceType::Var(v))
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

    fn into_ranged_seq_type(self, range: impl IntoIterator<Item = Self>) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::RANGED_SEQUENCE_TYPE.clone(),
                presentation: None,
            },
            Box::new([
                Argument::Simple(self),
                Argument::Sequence(MaybeSequence::Seq(range.into_iter().collect())),
            ]),
            None,
        ))
    }

    /*
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
     */
}
 */

#[allow(clippy::match_like_matches_macro)]
const fn is_index(t: &Argument) -> bool {
    match t {
        Argument::Simple(Term::Number(Numeric::Int(_))) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqConcatInferenceRule;
impl SizedSolverRule for SeqConcatInferenceRule {
    fn priority(&self) -> isize {
        1000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("sequence concatenation inference")
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for SeqConcatInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Application(app) = term
            && app.head.is(&*ftml_uris::metatheory::SEQUENCE_CONC)
            && let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments
            && seq.iter().all(|s| s.is_sequence())
        {
            true
        } else {
            false
        }
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let Term::Application(app) = term else {
            return None;
        };
        let [Argument::Sequence(MaybeSequence::Seq(seq))] = &*app.arguments else {
            return None;
        };
        let mut curr = None;
        for s in seq {
            let tp = checker.infer_type(s)?;
            if let Some(otp) = curr.as_ref() {
                if checker.scoped(|c| c.check_equality(otp, &tp)) != Some(true) {
                    return None;
                }
            } else {
                curr = Some(tp);
            }
        }
        curr
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
impl<Split: SplitStrategy> InferenceRule<Split> for SeqIndexRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Application(app) if app.head.is_sequence_variable() && app.arguments.len() == 1
            && matches!(app.arguments.first(),Some(Argument::Simple(_)))
            //&& app.arguments.first().is_some_and(is_index)
        )
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let Term::Application(app) = term else {
            return None;
        };
        let tp = checker.infer_type(&app.head)?;
        if let SequenceType::SeqType(t, _) = tp.as_sequence_type()? {
            Some(t.clone())
        } else {
            let Some(Argument::Simple(index)) = app.arguments.first() else {
                checker.failure("First argument is not simple");
                return None;
            }; //.clone();
            let indextp = checker.infer_type(index)?;
            if NumberRule::is_number_term(&indextp, &checker)
                .is_none_or(|t| !(t <= NumberType::Integers))
            {
                checker.failure("index is not an ordinal type");
                return None;
            }
            Some(Term::Application(ApplicationTerm::new(
                tp,
                Box::new([Argument::Simple(index.clone())]),
                None,
            )))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqUniverseRule;
impl SizedSolverRule for SeqUniverseRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("sequences of inhabitables/universes are inhabitable/universes")
    }
}
impl<Split: SplitStrategy> UniverseRule<Split> for SeqUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        term.as_sequence_type().is_some()
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        match term.as_sequence_type()? {
            SequenceType::SeqType(univ, _) => checker.check_universe(univ),
            SequenceType::SequenceExpression(expr) => {
                for e in expr {
                    if checker.check_universe(e) != Some(true) {
                        return None;
                    }
                }
                Some(true)
            }
            _ => None, // TODO?
        }
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for SeqUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        term.as_sequence_type().is_some()
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        //let univ = term.as_sequence_type()?;
        match term.as_sequence_type()? {
            SequenceType::SeqType(inh, _) => checker.check_inhabitable(inh),
            SequenceType::SequenceExpression(expr) => {
                for e in expr {
                    if checker.check_inhabitable(e) != Some(true) {
                        return None;
                    }
                }
                Some(true)
            }
            _ => None, // TODO?
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqInferenceRule;
impl SizedSolverRule for SeqInferenceRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("sequences have sequence types")
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for SeqInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term.as_sequence(), Some(Sequence::SequenceExpression(_)))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        //checker.comment(format!("Here: {:?}", term.debug_short()));
        let Some(Sequence::SequenceExpression(elems)) = term.as_sequence() else {
            return None;
        };
        let mut curr = None;
        for e in elems {
            if let Some(tp) = &curr {
                if checker.scoped(|checker| checker.check_type(e, tp)) != Some(true) {
                    return None;
                }
            } else {
                curr = Some(checker.infer_type(e)?);
            }
        }
        curr.map(Term::into_seq_type)
    }
}

/*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqTypeEqRule;
impl SizedSolverRule for SeqTypeEqRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("sequences of types are sequence types")
    }
}
impl<Split: SplitStrategy> EqualityRule<Split> for SeqTypeEqRule {
    fn applicable(&self, lhs: &Term, rhs: &Term) -> bool {
        (lhs.as_sequence_type().is_some() && rhs.as_sequence().is_some_and(|r| r.is_concrete()))
            || (rhs.as_sequence_type().is_some()
                && lhs.as_sequence().is_some_and(|r| r.is_concrete()))
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        lhs: &'t Term,
        rhs: &'t Term,
    ) -> Option<bool> {
        if rhs.as_sequence_type().is_some() && lhs.as_sequence().is_some_and(|r| r.is_concrete()) {
            return self.apply(checker, rhs, lhs);
        }
        let Some(SequenceType::SeqType(seqtp, _)) = lhs.as_sequence_type() else {
            return None;
        };
        let seq = rhs.as_sequence()?.to_concrete()?;
        checker.comment(format!(
            "Here: {seq:#?}\n    Type: {:?}",
            seqtp.debug_short()
        ));
        for t in seq {
            if checker.scoped(|c| c.check_equality(&t, seqtp)) != Some(true) {
                return None;
            }
        }
        Some(true)
    }
}
 */
