use crate::{
    context::Context,
    rules::{CheckingRule, InhabitableRule, SizedSolverRule, SubtypeRule, UniverseRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{Argument, Term};
use ftml_uris::SymbolUri;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleInhabitableRule(pub SymbolUri, pub u8);
impl SizedSolverRule for SimpleInhabitableRule {}

impl std::fmt::Display for SimpleInhabitableRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is inhabitable", self.0)
    }
}

impl<Split: SplitStrategy> InhabitableRule<Split> for SimpleInhabitableRule {
    fn applicable(&self, term: &Term) -> bool {
        if self.1 == 0 {
            matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
        } else {
            matches!(term,Term::Application(a)
                if matches!(&a.head,Term::Symbol { uri, .. } if *uri == self.0)
                && a.arguments.len() == self.1 as usize
            )
        }
    }
    fn apply<'t>(
        &self,
        solver: crate::SolverRef<Split>,
        trace: &mut crate::trace::SolverTrace,
        mut context: Context<'t, '_>,
        t: &'t Term,
    ) -> Option<bool> {
        if self.1 == 0 {
            return Some(true);
        }
        let Term::Application(a) = t else { return None };
        for (i, arg) in a.arguments.iter().enumerate() {
            trace.comment(format!("Checking argument {}", i + 1));
            match arg {
                Argument::Simple(t) => {
                    if !solver.check_inhabitable(trace, context.branch(), t)? {
                        return Some(false);
                    }
                }
                _ => return None,
            }
        }
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUniverseRule(pub SymbolUri);
impl SizedSolverRule for SimpleUniverseRule {}
impl<Split: SplitStrategy> InhabitableRule<Split> for SimpleUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply(
        &self,
        _: crate::SolverRef<Split>,
        _: &mut crate::trace::SolverTrace,
        _: Context,
        _: &Term,
    ) -> Option<bool> {
        Some(true)
    }
}
impl<Split: SplitStrategy> UniverseRule<Split> for SimpleUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(
        &self,
        _: crate::SolverRef<Split>,
        _: &mut crate::trace::SolverTrace,
        _: Context<'t, '_>,
        _: &'t Term,
    ) -> Option<bool> {
        Some(true)
    }
}
impl std::fmt::Display for SimpleUniverseRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a universe", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyRule(pub SymbolUri);
impl SizedSolverRule for AnyRule {}

impl std::fmt::Display for AnyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is any-type", self.0)
    }
}
impl<Split: SplitStrategy> SubtypeRule<Split> for AnyRule {
    fn applicable(&self, _: &Term, sup: &Term) -> bool {
        matches!(sup,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(
        &self,
        solver: crate::SolverRef<Split>,
        trace: &mut crate::trace::SolverTrace,
        context: Context<'t, '_>,
        tm: &'t Term,
        _: &'t Term,
    ) -> Option<bool> {
        solver.check_inhabitable(trace, context, tm)
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for AnyRule {
    fn applicable(&self, _: &Term, tp: &Term) -> bool {
        matches!(tp,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(
        &self,
        _: crate::SolverRef<Split>,
        _: &mut crate::trace::SolverTrace,
        _: Context<'t, '_>,
        _: &'t Term,
        _: &'t Term,
    ) -> Option<bool> {
        Some(true)
    }
}
