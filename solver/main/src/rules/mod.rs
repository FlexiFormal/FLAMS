pub mod extractors;
pub mod fixity;
pub mod implicits;
pub mod operators;
pub mod sequences;
pub use ftml_solver_trace::{CheckerRule, SizedSolverRule};
use ftml_uris::SymbolUri;

use crate::{CheckRef, split::SplitStrategy};
use ftml_ontology::terms::{Term, termpaths::TermPath};
use std::{fmt::Debug, ops::ControlFlow};

macro_rules! rules{
    ($($name:ident = $tp:ident $(($($e:expr),*))? ),*$(,)?) => {
        #[derive(Debug)]
        pub struct RuleSet<Split: SplitStrategy> {
            $(
                $name:Vec<Box<dyn $tp<Split>>>
            ),*
        }

        paste::paste!{
            $(
            trait [< ClonableDyn $tp>]<Split:SplitStrategy>:$tp<Split> {
                fn [<to_dyn_ $name>](&self) -> Box<dyn $tp<Split>>;
            }
            impl<Split:SplitStrategy,T:$tp<Split>+Sized+Clone> [< ClonableDyn $tp>]<Split> for T {
                fn [<to_dyn_ $name>](&self) -> Box<dyn $tp<Split>> {
                    Box::new(self.clone()) as _
                }
            }
            )*
        }


        impl<Split:SplitStrategy> Default for RuleSet<Split> {
            paste::paste!{
                fn default() -> Self {
                    Self {
                        $(
                            $name: Self::[<DEFAULT_ $name:snake:upper >].iter().map(|r| r.[<to_dyn_ $name>]()).collect()
                        ),*
                    }
                }
            }
        }

        impl<Split: SplitStrategy> RuleSet<Split> {
            paste::paste!{
                $(

                    const [<DEFAULT_ $name:snake:upper >] : &[&dyn [< ClonableDyn $tp>]<Split>] = &[
                        $($(
                            &$e as _
                        ),*)?
                    ];

                    pub fn [<push_ $name>](&mut self,rule:Box<dyn $tp<Split>>) {
                        if !self.$name.iter().any(|v| v.eq(&*rule)) {
                            let i = self
                                .$name
                                .binary_search_by_key(&(-rule.priority()), |e| -e.priority())
                                .map_or_else(|i| i, |i| i);
                            self.$name.insert(i, rule);
                        }
                    }
                    #[inline]
                    #[must_use]
                    pub fn $name(&self) -> &[Box<dyn $tp<Split>>] {
                        &self.$name
                    }
                )*
            }
        }
    }
}

rules! {
    inference = InferenceRule(
        sequences::SeqIndexRule,
        sequences::SeqInferenceRule,
        operators::numbers::NumberTypes,
        implicits::ImplicitRule
    ),
    subtyping = SubtypeRule(operators::numbers::NumberTypes),
    checking = CheckingRule(operators::numbers::NumberTypes),
    inhabitable = InhabitableRule(sequences::SeqUniverseRule),
    equality = EqualityRule,
    universe = UniverseRule(sequences::SeqUniverseRule),
    preparation = PreparationRule,
    simplification = SimplificationRule,
    marker = MarkerRule,
    proof = ProofRule
}

pub trait SimplificationRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn apply<'t>(
        &self,
        checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
    ) -> Result<Term, Option<TermPath>>;
}

pub trait EqualityRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, lhs: &Term, rhs: &Term) -> bool;
    fn apply<'t>(
        &self,
        checker: CheckRef<'t, '_, Split>,
        lhs: &'t Term,
        rhs: &'t Term,
    ) -> Option<bool>;
}

pub trait InferenceRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn infer<'t>(&self, checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<Term>;
}

pub trait CheckingRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, term: &Term, tp: &Term) -> bool;
    fn apply<'t>(
        &self,
        checker: CheckRef<'t, '_, Split>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool>;
}

pub trait InhabitableRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn apply<'t>(&self, checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool>;
}

pub trait UniverseRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn apply<'t>(&self, checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool>;
}

pub trait SubtypeRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, sub: &Term, sup: &Term) -> bool;
    fn apply<'t>(
        &self,
        checker: CheckRef<'t, '_, Split>,
        sub: &'t Term,
        sup: &'t Term,
    ) -> Option<bool>;
}

pub trait PreparationRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, checker: &CheckRef<'_, '_, Split>, t: &Term) -> bool;
    fn apply(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        t: Term,
        path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term>;
    fn applicable_revert(&self, checker: &CheckRef<'_, '_, Split>, t: &Term) -> bool;
    fn revert(&self, checker: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term>;
}
pub trait MarkerRule<Split: SplitStrategy>: CheckerRule {}

pub trait ProofRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn prove<'t>(&self, checker: CheckRef<'t, '_, Split>, goal: &'t Term) -> Option<Term>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsJudgmentRule(pub SymbolUri);
impl SizedSolverRule for IsJudgmentRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, "is a judgment")
    }
}
impl<Split: SplitStrategy> MarkerRule<Split> for IsJudgmentRule {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HOASRule {
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
    pub apply: Option<SymbolUri>,
}
impl SizedSolverRule for HOASRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("HOAS using ", &self.lambda, " and ", &self.pi)
    }
}
impl<Split: SplitStrategy> MarkerRule<Split> for HOASRule {}
