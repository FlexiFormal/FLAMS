use std::borrow::Cow;

use ftml_ontology::terms::{
    ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence, Term,
};
use ftml_solver_trace::{CheckerRule, CheckingTask, traceref};

use crate::{
    CheckRef,
    impls::solving::{TermExtSolvable, is_solvable_var},
    rules::{
        implicits::{ImplicitExtApp, ImplicitExtBound},
        unknowns::{beta_unknowns, beta_unknowns_cow},
    },
    split::SplitStrategy,
};

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub fn simplify_full(&mut self, expand: bool, term: &'t Term) -> Option<Term> {
        if matches!(term, Term::Symbol { .. } | Term::Number(_)) {
            return None;
        }
        self./*wrap_check*/untraced(CheckingTask::Simplify(term), |slf| {
            slf.simplify_full_i(expand, term)
        }).inspect(|t| {
            self.add_msg(traceref!("Simplified: ",t.clone()).into());
        })
    }
    fn simplify_full_i(&mut self, expand: bool, term: &'t Term) -> Option<Term> {
        tracing::debug!(
            "Fully Simplifying {:?} (expand:{expand})",
            term.debug_short()
        );
        let mut current = if expand && let Some(t) = self.simplify_implicit(term) {
            Cow::Owned(t)
        } else
        /*if term.unapply_implicits().is_some() {
            return None;
        } else*/
        {
            match term {
                Term::Symbol { uri, .. } if expand => self.get_symbol_definiens(uri).map(|t| {
                    self.comment("expanded definition");
                    Cow::Owned(t)
                })?,
                Term::Var { variable, .. } => {
                    if let Some(name) = is_solvable_var(variable)
                        && let Some(t) = self.get_solution(name)
                    {
                        Cow::Owned(t)
                    } else if expand && let Some(df) = self.get_var_definiens(variable) {
                        Cow::Owned(df)
                    } else {
                        return None;
                    }
                }
                Term::Number(_) => return None,
                Term::Application(app) => {
                    let mut changed = false;
                    let nhead = self.simplify_full_i(true, &app.head).map_or(
                        Cow::Borrowed(&app.head),
                        |t| {
                            changed = true;
                            Cow::Owned(t)
                        },
                    );
                    let args = app
                        .arguments
                        .iter()
                        .map(|a| {
                            self.arg_full(expand, a).map_or(Cow::Borrowed(a), |a| {
                                changed = true;
                                Cow::Owned(a)
                            })
                        })
                        .collect::<Vec<_>>();
                    if changed {
                        Cow::Owned(Term::Application(ApplicationTerm::new(
                            nhead.into_owned(),
                            args.into_iter().map(Cow::into_owned).collect(),
                            app.presentation.clone(),
                        )))
                    } else {
                        Cow::Borrowed(term)
                    }
                }
                Term::Bound(app) => {
                    let mut changed = false;
                    let nhead = self.simplify_full_i(true, &app.head).map_or(
                        Cow::Borrowed(&app.head),
                        |t| {
                            changed = true;
                            Cow::Owned(t)
                        },
                    );
                    let args = app
                        .arguments
                        .iter()
                        .map(|a| {
                            self.bound_arg_full(expand, a)
                                .map_or(Cow::Borrowed(a), |a| {
                                    changed = true;
                                    Cow::Owned(a)
                                })
                        })
                        .collect::<Vec<_>>();
                    if changed {
                        Cow::Owned(Term::Bound(BindingTerm::new(
                            nhead.into_owned(),
                            args.into_iter().map(Cow::into_owned).collect(),
                            app.presentation.clone(),
                        )))
                    } else {
                        Cow::Borrowed(term)
                    }
                }
                _ => Cow::Borrowed(term),
            }
        };
        loop {
            if let Some(next) = self.scoped(|slf| slf.simplify_one(expand, &current)) {
                current = Cow::Owned(next);
            } else {
                return match current {
                    Cow::Borrowed(_) => None,
                    Cow::Owned(t) => Some(t),
                };
            }
        }
    }

    pub fn simplify_until(
        &mut self,
        term: &'t Term,
        mut until: impl FnMut(&Self, &Term) -> bool,
    ) -> Option<Cow<'t, Term>> {
        if until(self, term) {
            return Some(Cow::Borrowed(term));
        }
        self.wrap_check(CheckingTask::Simplify(term), |slf| {
            slf.simplify_until_i(term, until)
        })
    }
    fn simplify_until_i(
        &mut self,
        term: &'t Term,
        mut until: impl FnMut(&Self, &Term) -> bool,
    ) -> Option<Cow<'t, Term>> {
        let mut current = Cow::<'t, _>::Borrowed(term);
        loop {
            let Some(next) = self.scoped(|slf| slf.simplify_one(true, &current)) else {
                //self.comment(format!("Final simplification: {:?}", current.debug_short()));
                return None;
            };
            if until(self, &next) {
                return Some(Cow::Owned(next));
            }
            current = Cow::Owned(next);
        }
    }

    pub(crate) fn simplify_rules<Rl: CheckerRule + ?Sized, R>(
        &mut self,
        rules: &'t [Box<Rl>],
        term: &'t Term,
        applicable: impl Fn(&Rl, &Term) -> bool,
        apply: impl for<'s> Fn(CheckRef<'s, '_, Split>, &Rl, &'s Term) -> Option<R> + Send + Sync,
    ) -> Option<R>
    where
        R: Send + Sync + std::fmt::Debug + Clone + 'static,
    {
        let mut applicables = smallvec::SmallVec::<_, 2>::default();
        match self.simplify_until(term, |_, t| {
            applicables = rules
                .iter()
                .filter_map(|rl| {
                    if applicable(&**rl, t) {
                        Some(&**rl)
                    } else {
                        None
                    }
                })
                .collect();
            !applicables.is_empty()
        }) {
            Some(Cow::Borrowed(term)) => {
                if let Some(r) =
                    Split::split(self, true, applicables, |slf, rl| apply(slf, rl, term))
                {
                    return Some(r);
                }
                self.simplify_one(true, term).and_then(|term| {
                    self.scoped(|slf| slf.simplify_rules(rules, &term, applicable, apply))
                })
            }
            Some(Cow::Owned(term)) => self.scoped(|slf| {
                if let Some(r) =
                    Split::split(slf, true, applicables, |slf, rl| apply(slf, rl, &term))
                {
                    return Some(r);
                }
                slf.simplify_one(true, &term).and_then(|term| {
                    slf.scoped(|slf| slf.simplify_rules(rules, &term, applicable, apply))
                })
            }),
            None => {
                self.failure("No rule applicable");
                None
            }
        }
    }

    pub(crate) fn simplify_until_two(
        &mut self,
        term1: &'t Term,
        term2: &'t Term,
        mut until: impl FnMut(&Self, &Term, &Term) -> bool,
    ) -> Option<(Cow<'t, Term>, Cow<'t, Term>)> {
        let mut left = true;
        let mut right = true;
        let mut next_left = true;
        let mut t1 = Cow::Borrowed(term1);
        let mut t2 = Cow::Borrowed(term2);
        loop {
            if next_left && left {
                if until(self, &t1, &t2) {
                    return Some((t1, t2));
                }
                next_left = false;
                if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t1)) {
                    t1 = Cow::Owned(next);
                    continue;
                }
                left = false;
            }
            if right {
                if until(self, &t1, &t2) {
                    return Some((t1, t2));
                }
                next_left = true;
                if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t2)) {
                    t2 = Cow::Owned(next);
                    continue;
                }
                right = false;
                continue;
            }
            break;
        }
        self.add_msg(
            traceref!(FAIL
                "Final simplifications: ",
                t1.into_owned(),
                " and ",
                t2.into_owned()
            )
            .into(),
        );
        None
    }

    pub(crate) fn simplify_rules_two<Rl: CheckerRule + ?Sized + 'static, R>(
        &mut self,
        rules: &'t [Box<Rl>],
        term1: &'t Term,
        term2: &'t Term,
        applicable: impl Fn(&CheckRef<'_, '_, Split>, &Rl, &Term, &Term) -> bool,
        apply: impl for<'s> Fn(CheckRef<'s, '_, Split>, &Rl, &'s Term, &'s Term) -> Option<R> + Sync,
        abort: impl Fn(&Term, &Term) -> bool + Send + Sync,
    ) -> either::Either<Option<R>, (Term, Term)>
    where
        R: Send + Sync + std::fmt::Debug + Clone + 'static,
    {
        let mut applicables = smallvec::SmallVec::<_, 2>::default();
        let mut left = true;
        let mut right = true;
        let mut next_left = true;
        let mut t1 = Cow::Borrowed(term1);
        let mut t2 = Cow::Borrowed(term2);
        loop {
            macro_rules! set {
                () => {
                    if abort(&*t1, &*t2) {
                        return either::Right((t1.into_owned(), t2.into_owned()));
                    }
                    applicables = rules
                        .iter()
                        .filter_map(|rl| {
                            if applicable(self, &**rl, &*t1, &*t2) {
                                Some(&**rl)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !applicables.is_empty() {
                        break;
                    }
                };
            }
            loop {
                if next_left && left {
                    set!();
                    next_left = false;
                    if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t1)) {
                        t1 = Cow::Owned(next);
                        continue;
                    }
                    left = false;
                }
                if right {
                    set!();
                    next_left = true;
                    if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t2)) {
                        t2 = Cow::Owned(next);
                        continue;
                    }
                    right = false;
                    continue;
                }
                break;
            }
            if applicables.is_empty() {
                self.failure("No rule applicable");
                return either::Left(None);
            }
            if let Some(r) = self.scoped(|slf| {
                Split::split(slf, true, std::mem::take(&mut applicables), |slf, rl| {
                    apply(slf, rl, &t1, &t2)
                })
            }) {
                return either::Left(Some(r));
            }
            if next_left && left {
                next_left = false;
                if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t1)) {
                    t1 = Cow::Owned(next);
                    set!();
                    continue;
                }
            }
            if right {
                next_left = true;
                if let Some(next) = self.scoped(|slf| slf.simplify_one(true, &t2)) {
                    t2 = Cow::Owned(next);
                    set!();
                    continue;
                }
                right = false;
                if left {
                    continue;
                }
            }
            break;
        }
        self.failure("No rule applicable");
        either::Left(None)
    }

    fn simplify_one(&mut self, expand: bool, term: &'t Term) -> Option<Term> {
        if term.has_solvable() {
            let nterm = self.subst(term.clone());
            if *term != nterm {
                return Some(nterm); //self.scoped(|slf| slf.simplify_one_i(expand, &nterm));
            }
        }
        if let Cow::Owned(nterm) = beta_unknowns_cow(term)
            && *term != nterm
        {
            return Some(nterm); //self.scoped(|slf| slf.simplify_one_i(expand, &nterm));
        }
        self.simplify_one_i(expand, term)
    }
    fn simplify_one_i(&mut self, expand: bool, term: &'t Term) -> Option<Term> {
        /**/
        let applicables = self
            .top
            .rules
            .simplification()
            .iter()
            .filter_map(|rl| {
                if rl.applicable(term) {
                    Some(&**rl)
                } else {
                    None
                }
            })
            .collect::<smallvec::SmallVec<_, 2>>();
        match Split::split(self, false, applicables, |slf, rl| {
            match rl.apply(slf, term) {
                Ok(t) => Some(either::Left(t)),
                Err(Some(v)) => Some(either::Right(v)),
                _ => None,
            }
        }) {
            Some(either::Left(t)) => return Some(t),
            Some(_) => {
                // TODO
            }
            None => (),
        }

        self.simplify_one_default(expand, term)
    }

    pub(crate) fn simplify_implicit(&mut self, term: &'t Term) -> Option<Term> {
        if let Some((Term::Symbol { uri, .. }, args)) = term.unapply_implicits()
            && let Some(def) = self.get_symbol_definiens(uri)
            && let Some((def, vars)) = def.get_bound_implicits()
            && args.len() == vars.len()
        {
            let mut substs = Vec::new();
            for (ComponentVar { var, tp, .. }, arg) in vars.iter().zip(args) {
                if let Some(tp) = tp {
                    let tp: Cow<Term> = tp / &*substs;
                    if self.scoped(|checker| checker.check_type(arg, &tp)) != Some(true) {
                        return None;
                    }
                    substs.push((var.name(), arg));
                }
            }
            let r = def / &*substs;
            tracing::debug!("Unapplied implicits: {:?}", r.debug_short());
            Some(r.into_owned())
        } else {
            None
        }
    }

    fn simplify_one_default(&mut self, expand: bool, term: &'t Term) -> Option<Term> {
        if expand && let Some(t) = self.simplify_implicit(term) {
            return Some(t);
        }
        match term {
            // Definition Expansion
            Term::Symbol { uri, .. } if expand => self.get_symbol_definiens(uri).inspect(|_| {
                self.comment("expanded definition");
            }),
            Term::Var { variable, .. } if expand || is_solvable_var(variable).is_some() => {
                self.get_var_definiens(variable).inspect(|_| {
                    self.comment("expanded definition");
                })
            }
            Term::Application(app) => self.simplify_one(expand, &app.head).map(|nh| {
                Term::Application(ApplicationTerm::new(
                    nh,
                    app.arguments.clone(),
                    app.presentation.clone(),
                ))
            }),
            Term::Bound(app) => self.simplify_one(true, &app.head).map(|nh| {
                Term::Bound(BindingTerm::new(
                    nh,
                    app.arguments.clone(),
                    app.presentation.clone(),
                ))
            }),
            _ => None,
        }
    }

    fn arg_full(&mut self, expand: bool, arg: &'t Argument) -> Option<Argument> {
        match arg {
            Argument::Simple(t) => self.simplify_full_i(expand, t).map(Argument::Simple),
            Argument::Sequence(MaybeSequence::One(t)) => self
                .simplify_full_i(expand, t)
                .map(|t| Argument::Sequence(MaybeSequence::One(t))),
            Argument::Sequence(MaybeSequence::Seq(ts)) => {
                let mut changed = false;
                let nts = ts
                    .iter()
                    .map(|t| {
                        self.simplify_full_i(expand, t)
                            .map_or(Cow::Borrowed(t), |a| {
                                changed = true;
                                Cow::Owned(a)
                            })
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Some(Argument::Sequence(MaybeSequence::Seq(
                        nts.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    None
                }
            }
        }
    }
    fn bound_arg_full(&mut self, expand: bool, arg: &'t BoundArgument) -> Option<BoundArgument> {
        match arg {
            BoundArgument::Simple(t) => self.simplify_full_i(expand, t).map(BoundArgument::Simple),
            BoundArgument::Bound(cv) => self.cv_full(expand, cv).map(BoundArgument::Bound),
            BoundArgument::Sequence(MaybeSequence::One(t)) => self
                .simplify_full_i(expand, t)
                .map(|t| BoundArgument::Sequence(MaybeSequence::One(t))),
            BoundArgument::BoundSeq(MaybeSequence::One(cv)) => self
                .cv_full(expand, cv)
                .map(|t| BoundArgument::BoundSeq(MaybeSequence::One(t))),
            BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                let mut changed = false;
                let nts = ts
                    .iter()
                    .map(|t| {
                        self.simplify_full_i(expand, t)
                            .map_or(Cow::Borrowed(t), |a| {
                                changed = true;
                                Cow::Owned(a)
                            })
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Some(BoundArgument::Sequence(MaybeSequence::Seq(
                        nts.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    None
                }
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(cvs)) => {
                let mut changed = false;
                let ncvs = cvs
                    .iter()
                    .map(|t| {
                        self.cv_full(expand, t).map_or(Cow::Borrowed(t), |a| {
                            changed = true;
                            Cow::Owned(a)
                        })
                    })
                    .collect::<Vec<_>>();
                if changed {
                    Some(BoundArgument::BoundSeq(MaybeSequence::Seq(
                        ncvs.into_iter().map(Cow::into_owned).collect(),
                    )))
                } else {
                    None
                }
            }
        }
    }

    fn cv_full(&mut self, expand: bool, arg: &'t ComponentVar) -> Option<ComponentVar> {
        match (arg.tp.as_ref(), arg.df.as_ref()) {
            (None, None) => None,
            (Some(tp), None) => self.simplify_full_i(expand, tp).map(|tp| ComponentVar {
                var: arg.var.clone(),
                tp: Some(tp),
                df: None,
            }),
            (None, Some(df)) => self.simplify_full_i(expand, df).map(|df| ComponentVar {
                var: arg.var.clone(),
                tp: None,
                df: Some(df),
            }),
            (Some(tp), Some(df)) => {
                let ntp = self.simplify_full_i(expand, tp);
                let ndf = self.simplify_full_i(expand, df);
                if ntp.is_none() && ndf.is_none() {
                    return None;
                }
                Some(ComponentVar {
                    var: arg.var.clone(),
                    tp: Some(ntp.unwrap_or_else(|| tp.clone())),
                    df: Some(ndf.unwrap_or_else(|| df.clone())),
                })
            }
        }
    }
}
