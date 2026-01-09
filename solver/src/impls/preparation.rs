use std::{hint::unreachable_unchecked, mem::MaybeUninit};

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

use crate::{
    SolverRef,
    context::Context,
    split::SplitStrategy,
    trace::{CheckingTask, SolverTrace},
};

impl<Split: SplitStrategy> SolverRef<'_, Split> {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn prepare(self, t: Term) -> Term {
        tracing::trace!("preparing {:?}", t.debug_short());
        let mut ctx = Context::new_top();
        self.prepare_i(ctx.build(), t)
    }

    pub(crate) fn bind_implicits(self, nt: Term) -> Term {
        let allvars = nt
            .free_variables()
            .into_iter()
            .cloned()
            .collect::<SmallVec<_, 4>>();
        if allvars.is_empty() {
            return nt;
        }
        tracing::trace!("All variables: {allvars:?}");

        let mut ctp = Context::new_top();
        let mut ctx = smallvec::SmallVec::<_, 4>::new();
        for v in allvars {
            if !ctx.iter().any(|(var, _)| *var == v) {
                let mut trace = SolverTrace::new(CheckingTask::VariableInference(v.name()));
                let name = self.new_solvable();
                let Variable::Name { name: id, .. } = &name else {
                    // SAFETY: new_solvable always returns Variable::Name
                    unsafe { unreachable_unchecked() }
                };
                let tp = self.infer_var_type_i(&mut trace, &ctp.build(), &v);
                if let Some(tp) = tp {
                    self.state.solve_type(id.clone(), tp / ctx.as_slice());
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
        /*
        let mut allvars = nt
            .all_variables()
            .into_iter()
            .map(|(v, t)| (v.clone(), t, None))
            .collect::<SmallVec<_, 4>>();
        tracing::warn!("All variables: {allvars:?}");
        if allvars.is_empty() {
            return nt;
        }

        let mut curr = allvars.len() - 1;
        let mut ctp = Context::new_top();
        loop {
            let (v, _, otp) = &mut allvars[curr];
            if otp.is_some() {
                if curr == 0 {
                    break;
                }
                curr -= 1;
                continue;
            }
            let mut trace = SolverTrace::new(CheckingTask::VarInfer(v));
            if let Some(tp) = self.infer_var_type_i(&mut trace, &ctp.build(), v) {
                *otp = Some(tp.clone());
                let mut nfv = tp
                    .free_variables()
                    .into_iter()
                    .map(|v| (v.clone(), FreeOrBound::Free, None))
                    .collect::<SmallVec<_, 4>>();
                curr += nfv.len();
                nfv.append(&mut allvars);
                allvars = nfv;
            }
            if curr == 0 {
                break;
            }
            curr -= 1;
        }
        let mut dedup = SmallVec::<_, 4>::new();
        for (v, f, tp) in allvars {
            let p = (v, tp);
            if f == FreeOrBound::Free && !dedup.contains(&p) {
                dedup.push(p);
            }
        }
        if dedup.is_empty() {
            return nt;
        }
        //tracing::warn!("Free variables: {dedup:?}");
        let mut ctx = smallvec::SmallVec::<_, 4>::new();
        for (v, tp) in dedup {
            // TODO store types of implicits somewhere
            let tp = tp.map_or_else(
                || Term::Var {
                    variable: self.new_solvable(),
                    presentation: None,
                },
                |tp| tp / ctx.as_slice(),
            );
            ctx.push((
                v,
                Term::Var {
                    variable: self.new_solvable(),
                    presentation: None,
                },
            ));
        }
        /*
        let nt = dedup.into_iter().fold(nt, |t, (v, tp)| {
            Term::Bound(BindingTerm::new(
                Term::Symbol {
                    uri: ftml_uris::metatheory::IMPLICIT_BIND.clone(),
                    presentation: None,
                },
                Box::new([
                    BoundArgument::Bound(ComponentVar {
                        var: v,
                        tp: None,
                        df: None,
                    }),
                    BoundArgument::Simple(t),
                ]),
                None,
            ))
        });
        */
        tracing::warn!("New context: {ctx:?}");
        let n = nt / ctx.as_slice();
        tracing::warn!("Implicitified: {:?}", n.debug_short());
        n
        */
    }

    fn get_head(
        self,
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

    fn prepare_i(self, context: Context, t: Term) -> Term {
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
        self.prepare_recurse(context, unsafe { t.assume_init() })
    }

    fn prepare_recurse(self, mut context: Context, term: Term) -> Term {
        match term {
            Term::Application(a) => Term::Application(ApplicationTerm::new(
                self.prepare_i(context.branch(), a.head.clone()),
                a.arguments
                    .iter()
                    .map(|arg| match arg {
                        Argument::Simple(t) => {
                            Argument::Simple(self.prepare_i(context.branch(), t.clone()))
                        }
                        Argument::Sequence(MaybeSequence::One(t)) => Argument::Sequence(
                            MaybeSequence::One(self.prepare_i(context.branch(), t.clone())),
                        ),
                        Argument::Sequence(MaybeSequence::Seq(ts)) => {
                            Argument::Sequence(MaybeSequence::Seq(
                                ts.iter()
                                    .cloned()
                                    .map(|t| self.prepare_i(context.branch(), t))
                                    .collect(),
                            ))
                        }
                    })
                    .collect(),
                a.presentation.clone(),
            )),
            Term::Bound(b) => Term::Bound(BindingTerm::new(
                self.prepare_i(context.branch(), b.head.clone()),
                context.in_branch(|mut context| {
                    b.arguments
                        .iter()
                        .map(|arg| match arg {
                            BoundArgument::Simple(t) => {
                                BoundArgument::Simple(self.prepare_i(context.branch(), t.clone()))
                            }
                            BoundArgument::Sequence(MaybeSequence::One(t)) => {
                                BoundArgument::Sequence(MaybeSequence::One(
                                    self.prepare_i(context.branch(), t.clone()),
                                ))
                            }
                            BoundArgument::Sequence(MaybeSequence::Seq(ts)) => {
                                BoundArgument::Sequence(MaybeSequence::Seq(
                                    ts.iter()
                                        .cloned()
                                        .map(|t| self.prepare_i(context.branch(), t))
                                        .collect(),
                                ))
                            }
                            BoundArgument::Bound(cv) => {
                                BoundArgument::Bound(self.prepare_cv(&mut context, cv))
                            }
                            BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                                BoundArgument::BoundSeq(MaybeSequence::One(
                                    self.prepare_cv(&mut context, cv),
                                ))
                            }
                            BoundArgument::BoundSeq(MaybeSequence::Seq(vars)) => {
                                BoundArgument::BoundSeq(MaybeSequence::Seq(
                                    vars.iter()
                                        .map(|cv| self.prepare_cv(&mut context, cv))
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

    fn prepare_cv(self, context: &mut Context, cv: &ComponentVar) -> ComponentVar {
        let tp = match &cv.tp {
            Some(t) => Some(self.prepare_i(context.branch(), t.clone())),
            None => self.infer_var_type_i(
                &mut SolverTrace::new(CheckingTask::VariableInference(cv.var.name())),
                context,
                &cv.var,
            ),
        };
        let df = match &cv.df {
            Some(t) => Some(self.prepare_i(context.branch(), t.clone())),
            None => self.get_var_definiens(context, &cv.var),
        };
        let cv = ComponentVar {
            var: cv.var.clone(),
            tp,
            df,
        };
        context.extend(cv.clone());
        cv
    }
}
