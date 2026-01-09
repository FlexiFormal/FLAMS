pub mod defaults;
pub mod extractors;
pub mod fixity;
pub mod pi;
pub mod typing;
pub mod universe;

use std::{fmt::Debug, ops::ControlFlow};

use crate::{SolverRef, SolverTrace, context::Context, split::SplitStrategy, trace::RefCheckLog};
use ftml_ontology::{
    domain::declarations::symbols::Symbol, narrative::elements::VariableDeclaration, terms::Term,
};

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

pub trait CheckerRule: std::fmt::Display + std::fmt::Debug + Send + Sync + std::any::Any {
    fn priority(&self) -> isize {
        0
    }
    fn as_box_dyn(&self) -> Box<dyn CheckerRule>;
    fn as_dyn(&self) -> &dyn CheckerRule;
    fn as_any(&self) -> &dyn std::any::Any;
    fn eq(&self, o: &dyn CheckerRule) -> bool;
    //fn display(self: Box<Self>) -> RefCheckLog<'static>;
}

pub trait SizedSolverRule:
    std::fmt::Display + std::fmt::Debug + Send + Sync + std::any::Any + Clone + Sized + PartialEq + Eq
{
    fn priority(&self) -> isize {
        0
    }
    //fn display(self: Box<Self>) -> RefCheckLog<'static>;
}
impl<T: SizedSolverRule> CheckerRule for T {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn priority(&self) -> isize {
        <Self as SizedSolverRule>::priority(self)
    }
    /*
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn display(self: Box<Self>) -> RefCheckLog<'static> {
        <Self as SizedSolverRule>::display(self)
    }
     */

    fn as_box_dyn(&self) -> Box<dyn CheckerRule> {
        Box::new(self.clone()) as _
    }
    fn as_dyn(&self) -> &dyn CheckerRule {
        self as _
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
    fn eq(&self, o: &dyn CheckerRule) -> bool {
        o.as_any().downcast_ref::<T>().is_some_and(|v| v == self)
    }
}

pub trait EqualityRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, lhs: &Term, rhs: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        lhs: &'t Term,
        rhs: &'t Term,
    ) -> Option<bool>;
}

pub trait InferenceRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn infer<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<Term>;
}

pub trait CheckingRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term, tp: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        term: &'t Term,
        tp: &'t Term,
    ) -> Option<bool>;
}

pub trait InhabitableRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<bool>;
}

pub trait UniverseRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, term: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
        term: &'t Term,
    ) -> Option<bool>;
}

pub trait SubtypeRule<Split: SplitStrategy>: CheckerRule {
    fn applicable(&self, sub: &Term, sup: &Term) -> bool;
    fn apply<'t>(
        &self,
        solver: SolverRef<Split>,
        trace: &mut SolverTrace,
        context: Context<'t, '_>,
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
}
