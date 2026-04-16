use ftml_ontology::terms::{ComponentVar, Term, Variable};

use crate::{
    CheckRef, impls::solving::is_solvable_var, rules::InferenceRule, split::SplitStrategy,
    trace::CheckingTask,
};

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub fn infer_type(&mut self, t: &'t Term) -> Option<Term> {
        tracing::debug!("Inferring type of {:?}", t.debug_short());
        let r = self.wrap_check(CheckingTask::Inference(t), |slf| slf.infer_type_i(t));
        if let Some(r) = &r {
            tracing::debug!("Inferred: {:?}", r.debug_short());
        } else {
            tracing::debug!("Inferrence failed");
        }
        r
    }
    pub(crate) fn infer_type_i(&mut self, t: &'t Term) -> Option<Term> {
        match t {
            Term::Symbol { uri, .. } => {
                self.comment("Looking up symbol");
                let ret = self.get_symbol_type(uri);

                if ret.is_none() {
                    self.failure("Symbol has no type");
                }
                return ret;
            }
            Term::Var { variable, .. } => {
                return self.infer_var_type_i(variable);
            }
            _ => (),
        }
        self.simplify_rules(
            self.top.rules.inference(),
            t,
            InferenceRule::applicable,
            |slf, rl, t| rl.infer(slf, t),
        )
        /*
        let rules = self
            .top
            .rules
            .inference()
            .iter()
            .filter_map(|rl| if rl.applicable(t) { Some(&**rl) } else { None })
            .collect::<smallvec::SmallVec<_, 2>>();
        let r = Split::split(self, true, rules, |slf, rl| rl.infer(slf, t));
        r.map(|t| self.subst(t))
        */
        /*r.map(|t| {
            let simp = self.scoped(|slf| slf.simplify_full(false, &t)).unwrap_or(t);
            self.subst(simp)
        })*/
    }

    pub fn infer_var_type(&mut self, var: &'t Variable) -> Option<Term> {
        self.wrap_check(CheckingTask::VariableInference(var.name()), |slf| {
            slf.infer_var_type_i(var)
        })
    }
    pub(crate) fn infer_var_type_i(&mut self, var: &Variable) -> Option<Term> {
        if let Some(id) = is_solvable_var(var) {
            return Some(self.get_solvable_type(id));
        }
        let (ctx, mut msgs) = self.split();
        for v in ctx.iter().rev().map(|v| &**v) {
            match (v, var) {
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        tp,
                        ..
                    },
                    Variable::Name { name: n2, .. },
                ) if *name == *n2 => {
                    if tp.is_some() {
                        msgs.comment("Found type in context");
                    } else {
                        msgs.failure("variable untyped in context");
                    }
                    return tp.clone().map(|t| self.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        tp,
                        ..
                    },
                    Variable::Ref { declaration, .. },
                ) if name.as_ref() == declaration.name().last() && tp.is_some() => {
                    if tp.is_some() {
                        msgs.comment("Found type in context");
                    } else {
                        msgs.failure("Variable untyped in context");
                    }
                    return tp.clone().map(|t| self.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        tp,
                        ..
                    },
                    Variable::Name { name, .. },
                ) if name.as_ref() == declaration.name().last() => {
                    return if tp.is_some() {
                        msgs.comment("Found type in context");
                        tp.clone().map(|t| self.subst(t))
                    } else {
                        msgs.comment("Getting variable globally");
                        let declaration = declaration.clone();
                        let var = self.get_variable(&declaration).ok()?;
                        var.data.tp.checked_or_parsed().map(|(t, _)| {
                            if var.data.is_seq && t.as_sequence_type().is_none() {
                                t.into_seq_type()
                            } else {
                                t
                            }
                        })
                    };
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        tp,
                        ..
                    },
                    Variable::Ref {
                        declaration: d2, ..
                    },
                ) if *declaration == *d2 => {
                    return if tp.is_some() {
                        msgs.comment("Found type in context");
                        tp.clone().map(|t| self.subst(t))
                    } else {
                        msgs.comment("Getting variable globally");
                        let declaration = declaration.clone();
                        let var = self.get_variable(&declaration).ok()?;
                        var.data.tp.checked_or_parsed().map(|(t, _)| {
                            if var.data.is_seq && t.as_sequence_type().is_none() {
                                t.into_seq_type()
                            } else {
                                t
                            }
                        })
                    };
                }
                _ => (),
            }
        }
        if let Variable::Ref { declaration, .. } = var {
            self.comment("Getting variable globally");
            let var = self.get_variable(declaration).ok()?;
            var.data.tp.checked_or_parsed().map(|(t, _)| {
                if var.data.is_seq && t.as_sequence_type().is_none() {
                    t.into_seq_type()
                } else {
                    t
                }
            })
        } else {
            Some(self.new_solvable())
        }
    }
}
