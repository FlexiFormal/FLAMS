use crate::{Checker, impls::solving::Solvable, split::SplitStrategy};
use ftml_ontology::{
    narrative::elements::{LogicalParagraph, paragraphs::ParagraphStep},
    terms::{ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, Term, Variable},
    utils::SourceRange,
};
use ftml_solver_trace::{
    CheckLog, CheckingTask, PreCheckLog,
    results::{
        CheckResult, ContentCheckResult, ProofStepCheckResult, ProofStepResult, SymbolCheckResult,
        TypeCheckResult,
    },
};
use ftml_uris::{DocumentElementUri, Id, SymbolUri};
use std::borrow::Cow;

impl<Split: SplitStrategy> Checker<Split> {
    pub fn check_definition(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here! Definition {}", p.uri);
        None
    }

    pub fn check_proof(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here: {:?}", &p.children);
        let mut ret = Vec::new();
        let mut ctx = Vec::new();
        let hoas = HOASSymbols::get(self)?;
        let mut state = ProofState {
            hoas: &hoas,
            context: &mut ctx,
            counter: 0,
            conclusion_df: None,
            conclusion_type: None,
        };
        for s in &p.steps {
            if let Some(res) = self.proof_step(s, &mut state) {
                ret.push(res);
            }
        }
        if matches!(p.steps.last(), Some(ParagraphStep::ProofConclusion { .. })) {
            state.context.pop();
        }
        let (tp, df) = state.make_conclusion(0);
        if (tp.is_some() || df.is_some())
            && let Some((sym, _)) = p.fors.first()
            && let Ok(sym) = self.get_symbol(sym, |t| self.prepare(None, t).1)
        {
            let orig_tp = &sym.data.tp;
            let orig_df = &sym.data.df;
            if let Some(tp) = tp {
                if let Some((orig, _)) = orig_tp.checked_or_parsed() {
                    let (b, _, l) = self.check_subtype(None, &tp, &orig);
                    ret.push(ProofStepResult::Conclusion {
                        var: None,
                        result: ProofStepCheckResult::GoalOnly {
                            result: TypeCheckResult {
                                success: b.unwrap_or_default(),
                                log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                            },
                        },
                    });
                } else {
                    orig_tp.set_checked(tp.clone());
                    orig_tp.set_presentation(self.revert_prepare(tp));
                }
                if let Some(df) = df
                    && orig_df.is_none()
                {
                    orig_df.set_checked(df.clone());
                    orig_df.set_presentation(self.revert_prepare(df));
                }
            }
        }
        Some(CheckResult::Proof(p.uri.clone(), ret))
    }

    fn proof_step(
        &mut self,
        s: &ParagraphStep,
        context: &mut ProofState,
    ) -> Option<ProofStepResult> {
        match s {
            ParagraphStep::EquationStep => None,
            ParagraphStep::ProofAssumption {
                var_name,
                method,
                justification,
                arguments,
                yields,
            } => self
                .step_data(
                    context,
                    var_name.as_ref(),
                    method.as_ref().map(|(t, _)| t),
                    justification.as_ref().map(|(t, _)| t),
                    arguments,
                    yields.as_ref().map(|(t, _)| t),
                    true,
                )
                .map(|r| ProofStepResult::Assumption {
                    var: var_name.clone(),
                    result: r,
                }),
            ParagraphStep::ProofConclusion {
                var_name,
                method,
                justification,
                arguments,
                yields,
            } => self
                .conclusion_step(
                    context,
                    var_name.as_ref(),
                    method.as_ref().map(|(t, _)| t),
                    justification.as_ref().map(|(t, _)| t),
                    arguments,
                    yields.as_ref().map(|(t, _)| t),
                )
                .map(|r| ProofStepResult::Conclusion {
                    var: var_name.clone(),
                    result: r,
                }),
            ParagraphStep::ProofStep {
                var_name,
                method,
                justification,
                arguments,
                yields,
            } => self
                .step_data(
                    context,
                    var_name.as_ref(),
                    method.as_ref().map(|(t, _)| t),
                    justification.as_ref().map(|(t, _)| t),
                    arguments,
                    yields.as_ref().map(|(t, _)| t),
                    false,
                )
                .map(|r| ProofStepResult::Step {
                    var: var_name.clone(),
                    result: r,
                }),
            ParagraphStep::Subproof {
                uri,
                var_name,
                steps,
                .. /*
                method,
                justification,
                arguments,
                yields,
                */
            } => {
                let curr = context.context.len();
                let results = steps
                    .iter()
                    .filter_map(|s| self.proof_step(s, context))
                    .collect();
                if matches!(steps.last(), Some(ParagraphStep::ProofConclusion { .. })) {
                    context.context.pop();
                }
                let (tp, df) = context.make_conclusion(curr);
                let var = var_name.as_ref().map_or_else(
                    || Variable::Name {
                        name: context.dummy(),
                        notated: None,
                    },
                    |uri| Variable::Ref {
                        declaration: uri.clone(),
                        is_sequence: None,
                    },
                );
                if let Some(v) = var_name.as_ref().and_then(|vn| self.get_variable(vn).ok()) {
                    if let Some(tp) = &tp {
                        v.data.tp.set_checked(tp.clone());
                        let pres = self.revert_prepare(tp.clone());
                        v.data.tp.set_presentation(pres);
                    }
                    if let Some(df) = &df {
                        v.data.df.set_checked(df.clone());
                        let pres = self.revert_prepare(df.clone());
                        v.data.df.set_presentation(pres);
                    }
                }
                context.context.push((ComponentVar { var, tp, df }, false));
                // TODO something reasonable
                //r
                Some(ProofStepResult::Subproof {
                    uri: uri.clone(),
                    var: var_name.clone(),
                    results,
                })
            }
        }
    }

    fn step_data_i(
        &self,
        context: &mut ProofState,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
        needs_def: bool,
    ) -> (Option<ProofStepCheckResult>, Option<Term>, Option<Term>) {
        let mut tp = None;
        let mut df = None;
        let mut proof_log = None;
        let var = var_name.and_then(|vn| self.get_variable(vn).ok());
        if let Some(tm) = yields {
            let (unks, tm) = self.prepare(None, tm.clone());
            let (b, unks, l) = context.check_inhabitable(self, unks, &tm);
            proof_log = Some(ProofStepCheckResult::GoalOnly {
                result: TypeCheckResult {
                    success: b.unwrap_or_default(),
                    log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                },
            });
            if Some(true) == b {
                tp = Some(tm);
            } else {
                let tm = context.hoas.wrap_judg(&tm);
                let (b, unks, l) = context.check_inhabitable(self, unks, &tm);
                proof_log = Some(ProofStepCheckResult::GoalOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or_default(),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                });
                tp = Some(tm.into_owned());
            }
        }
        if let Some(tm) = justification {
            let (unks, tm) = self.prepare(None, tm.clone());
            if let Some(tp) = tp.as_ref() {
                let Some(ProofStepCheckResult::GoalOnly { result }) = proof_log.take() else {
                    panic!("bug");
                };
                let (b, unks, l) = context.check_type(self, unks, &tm, tp);
                df = Some(tm);
                proof_log = Some(ProofStepCheckResult::Both {
                    inhabitable: result,
                    matches: Some(TypeCheckResult {
                        success: b.unwrap_or_default(),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    }),
                });
            } else {
                let (r, unks, l) = context.infer(self, unks, &tm);
                let infed = r.clone().map(|t| self.revert_prepare(t));
                proof_log = Some(ProofStepCheckResult::ProofOnly {
                    inferred: infed,
                    log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                });
                tp = r;
                df = Some(tm);
            }
        }
        if df.is_none()
            && let Some(tm) = method
        {
            let (tm, unks) = context.hoas.apply(
                self,
                tm,
                arguments.iter().map(|o| o.as_ref().map(|(t, _)| t.clone())),
            );
            if let Some(tp) = &tp {
                let Some(ProofStepCheckResult::GoalOnly { result }) = proof_log.take() else {
                    panic!("bug");
                };
                let (b, unks, l) = context.check_type(self, unks, &tm, tp);
                df = Some(tm.into_owned());
                proof_log = Some(ProofStepCheckResult::Both {
                    inhabitable: result,
                    matches: Some(TypeCheckResult {
                        success: b.unwrap_or_default(),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    }),
                });
            } else {
                let (r, unks, l) = context.infer(self, unks, &tm);
                let infed = r.clone().map(|t| self.revert_prepare(t));
                proof_log = Some(ProofStepCheckResult::ProofOnly {
                    inferred: infed,
                    log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                });
                tp = r;
                df = Some(tm.into_owned());
            }
        }
        if let Some(v) = var {
            if let Some(tp) = &tp {
                v.data.tp.set_checked(tp.clone());
                let pres = self.revert_prepare(tp.clone());
                v.data.tp.set_presentation(pres);
            }
            if let Some(df) = &df {
                v.data.df.set_checked(df.clone());
                let pres = self.revert_prepare(df.clone());
                v.data.df.set_presentation(pres);
            }
        }
        if needs_def
            && df.is_none()
            && let Some(ProofStepCheckResult::GoalOnly { result }) = proof_log.as_mut()
        {
            result.success = false;
            result.log.add_failure("Unproven goal");
        }
        (proof_log, tp, df)
    }

    fn step_data(
        &mut self,
        context: &mut ProofState,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
        is_assumption: bool,
    ) -> Option<ProofStepCheckResult> {
        let (r, tp, df) = self.step_data_i(
            context,
            var_name,
            method,
            justification,
            arguments,
            yields,
            !is_assumption,
        );
        let var = var_name.map_or_else(
            || Variable::Name {
                name: context.dummy(),
                notated: None,
            },
            |uri| Variable::Ref {
                declaration: uri.clone(),
                is_sequence: None,
            },
        );
        let cv = ComponentVar { var, tp, df };
        context.context.push((cv, is_assumption));
        r
    }

    fn conclusion_step(
        &self,
        context: &mut ProofState,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
    ) -> Option<ProofStepCheckResult> {
        let (r, tp, df) = self.step_data_i(
            context,
            var_name,
            method,
            justification,
            arguments,
            yields,
            true,
        );
        let var = var_name.map_or_else(
            || Variable::Name {
                name: context.dummy(),
                notated: None,
            },
            |uri| Variable::Ref {
                declaration: uri.clone(),
                is_sequence: None,
            },
        );
        context.conclusion_type.clone_from(&tp);
        context.conclusion_df.clone_from(&df);
        let cv = ComponentVar { var, tp, df };
        context.context.push((cv, false));
        r
    }

    pub fn check_assertion(&mut self, p: &LogicalParagraph) -> Option<Vec<CheckResult>> {
        let hoas = HOASSymbols::get(self)?;
        let mut ret = Vec::new();
        for (target, term) in &p.fors {
            let Ok(target) = self.get_symbol(target, |t| t) else {
                continue;
            };
            let Some(term) = term else { continue };
            let Some(term) = term.get_parsed() else {
                continue;
            };
            let wrapped = hoas.wrap_types(&p.premises, term);
            let (unks, tp) = self.prepare(None, wrapped.into_owned());

            tracing::trace!("Checking assertion for {}", target.uri);
            let (b, _, l) = self.check_inhabitable(Some(unks), &tp);
            target
                .data
                .tp
                .set_presentation(self.revert_prepare(tp.clone()));
            target.data.tp.set_checked(tp);
            ret.push(CheckResult::Content(ContentCheckResult::Symbol(
                target.uri.clone(),
                SymbolCheckResult::TypeOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                },
            )));
        }
        Some(ret)
    }
}

#[derive(Debug)]
struct ProofState<'c> {
    hoas: &'c HOASSymbols,
    context: &'c mut Vec<(ComponentVar, bool)>,
    counter: usize,
    conclusion_type: Option<Term>,
    conclusion_df: Option<Term>,
}

impl ProofState<'_> {
    fn dummy(&mut self) -> Id {
        // SAFETY: valid ID
        let r = unsafe {
            format!("DUMMY_{}", self.counter + 1)
                .parse()
                .unwrap_unchecked()
        };
        self.counter += 1;
        r
    }
    fn make_conclusion(&mut self, off: usize) -> (Option<Term>, Option<Term>) {
        self.context.drain(off..).rev().fold(
            (self.conclusion_type.take(), self.conclusion_df.take()),
            |(tp, df), (v, is_ass)| match (tp, df) {
                (Some(tp), None) if is_ass => (Some(self.hoas.pi(v, tp)), None),
                (None, Some(df)) if is_ass => (None, Some(self.hoas.lambda(v, df))),
                (Some(tp), Some(df)) if is_ass => (
                    Some(self.hoas.pi(v.clone(), tp)),
                    Some(self.hoas.lambda(v, df)),
                ),
                (Some(tp), None) => (
                    (Some(if tp.has_free_such_that(|v2| v2.name() == v.var.name()) {
                        if let Some(df) = v.df {
                            tp / (v.var.name(), &df)
                        } else {
                            self.hoas.let_in(v, tp)
                        }
                    } else {
                        tp
                    })),
                    None,
                ),
                (None, Some(df)) => (
                    None,
                    Some(if let Some(d) = v.df {
                        df / (v.var.name(), &d)
                    } else {
                        self.hoas.let_in(v, df)
                    }),
                ),
                (Some(tp), Some(df)) => (
                    (Some(if tp.has_free_such_that(|v2| v2.name() == v.var.name()) {
                        if let Some(df) = &v.df {
                            tp / (v.var.name(), df)
                        } else {
                            self.hoas.let_in(v.clone(), tp)
                        }
                    } else {
                        tp
                    })),
                    Some(if let Some(d) = v.df {
                        df / (v.var.name(), &d)
                    } else {
                        self.hoas.let_in(v, df)
                    }),
                ),
                (None, None) => (None, None),
            },
        )
    }

    fn forget(&mut self, off: usize) {
        self.context.truncate(off);
    }

    fn check_inhabitable<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: rustc_hash::FxHashSet<Solvable>,
        t: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        checker.wrap_task(CheckingTask::Inhabitable(t), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            slf.check_inhabitable_i(t)
        })
    }
    fn check_type<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: rustc_hash::FxHashSet<Solvable>,
        tm: &Term,
        tp: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        checker.wrap_task(CheckingTask::HasType(tm, tp), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            slf.check_type_i(tm, tp)
        })
    }
    fn check_equal<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: rustc_hash::FxHashSet<Solvable>,
        lhs: &Term,
        rhs: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        checker.wrap_task(CheckingTask::Equality(lhs, rhs), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            slf.check_equality_i(lhs, rhs)
        })
    }
    fn infer<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: rustc_hash::FxHashSet<Solvable>,
        t: &Term,
    ) -> (Option<Term>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        checker.wrap_task(CheckingTask::Inference(t), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            slf.infer_type_i(t)
        })
    }
}

#[derive(Debug)]
struct HOASSymbols {
    judgment: Option<SymbolUri>,
    lambda: SymbolUri,
    pi: SymbolUri,
    apply: Option<SymbolUri>,
    //dummies: std::sync::atomic::AtomicUsize,
}
impl HOASSymbols {
    //fn new_dummy(&self) {}

    fn apply<'t, Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        head: &'t Term,
        arguments: impl Iterator<Item = Option<Term>>,
    ) -> (Cow<'t, Term>, rustc_hash::FxHashSet<Solvable>) {
        let mut ret = rustc_hash::FxHashSet::default();
        if let Some(app) = self.apply.as_ref() {
            (
                arguments.fold(Cow::Borrowed(head), |h, a| {
                    Cow::Owned(Term::Application(ApplicationTerm::new(
                        Term::Symbol {
                            uri: app.clone(),
                            presentation: None,
                        },
                        Box::new([
                            Argument::Simple(h.into_owned()),
                            Argument::Simple(a.unwrap_or_else(|| {
                                let name = checker.new_solvable();
                                ret.insert(Solvable {
                                    name: name.clone(),
                                    solution: crate::impls::solving::BoundedValue::None,
                                    tp: crate::impls::solving::BoundedValue::None,
                                });
                                Term::Var {
                                    variable: Variable::Name {
                                        name,
                                        notated: None,
                                    },
                                    presentation: None,
                                }
                            })),
                        ]),
                        None,
                    )))
                }),
                ret,
            )
        } else {
            let args = arguments
                .map(|t| {
                    Argument::Simple(t.unwrap_or_else(|| {
                        let name = checker.new_solvable();
                        ret.insert(Solvable {
                            name: name.clone(),
                            solution: crate::impls::solving::BoundedValue::None,
                            tp: crate::impls::solving::BoundedValue::None,
                        });
                        Term::Var {
                            variable: Variable::Name {
                                name,
                                notated: None,
                            },
                            presentation: None,
                        }
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

    fn get<Split: SplitStrategy>(checker: &Checker<Split>) -> Option<Self> {
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

    fn wrap_vars<'c>(
        &self,
        args: impl DoubleEndedIterator<Item = ComponentVar>,
        ret: &'c Term,
    ) -> Cow<'c, Term> {
        args.rev().fold(Cow::Borrowed(ret), |c, v| {
            //let premise = self.wrap_judg(p).into_owned();
            Cow::Owned(self.pi(v, c.into_owned()))
        })
    }

    fn wrap_types<'c>(&self, args: &[Term], ret: &'c Term) -> Cow<'c, Term> {
        let ret = self.wrap_judg(ret);
        args.iter().rev().fold(ret, |c, p| {
            let premise = self.wrap_judg(p).into_owned();
            Cow::Owned(self.pi(
                ComponentVar {
                    var: Variable::Name {
                        name: crate::DUMMY.clone(),
                        notated: None,
                    },
                    tp: Some(premise),
                    df: None,
                },
                c.into_owned(),
            ))
        })
    }

    fn lambda(&self, var: ComponentVar, body: Term) -> Term {
        Self::simple_bind(var, body, self.lambda.clone())
    }

    fn pi(&self, var: ComponentVar, body: Term) -> Term {
        Self::simple_bind(var, body, self.pi.clone())
    }

    fn let_in(&self, var: ComponentVar, body: Term) -> Term {
        Self::simple_bind(var, body, ftml_uris::metatheory::LET_IN.clone())
    }

    fn simple_bind(var: ComponentVar, body: Term, uri: SymbolUri) -> Term {
        Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri,
                presentation: None,
            },
            Box::new([BoundArgument::Bound(var), BoundArgument::Simple(body)]),
            None,
        ))
    }

    fn wrap_judg<'c>(&self, ret: &'c Term) -> Cow<'c, Term> {
        self.judgment.as_ref().map_or_else(
            || Cow::Borrowed(ret),
            |j| {
                Cow::Owned(self.apply.as_ref().map_or_else(
                    || {
                        Term::Application(ApplicationTerm::new(
                            Term::Symbol {
                                uri: j.clone(),
                                presentation: None,
                            },
                            Box::new([Argument::Simple(ret.clone())]),
                            None,
                        ))
                    },
                    |app| {
                        Term::Application(ApplicationTerm::new(
                            Term::Symbol {
                                uri: app.clone(),
                                presentation: None,
                            },
                            Box::new([
                                Argument::Simple(Term::Symbol {
                                    uri: j.clone(),
                                    presentation: None,
                                }),
                                Argument::Simple(ret.clone()),
                            ]),
                            None,
                        ))
                    },
                ))
            },
        )
    }
}
