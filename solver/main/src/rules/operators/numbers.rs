use ftml_ontology::terms::{Numeric, Term};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{
    CheckRef,
    rules::{CheckingRule, InferenceRule, MarkerRule, SubtypeRule},
    split::SplitStrategy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberType {
    PositiveNaturals,
    Naturals,
    NegativeIntegers,
    Integers, // PositiveIntegers = Naturals
    NonZeroIntegers,
    Rationals,
    PositiveRationals,
    NegativeRationals,
    NonZeroRationals,
    Reals,
    PositiveReals,
    NegativeReals,
    NonZeroReals,
}
impl NumberType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveNaturals => "natural numbers (excluding zero)",
            Self::Naturals => "natural numbers (including 0)",
            Self::NegativeIntegers => "negative integers",
            Self::Integers => "integers",
            Self::NonZeroIntegers => "non-zero integers",
            Self::Rationals => "rational numbers",
            Self::PositiveRationals => "positive rational numbers (including 0)",
            Self::NegativeRationals => "negative rational numbers",
            Self::NonZeroRationals => "non-zero rational numbers",
            Self::Reals => "real numbers",
            Self::PositiveReals => "positive real numbers (including zero)",
            Self::NegativeReals => "negative real numbers",
            Self::NonZeroReals => "non-zero real numbers",
        }
    }
    pub fn contains(self, num: &Numeric) -> bool {
        match self {
            Self::Reals => true,
            Self::NonZeroReals => match num {
                Numeric::Int(i) => *i != 0,
                Numeric::Float(i) => **i != 0.0,
            },
            Self::PositiveReals => match num {
                Numeric::Int(i) => *i >= 0,
                Numeric::Float(i) => **i >= 0.0,
            },
            Self::NegativeReals => match num {
                Numeric::Int(i) => *i < 0,
                Numeric::Float(i) => **i < 0.0,
            },
            Self::Integers => num.as_int().is_some(),
            Self::NegativeIntegers => num.as_int().is_some_and(|i| i < 0),
            Self::NonZeroIntegers => num.as_int().is_some_and(|i| i != 0),
            Self::Naturals => num.as_int().is_some_and(|i| i >= 0),
            Self::PositiveNaturals => num.as_int().is_some_and(|i| i > 0),
            _ => false,
        }
    }
    pub fn get<'c, Split: SplitStrategy>(
        self,
        checker: &'c CheckRef<'_, '_, Split>,
    ) -> Option<&'c SymbolUri> {
        checker.rules().marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<NumberRule>()
                .and_then(|rl| if rl.typ == self { Some(&rl.sym) } else { None })
        })
    }
}
impl PartialOrd for NumberType {
    #[allow(clippy::match_same_arms)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::{Greater, Less};
        if *self == *other {
            return Some(std::cmp::Ordering::Equal);
        }
        match (*self, *other) {
            (_, Self::Reals) => Some(Less),
            (Self::Reals, _) => Some(Greater),
            (
                Self::PositiveNaturals,
                Self::NegativeIntegers | Self::NegativeRationals | Self::NegativeReals,
            ) => None,
            (Self::PositiveNaturals, _) => Some(Less),
            (Self::Naturals, Self::PositiveNaturals) => Some(Greater),
            (
                Self::Naturals,
                Self::Integers | Self::Rationals | Self::PositiveRationals | Self::PositiveReals,
            ) => Some(Less),
            (
                Self::NegativeIntegers,
                Self::Integers
                | Self::NonZeroIntegers
                | Self::Rationals
                | Self::NegativeRationals
                | Self::NonZeroRationals
                | Self::NegativeReals
                | Self::NonZeroReals,
            ) => Some(Less),
            (Self::Integers, Self::PositiveNaturals | Self::Naturals) => Some(Greater),
            (Self::Integers, Self::Rationals | Self::PositiveReals) => Some(Less),
            (Self::NonZeroIntegers, Self::PositiveNaturals | Self::NegativeIntegers) => {
                Some(Greater)
            }
            (
                Self::Rationals,
                Self::PositiveNaturals
                | Self::Naturals
                | Self::NegativeIntegers
                | Self::Integers
                | Self::NonZeroIntegers,
            ) => Some(Greater),
            (Self::PositiveRationals, Self::PositiveNaturals | Self::Naturals) => Some(Greater),
            (Self::PositiveRationals, Self::PositiveReals) => Some(Less),
            (Self::NegativeRationals, Self::NegativeIntegers | Self::NonZeroIntegers) => {
                Some(Greater)
            }
            (
                Self::NegativeRationals,
                Self::NonZeroRationals | Self::NegativeReals | Self::NonZeroReals,
            ) => Some(Less),
            (
                Self::NonZeroRationals,
                Self::PositiveNaturals | Self::NegativeIntegers | Self::NonZeroIntegers,
            ) => Some(Greater),
            (Self::NonZeroRationals, Self::Rationals | Self::NonZeroReals) => Some(Less),
            (
                Self::PositiveReals,
                Self::PositiveNaturals | Self::Naturals | Self::PositiveRationals,
            ) => Some(Greater),
            (Self::PositiveReals, Self::NonZeroReals) => Some(Less),
            (Self::NegativeReals, Self::NegativeIntegers | Self::NegativeRationals) => {
                Some(Greater)
            }
            (Self::NegativeReals, Self::NonZeroReals) => Some(Less),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberRule {
    pub typ: NumberType,
    pub sym: SymbolUri,
}
impl SizedSolverRule for NumberRule {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.sym, "is type of", self.typ.as_str())
    }
}
impl<Split: SplitStrategy> MarkerRule<Split> for NumberRule {}
impl NumberRule {
    pub fn is_number<Split: SplitStrategy>(
        uri: &SymbolUri,
        checker: &CheckRef<'_, '_, Split>,
    ) -> Option<NumberType> {
        checker.rules().marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<Self>()
                .and_then(|rl| if rl.sym == *uri { Some(rl.typ) } else { None })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberTypes;
impl SizedSolverRule for NumberTypes {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!("number type")
    }
}
impl<Split: SplitStrategy> SubtypeRule<Split> for NumberTypes {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, sub: &Term, sup: &Term) -> bool {
        if let Term::Symbol { uri: n1, .. } = sub
            && let Term::Symbol { uri: n2, .. } = sup
            && let Some(type1) = NumberRule::is_number(n1, checker)
            && let Some(type2) = NumberRule::is_number(n2, checker)
        {
            type1 <= type2
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: CheckRef<'t, '_, Split>,
        sub: &'t Term,
        sup: &'t Term,
    ) -> Option<bool> {
        let Term::Symbol { uri: n1, .. } = sub else {
            return None;
        };
        let Term::Symbol { uri: n2, .. } = sup else {
            return None;
        };
        let type1 = NumberRule::is_number(n1, &checker)?;
        let type2 = NumberRule::is_number(n2, &checker)?;
        checker.comment(format!("{} <= {}", type1.as_str(), type2.as_str()));
        // by applicability
        Some(true)
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for NumberTypes {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, term: &Term, tp: &Term) -> bool {
        if let Term::Number(n) = term
            && let Term::Symbol { uri, .. } = tp
            && let Some(typ) = NumberRule::is_number(uri, checker)
        {
            typ.contains(n)
        } else {
            false
        }
    }
    fn apply<'t>(&self, _: CheckRef<'t, '_, Split>, _: &'t Term, _: &'t Term) -> Option<bool> {
        // by applicability
        Some(true)
    }
}
impl<Split: SplitStrategy> InferenceRule<Split> for NumberTypes {
    fn applicable(&self, term: &Term) -> bool {
        matches!(term, Term::Number(_))
    }
    fn infer<'t>(&self, checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term> {
        macro_rules! get {
            ($typ:ident,$cond:expr) => {
                if $cond && let Some(uri) = NumberType::$typ.get(&checker) {
                    return Some(Term::Symbol {
                        uri: uri.clone(),
                        presentation: None,
                    });
                }
            };
        }
        let Term::Number(num) = term else {
            return None;
        };
        if let Some(n) = num.as_int() {
            get!(PositiveNaturals, n > 0);
            get!(Naturals, n >= 0);
            get!(NegativeIntegers, n < 0);
        }
        let f = num.as_float();
        get!(PositiveReals, f >= 0.0);
        get!(NegativeReals, f < 0.0);
        None
    }
}
