use std::{borrow::Cow, fmt::Write};

use crate::{
    TermExtSeq,
    impls::{equality::Alpha, solving::TermExtSolvable},
};
use ftml_ontology::terms::{Argument, BoundArgument, ComponentVar, MaybeSequence, Term, Variable};
use ftml_uris::Id;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub vars: Box<[Id]>,
    pub body: Term,
    allow_references: bool,
}
impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pattern{{{:?} => {:?}}}",
            self.vars,
            self.body.debug_short()
        )
    }
}
impl Pattern {
    #[must_use]
    pub fn from(t: Term, allow_references: bool) -> Self {
        //let mut substs = Vec::<(String, Term)>::new();
        let vars = t
            .free_variables()
            .into_iter()
            .filter_map(|v| {
                if allow_references {
                    Some(v.name_id().into_owned())
                }
                /*else if v.is_solvable().is_some() {
                    let fv = t.fresh_variable(&crate::DUMMY, None);
                    substs.push((v.name().to_string(), fv.0.clone().into()));
                    Some(fv.0.name_id().into_owned())
                }*/
                else if let Variable::Name { name, .. } = v {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Self {
            vars,
            body: t, // / &*substs,
            allow_references,
        }
    }
    #[must_use]
    pub fn matches<'t>(&self, term: &'t Term) -> Option<Vec<Cow<'t, Term>>> {
        let mut vars = vec![None; self.vars.len()];
        let mut alpha = Alpha::new();
        self.match_rec(term, &self.body, &mut vars, &mut alpha)?;
        let mut terms = Vec::with_capacity(vars.len());
        for v in vars {
            terms.push(v?);
        }
        Some(terms)
    }

    #[allow(clippy::option_if_let_else, clippy::too_many_lines)]
    fn match_rec<'t: 's, 's>(
        &self,
        term: &'t Term,
        body: &'s Term,
        vars: &mut [Option<Cow<'t, Term>>],
        alpha: &mut Alpha<'s>,
    ) -> Option<()> {
        match (term, body) {
            (
                _,
                Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                },
            ) if self.vars.contains(name) => {
                let idx = self.vars.iter().position(|n| *n == *name)?;
                if let Some(t) = &vars[idx] {
                    if crate::impls::equality::alpha_equal(term, t) {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    vars[idx] = Some(Cow::Borrowed(term));
                    Some(())
                }
            }
            (
                _,
                Term::Var {
                    variable: Variable::Ref { declaration, .. },
                    ..
                },
            ) if self.allow_references
                && self
                    .vars
                    .iter()
                    .any(|v| v.as_ref() == declaration.name().last()) =>
            {
                let idx = self
                    .vars
                    .iter()
                    .position(|n| n.as_ref() == declaration.name().last())?;
                if let Some(t) = &vars[idx] {
                    if crate::impls::equality::alpha_equal(term, t) {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    vars[idx] = Some(Cow::Borrowed(term));
                    Some(())
                }
            }
            (Term::Symbol { .. }, Term::Symbol { .. }) => {
                if term == body {
                    Some(())
                } else {
                    None
                }
            }
            (Term::Var { variable: v1, .. }, Term::Var { variable: v2, .. }) => {
                if v1.name() == v2.name()
                    || alpha.iter().any(|(a, b)| {
                        (*a == v1.name() && b.name() == v2.name())
                            || (b.name() == v1.name() && *a == v2.name())
                    })
                {
                    Some(())
                } else {
                    None
                }
            }
            (Term::Application(a), Term::Application(b))
                if a.arguments.len() == b.arguments.len() =>
            {
                self.match_rec(&a.head, &b.head, vars, alpha)?;
                for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
                    self.match_args(a, b, vars, alpha)?;
                }
                Some(())
            }
            (Term::Bound(a), Term::Bound(b)) if a.arguments.len() == b.arguments.len() => {
                self.match_rec(&a.head, &b.head, vars, alpha)?;
                let mut acc = 0;
                for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
                    acc += self.match_bargs(a, b, vars, alpha)?;
                }
                for _ in 0..acc {
                    alpha.pop();
                }
                Some(())
            }
            (Term::Field(a), Term::Field(b)) => {
                if a.key != b.key {
                    return None;
                }
                self.match_rec(&a.record, &b.record, vars, alpha)
            }
            (
                Term::Label {
                    name: na,
                    df: da,
                    tp: ta,
                },
                Term::Label {
                    name: nb,
                    df: db,
                    tp: tb,
                },
            ) if *na == *nb => {
                match (da, db) {
                    (Some(a), Some(b)) => {
                        self.match_rec(a, b, vars, alpha)?;
                    }
                    (None, None) => (),
                    _ => return None,
                }
                match (ta, tb) {
                    (Some(a), Some(b)) => self.match_rec(a, b, vars, alpha),
                    (None, None) => Some(()),
                    _ => None,
                }
            }
            (Term::Number(a), Term::Number(b)) if a == b => Some(()),
            _ => None,
            //_ => todo!(),
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn match_args<'t: 's, 's>(
        &self,
        term: &'t Argument,
        body: &'s Argument,
        vars: &mut [Option<Cow<'t, Term>>],
        alpha: &mut Alpha<'s>,
    ) -> Option<()> {
        match (term, body) {
            (Argument::Simple(a), Argument::Simple(b))
            | (
                Argument::Sequence(MaybeSequence::One(a)),
                Argument::Sequence(MaybeSequence::One(b)),
            ) => self.match_rec(a, b, vars, alpha),
            (
                Argument::Sequence(MaybeSequence::Seq(a)),
                Argument::Sequence(MaybeSequence::Seq(b)),
            ) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b.iter()) {
                    self.match_rec(a, b, vars, alpha)?;
                }
                Some(())
            }
            (
                Argument::Sequence(MaybeSequence::Seq(a)),
                Argument::Sequence(MaybeSequence::One(Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                })),
            ) if self.vars.contains(name) => {
                let idx = self.vars.iter().position(|n| *n == *name)?;
                if let Some(t) = &vars[idx] {
                    if let Some(seq) = t.as_sequence()
                        && seq.len() == a.len()
                        && seq
                            .iter()
                            .zip(a.iter())
                            .all(|(a, b)| crate::impls::equality::alpha_equal(a, b))
                    {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    vars[idx] = Some(Cow::Owned(Term::into_seq(a.iter().cloned())));
                    Some(())
                }
            }
            _ => None,
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn match_bargs<'t: 's, 's>(
        &self,
        term: &'t BoundArgument,
        body: &'s BoundArgument,
        vars: &mut [Option<Cow<'t, Term>>],
        alpha: &mut Alpha<'s>,
    ) -> Option<usize> {
        match (term, body) {
            (BoundArgument::Simple(a), BoundArgument::Simple(b)) => {
                self.match_rec(a, b, vars, alpha).map(|()| 0)
            }
            (
                BoundArgument::Sequence(MaybeSequence::One(a)),
                BoundArgument::Sequence(MaybeSequence::One(b)),
            ) => self.match_rec(a, b, vars, alpha).map(|()| 0),
            (
                BoundArgument::Sequence(MaybeSequence::Seq(a)),
                BoundArgument::Sequence(MaybeSequence::Seq(b)),
            ) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b.iter()) {
                    self.match_rec(a, b, vars, alpha)?;
                }
                Some(0)
            }
            (
                BoundArgument::Sequence(MaybeSequence::Seq(a)),
                BoundArgument::Sequence(MaybeSequence::One(Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                })),
            ) if self.vars.contains(name) => {
                let idx = self.vars.iter().position(|n| *n == *name)?;
                if let Some(t) = &vars[idx] {
                    if let Some(seq) = t.as_sequence()
                        && seq.len() == a.len()
                        && seq
                            .iter()
                            .zip(a.iter())
                            .all(|(a, b)| crate::impls::equality::alpha_equal(a, b))
                    {
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    vars[idx] = Some(Cow::Owned(Term::into_seq(a.iter().cloned())));
                    Some(0)
                }
            }
            (BoundArgument::Bound(lhs), BoundArgument::Bound(rhs))
            | (
                BoundArgument::BoundSeq(MaybeSequence::One(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::One(rhs)),
            ) => {
                self.match_cv(lhs, rhs, vars, alpha)?;
                Some(1)
            }
            (
                BoundArgument::BoundSeq(MaybeSequence::Seq(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::Seq(rhs)),
            ) if lhs.len() == rhs.len() => {
                if lhs
                    .iter()
                    .zip(rhs.iter())
                    .all(|(a, b)| self.match_cv(a, b, vars, alpha).is_some())
                {
                    Some(lhs.len())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn match_cv<'t: 's, 's>(
        &self,
        term: &'t ComponentVar,
        body: &'s ComponentVar,
        vars: &mut [Option<Cow<'t, Term>>],
        alpha: &mut Alpha<'s>,
    ) -> Option<()> {
        match (term.tp.as_ref(), body.tp.as_ref()) {
            (Some(a), Some(b)) => {
                self.match_rec(a, b, vars, alpha)?;
            }
            (None, None) => (),
            _ => return None,
        }
        match (term.df.as_ref(), body.df.as_ref()) {
            (Some(a), Some(b)) => {
                self.match_rec(a, b, vars, alpha)?;
            }
            (None, None) => (),
            _ => return None,
        }
        alpha.push((term.var.name(), &body.var));
        Some(())
    }
}
