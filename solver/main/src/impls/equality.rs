use crate::{CheckRef, impls::solving::TermExtSolvable, split::SplitStrategy, trace::CheckingTask};
use ftml_ontology::terms::{ApplicationTerm, Argument, Term};

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    #[allow(clippy::unused_self)]
    pub(crate) fn trivially_equal(&self, lhs: &Term, rhs: &Term) -> bool {
        if lhs == rhs {
            return true;
        }
        match (lhs, rhs) {
            (Term::Var { variable: v1, .. }, Term::Var { variable: v2, .. }) => {
                v1.name() == v2.name()
            }
            _ => false,
        }
    }

    pub fn check_equality(&mut self, lhs: &'t Term, rhs: &'t Term) -> Option<bool> {
        self.wrap_check(CheckingTask::Equality(lhs, rhs), |slf| {
            if slf.trivially_equal(lhs, rhs) {
                slf.comment("trivial");
                return Some(true);
            }
            slf.check_equality_i(lhs, rhs)
        })
    }
    pub(crate) fn check_equality_i(&mut self, lhs: &'t Term, rhs: &'t Term) -> Option<bool> {
        if let Some(unk) = lhs.is_solvable() {
            return self.solve_equality(unk, rhs);
        }
        if let Some(unk) = rhs.is_solvable() {
            return self.solve_equality(unk, lhs);
        }

        let rules = self.top.rules.equality().iter().filter_map(|rl| {
            if rl.applicable(lhs, rhs) {
                Some(&**rl)
            } else {
                None
            }
        });
        let prev = match Split::split_i(self, rules, |slf, rl| rl.apply(slf, lhs, rhs)) {
            Ok(r) => return Some(r),
            Err(ls) => ls,
        };
        match (lhs, rhs) {
            (Term::Application(lhs), Term::Application(rhs))
                if lhs.arguments.len() == rhs.arguments.len() =>
            {
                match self.traced(CheckingTask::Strategy("Trying congruence"), |slf| {
                    slf.congruence(lhs, rhs)
                }) {
                    Ok(r) => Some(r),
                    Err(l) => {
                        for l in prev {
                            self.add_msg(l.into());
                        }
                        self.add_msg(l.into());
                        None
                    }
                }
            }
            _ => {
                for l in prev {
                    self.add_msg(l.into());
                }
                None
            }
        }
    }

    // invariant: lhs.arguments.len() == rhs.arguments.len()
    fn congruence(&mut self, lhs: &'t ApplicationTerm, rhs: &'t ApplicationTerm) -> Option<bool> {
        self.comment("Comparing operators");
        if !self.check_equality(&lhs.head, &rhs.head)? {
            return None;
        }
        for (i, (a, b)) in lhs.arguments.iter().zip(&rhs.arguments).enumerate() {
            self.counter("Comparing arguments ", i + 1);
            match (a, b) {
                (Argument::Simple(a), Argument::Simple(b)) => {
                    if !self.check_equality(a, b)? {
                        return None;
                    }
                }
                _ => {
                    self.failure("Argument not simple");
                    return None;
                }
            }
        }
        Some(true)
    }
}
