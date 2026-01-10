use ftml_ontology::terms::{ComponentVar, Term, Variable};

use crate::{CheckRef, split::SplitStrategy, trace::CheckingTask};

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub fn infer_type(&mut self, t: &'t Term) -> Option<Term> {
        self.wrap_check(CheckingTask::Inference(t), |slf| slf.infer_type_i(t))
    }
    pub(crate) fn infer_type_i(&mut self, t: &'t Term) -> Option<Term> {
        match t {
            Term::Symbol { uri, .. } => {
                self.comment("Looking up symbol");
                let Ok(s) = self.get_symbol(uri) else {
                    self.failure("Symbol not found");
                    return None;
                };
                let ret = s
                    .data
                    .tp
                    .checked_or_parsed()
                    .map(|(t, _)| self.bind_implicits(t));

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
        let rules = self
            .top
            .rules
            .inference()
            .iter()
            .filter_map(|rl| if rl.applicable(t) { Some(&**rl) } else { None });
        let r = Split::split(self, rules, |slf, rl| rl.infer(slf, t));
        r.map(|t| self.subst(t))
    }

    pub fn infer_var_type(&mut self, var: &'t Variable) -> Option<Term> {
        self.wrap_check(CheckingTask::VariableInference(var.name()), |slf| {
            slf.infer_var_type_i(var)
        })
    }
    pub(crate) fn infer_var_type_i(&mut self, var: &Variable) -> Option<Term> {
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
                        self.get_variable(&declaration)
                            .ok()?
                            .data
                            .tp
                            .checked_or_parsed()
                            .map(|(t, _)| t)
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
                        self.get_variable(&declaration)
                            .ok()?
                            .data
                            .tp
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                _ => (),
            }
        }
        if let Variable::Ref { declaration, .. } = var {
            self.comment("Getting variable globally");
            self.get_variable(declaration)
                .ok()?
                .data
                .tp
                .checked_or_parsed()
                .map(|(t, _)| t)
        } else {
            None
        }
    }
}
