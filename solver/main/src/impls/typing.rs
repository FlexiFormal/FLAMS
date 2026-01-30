use crate::{CheckRef, impls::solving::TermExtSolvable, split::SplitStrategy, trace::CheckingTask};
use ftml_ontology::terms::Term;

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    pub fn check_type(&mut self, tm: &'t Term, tp: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::HasType(tm, tp), |slf| {
            slf.check_type_i(tm, tp)
        })
    }

    pub fn check_subtype(&mut self, sub: &'t Term, sup: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::Subtype(sub, sup), |slf| {
            slf.check_subtype_i(sub, sup)
        })
    }

    pub fn check_inhabitable(&mut self, t: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::Inhabitable(t), |slf| {
            slf.check_inhabitable_i(t)
        })
    }

    pub fn check_universe(&mut self, t: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::Universe(t), |slf| slf.check_universe_i(t))
    }

    pub(crate) fn check_type_i(&mut self, tm: &'t Term, tp: &'t Term) -> Option<bool> {
        self.cancellable(|slf| {
            Split::strategies(
                slf,
                "Using type inference",
                |slf| {
                    let subtp = slf.infer_type(tm)?;
                    slf.scoped(|slf| slf.check_subtype(&subtp, tp))
                },
                "Using checking rules",
                |slf| {
                    let rules = self.top.rules.checking().iter().filter_map(|rl| {
                        if rl.applicable(tm, tp) {
                            Some(&**rl)
                        } else {
                            None
                        }
                    });
                    Split::split(slf, rules, |slf, rl| rl.apply(slf, tm, tp))
                },
            )
        })
    }

    pub(crate) fn check_subtype_i(&mut self, sub: &'t Term, sup: &'t Term) -> Option<bool> {
        if self.trivially_equal(sub, sup) {
            self.comment("trivial");
            return Some(true);
        }
        if let Some(unk) = sub.is_solvable() {
            return self.solve_upper_bound(unk, sup);
        }
        if let Some(unk) = sup.is_solvable() {
            return self.solve_lower_bound(unk, sub);
        }
        let rules = self.top.rules.subtyping().iter().filter_map(|rl| {
            if rl.applicable(sub, sup) {
                Some(&**rl)
            } else {
                None
            }
        });
        let lines = match Split::split_i(self, rules, |slf, rl| rl.apply(slf, sub, sup)) {
            Ok(r) => return Some(r),
            Err(ls) => ls,
        };
        match self.traced(
            CheckingTask::Strategy("Proving subtyping failed; Falling back to checking equality"),
            |slf| slf.check_equality_i(sub, sup),
        ) {
            Ok(b) => Some(b),
            Err(l) => {
                for l in lines {
                    self.add_msg(l.into());
                }
                self.add_msg(l.into());
                None
            }
        }
    }

    pub(crate) fn check_inhabitable_i(&mut self, tm: &'t Term) -> Option<bool> {
        Split::strategies(
            self,
            "Using type inference",
            |slf| {
                let tp = slf.infer_type(tm)?;
                slf.scoped(|slf| slf.check_universe(&tp))
            },
            "Using inhabitable rules",
            |slf| {
                let rules = slf
                    .top
                    .rules
                    .inhabitable()
                    .iter()
                    .filter_map(|rl| if rl.applicable(tm) { Some(&**rl) } else { None });
                Split::split(slf, rules, |slf, rl| rl.apply(slf, tm))
            },
        )
    }

    fn check_universe_i(&mut self, t: &'t Term) -> Option<bool> {
        let rules = self
            .top
            .rules
            .universe()
            .iter()
            .filter_map(|rl| if rl.applicable(t) { Some(&**rl) } else { None });
        Split::split(self, rules, |slf, rl| rl.apply(slf, t))
    }
}
