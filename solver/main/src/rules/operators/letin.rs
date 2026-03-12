use ftml_ontology::terms::{BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{
    rules::{PreparationRule, SimplificationRule},
    split::SplitStrategy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetinComputation(pub SymbolUri);

impl SizedSolverRule for LetinComputation {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " x := t in s ==> s[x/t]")
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for LetinComputation {
    fn applicable(&self, checker: &crate::CheckRef<'_, '_, Split>, t: &Term) -> bool {
        false
    }
    fn applicable_revert(&self, checker: &crate::CheckRef<'_, '_, Split>, t: &Term) -> bool {
        if let Term::Bound(b) = t
            && let Term::Symbol { uri, .. } = &b.head
            && *uri == self.0
            && let [
                BoundArgument::Bound(_) | BoundArgument::BoundSeq(MaybeSequence::Seq(_)),
                BoundArgument::Simple(body),
            ] = &*b.arguments
        {
            <Self as SimplificationRule<Split>>::applicable(&self, body)
        } else {
            false
        }
    }
    fn apply(
        &self,
        checker: &mut crate::CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> std::ops::ControlFlow<Term, Term> {
        std::ops::ControlFlow::Continue(t)
    }
    fn revert(
        &self,
        checker: &crate::CheckRef<'_, '_, Split>,
        t: Term,
    ) -> std::ops::ControlFlow<Term, Term> {
        let Term::Bound(b) = t else {
            return std::ops::ControlFlow::Continue(t);
        };
        let [
            var @ (BoundArgument::Bound(_) | BoundArgument::BoundSeq(MaybeSequence::Seq(_))),
            BoundArgument::Simple(body),
        ] = &*b.arguments
        else {
            return std::ops::ControlFlow::Continue(Term::Bound(b));
        };
        let Term::Bound(b2) = body else {
            return std::ops::ControlFlow::Continue(Term::Bound(b));
        };
        let [
            var2 @ (BoundArgument::Bound(_) | BoundArgument::BoundSeq(MaybeSequence::Seq(_))),
            BoundArgument::Simple(body),
        ] = &*b2.arguments
        else {
            return std::ops::ControlFlow::Continue(Term::Bound(b));
        };
        let mut vars = Vec::new();
        match var {
            BoundArgument::Bound(v) => vars.push(v.clone()),
            BoundArgument::BoundSeq(MaybeSequence::Seq(v)) => vars = v.clone().into_vec(),
            _ => unreachable!(),
        }
        match var2 {
            BoundArgument::Bound(v) => vars.push(v.clone()),
            BoundArgument::BoundSeq(MaybeSequence::Seq(v)) => vars.extend(v.iter().cloned()),
            _ => unreachable!(),
        }
        std::ops::ControlFlow::Continue(Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri: self.0.clone(),
                presentation: None,
            },
            Box::new([
                BoundArgument::BoundSeq(MaybeSequence::Seq(vars.into_boxed_slice())),
                BoundArgument::Simple(body.clone()),
            ]),
            None,
        )))
    }
}

impl<Split: SplitStrategy> SimplificationRule<Split> for LetinComputation {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Bound(b) = term
            && let Term::Symbol { uri, .. } = &b.head
            && *uri == self.0
            && let [
                BoundArgument::Bound(_) | BoundArgument::BoundSeq(MaybeSequence::Seq(_)),
                BoundArgument::Simple(_),
            ] = &*b.arguments
        {
            true
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        checker: crate::CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Bound(b) = term else {
            return Err(None);
        };
        let [
            var @ (BoundArgument::Bound(_) | BoundArgument::BoundSeq(MaybeSequence::Seq(_))),
            BoundArgument::Simple(body),
        ] = &*b.arguments
        else {
            return Err(None);
        };
        match var {
            BoundArgument::Bound(ComponentVar {
                var, df: Some(d), ..
            }) => Ok((body / (var.name(), d)).into_owned()),
            BoundArgument::Bound(v) => Ok((body
                / (
                    v.var.name(),
                    &checker.get_var_definiens(&v.var).ok_or(None)?,
                ))
                .into_owned()),
            BoundArgument::BoundSeq(MaybeSequence::Seq(s)) => s
                .iter()
                .rev()
                .try_fold(body.clone(), |b, v| match v {
                    ComponentVar {
                        var, df: Some(d), ..
                    } => Some(b / (var.name(), d)),
                    ComponentVar { var, .. } => {
                        Some(b / (var.name(), &checker.get_var_definiens(&var)?))
                    }
                })
                .ok_or(None),
            _ => unreachable!(),
        }
    }
}
