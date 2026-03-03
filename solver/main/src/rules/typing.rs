use crate::{
    CheckRef,
    rules::{PreparationRule, SizedSolverRule},
    split::SplitStrategy,
};
use ftml_ontology::terms::{
    Argument, ArgumentMode, BindingTerm, BoundArgument, ComponentVar, IsTerm, MaybeSequence, Term,
    Variable,
};
use ftml_uris::SymbolUri;
use std::{hint::unreachable_unchecked, ops::ControlFlow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleTypeOperatorRule(pub SymbolUri);

impl SizedSolverRule for SimpleTypeOperatorRule {
    fn priority(&self) -> isize {
        100_000
    }
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!(&self.0, " is a typing operator")
    }
}
impl std::fmt::Display for SimpleTypeOperatorRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is a type operator", self.0)
    }
}
impl SimpleTypeOperatorRule {
    fn is_app(&self, t: &Term) -> bool {
        //ftml_ontology::matchtm!(app({sym(=self.0)},[]) = t);
        matches!(
            t,
            Term::Application(app)
            if matches!(
                &app.head,
                Term::Symbol{uri,..} if *uri == self.0
            ) && (matches!(
                app.arguments.first(),
                Some(Argument::Simple(Term::Var{..}))
            ) || matches!(
                app.arguments.first(),
                Some(Argument::Sequence(MaybeSequence::Seq(s)))
                if s.iter().all(|t| matches!(t,Term::Var{..}))
            )) && matches!(app.arguments.get(1),Some(Argument::Simple(_)))
        )
    }
    /// Only safe to call iff `self.is_app(&t)`
    unsafe fn get_var(t: &Term) -> (Vec<Variable>, Term) {
        let Term::Application(a) = t else {
            unsafe { unreachable_unchecked() }
        };
        let Argument::Simple(tp) = a.arguments[1].clone() else {
            unsafe { unreachable_unchecked() }
        };
        (
            match a.arguments[0].clone() {
                Argument::Simple(Term::Var { variable, .. }) => vec![variable],
                Argument::Sequence(MaybeSequence::Seq(vars)) => vars
                    .into_iter()
                    .map(|v| {
                        let Term::Var { variable, .. } = v else {
                            unsafe { unreachable_unchecked() }
                        };
                        variable
                    })
                    .collect(),
                _ => unsafe { unreachable_unchecked() },
            },
            tp,
        )
    }
}
impl<Split: SplitStrategy> PreparationRule<Split> for SimpleTypeOperatorRule {
    fn applicable(&self, checker: &crate::CheckRef<'_, '_, Split>, t: &Term) -> bool {
        let Term::Bound(b) = t else {
            tracing::trace!("Not bound");
            return false;
        };
        let Some(head) = checker.get_head(t) else {
            return false;
        };
        let spec = head.as_ref().either(|s| &s.data.arity, |v| &v.data.arity);
        if spec.num() as usize != b.arguments.len() {
            tracing::trace!("Arguments don't match: {spec:?} != {:?}", b.arguments);
            return false;
        }
        spec.iter()
            .zip(b.arguments.iter())
            .any(|(a, b)| match (a, b) {
                (ArgumentMode::BoundVariable, BoundArgument::Simple(t)) => t
                    .head()
                    .is_some_and(|v| matches!(v,either::Left(uri) if *uri == self.0)),
                (
                    ArgumentMode::BoundVariableSequence,
                    BoundArgument::Sequence(MaybeSequence::One(t)),
                ) => t
                    .head()
                    .is_some_and(|v| matches!(v,either::Left(uri) if *uri == self.0)),
                (
                    ArgumentMode::BoundVariableSequence,
                    BoundArgument::Sequence(MaybeSequence::Seq(ts)),
                ) => ts.iter().any(|t| {
                    t.head()
                        .is_some_and(|v| matches!(v,either::Left(uri) if *uri == self.0))
                }),
                _ => false,
            })
    }
    /*
       fn make_bound<'t>(
           &self,
           _: crate::CheckRef<'t, '_, Split>,
           t: &BoundArgument,
       ) -> Option<BoundArgument> {
           match t {
               BoundArgument::Simple(t) => {
                   if self.is_app(&t) {
                       // SAFETY: is_app
                       let (mut v, tp) = unsafe { Self::get_var(&t) };
                       if v.len() == 1 {
                           // SAFETY: len==1
                           let var = unsafe { v.pop().unwrap_unchecked() };
                           return Some(BoundArgument::Bound(ComponentVar {
                               var,
                               tp: Some(tp),
                               df: None,
                           }));
                       }
                   }
                   None
               }
               BoundArgument::Sequence(MaybeSequence::Seq(s)) => {
                   let mut works = true;
                   let mut types = Vec::new();
                   let ns = s
                       .into_iter()
                       .flat_map(|t| {
                           if self.is_app(&t) {
                               // SAFETY: is_app
                               let (v, tp) = unsafe { Self::get_var(&t) };
                               for _ in &v {
                                   types.push(tp.clone());
                               }
                               v
                           } else {
                               works = false;
                               Vec::new()
                           }
                       })
                       .collect::<Vec<_>>();
                   if works {
                       return Some(BoundArgument::BoundSeq(MaybeSequence::Seq(
                           ns.into_iter()
                               .zip(types)
                               .map(|(var, tp)| ComponentVar {
                                   var,
                                   tp: Some(tp),
                                   df: None,
                               })
                               .collect(),
                       )));
                   }
                   None
               }
               _ => None,
           }
       }
    */
    fn apply(
        &self,
        checker: &mut CheckRef<'_, '_, Split>,
        t: Term,
        _: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ControlFlow<Term, Term> {
        let Some(head) = checker.get_head(&t) else {
            return ControlFlow::Continue(t);
        };
        let Term::Bound(b) = t else {
            unreachable!("wut");
        };
        let spec = head.as_ref().either(|s| &s.data.arity, |v| &v.data.arity);

        let nargs = spec
            .iter()
            .zip(b.arguments.clone())
            .map(|(m, a)| match (m, a) {
                (ArgumentMode::BoundVariable, BoundArgument::Simple(t)) => {
                    if self.is_app(&t) {
                        // SAFETY: is_app
                        let (mut v, tp) = unsafe { Self::get_var(&t) };
                        if v.len() == 1 {
                            // SAFETY: len==1
                            let var = unsafe { v.pop().unwrap_unchecked() };
                            BoundArgument::Bound(ComponentVar {
                                var,
                                tp: Some(tp),
                                df: None,
                            })
                        } else {
                            BoundArgument::Simple(t)
                        }
                    } else {
                        BoundArgument::Simple(t)
                    }
                }
                (
                    ArgumentMode::BoundVariableSequence,
                    BoundArgument::Sequence(MaybeSequence::Seq(s)),
                ) => {
                    let mut works = true;
                    let mut types = Vec::new();
                    let ns = s
                        .into_iter()
                        .flat_map(|t| {
                            if self.is_app(&t) {
                                // SAFETY: is_app
                                let (v, tp) = unsafe { Self::get_var(&t) };
                                for _ in &v {
                                    types.push(tp.clone());
                                }
                                v.into_iter()
                                    .map(|v| Term::Var {
                                        variable: v,
                                        presentation: None,
                                    })
                                    .collect::<Vec<_>>()
                            } else {
                                works = false;
                                vec![t]
                            }
                        })
                        .collect::<Vec<_>>();
                    if works {
                        BoundArgument::BoundSeq(MaybeSequence::Seq(
                            ns.into_iter()
                                .zip(types)
                                .map(|(t, tp)| {
                                    let Term::Var { variable, .. } = t else {
                                        // SAFETY: works == true
                                        unsafe { unreachable_unchecked() }
                                    };
                                    ComponentVar {
                                        var: variable,
                                        tp: Some(tp),
                                        df: None,
                                    }
                                })
                                .collect(),
                        ))
                    } else {
                        BoundArgument::Sequence(MaybeSequence::Seq(ns.into_boxed_slice()))
                    }
                }
                (m, a) => {
                    tracing::trace!("Other: {m:?} = {a:?}");
                    a
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        ControlFlow::Continue(Term::Bound(BindingTerm::new(
            b.head.clone(),
            nargs,
            b.presentation.clone(),
        )))
    }
    fn applicable_revert(&self, _: &CheckRef<'_, '_, Split>, _: &Term) -> bool {
        false
    }
    fn revert(&self, _: &CheckRef<'_, '_, Split>, t: Term) -> ControlFlow<Term, Term> {
        ControlFlow::Continue(t)
    }
}
