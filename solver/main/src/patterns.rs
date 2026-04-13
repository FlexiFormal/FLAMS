use std::borrow::Cow;

use crate::impls::equality::Alpha;
use ftml_ontology::terms::{
    Argument, BoundArgument, ComponentVar, MaybeSequence, Term, Variable, sequences::Sequence,
};
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

    pub fn from_with_vars(t: &Term, allow_references: bool, vars: &mut Vec<Id>) {
        for v in t.free_variables() {
            if vars.iter().any(|v2| v2.as_ref() == v.name()) {
                continue;
            }
            if allow_references {
                vars.push(v.name_id().into_owned());
            } else if let Variable::Name { name, .. } = v {
                vars.push(name.clone());
            }
        }
    }

    #[must_use]
    pub fn matches<'t>(&self, term: &'t Term) -> Option<Vec<Cow<'t, Term>>> {
        let vars = Self::r#match(term, &self.body, &self.vars, self.allow_references)?;
        let mut terms = Vec::with_capacity(vars.len());
        for v in vars {
            terms.push(v?);
        }
        Some(terms)
    }

    #[must_use]
    pub(crate) fn r#match<'t: 's, 's>(
        term: &'t Term,
        pattern: &'s Term,
        variables: &[Id],
        allow_references: bool,
    ) -> Option<Vec<Option<Cow<'t, Term>>>> {
        let mut vars: Vec<Option<Cow<'t, _>>> = vec![None; variables.len()];
        let mut alpha = Alpha::new();
        Self::match_i(
            term,
            pattern,
            variables,
            allow_references,
            &mut vars,
            &mut alpha,
        )?;
        Some(vars)
    }

    #[must_use]
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn match_i<'t: 's, 's>(
        term: &'t Term,
        pattern: &'s Term,
        variables: &[Id],
        allow_references: bool,
        vars: &mut [Option<Cow<'t, Term>>],
        alpha: &mut Alpha<'s>,
    ) -> Option<()> {
        Self::match_rec(
            term,
            pattern,
            variables,
            allow_references,
            alpha,
            &mut |v, t| {
                let idx = variables.iter().position(|n| *n == *v)?;
                if let Some(ot) = &vars[idx] {
                    match t {
                        either::Left(t) => {
                            if crate::impls::equality::alpha_equal(ot, t) {
                                Some(())
                            } else {
                                None
                            }
                        }
                        either::Right(ts) => {
                            if let Some(Sequence::SequenceExpression(seq)) = ot.as_sequence()
                                && seq.len() == ts.len()
                                && seq
                                    .iter()
                                    .zip(ts.iter())
                                    .all(|(a, b)| crate::impls::equality::alpha_equal(a, b))
                            {
                                Some(())
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    vars[idx] = Some(match t {
                        either::Left(t) => Cow::Borrowed(t),
                        either::Right(ts) => Cow::Owned(Term::into_seq(ts.iter().cloned())),
                    });
                    Some(())
                }
            },
        )
    }

    #[allow(clippy::option_if_let_else, clippy::too_many_lines)]
    pub(crate) fn match_rec<'t: 's, 's>(
        //&self,
        term: &'t Term,
        pattern: &'s Term,
        variables: &[Id],
        allow_references: bool,
        alpha: &mut Alpha<'s>,
        add: &mut dyn FnMut(&Id, either::Either<&'t Term, &'t [Term]>) -> Option<()>,
        //vars: &mut [Option<Cow<'t, Term>>],
    ) -> Option<()> {
        match (term, pattern) {
            (
                _,
                Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                },
            ) if variables.contains(name) => add(name, either::Left(term)),
            (
                _,
                Term::Var {
                    variable: Variable::Ref { declaration, .. },
                    ..
                },
            ) if allow_references
                && variables
                    .iter()
                    .any(|v| v.as_ref() == declaration.name().last()) =>
            {
                let id = variables
                    .iter()
                    .find(|v| v.as_ref() == declaration.name().last())?;
                add(id, either::Left(term))
            }
            (Term::Symbol { .. }, Term::Symbol { .. }) => {
                if term == pattern {
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
                Self::match_rec(&a.head, &b.head, variables, allow_references, alpha, add)?;
                for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
                    Self::match_args(a, b, variables, allow_references, alpha, add)?;
                }
                Some(())
            }
            (Term::Bound(a), Term::Bound(b)) if a.arguments.len() == b.arguments.len() => {
                Self::match_rec(&a.head, &b.head, variables, allow_references, alpha, add)?;
                let mut acc = 0;
                for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
                    acc += Self::match_bargs(a, b, variables, allow_references, alpha, add)?;
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
                Self::match_rec(
                    &a.record,
                    &b.record,
                    variables,
                    allow_references,
                    alpha,
                    add,
                )
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
                        Self::match_rec(a, b, variables, allow_references, alpha, add)?;
                    }
                    (None, None) => (),
                    _ => return None,
                }
                match (ta, tb) {
                    (Some(a), Some(b)) => {
                        Self::match_rec(a, b, variables, allow_references, alpha, add)
                    }
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
        //&self,
        term: &'t Argument,
        body: &'s Argument,
        variables: &[Id],
        allow_references: bool,
        alpha: &mut Alpha<'s>,
        add: &mut dyn FnMut(&Id, either::Either<&'t Term, &'t [Term]>) -> Option<()>,
    ) -> Option<()> {
        match (term, body) {
            (Argument::Simple(a), Argument::Simple(b))
            | (
                Argument::Sequence(MaybeSequence::One(a)),
                Argument::Sequence(MaybeSequence::One(b)),
            ) => Self::match_rec(a, b, variables, allow_references, alpha, add),
            (
                Argument::Sequence(MaybeSequence::Seq(a)),
                Argument::Sequence(MaybeSequence::Seq(b)),
            ) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b.iter()) {
                    Self::match_rec(a, b, variables, allow_references, alpha, add)?;
                }
                Some(())
            }
            (
                Argument::Sequence(MaybeSequence::Seq(a)),
                Argument::Sequence(MaybeSequence::One(Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                })),
            ) if variables.contains(name) => add(name, either::Right(a)),
            _ => None,
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn match_bargs<'t: 's, 's>(
        //&self,
        term: &'t BoundArgument,
        body: &'s BoundArgument,
        variables: &[Id],
        allow_references: bool,
        alpha: &mut Alpha<'s>,
        add: &mut dyn FnMut(&Id, either::Either<&'t Term, &'t [Term]>) -> Option<()>,
    ) -> Option<usize> {
        match (term, body) {
            (BoundArgument::Simple(a), BoundArgument::Simple(b)) => {
                Self::match_rec(a, b, variables, allow_references, alpha, add).map(|()| 0)
            }
            (
                BoundArgument::Sequence(MaybeSequence::One(a)),
                BoundArgument::Sequence(MaybeSequence::One(b)),
            ) => Self::match_rec(a, b, variables, allow_references, alpha, add).map(|()| 0),
            (
                BoundArgument::Sequence(MaybeSequence::Seq(a)),
                BoundArgument::Sequence(MaybeSequence::Seq(b)),
            ) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b.iter()) {
                    Self::match_rec(a, b, variables, allow_references, alpha, add)?;
                }
                Some(0)
            }
            (
                BoundArgument::Sequence(MaybeSequence::Seq(a)),
                BoundArgument::Sequence(MaybeSequence::One(Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                })),
            ) if variables.contains(name) => add(name, either::Right(a)).map(|()| 0),
            (BoundArgument::Bound(lhs), BoundArgument::Bound(rhs))
            | (
                BoundArgument::BoundSeq(MaybeSequence::One(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::One(rhs)),
            ) => {
                Self::match_cv(lhs, rhs, variables, allow_references, alpha, add)?;
                Some(1)
            }
            (
                BoundArgument::BoundSeq(MaybeSequence::Seq(lhs)),
                BoundArgument::BoundSeq(MaybeSequence::Seq(rhs)),
            ) if lhs.len() == rhs.len() => {
                if lhs.iter().zip(rhs.iter()).all(|(a, b)| {
                    Self::match_cv(a, b, variables, allow_references, alpha, add).is_some()
                }) {
                    Some(lhs.len())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn match_cv<'t: 's, 's>(
        //&self,
        term: &'t ComponentVar,
        body: &'s ComponentVar,
        variables: &[Id],
        allow_references: bool,
        alpha: &mut Alpha<'s>,
        add: &mut dyn FnMut(&Id, either::Either<&'t Term, &'t [Term]>) -> Option<()>,
    ) -> Option<()> {
        match (term.tp.as_ref(), body.tp.as_ref()) {
            (Some(a), Some(b)) => {
                Self::match_rec(a, b, variables, allow_references, alpha, add)?;
            }
            (None, None) => (),
            _ => return None,
        }
        match (term.df.as_ref(), body.df.as_ref()) {
            (Some(a), Some(b)) => {
                Self::match_rec(a, b, variables, allow_references, alpha, add)?;
            }
            (None, None) => (),
            _ => return None,
        }
        alpha.push((term.var.name(), &body.var));
        Some(())
    }
}
