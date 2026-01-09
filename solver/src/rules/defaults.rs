use ftml_ontology::terms::{Argument, Numeric, Term, Variable};

use crate::{
    TermExtSeq,
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
    /*
    fn display(self: Box<Self>) -> crate::trace::RefCheckLog<'static> {
        crate::traceline!("sequence index")
    }
     */
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
    fn infer<'t>(
        &self,
        solver: crate::SolverRef<Split>,
        trace: &mut crate::trace::SolverTrace,
        context: crate::context::Context<'t, '_>,
        term: &'t Term,
    ) -> Option<Term> {
        let Term::Application(app) = term else {
            return None;
        };
        solver
            .infer_type(trace, context, &app.head)?
            .is_sequence_type()
            .cloned()
    }
}

/*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqInhabitableRule;
impl SizedSolverRule for SeqInhabitableRule {}
impl std::fmt::Display for SeqInhabitableRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("sequence types are inhabitable")
    }
}

fn is_sequence(t: &Term) -> Option<&Term> {
    if let Term::Application(app) = t
        && let Term::Symbol { uri, .. } = &app.head
        && *uri == *ftml_uris::metatheory::SEQUENCE_TYPE
        && app.arguments.len() == 1
        && let Some(Argument::Simple(t)) = app.arguments.first()
    {
        Some(t)
    } else {
        None
    }
}

impl<Split: SplitStrategy> InhabitableRule<Split> for SeqInhabitableRule {
    fn applicable(&self, term: &ftml_ontology::terms::Term) -> bool {
        is_sequence(term).is_some()
    }
    fn apply<'t>(
        &self,
        solver: crate::SolverRef<Split>,
        trace: &mut crate::trace::SolverTrace,
        context: crate::context::Context<'t, '_>,
        term: &'t Term,
    ) -> Option<bool> {
        let arg = is_sequence(term)?;
        solver.check_inhabitable(trace, context, arg)
    }
}
*/
