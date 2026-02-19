use crate::{CheckRef, split::SplitStrategy};
use either::Either;
use ftml_ontology::{
    domain::{SharedDeclaration, declarations::symbols::Symbol},
    narrative::{SharedDocumentElement, elements::VariableDeclaration},
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, IsTerm, MaybeSequence,
        Term, Variable, termpaths::TermPath,
    },
};
use smallvec::SmallVec;
use std::{hint::unreachable_unchecked, mem::MaybeUninit};

impl<Split: SplitStrategy> CheckRef<'_, '_, Split> {
    #[inline]
    #[must_use]
    pub const fn rules(&self) -> &crate::rules::RuleSet<Split> {
        &self.top.rules
    }

    pub(crate) fn prepare(&self, t: Term, path: Option<&mut TermPath>) -> Term {
        tracing::trace!("preparing {:?}", t.debug_short());
        let mut cp = self.copied();
        let mut ncp = cp.get_ref();
        let old = std::mem::replace(ncp.context.0, SmallVec::new());
        let r = ncp.prepare_i(t, path.map(|p| (p.inner_mut(), 0)));
        *ncp.context.0 = old;
        r
    }

    pub(crate) fn revert_prepare(&self, t: Term) -> Term {
        tracing::trace!("reverting preparation {:?}", t.debug_short());
        let mut cp = self.copied();
        let mut ncp = cp.get_ref();
        let old = std::mem::replace(ncp.context.0, SmallVec::new());
        let r = ncp.revert_i(t);
        *ncp.context.0 = old;
        r
    }

    pub(crate) fn bind_implicits(&mut self, nt: Term) -> Term {
        tracing::trace!("Binding implicits for {:?}", nt.debug_short());
        let allvars = nt
            .free_variables()
            .into_iter()
            .cloned()
            .collect::<SmallVec<_, 4>>();
        if allvars.is_empty() {
            return nt;
        }
        tracing::trace!("All variables: {allvars:?}");

        let mut ctx = smallvec::SmallVec::<_, 4>::new();
        for v in allvars {
            if !ctx.iter().any(|(var, _)| *var == v) {
                let name = self.new_solvable();
                let Variable::Name { name: id, .. } = &name else {
                    // SAFETY: new_solvable always returns Variable::Name
                    unsafe { unreachable_unchecked() }
                };
                let tp = self.infer_var_type_i(&v);
                if let Some(tp) = tp {
                    self.solve_type(id.clone(), tp / ctx.as_slice());
                }

                ctx.push((
                    v,
                    Term::Var {
                        variable: name,
                        presentation: None,
                    },
                ));
            }
        }
        if ctx.is_empty() {
            return nt;
        }
        tracing::trace!("New context: {ctx:?}");
        let n = nt / ctx.as_slice();
        tracing::trace!("Implicitified: {:?}", n.debug_short());
        n
    }

    pub fn get_head(
        &self,
        t: &Term,
    ) -> Option<Either<SharedDeclaration<Symbol>, SharedDocumentElement<VariableDeclaration>>> {
        let head = t.head()?;
        Some(match head {
            Either::Left(uri) => Either::Left(self.get_symbol(uri).ok()?),
            Either::Right(Variable::Ref { declaration, .. }) => {
                Either::Right(self.get_variable(declaration).ok()?)
            }
            either::Right(Variable::Name { .. }) => return None,
        })
    }

    fn prepare_i(
        &mut self,
        t: Term,
        mut path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> Term {
        match &t {
            Term::Symbol { .. } | Term::Var { .. } => return t,
            _ => (),
        }

        // this may very much be overkill, but it's nice to do things without cloning,
        // and terms are composed of potentially multiple Arcs :)
        let mut t = MaybeUninit::new(t);

        let rules = self.top.rules.preparation();

        for rl in self.top.rules.preparation() {
            // SAFETY: not yet replaced
            if !rl.applicable(self, unsafe { t.assume_init_ref() }) {
                continue;
            }
            // SAFETY: not yet replaced
            let tm = unsafe { t.assume_init_read() };
            let path = match &mut path {
                Some((p, t)) => Some((&mut **p, *t)),
                _ => None,
            };
            match rl.apply(self, tm, path) {
                //                                 MaybeUninit doesn't drop the inner value,
                //                                 vvvvvvvvv  so this is fine
                std::ops::ControlFlow::Break(t) => return t,
                std::ops::ControlFlow::Continue(tm) => {
                    // t is initialized again
                    t.write(tm);
                }
            }
            tracing::trace!(
                "Rule {rl:?} applied; result: {:?}",
                unsafe { t.assume_init_ref() }.debug_short()
            );
        }
        // SAFETY: t has been restored or we've returned early anyway
        self.prepare_recurse(
            unsafe { t.assume_init() },
            |s, t, p| s.prepare_i(t, p),
            path,
        )
    }

    fn revert_i(&mut self, t: Term) -> Term {
        match &t {
            Term::Symbol { .. } | Term::Var { .. } => return t,
            _ => (),
        }

        // this may very much be overkill, but it's nice to do things without cloning,
        // and terms are composed of potentially multiple Arcs :)
        let mut t = MaybeUninit::new(t);

        for rl in self.top.rules.preparation().iter().rev() {
            // SAFETY: not yet replaced
            if !rl.applicable_revert(self, unsafe { t.assume_init_ref() }) {
                continue;
            }
            // SAFETY: not yet replaced
            let tm = unsafe { t.assume_init_read() };
            match rl.revert(self, tm) {
                //                                 MaybeUninit doesn't drop the inner value,
                //                                 vvvvvvvvv  so this is fine
                std::ops::ControlFlow::Break(t) => return t,
                std::ops::ControlFlow::Continue(tm) => {
                    // t is initialized again
                    t.write(tm);
                }
            }
            tracing::trace!(
                "Rule {rl:?} applied; result: {:?}",
                unsafe { t.assume_init_ref() }.debug_short()
            );
        }
        // SAFETY: t has been restored or we've returned early anyway
        self.prepare_recurse(unsafe { t.assume_init() }, |s, t, _| s.revert_i(t), None)
    }

    fn prepare_recurse(
        &mut self,
        term: Term,
        then: fn(
            &mut CheckRef<'_, '_, Split>,
            Term,
            Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
        ) -> Term,
        mut path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> Term {
        match term {
            Term::Application(a) => Term::Application(ApplicationTerm::new(
                then(self, a.head.clone(), get_path(&mut path, 0)),
                {
                    let mut idx = 0;
                    a.arguments
                        .iter()
                        .map(|arg| match arg {
                            Argument::Simple(t) => Argument::Simple(then(
                                self,
                                t.clone(),
                                get_path(&mut path, {
                                    idx += 1;
                                    idx
                                }),
                            )),
                            Argument::Sequence(MaybeSequence::One(t)) => {
                                Argument::Sequence(MaybeSequence::One(then(
                                    self,
                                    t.clone(),
                                    get_path(&mut path, {
                                        idx += 1;
                                        idx
                                    }),
                                )))
                            }
                            Argument::Sequence(MaybeSequence::Seq(ts)) => {
                                idx += 1;
                                let mut npath = get_path(&mut path, idx);
                                Argument::Sequence(MaybeSequence::Seq(
                                    ts.iter()
                                        .cloned()
                                        .enumerate()
                                        .map(|(i, t)| then(self, t, get_path(&mut npath, i)))
                                        .collect(),
                                ))
                            }
                        })
                        .collect()
                },
                a.presentation.clone(),
            )),
            Term::Bound(b) => Term::Bound(BindingTerm::new(
                then(self, b.head.clone(), get_path(&mut path, 0)),
                self.scoped(|slf| {
                    let mut idx = 0;
                    b.arguments
                        .iter()
                        .map(|arg| match arg {
                            BoundArgument::Simple(t) => BoundArgument::Simple(then(
                                slf,
                                t.clone(),
                                get_path(&mut path, {
                                    idx += 1;
                                    idx
                                }),
                            )),
                            BoundArgument::Sequence(MaybeSequence::One(t)) => {
                                BoundArgument::Sequence(MaybeSequence::One(then(
                                    slf,
                                    t.clone(),
                                    get_path(&mut path, {
                                        idx += 1;
                                        idx
                                    }),
                                )))
                            }
                            BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                                idx += 1;
                                let mut npath = get_path(&mut path, idx);
                                BoundArgument::Sequence(MaybeSequence::Seq(
                                    ts.iter()
                                        .cloned()
                                        .enumerate()
                                        .map(|(i, t)| then(slf, t, get_path(&mut npath, i)))
                                        .collect(),
                                ))
                            }
                            BoundArgument::Bound(cv) => BoundArgument::Bound(slf.prepare_cv(
                                cv,
                                then,
                                get_path(&mut path, {
                                    idx += 1;
                                    idx
                                }),
                            )),
                            BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                                BoundArgument::BoundSeq(MaybeSequence::One(slf.prepare_cv(
                                    cv,
                                    then,
                                    get_path(&mut path, {
                                        idx += 1;
                                        idx
                                    }),
                                )))
                            }
                            BoundArgument::BoundSeq(MaybeSequence::Seq(vars)) => {
                                idx += 1;
                                let mut npath = get_path(&mut path, idx);
                                BoundArgument::BoundSeq(MaybeSequence::Seq(
                                    vars.iter()
                                        .enumerate()
                                        .map(|(i, cv)| {
                                            slf.prepare_cv(cv, then, get_path(&mut npath, i))
                                        })
                                        .collect(),
                                ))
                            }
                        })
                        .collect()
                }),
                b.presentation.clone(),
            )),
            t => t,
        }
    }

    fn prepare_cv(
        &mut self,
        cv: &ComponentVar,
        then: fn(&mut Self, Term, Option<(&mut smallvec::SmallVec<u8, 16>, usize)>) -> Term,
        mut path: Option<(&mut smallvec::SmallVec<u8, 16>, usize)>,
    ) -> ComponentVar {
        let mut next = 0;
        let tp = match &cv.tp {
            Some(t) => {
                next += 1;
                Some(then(self, t.clone(), get_path(&mut path, 0)))
            }
            None => {
                let r = self.infer_var_type_i(&cv.var);
                if r.is_some()
                    && let Some((p, i)) = &mut path
                    && let Some(i) = p.get_mut(*i)
                {
                    *i += 1;
                    next += 1;
                }
                r
            }
        };
        let df = match &cv.df {
            Some(t) => Some(then(self, t.clone(), get_path(&mut path, next))),
            None => {
                let r = self.get_var_definiens(&cv.var);
                if r.is_some()
                    && let Some((p, i)) = &mut path
                    && let Some(i) = p.get_mut(*i)
                {
                    *i += 1;
                    next += 1;
                }
                r
            }
        };
        let cv = ComponentVar {
            var: cv.var.clone(),
            tp,
            df,
        };
        self.extend_context(cv.clone());
        cv
    }
}

fn get_path<'a, 'b>(
    op: &'a mut Option<(&'b mut smallvec::SmallVec<u8, 16>, usize)>,
    idx: usize,
) -> Option<(&'a mut smallvec::SmallVec<u8, 16>, usize)> {
    op.as_mut().and_then(|(s, i)| {
        if s.get(*i).copied() == Some(idx as u8) {
            Some((&mut **s, *i + 1))
        } else {
            None
        }
    })
}
