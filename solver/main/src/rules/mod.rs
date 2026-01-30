pub mod defaults;
pub mod extractors;
pub mod fixity;
pub mod pi;
pub mod typing;
pub mod universe;
pub use ftml_solver_trace::{CheckerRule, SizedSolverRule};

use crate::{CheckRef, split::SplitStrategy};
use ftml_ontology::{
    domain::declarations::symbols::Symbol, narrative::elements::VariableDeclaration, terms::Term,
};
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
    inference = InferenceRule(defaults::SeqIndexRule),
    subtyping = SubtypeRule,
    checking = CheckingRule,
    inhabitable = InhabitableRule,//(defaults::SeqInhabitableRule),
    equality = EqualityRule,
    universe = UniverseRule,
    preparation = PreparationRule,
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
    fn applicable(&self, term: &Term, tp: &Term) -> bool;
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
    fn applicable(&self, sub: &Term, sup: &Term) -> bool;
    fn apply<'t>(
        &self,
        checker: CheckRef<'t, '_, Split>,
        sub: &'t Term,
        sup: &'t Term,
    ) -> Option<bool>;
}

pub trait PreparationRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, t: &Term, head: either::Either<&Symbol, &VariableDeclaration>) -> bool;
    fn apply(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term>;
    fn applicable_revert(
        &self,
        t: &Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> bool;
    fn revert(
        &self,
        rules: &RuleSet<Split>,
        t: Term,
        head: either::Either<&Symbol, &VariableDeclaration>,
    ) -> ControlFlow<Term, Term>;
}
