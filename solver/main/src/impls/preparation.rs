use crate::{CheckRef, split::SplitStrategy};
use either::Either;
use ftml_ontology::{
    domain::{SharedDeclaration, declarations::symbols::Symbol},
    narrative::{SharedDocumentElement, elements::VariableDeclaration},
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, IsTerm, MaybeSequence,
        Term, Variable,
    },
};
use smallvec::SmallVec;
use std::{hint::unreachable_unchecked, mem::MaybeUninit};

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub(crate) fn prepare(&self, t: Term) -> Term {
        tracing::trace!("preparing {:?}", t.debug_short());
        let mut cp = self.copied();
        let mut ncp = cp.get_ref();
        let old = std::mem::replace(ncp.context.0, SmallVec::new());
        let r = ncp.prepare_i(t);
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

    fn get_head(
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

    fn prepare_i(&mut self, t: Term) -> Term {
        match &t {
            Term::Symbol { .. } | Term::Var { .. } => return t,
            _ => (),
        }
        let Some(head) = self.get_head(&t) else {
            return t;
        };
        let head = head.as_ref().map_either(|v| &**v, |v| &**v);
        tracing::trace!("Head: {:?}", head);

        // this may very much be overkill, but it's nice to do things without cloning,
        // and terms are composed of potentially multiple Arcs :)
        let mut t = MaybeUninit::new(t);

        let rules = self.top.rules.preparation();
        tracing::trace!("Rules: {rules:#?}");

        for rl in self.top.rules.preparation() {
            tracing::trace!("Rule {rl:?}?");
            // SAFETY: not yet replaced
            if !rl.applicable(unsafe { t.assume_init_ref() }, head) {
                continue;
            }
            // SAFETY: not yet replaced
            let tm = unsafe { t.assume_init_read() };
            match rl.apply(&self.top.rules, tm, head) {
                //                                 MaybeUninit doesn't drop the inner value,
                //                                 vvvvvvvvv  so this is fine
                std::ops::ControlFlow::Break(t) => return t,
                std::ops::ControlFlow::Continue(tm) => {
                    // t is initialized again
                    t.write(tm);
                }
            }
        }
        // SAFETY: t has been restored or we've returned early anyway
        self.prepare_recurse(unsafe { t.assume_init() }, |s, t| s.prepare_i(t))
    }

    fn revert_i(&mut self, t: Term) -> Term {
        match &t {
            Term::Symbol { .. } | Term::Var { .. } => return t,
            _ => (),
        }
        let Some(head) = self.get_head(&t) else {
            return t;
        };
        let head = head.as_ref().map_either(|v| &**v, |v| &**v);
        tracing::trace!("Head: {:?}", head);

        // this may very much be overkill, but it's nice to do things without cloning,
        // and terms are composed of potentially multiple Arcs :)
        let mut t = MaybeUninit::new(t);

        let rules = self.top.rules.preparation();
        tracing::trace!("Rules: {rules:#?}");

        for rl in self.top.rules.preparation().iter().rev() {
            tracing::trace!("Rule {rl:?}?");
            // SAFETY: not yet replaced
            if !rl.applicable_revert(unsafe { t.assume_init_ref() }, head) {
                continue;
            }
            // SAFETY: not yet replaced
            let tm = unsafe { t.assume_init_read() };
            match rl.revert(&self.top.rules, tm, head) {
                //                                 MaybeUninit doesn't drop the inner value,
                //                                 vvvvvvvvv  so this is fine
                std::ops::ControlFlow::Break(t) => return t,
                std::ops::ControlFlow::Continue(tm) => {
                    // t is initialized again
                    t.write(tm);
                }
            }
        }
        // SAFETY: t has been restored or we've returned early anyway
        self.prepare_recurse(unsafe { t.assume_init() }, |s, t| s.revert_i(t))
    }

    fn prepare_recurse(
        &mut self,
        term: Term,
        then: fn(&mut CheckRef<'_, '_, Split>, Term) -> Term,
    ) -> Term {
        match term {
            Term::Application(a) => Term::Application(ApplicationTerm::new(
                then(self, a.head.clone()),
                a.arguments
                    .iter()
                    .map(|arg| match arg {
                        Argument::Simple(t) => Argument::Simple(then(self, t.clone())),
                        Argument::Sequence(MaybeSequence::One(t)) => {
                            Argument::Sequence(MaybeSequence::One(then(self, t.clone())))
                        }
                        Argument::Sequence(MaybeSequence::Seq(ts)) => Argument::Sequence(
                            MaybeSequence::Seq(ts.iter().cloned().map(|t| then(self, t)).collect()),
                        ),
                    })
                    .collect(),
                a.presentation.clone(),
            )),
            Term::Bound(b) => Term::Bound(BindingTerm::new(
                then(self, b.head.clone()),
                self.scoped(|slf| {
                    b.arguments
                        .iter()
                        .map(|arg| match arg {
                            BoundArgument::Simple(t) => BoundArgument::Simple(then(slf, t.clone())),
                            BoundArgument::Sequence(MaybeSequence::One(t)) => {
                                BoundArgument::Sequence(MaybeSequence::One(then(slf, t.clone())))
                            }
                            BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                                BoundArgument::Sequence(MaybeSequence::Seq(
                                    ts.iter().cloned().map(|t| then(slf, t)).collect(),
                                ))
                            }
                            BoundArgument::Bound(cv) => {
                                BoundArgument::Bound(slf.prepare_cv(cv, then))
                            }
                            BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                                BoundArgument::BoundSeq(MaybeSequence::One(
                                    slf.prepare_cv(cv, then),
                                ))
                            }
                            BoundArgument::BoundSeq(MaybeSequence::Seq(vars)) => {
                                BoundArgument::BoundSeq(MaybeSequence::Seq(
                                    vars.iter().map(|cv| slf.prepare_cv(cv, then)).collect(),
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

    fn prepare_cv(&mut self, cv: &ComponentVar, then: fn(&mut Self, Term) -> Term) -> ComponentVar {
        let tp = match &cv.tp {
            Some(t) => Some(then(self, t.clone())),
            None => self.infer_var_type_i(&cv.var),
        };
        let df = match &cv.df {
            Some(t) => Some(then(self, t.clone())),
            None => self.get_var_definiens(&cv.var),
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
