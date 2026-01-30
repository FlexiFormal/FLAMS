use ftml_ontology::terms::{Argument, Numeric, Term, Variable};

use crate::{
    CheckRef, TermExtSeq,
    rules::{InferenceRule, SizedSolverRule},
    split::SplitStrategy,
};

#[allow(clippy::match_like_matches_macro)]
const fn is_sequence(t: &Term) -> bool {
    match t {
        Term::Var {
            variable:
                Variable::Ref {
                    is_sequence: Some(true),
                    ..
                },
            ..
        } => true,
        _ => false,
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
        matches!(term,Term::Application(app) if is_sequence(&app.head) && app.arguments.len() == 1
            && app.arguments.first().is_some_and(is_index))
    }
    fn infer<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        let Term::Application(app) = term else {
            return None;
        };
        checker.infer_type(&app.head)?.is_sequence_type().cloned()
    }
}
