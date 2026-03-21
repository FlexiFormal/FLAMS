use crate::{
    CheckRef,
    rules::{PreparationRule, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::{
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
        100_000
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
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
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
        _: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term> {
        if let Some(i) = path.and_then(|(v, i)| {
            v.get_mut(i)
                .and_then(|i| if *i > 0 { Some(i) } else { None })
        }) {
            *i = self.reorder.of(*i).unwrap_or(*i);
        }

        //tracing::debug!("Reordering {:?}", t.debug_short());
        let r = match t {
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
        };
        //tracing::debug!("Result: {:?}", r.debug_short());
        ControlFlow::Continue(r)
    }
    fn applicable_revert(&self, checker: &CheckRef<'_, '_, Split>, t: &Term) -> bool {
        <Self as PreparationRule<Split>>::applicable(self, checker, t)
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term> {
        //tracing::debug!("Reverting Reordering {:?}", t.debug_short());
        let r = match t {
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
        };
        //tracing::debug!("Result: {:?}", r.debug_short());
        ControlFlow::Continue(r)
    }
}
