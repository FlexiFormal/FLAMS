mod bin;
mod conj;
mod pre;
mod reorder;

pub use bin::*;
pub use conj::*;
pub use pre::*;
pub use reorder::*;

use ftml_ontology::{
    domain::declarations::symbols::Symbol,
    narrative::elements::VariableDeclaration,
    terms::{ApplicationTerm, Argument, ArgumentMode, MaybeSequence, Term},
};
use ftml_uris::SymbolUri;

fn is_sequence_binary<'t>(
    uri: &SymbolUri,
    t: &'t Term,
    head: either::Either<&Symbol, &VariableDeclaration>,
) -> Option<(&'t ApplicationTerm, &'t MaybeSequence<Term>, usize)> {
    let either::Left(sym) = head else {
        return None;
    };
    if sym.uri != *uri {
        return None;
    }
    let Term::Application(a) = t else {
        tracing::trace!("Not a binder");
        return None;
    };
    let Some(seq_index) = sym
        .data
        .arity
        .iter()
        .position(|m| matches!(m, ArgumentMode::Sequence))
    else {
        tracing::trace!("No sequence index");
        return None;
    };
    match a.arguments.get(seq_index) {
        Some(Argument::Sequence(s)) => Some((a, s, seq_index)),
        _ => None,
    }
}
