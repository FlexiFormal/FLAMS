use crate::{
    rules::{PreparationRule, RuleSet, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::{
    domain::declarations::symbols::Symbol,
    narrative::elements::VariableDeclaration,
    terms::{ApplicationTerm, BindingTerm, Term},
    utils::Permutation,
};
use ftml_uris::SymbolUri;
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReorderRule {
    pub symbol: SymbolUri,
    pub reorder: Permutation,
}
impl SizedSolverRule for ReorderRule {
    fn priority(&self) -> isize {
        100
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(
            &self.symbol,
            format!("reorders argument {:?}", self.reorder)
        )
    }
}
impl std::fmt::Display for ReorderRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} reorder arguments {:?}", self.symbol, self.reorder)
    }
}

impl<Split: SplitStrategy> PreparationRule<Split> for ReorderRule {
    fn applicable(&self, t: &Term, _: either::Either<&Symbol, &VariableDeclaration>) -> bool {
        match t {
            Term::Application(a) => {
                matches!(&a.head,Term::Symbol { uri, .. } if *uri == self.symbol)
                    && a.arguments.len() == self.reorder.len()
            }
            Term::Bound(b) => {
                matches!(&b.head,Term::Symbol { uri, .. } if *uri == self.symbol)
                    && b.arguments.len() == self.reorder.len()
            }
            _ => false,
        }
    }
    fn apply(
        &self,
        _: &RuleSet<Split>,
        t: Term,
        _: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(match t {
            Term::Application(app) => Term::Application(ApplicationTerm::new(
                app.head.clone(),
                // SAFETY: applicable checks `arguments.len() == self.reorder.len()`
                unsafe {
                    debug_assert_eq!(app.arguments.len(), self.reorder.len());
                    self.reorder.apply_unchecked(&app.arguments)
                }
                .into_boxed_slice(),
                app.presentation.clone(),
            )),
            Term::Bound(app) => Term::Bound(BindingTerm::new(
                app.head.clone(),
                // SAFETY: applicable checks `arguments.len() == self.reorder.len()`
                unsafe {
                    debug_assert_eq!(app.arguments.len(), self.reorder.len());
                    self.reorder.apply_unchecked(&app.arguments)
                }
                .into_boxed_slice(),
                app.presentation.clone(),
            )),
            t => t,
        })
    }
    fn applicable_revert(
        &self,
        t: &Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> bool {
        <Self as PreparationRule<Split>>::applicable(self, t, head)
    }
    fn revert(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(match t {
            Term::Application(app) => Term::Application(ApplicationTerm::new(
                app.head.clone(),
                // SAFETY: applicable checks `arguments.len() == self.reorder.len()`
                unsafe {
                    debug_assert_eq!(app.arguments.len(), self.reorder.len());
                    self.reorder.revert_unchecked(&app.arguments)
                }
                .into_boxed_slice(),
                app.presentation.clone(),
            )),
            Term::Bound(app) => Term::Bound(BindingTerm::new(
                app.head.clone(),
                // SAFETY: applicable checks `arguments.len() == self.reorder.len()`
                unsafe {
                    debug_assert_eq!(app.arguments.len(), self.reorder.len());
                    self.reorder.revert_unchecked(&app.arguments)
                }
                .into_boxed_slice(),
                app.presentation.clone(),
            )),
            t => t,
        })
    }
}
