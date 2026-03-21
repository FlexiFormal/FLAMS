use std::borrow::Cow;

use ftml_ontology::terms::{ApplicationTerm, Argument, ComponentVar, Term, helpers::IntoTerm};
use ftml_uris::SymbolUri;

use crate::{
    Checker,
    impls::solving::{Solutions, Solvable},
    split::SplitStrategy,
};

#[derive(Debug, Clone)]
pub struct HOASSymbols {
    pub judgment: Option<SymbolUri>,
    pub lambda: SymbolUri,
    pub pi: SymbolUri,
    pub apply: Option<SymbolUri>,
    //dummies: std::sync::atomic::AtomicUsize,
}
impl HOASSymbols {
    pub fn apply<'t, Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        head: &'t Term,
        arguments: impl Iterator<Item = Option<Term>>,
    ) -> (Cow<'t, Term>, Solutions) {
        let mut ret = Solutions::default();
        if let Some(app) = self.apply.as_ref() {
            (
                arguments.fold(Cow::Borrowed(head), |h, a| {
                    Cow::Owned(app.clone().apply_tms([
                        h.into_owned(),
                        a.unwrap_or_else(|| {
                            let name = checker.new_solvable();
                            ret.0.insert(Solvable {
                                name: name.clone(),
                                solution: crate::impls::solving::BoundedValue::None,
                                tp: crate::impls::solving::BoundedValue::None,
                            });
                            name.into()
                        }),
                    ]))
                }),
                ret,
            )
        } else {
            let args = arguments
                .map(|t| {
                    Argument::Simple(t.unwrap_or_else(|| {
                        let name = checker.new_solvable();
                        ret.0.insert(Solvable {
                            name: name.clone(),
                            solution: crate::impls::solving::BoundedValue::None,
                            tp: crate::impls::solving::BoundedValue::None,
                        });
                        name.into()
                    }))
                })
                .collect::<Box<[_]>>();
            (
                if args.is_empty() {
                    Cow::Borrowed(head)
                } else {
                    Cow::Owned(Term::Application(ApplicationTerm::new(
                        head.clone(),
                        args,
                        None,
                    )))
                },
                ret,
            )
        }
    }

    pub fn get<Split: SplitStrategy>(checker: &Checker<Split>) -> Option<Self> {
        let judgment = checker.rules.marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<super::rules::IsJudgmentRule>()
                .map(|rl| rl.0.clone())
        });
        let (lambda, pi, apply) = checker.rules.marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<super::rules::HOASRule>()
                .map(|rl| (rl.lambda.clone(), rl.pi.clone(), rl.apply.clone()))
        })?;
        Some(Self {
            judgment,
            lambda,
            pi,
            apply,
            //dummies: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    pub fn wrap_vars<'c>(
        &self,
        args: impl DoubleEndedIterator<Item = ComponentVar>,
        ret: &'c Term,
    ) -> Cow<'c, Term> {
        args.rev().fold(Cow::Borrowed(ret), |c, v| {
            //let premise = self.wrap_judg(p).into_owned();
            Cow::Owned(
                self.pi
                    .clone()
                    .simple_bind(v.var, v.tp, v.df, c.into_owned()),
            )
        })
    }

    pub fn wrap_types<'c>(&self, args: &[Term], ret: &'c Term) -> Cow<'c, Term> {
        let ret = self.wrap_judg(ret);
        args.iter().rev().fold(ret, |c, p| {
            let premise = self.wrap_judg(p).into_owned();
            Cow::Owned(self.pi.clone().simple_bind(
                crate::DUMMY.clone().into(),
                Some(premise),
                None,
                c.into_owned(),
            ))
        })
    }

    #[inline]
    pub fn let_in(var: ComponentVar, body: Term) -> Term {
        ftml_uris::metatheory::LET_IN
            .clone()
            .simple_bind(var.var, var.tp, var.df, body)
    }

    #[inline]
    pub fn lambda(&self, var: ComponentVar, body: Term) -> Term {
        self.lambda
            .clone()
            .simple_bind(var.var, var.tp, var.df, body)
    }

    #[inline]
    pub fn pi(&self, var: ComponentVar, body: Term) -> Term {
        self.pi.clone().simple_bind(var.var, var.tp, var.df, body)
    }

    pub fn wrap_judg<'c>(&self, ret: &'c Term) -> Cow<'c, Term> {
        self.judgment.as_ref().map_or_else(
            || Cow::Borrowed(ret),
            |j| {
                Cow::Owned(self.apply.as_ref().map_or_else(
                    || j.clone().apply_tms([ret.clone()]),
                    |app| app.clone().apply_tms([j.clone().into(), ret.clone()]),
                ))
            },
        )
    }
}
