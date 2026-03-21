use crate::{
    CheckRef,
    impls::solving::TermExtSolvable,
    patterns::Pattern,
    rules::{CheckingRule, InhabitableRule, SizedSolverRule, SubtypeRule, UniverseRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{Argument, Term};
use ftml_uris::SymbolUri;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleInhabitableRule(pub SymbolUri, pub u8);
impl SizedSolverRule for SimpleInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is inhabitable")
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for SimpleInhabitableRule {
    fn applicable(&self, term: &Term) -> bool {
        if self.1 == 0 {
            //ftml_ontology::matchtm!(sym(= &self.0) = term)
            matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
        } else {
            /*ftml_ontology::matchtm!(app({sym(=self.0)},[args]) = term
                => { args.len() == self.1 as usize} else {false}
            )*/
            matches!(term,Term::Application(a)
                if matches!(&a.head,Term::Symbol { uri, .. } if *uri == self.0)
                && a.arguments.len() == self.1 as usize
            )
        }
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, t: &'t Term) -> Option<bool> {
        if self.1 == 0 {
            return Some(true);
        }
        let Term::Application(a) = t else { return None };
        for (i, arg) in a.arguments.iter().enumerate() {
            checker.comment(format!("Checking argument {}", i + 1));
            match arg {
                Argument::Simple(t) => {
                    if !checker.check_inhabitable(t)? {
                        return Some(false);
                    }
                }
                Argument::Sequence(_) => return None,
            }
        }
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexInhabitableRule(pub Pattern);
impl SizedSolverRule for ComplexInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(self.0.body.clone(), "is inhabitable")
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for ComplexInhabitableRule {
    fn applicable(&self, term: &Term) -> bool {
        self.0.matches(term).is_some()
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, t: &'t Term) -> Option<bool> {
        // To be sure:
        checker.infer_type(t)?;
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUniverseRule(pub SymbolUri);

impl SizedSolverRule for SimpleUniverseRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is a universe")
    }
}
impl std::fmt::Display for SimpleUniverseRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a universe", self.0)
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for SimpleUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(&self, _: CheckRef<'t, '_, Split>, _: &'t Term) -> Option<bool> {
        Some(true)
    }
}
impl<Split: SplitStrategy> UniverseRule<Split> for SimpleUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(&self, _: CheckRef<'t, '_, Split>, _: &'t Term) -> Option<bool> {
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexUniverseRule(pub Pattern);
impl SizedSolverRule for ComplexUniverseRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(self.0.body.clone(), "is a universe")
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for ComplexUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        self.0.matches(term).is_some()
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, t: &'t Term) -> Option<bool> {
        // To be sure:
        checker.infer_type(t)?;
        Some(true)
    }
}
impl<Split: SplitStrategy> UniverseRule<Split> for ComplexUniverseRule {
    fn applicable(&self, term: &Term) -> bool {
        self.0.matches(term).is_some()
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, t: &'t Term) -> Option<bool> {
        // To be sure:
        checker.infer_type(t)?;
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyRule(pub SymbolUri);

impl SizedSolverRule for AnyRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is any-type")
    }
}

impl std::fmt::Display for AnyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is any-type", self.0)
    }
}
impl<Split: SplitStrategy> SubtypeRule<Split> for AnyRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, _: &Term, sup: &Term) -> bool {
        matches!(sup,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        tm: &'t Term,
        _: &'t Term,
    ) -> Option<bool> {
        checker.check_inhabitable(tm)
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for AnyRule {
    fn applicable(&self, _: &CheckRef<'_, '_, Split>, _: &Term, tp: &Term) -> bool {
        matches!(tp,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        tm: &'t Term,
        tp: &'t Term,
    ) -> Option<bool> {
        if let Some(ntp) = checker.infer_type(tm)
            && let Some(unk) = ntp.is_solvable()
        {
            return checker.solve_upper_bound(unk, tp);
        }
        Some(true)
    }
}
