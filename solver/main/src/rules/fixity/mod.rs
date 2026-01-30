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
    terms::{ApplicationTerm, Argument, ArgumentMode, MaybeSequence, Term, Variable},
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
        tracing::trace!("Not an application");
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

fn was_sequence_binary<'t>(
    uri: &SymbolUri,
    t: &'t Term,
    head: either::Either<&Symbol, &VariableDeclaration>,
) -> Option<(&'t ApplicationTerm, &'t Term, &'t Term, usize)> {
    let either::Left(sym) = head else {
        return None;
    };
    if sym.uri != *uri {
        return None;
    }
    let Term::Application(a) = t else {
        return None;
    };
    let Some(seq_index) = sym
        .data
        .arity
        .iter()
        .position(|m| matches!(m, ArgumentMode::Sequence))
    else {
        return None;
    };
    let num_args = sym.data.arity.num() as usize;
    let actual_args = a.arguments.len();
    if actual_args < num_args {
        return None;
    }
    let num_later = num_args - 1 - seq_index;
    let range_end = actual_args - num_later;
    if range_end != seq_index + 2 {
        return None;
    }
    let Some(Argument::Simple(first)) = a.arguments.get(seq_index) else {
        return None;
    };
    let Some(Argument::Simple(second)) = a.arguments.get(seq_index + 1) else {
        return None;
    };
    Some((a, first, second, seq_index))
}

fn match_head(
    lhs: either::Either<&Symbol, &VariableDeclaration>,
    rhs: Option<either::Either<&SymbolUri, &Variable>>,
) -> bool {
    match (lhs, rhs) {
        (either::Either::Left(s), Some(either::Either::Left(uri))) => *uri == s.uri,
        (either::Either::Right(vd), Some(either::Either::Right(v))) => {
            v.name() == vd.uri.name().last()
        }
        _ => false,
    }
}
