use crate::{
    Checker, facts::GlobalOrLocal, hoas::HOASSymbols, impls::solving::Solutions,
    split::SplitStrategy,
};
use ftml_ontology::{
    narrative::elements::{LogicalParagraph, paragraphs::ParagraphStep},
    terms::{ComponentVar, Term, Variable},
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

impl<Split: SplitStrategy> Checker<Split> {
    pub fn check_definition(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here! Definition {}", p.uri);
        None
    }

    pub fn check_proof(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here: {:?}", &p.children);
        let mut ret = Vec::new();
        let mut ctx = Vec::new();
        let _ = self.hoas()?;
        let mut state = ProofCheckState {
            context: &mut ctx,
            counter: 0,
            conclusion_df: None,
            conclusion_type: None,
        };
        let for_symbol = p
            .fors
            .first()
            .and_then(|sym| self.get_symbol(&sym.0, |t| self.prepare(None, t).1).ok());
        let block = for_symbol.as_ref().map(|sym| &sym.uri);
        for s in &p.steps {
            if let Some(res) = self.proof_step(s, &mut state, block) {
                ret.push(res);
            }
        }
        if matches!(p.steps.last(), Some(ParagraphStep::ProofConclusion { .. })) {
            state.context.pop();
        }
        let (tp, df) = state.make_conclusion(self.hoas()?, 0);
        if (tp.is_some() || df.is_some())
            && let Some(sym) = for_symbol.as_ref()
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
        context: &mut ProofCheckState,
        block: Option<&SymbolUri>,
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
                    block,
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
                    block,
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
                    block,
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
                    .filter_map(|s| self.proof_step(s, context,block))
                    .collect();
                if matches!(steps.last(), Some(ParagraphStep::ProofConclusion { .. })) {
                    context.context.pop();
                }
                let (tp, df) = context.make_conclusion(self.hoas().expect("checked earlier"),curr);
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
        context: &mut ProofCheckState,
        hoas: &HOASSymbols,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
        needs_def: bool,
        block: Option<&SymbolUri>,
    ) -> (Option<ProofStepCheckResult>, Option<Term>, Option<Term>) {
        let mut tp = None;
        let mut df = None;
        let mut proof_log = None;
        let var = var_name.and_then(|vn| self.get_variable(vn).ok());
        if let Some(tm) = yields {
            let (unks, tm) = self.prepare(None, tm.clone());
            let (b, unks, l) = context.check_inhabitable(self, unks, &tm, block);
            proof_log = Some(ProofStepCheckResult::GoalOnly {
                result: TypeCheckResult {
                    success: b.unwrap_or_default(),
                    log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                },
            });
            if Some(true) == b {
                tp = Some(tm);
            } else {
                let tm = hoas.wrap_judg(&tm);
                let (b, unks, l) = context.check_inhabitable(self, unks, &tm, block);
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
                let (b, unks, l) = context.check_type(self, unks, &tm, tp, block);
                df = Some(tm);
                proof_log = Some(ProofStepCheckResult::Both {
                    inhabitable: result,
                    matches: Some(TypeCheckResult {
                        success: b.unwrap_or_default(),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    }),
                });
            } else {
                let (r, unks, l) = context.infer(self, unks, &tm, block);
                let infed = r.clone().map(|t| self.revert_prepare(t));
                proof_log = Some(ProofStepCheckResult::ProofOnly {
                    inferred: infed,
                    log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                });
                tp = r;
                df = Some(tm);
            }
        }
        if df.is_none() {
            if let Some(tm) = method {
                let (tm, unks) = hoas.apply(
                    self,
                    tm,
                    arguments.iter().map(|o| o.as_ref().map(|(t, _)| t.clone())),
                );
                if let Some(tp) = &tp {
                    let Some(ProofStepCheckResult::GoalOnly { result }) = proof_log.take() else {
                        panic!("bug");
                    };
                    let (b, unks, l) = context.check_type(self, unks, &tm, tp, block);
                    df = Some(tm.into_owned());
                    proof_log = Some(ProofStepCheckResult::Both {
                        inhabitable: result,
                        matches: Some(TypeCheckResult {
                            success: b.unwrap_or_default(),
                            log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                        }),
                    });
                } else {
                    let (r, unks, l) = context.infer(self, unks, &tm, block);
                    let infed = r.clone().map(|t| self.revert_prepare(t));
                    proof_log = Some(ProofStepCheckResult::ProofOnly {
                        inferred: infed,
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    });
                    tp = r;
                    df = Some(tm.into_owned());
                }
            } else if needs_def && let Some(tp) = tp.as_ref() {
                let Some(ProofStepCheckResult::GoalOnly { result }) = proof_log.take() else {
                    panic!("bug");
                };
                let (r, unks, l) = context.prove(self, Solutions::default(), tp, block);
                df = r;
                proof_log = Some(ProofStepCheckResult::Both {
                    inhabitable: result,
                    matches: Some(TypeCheckResult {
                        success: df.is_some(),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    }),
                });
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
        context: &mut ProofCheckState,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
        is_assumption: bool,
        block: Option<&SymbolUri>,
    ) -> Option<ProofStepCheckResult> {
        let (r, tp, df) = self.step_data_i(
            context,
            self.hoas()?,
            var_name,
            method,
            justification,
            arguments,
            yields,
            !is_assumption,
            block,
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
        context: &mut ProofCheckState,
        var_name: Option<&DocumentElementUri>,
        method: Option<&Term>,
        justification: Option<&Term>,
        arguments: &[Option<(Term, SourceRange)>],
        yields: Option<&Term>,
        block: Option<&SymbolUri>,
    ) -> Option<ProofStepCheckResult> {
        let (r, tp, df) = self.step_data_i(
            context,
            self.hoas()?,
            var_name,
            method,
            justification,
            arguments,
            yields,
            true,
            block,
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
        let hoas = self.hoas()?;
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
struct ProofCheckState<'c> {
    context: &'c mut Vec<(ComponentVar, bool)>,
    counter: usize,
    conclusion_type: Option<Term>,
    conclusion_df: Option<Term>,
}

impl ProofCheckState<'_> {
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
    fn make_conclusion(&mut self, hoas: &HOASSymbols, off: usize) -> (Option<Term>, Option<Term>) {
        self.context.drain(off..).rev().fold(
            (self.conclusion_type.take(), self.conclusion_df.take()),
            |(tp, df), (v, is_ass)| match (tp, df) {
                (Some(tp), None) if is_ass => (Some(hoas.pi(v, tp)), None),
                (None, Some(df)) if is_ass => (None, Some(hoas.lambda(v, df))),
                (Some(tp), Some(df)) if is_ass => {
                    (Some(hoas.pi(v.clone(), tp)), Some(hoas.lambda(v, df)))
                }
                (Some(tp), None) => (
                    (Some(if tp.has_free_such_that(|v2| v2.name() == v.var.name()) {
                        if let Some(df) = v.df {
                            tp / (v.var.name(), &df)
                        } else {
                            HOASSymbols::let_in(v, tp)
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
                        HOASSymbols::let_in(v, df)
                    }),
                ),
                (Some(tp), Some(df)) => (
                    (Some(if tp.has_free_such_that(|v2| v2.name() == v.var.name()) {
                        if let Some(df) = &v.df {
                            tp / (v.var.name(), df)
                        } else {
                            HOASSymbols::let_in(v.clone(), tp)
                        }
                    } else {
                        tp
                    })),
                    Some(if let Some(d) = v.df {
                        df / (v.var.name(), &d)
                    } else {
                        HOASSymbols::let_in(v, df)
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
        unks: Solutions,
        t: &Term,
        block: Option<&SymbolUri>,
    ) -> (Option<bool>, Solutions, PreCheckLog) {
        checker.wrap_task(CheckingTask::Inhabitable(t), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            if let Some(blocked) = block {
                slf.context
                    .block_fact(GlobalOrLocal::Global(blocked.clone()));
            }
            slf.check_inhabitable_i(t)
        })
    }
    fn check_type<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: Solutions,
        tm: &Term,
        tp: &Term,
        block: Option<&SymbolUri>,
    ) -> (Option<bool>, Solutions, PreCheckLog) {
        checker.wrap_task(CheckingTask::HasType(tm, tp), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            if let Some(blocked) = block {
                slf.context
                    .block_fact(GlobalOrLocal::Global(blocked.clone()));
            }
            slf.check_type_i(tm, tp)
        })
    }
    fn check_equal<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: Solutions,
        lhs: &Term,
        rhs: &Term,
        block: Option<&SymbolUri>,
    ) -> (Option<bool>, Solutions, PreCheckLog) {
        checker.wrap_task(CheckingTask::Equality(lhs, rhs), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            if let Some(blocked) = block {
                slf.context
                    .block_fact(GlobalOrLocal::Global(blocked.clone()));
            }
            slf.check_equality_i(lhs, rhs)
        })
    }
    fn infer<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: Solutions,
        t: &Term,
        block: Option<&SymbolUri>,
    ) -> (Option<Term>, Solutions, PreCheckLog) {
        checker.wrap_task(CheckingTask::Inference(t), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            if let Some(blocked) = block {
                slf.context
                    .block_fact(GlobalOrLocal::Global(blocked.clone()));
            }
            slf.infer_type_i(t)
        })
    }
    fn prove<Split: SplitStrategy>(
        &self,
        checker: &Checker<Split>,
        unks: Solutions,
        t: &Term,
        block: Option<&SymbolUri>,
    ) -> (Option<Term>, Solutions, PreCheckLog) {
        checker.wrap_task(CheckingTask::Proving(t), Some(unks), |mut slf| {
            for (c, _) in &*self.context {
                slf.extend_context(c);
            }
            if let Some(blocked) = block {
                slf.context
                    .block_fact(GlobalOrLocal::Global(blocked.clone()));
            }
            slf.prove_i(t)
        })
    }
}
