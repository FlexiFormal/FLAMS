use ftml_ontology::terms::{Argument, Numeric, Term, helpers::IntoTerm};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::{Id, SymbolUri};

use crate::{
    CheckRef,
    rules::{
        CheckingRule, EqualityRule, InferenceRule, MarkerRule, ProofRule, SimplificationRule,
        SubtypeRule,
    },
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
    Complex,
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
            Self::Complex => "complex numbers",
        }
    }
    pub fn contains(self, num: &Numeric) -> bool {
        match self {
            Self::Complex => true,
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
            (_, Self::Complex) => Some(Less),
            (Self::Complex, _) => Some(Greater),
            //(_, Self::Reals) => Some(Less),
            //(Self::Reals, _) => Some(Greater),
            (
                Self::PositiveNaturals,
                Self::NegativeIntegers | Self::NegativeRationals | Self::NegativeReals,
            ) => None,
            (Self::PositiveNaturals, _) => Some(Less),
            (Self::Naturals, Self::PositiveNaturals) => Some(Greater),
            (
                Self::Naturals,
                Self::Integers
                | Self::Rationals
                | Self::PositiveRationals
                | Self::PositiveReals
                | Self::Reals,
            ) => Some(Less),
            (
                Self::NegativeIntegers,
                Self::Integers
                | Self::NonZeroIntegers
                | Self::Rationals
                | Self::NegativeRationals
                | Self::NonZeroRationals
                | Self::NegativeReals
                | Self::NonZeroReals
                | Self::Reals,
            ) => Some(Less),
            (Self::Integers, Self::PositiveNaturals | Self::Naturals) => Some(Greater),
            (Self::Integers, Self::Rationals | Self::PositiveReals | Self::Reals) => Some(Less),
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
            (Self::PositiveRationals, Self::PositiveReals | Self::Reals) => Some(Less),
            (Self::NegativeRationals, Self::NegativeIntegers | Self::NonZeroIntegers) => {
                Some(Greater)
            }
            (
                Self::NegativeRationals,
                Self::NonZeroRationals | Self::NegativeReals | Self::NonZeroReals | Self::Reals,
            ) => Some(Less),
            (
                Self::NonZeroRationals,
                Self::PositiveNaturals | Self::NegativeIntegers | Self::NonZeroIntegers,
            ) => Some(Greater),
            (Self::NonZeroRationals, Self::Rationals | Self::NonZeroReals | Self::Reals) => {
                Some(Less)
            }
            (
                Self::PositiveReals,
                Self::PositiveNaturals | Self::Naturals | Self::PositiveRationals,
            ) => Some(Greater),
            (Self::PositiveReals, Self::NonZeroReals | Self::Reals) => Some(Less),
            (Self::NegativeReals, Self::NegativeIntegers | Self::NegativeRationals) => {
                Some(Greater)
            }
            (Self::NegativeReals, Self::NonZeroReals | Self::Reals) => Some(Less),
            (Self::NonZeroReals, Self::Reals) => Some(Less),
            (Self::Reals, o) if o != Self::Complex => Some(Greater),
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
    pub fn is_number_term<Split: SplitStrategy>(
        term: &Term,
        checker: &CheckRef<'_, '_, Split>,
    ) -> Option<NumberType> {
        if let Term::Symbol { uri, .. } = term {
            Self::is_number_sym(uri, checker)
        } else {
            None
        }
    }
    pub fn is_number_sym<Split: SplitStrategy>(
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
            && let Some(type1) = NumberRule::is_number_sym(n1, checker)
            && let Some(type2) = NumberRule::is_number_sym(n2, checker)
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
        let type1 = NumberRule::is_number_sym(n1, &checker)?;
        let type2 = NumberRule::is_number_sym(n2, &checker)?;
        checker.comment(format!("{} <= {}", type1.as_str(), type2.as_str()));
        // by applicability
        Some(true)
    }
}
impl<Split: SplitStrategy> CheckingRule<Split> for NumberTypes {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, term: &Term, tp: &Term) -> bool {
        if let Term::Number(n) = term
            && let Term::Symbol { uri, .. } = tp
            && let Some(typ) = NumberRule::is_number_sym(uri, checker)
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

impl<Split: SplitStrategy> EqualityRule<Split> for NumberTypes {
    fn applicable(&self, lhs: &Term, rhs: &Term) -> bool {
        matches!(lhs, Term::Number(_)) && matches!(rhs, Term::Number(_))
    }
    fn apply<'t>(&self, _: CheckRef<'t, '_, Split>, lhs: &'t Term, rhs: &'t Term) -> Option<bool> {
        let (Term::Number(lhs), Term::Number(rhs)) = (lhs, rhs) else {
            return None;
        };
        Some(lhs == rhs)
    }
}

// -------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Max(pub SymbolUri);
impl SizedSolverRule for Max {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is maximum")
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for Max {
    fn applicable(&self, term: &Term) -> bool {
        let Term::Application(app) = term else {
            return false;
        };
        if app.head.is(&self.0)
            && let [
                Argument::Simple(Term::Number(_)),
                Argument::Simple(Term::Number(_)),
            ] = &*app.arguments
        {
            true
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        _: CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Application(app) = term else {
            return Err(None);
        };
        if let [
            Argument::Simple(Term::Number(a)),
            Argument::Simple(Term::Number(b)),
        ] = &*app.arguments
        {
            let a = a.as_float();
            let b = b.as_float();
            Ok(Term::Number(Numeric::Float(a.max(b).into())))
        } else {
            Err(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessThan(pub SymbolUri);
impl SizedSolverRule for LessThan {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is <=")
    }
}
impl<Split: SplitStrategy> ProofRule<Split> for LessThan {
    fn applicable(&self, term: &Term) -> bool {
        let Term::Application(app) = term else {
            return false;
        };
        if app.head.is(&self.0)
            && let [
                Argument::Simple(Term::Number(a)),
                Argument::Simple(Term::Number(b)),
            ] = &*app.arguments
        {
            true
        } else {
            false
        }
    }
    fn prove<'t>(&self, _: CheckRef<'t, '_, Split>, goal: &'t Term) -> Option<Term> {
        let Term::Application(app) = goal else {
            return None;
        };
        let [
            Argument::Simple(Term::Number(a)),
            Argument::Simple(Term::Number(b)),
        ] = &*app.arguments
        else {
            return None;
        };

        if a.as_float() <= b.as_float() {
            Some(ftml_uris::metatheory::AUTO_PROVE.clone().into())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logarithm(pub SymbolUri);
impl SizedSolverRule for Logarithm {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is logarithm")
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for Logarithm {
    fn applicable(&self, term: &Term) -> bool {
        let Term::Application(app) = term else {
            return false;
        };
        if app.head.is(&self.0)
            && let [
                Argument::Simple(Term::Number(b)),
                Argument::Simple(Term::Number(x)),
            ] = &*app.arguments
        {
            let b = b.as_float();
            let x = x.as_float();
            b > 0.0 && x > 0.0 && b != 1.0
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        _: CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Application(app) = term else {
            return Err(None);
        };
        if let [
            Argument::Simple(Term::Number(b)),
            Argument::Simple(Term::Number(x)),
        ] = &*app.arguments
        {
            let b = b.as_float();
            let x = x.as_float();
            let r = if b == 2.0 {
                x.log2()
            } else if b == 10.0 {
                x.log10()
            } else if b == std::f64::consts::E {
                x.ln()
            } else {
                x.log(b)
            };
            Ok(Term::Number(Numeric::Float(r.into())))
        } else {
            Err(None)
        }
    }
}

fn applicable(uri: &SymbolUri, term: &Term, unit: f64) -> bool {
    let Term::Application(app) = term else {
        return false;
    };
    app.head.is(uri)
        && (matches!(
            &*app.arguments,
            [
                Argument::Simple(Term::Number(_)),
                Argument::Simple(Term::Number(_))
            ] | [Argument::Sequence(_)]
        ) || matches!(
        &*app.arguments,
        [
            Argument::Simple(Term::Number(n)),
            Argument::Simple(_)
        ]
        if n.as_float() == unit
        ) || matches!(
        &*app.arguments,
        [
            Argument::Simple(_),
            Argument::Simple(Term::Number(n)),
        ]
        if n.as_float() == unit
        ))
}

macro_rules! arith {
    ($($name:ident $trace:literal $unit:literal = ($a:ident,$b:ident => $op:expr))*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $name(pub SymbolUri);
            impl SizedSolverRule for $name {
                fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
                    ftml_solver_trace::trace!(&self.0, $trace)
                }
            }
            impl<Split: SplitStrategy> SimplificationRule<Split> for $name {
                fn applicable(&self, term: &Term) -> bool {
                    applicable(&self.0,term,$unit)
                }
                fn apply<'t>(
                    &self,
                    _: CheckRef<'t, '_, Split>,
                    term: &'t Term,
                ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
                    let Term::Application(app) = term else {
                        return Err(None);
                    };
                    match &*app.arguments {
                        [Argument::Simple(Term::Number(z)), Argument::Simple(o)] if z.as_float() == $unit => {
                            Ok(o.clone())
                        }
                        [Argument::Simple(o), Argument::Simple(Term::Number(z))] if z.as_float() == $unit => {
                            Ok(o.clone())
                        }
                        [
                            Argument::Simple(Term::Number($a)),
                            Argument::Simple(Term::Number($b)),
                        ] => ($op).map_or(Err(None), |r| Ok(Term::Number(r))),
                        [Argument::Sequence(seq)] => Ok(super::super::sequences::fold::Fold::apply_init(
                            seq.clone(),
                            Term::Number(Numeric::Int(0)),
                            |x, y| self.0.clone().apply_tms([y.into(), x.into()]),
                        )),
                        _ => Err(None),
                    }
                }
            }
        )*
    };
}
arith! {
    AdditionRule " is addition" 0.0 = (a,b => *a + *b)
    SubtractionRule " is subtraction" 0.0 = (a,b => *a - *b)
    MultiplicationRule " is multiplication" 1.0 = (a,b => *a * *b)
    DivisionRule " is division" 1.0 = (a,b => *a / *b)
    ExponentiationRule " is exponentiation" 1.0 = (a,b => *a ^ *b)
}

/*
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdditionRule(pub SymbolUri);
impl SizedSolverRule for AdditionRule {
    fn display(&self) -> Vec<ftml_solver_trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is addition")
    }
}
impl<Split: SplitStrategy> SimplificationRule<Split> for AdditionRule {
    fn applicable(&self, term: &Term) -> bool {
        let Term::Application(app) = term else {
            return false;
        };
        app.head.is(&self.0)
            && (matches!(
                &*app.arguments,
                [
                    Argument::Simple(Term::Number(_)),
                    Argument::Simple(Term::Number(_))
                ] | [Argument::Sequence(_)]
            ) || matches!(
            &*app.arguments,
            [
                Argument::Simple(Term::Number(n)),
                Argument::Simple(_)
            ]
            if n.as_float() == 0.0
            ) || matches!(
            &*app.arguments,
            [
                Argument::Simple(_),
                Argument::Simple(Term::Number(n)),
            ]
            if n.as_float() == 0.0
            ))
    }
    fn apply<'t>(
        &self,
        _: CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<ftml_ontology::terms::termpaths::TermPath>> {
        let Term::Application(app) = term else {
            return Err(None);
        };
        match &*app.arguments {
            [Argument::Simple(Term::Number(z)), Argument::Simple(o)] if z.as_float() == 0.0 => {
                Ok(o.clone())
            }
            [Argument::Simple(o), Argument::Simple(Term::Number(z))] if z.as_float() == 0.0 => {
                Ok(o.clone())
            }
            [
                Argument::Simple(Term::Number(a)),
                Argument::Simple(Term::Number(b)),
            ] => (*a + *b).map_or(Err(None), |r| Ok(Term::Number(r))),
            [Argument::Sequence(seq)] => Ok(super::super::sequences::fold::Fold::apply_init(
                seq.clone(),
                Term::Number(Numeric::Int(0)),
                |x, y| self.0.clone().apply_tms([y.into(), x.into()]),
            )),
            _ => Err(None),
        }
    }
}
 */
