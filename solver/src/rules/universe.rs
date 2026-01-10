use crate::{
    CheckRef,
    rules::{CheckingRule, InhabitableRule, SizedSolverRule, SubtypeRule, UniverseRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{Argument, Term};
use ftml_uris::{FtmlUri, SymbolUri};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleInhabitableRule(pub SymbolUri, pub u8);
impl SizedSolverRule for SimpleInhabitableRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), " is inhabitable")
    }
}

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
                _ => return None,
            }
        }
        Some(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUniverseRule(pub SymbolUri);
impl SizedSolverRule for SimpleUniverseRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), " is a universe")
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
pub struct AnyRule(pub SymbolUri);
impl SizedSolverRule for AnyRule {
    fn display(
        &self,
        displayer: &dyn crate::trace::TraceDisplay,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        crate::trace!(displayer, f, self.0.as_uri(), " is any-type")
    }
}

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
        mut checker: CheckRef<'t, '_, Split>,
        tm: &'t Term,
        _: &'t Term,
    ) -> Option<bool> {
        checker.check_inhabitable(tm)
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for AnyRule {
    fn applicable(&self, _: &Term, tp: &Term) -> bool {
        matches!(tp,Term::Symbol { uri, .. } if *uri == self.0)
    }
    fn apply<'t>(&self, _: CheckRef<'t, '_, Split>, _: &'t Term, _: &'t Term) -> Option<bool> {
        Some(true)
    }
}
