use ftml_ontology::terms::{
    BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term, Variable,
};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::Id;

use crate::{
    rules::{InferenceRule, sequences::SequenceType},
    split::SplitStrategy,
};

static X: std::sync::LazyLock<Id> =
    std::sync::LazyLock::new(|| unsafe { "x".parse().unwrap_unchecked() });
static Y: std::sync::LazyLock<Id> =
    std::sync::LazyLock::new(|| unsafe { "y".parse().unwrap_unchecked() });

pub struct Fold;
impl Fold {
    pub fn apply_init(
        seq: MaybeSequence<Term>,
        init: Term,
        then: impl FnOnce(Variable, Variable) -> Term,
    ) -> Term {
        Term::Bound(BindingTerm::new(
            ftml_uris::metatheory::FOLD_RIGHT.clone().into(),
            Box::new([
                BoundArgument::Sequence(seq),
                BoundArgument::Simple(init),
                BoundArgument::Bound(ComponentVar {
                    var: X.clone().into(),
                    tp: None,
                    df: None,
                }),
                BoundArgument::Bound(ComponentVar {
                    var: Y.clone().into(),
                    tp: None,
                    df: None,
                }),
                BoundArgument::Simple(then(X.clone().into(), Y.clone().into())),
            ]),
            None,
        ))
    }
    pub fn unapply_init(
        term: &Term,
    ) -> Option<(
        &MaybeSequence<Term>,
        &Term,
        &ComponentVar,
        &ComponentVar,
        &Term,
    )> {
        let Term::Bound(b) = term else { return None };
        if !b.head.is(&*ftml_uris::metatheory::FOLD_RIGHT) {
            return None;
        }
        let [
            BoundArgument::Sequence(seq),
            BoundArgument::Simple(init),
            BoundArgument::Bound(cv1),
            BoundArgument::Bound(cv2),
            BoundArgument::Simple(ret),
        ] = &*b.arguments
        else {
            return None;
        };
        Some((seq, init, cv1, cv2, ret))
    }

    pub fn apply(seq: MaybeSequence<Term>, then: impl FnOnce(Variable, Variable) -> Term) -> Term {
        Term::Bound(BindingTerm::new(
            ftml_uris::metatheory::FOLD.clone().into(),
            Box::new([
                BoundArgument::Sequence(seq),
                BoundArgument::Bound(ComponentVar {
                    var: X.clone().into(),
                    tp: None,
                    df: None,
                }),
                BoundArgument::Bound(ComponentVar {
                    var: Y.clone().into(),
                    tp: None,
                    df: None,
                }),
                BoundArgument::Simple(then(X.clone().into(), Y.clone().into())),
            ]),
            None,
        ))
    }
    pub fn unapply(
        term: &Term,
    ) -> Option<(&MaybeSequence<Term>, &ComponentVar, &ComponentVar, &Term)> {
        let Term::Bound(b) = term else { return None };
        if !b.head.is(&*ftml_uris::metatheory::FOLD) {
            return None;
        }
        let [
            BoundArgument::Sequence(seq),
            BoundArgument::Bound(cv1),
            BoundArgument::Bound(cv2),
            BoundArgument::Simple(ret),
        ] = &*b.arguments
        else {
            return None;
        };
        Some((seq, cv1, cv2, ret))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FoldInferenceRule;
impl SizedSolverRule for FoldInferenceRule {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&*ftml_uris::metatheory::FOLD_RIGHT)
    }
}
impl FoldInferenceRule {
    fn infer_init<'t, Split: SplitStrategy>(
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<Term> {
        let (seq, init, cv1, cv2, ret) = Fold::unapply_init(term)?;
        let seqtp = match seq {
            MaybeSequence::One(t) => checker.infer_type(t)?,
            MaybeSequence::Seq(ts) => {
                let seq = Term::into_seq(ts.iter().cloned());
                checker.scoped(|checker| checker.infer_type(&seq))?
            }
        };
        let Some(SequenceType::SeqType(seqtp, _)) = seqtp.as_sequence_type() else {
            checker.failure("not a sequence type");
            return None;
        };
        let inittp = checker.infer_type(init)?;
        let ncv1 = ComponentVar {
            var: cv1.var.clone(),
            tp: Some(seqtp.clone()),
            df: None,
        };
        let ncv2 = ComponentVar {
            var: cv2.var.clone(),
            tp: Some(inittp),
            df: None,
        };
        checker.extend_context(ncv1);
        checker.extend_context(ncv2);
        checker.infer_type(ret)
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for FoldInferenceRule {
    fn applicable(&self, term: &Term) -> bool {
        Fold::unapply_init(term).is_some() || Fold::unapply(term).is_some()
    }
    fn infer<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Option<Term> {
        let Some((seq, cv1, cv2, ret)) = Fold::unapply(term) else {
            return Self::infer_init(checker, term);
        };
        let seqtp = match seq {
            MaybeSequence::One(t) => checker.infer_type(t)?,
            MaybeSequence::Seq(ts) => {
                let seq = Term::into_seq(ts.iter().cloned());
                checker.scoped(|checker| checker.infer_type(&seq))?
            }
        };
        let Some(SequenceType::SeqType(seqtp, _)) = seqtp.as_sequence_type() else {
            checker.failure("not a sequence type");
            return None;
        };
        if let Some(tp) = cv1.tp.as_ref()
            && checker.scoped(|c| c.check_subtype(seqtp, tp)) != Some(true)
        {
            return None;
        }
        if let Some(tp) = cv2.tp.as_ref()
            && checker.scoped(|c| c.check_subtype(seqtp, tp)) != Some(true)
        {
            return None;
        }
        let ncv1 = ComponentVar {
            var: cv1.var.clone(),
            tp: Some(seqtp.clone()),
            df: None,
        };
        let ncv2 = ComponentVar {
            var: cv2.var.clone(),
            tp: Some(seqtp.clone()),
            df: None,
        };
        checker.extend_context(ncv1);
        checker.extend_context(ncv2);
        checker.infer_type(ret)
    }
}
