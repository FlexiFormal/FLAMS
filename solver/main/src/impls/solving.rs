use std::borrow::Borrow;

use ftml_ontology::terms::{Term, Variable};
use ftml_uris::Id;

use crate::{CheckRef, split::SplitStrategy};

const PREFIX: &str = "SOLVE!";

pub trait TermExtSolvable {
    fn is_solvable(&self) -> Option<&Id>;
}

impl TermExtSolvable for Term {
    fn is_solvable(&self) -> Option<&Id> {
        if let Self::Var {
            variable: Variable::Name { name, .. },
            ..
        } = self
            && name.as_ref().starts_with(PREFIX)
            && name.as_ref().as_bytes()[PREFIX.len()..]
                .iter()
                .all(u8::is_ascii_digit)
        {
            Some(name)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Ancestor<'i> {
    pub(crate) p: &'i rustc_hash::FxHashSet<Solvable>,
    pub(crate) gp: Option<&'i Self>,
}

#[derive(Clone)]
pub enum BoundedValue {
    None,
    Solved(Term),
    Bounded(Option<Term>, Option<Term>),
}
impl std::fmt::Debug for BoundedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("(None)"),
            Self::Solved(t) => t.debug_short().fmt(f),
            Self::Bounded(a, b) => write!(
                f,
                "({:?} <= _ <= {:?})",
                a.as_ref().map(Term::debug_short),
                b.as_ref().map(Term::debug_short)
            ),
        }
    }
}

#[derive(Clone)]
pub struct Solvable {
    name: Id,
    pub(crate) solution: BoundedValue,
    pub(crate) tp: BoundedValue,
}
impl std::fmt::Debug for Solvable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} := {:?} (: {:?})", self.name, self.solution, self.tp)
    }
}
impl PartialEq for Solvable {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Solvable {}
impl Borrow<Id> for Solvable {
    #[inline]
    fn borrow(&self) -> &Id {
        &self.name
    }
}
impl std::hash::Hash for Solvable {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'t, Split: SplitStrategy> CheckRef<'t, '_, Split> {
    #[must_use]
    pub fn new_solvable(&mut self) -> Variable {
        let i = self
            .top
            .implicits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let name: Id =
            // SAFETY: is a valid Id
            unsafe { format!("{PREFIX}{i}").parse().unwrap_unchecked() };
        self.add_solvable(name.clone());
        Variable::Name {
            name,
            notated: None,
        }
    }
    pub(crate) fn solve_equality(&mut self, unk: &Id, solution: &'t Term) -> Option<bool> {
        self.comment(format!("solving unknown {unk}"));
        let Some(unks) = self.get_solvable(unk) else {
            self.failure("Unknown unknown!");
            return Some(false);
        };
        if let BoundedValue::Solved(tm) = &unks.solution {
            let tm = tm.clone();
            self.comment("already solved");
            return self.scoped(|slf| slf.check_equality(&tm, solution));
        }
        self.comment("Solved");
        self.solve(unk.clone(), solution.clone());
        Some(true)
    }

    pub(crate) fn solve_upper_bound(&mut self, unk: &Id, bound: &'t Term) -> Option<bool> {
        self.comment(format!("solving boundaries of unknown {unk}"));
        let Some(unks) = self.get_solvable(unk) else {
            self.failure("Unknown unknown!");
            return Some(false);
        };
        if let BoundedValue::Solved(tm) = &unks.solution {
            let tm = tm.clone();
            self.comment("already solved");
            return self.scoped(|slf| slf.check_subtype(&tm, bound));
        }
        /*
        if let BoundedValue::Bounded(lower, upper) = &unks.solution {
            let lower = lower.clone();
            let upper = upper.clone();
            drop(unks);
            return context.in_branch(|context| {
                if let Some(lower) = lower {
                    trace.comment("Checking against previous lower bound");
                    let r = self.check_subtype(trace, context, &lower, sup);
                    if r != Some(true) {
                        return r;
                    }
                }
                if let Some(upper) = upper {
                    trace.comment("Checking against previous upper bound");
                    let r = self.check_subtype(trace, context, &upper, sup);
                    if r != Some(true) {
                        return r;
                    }
                }
            });
        }

        trace.comment(format!("Solving upper type bound of {unk}"));
         */
        self.comment("Solved");
        self.solve(unk.clone(), bound.clone());
        Some(true)
    }

    pub(crate) fn solve_lower_bound(&mut self, unk: &Id, bound: &'t Term) -> Option<bool> {
        self.comment(format!("solving boundaries of unknown {unk}"));
        let Some(unks) = self.get_solvable(unk) else {
            self.failure("Unknown unknown!");
            return Some(false);
        };

        // todo boundary checks
        //
        if let BoundedValue::Solved(tm) = &unks.solution {
            let tm = tm.clone();
            self.comment("already solved");
            return self.scoped(|slf| slf.check_subtype(bound, &tm));
        }
        self.comment("Solved");
        self.solve(unk.clone(), bound.clone());
        Some(true)
    }

    pub(crate) fn merge_solutions(&mut self, solutions: rustc_hash::FxHashSet<Solvable>) {
        for s in solutions {
            self.solutions.remove(&s);
            self.solutions.insert(s);
        }
    }
    pub(crate) fn add_solvable(&mut self, name: Id) {
        self.solutions.insert(Solvable {
            name,
            solution: BoundedValue::None,
            tp: BoundedValue::None,
        });
    }
    pub(crate) fn get_solvable(&self, name: &Id) -> Option<&Solvable> {
        fn get<'i>(
            sols: &'i rustc_hash::FxHashSet<Solvable>,
            anc: Option<Ancestor<'i>>,
            name: &Id,
        ) -> Option<&'i Solvable> {
            sols.get(name)
                .or_else(|| anc.and_then(|Ancestor { p, gp }| get(p, gp.copied(), name)))
        }
        get(self.solutions, self.parent_solutions, name)
    }
    pub(crate) fn solve(&mut self, name: Id, solution: Term) {
        let solution = self.subst(solution);
        let tp = self
            .get_solvable(&name)
            .map_or(BoundedValue::None, |e| e.tp.clone());
        self.solutions.remove(&name);
        let ne = Solvable {
            name,
            solution: BoundedValue::Solved(solution),
            tp,
        };
        self.solutions.insert(ne);
    }
    pub(crate) fn solve_type(&mut self, name: Id, tp: Term) {
        let solution = self
            .get_solvable(&name)
            .map_or(BoundedValue::None, |e| e.solution.clone());
        let ne = Solvable {
            name,
            solution,
            tp: BoundedValue::Solved(tp),
        };
        self.solutions.remove(&ne);
        self.solutions.insert(ne);
    }

    pub(crate) fn subst(&self, term: Term) -> Term {
        let freevars = term.free_variables();
        if freevars.is_empty() {
            drop(freevars);
            return term;
        }
        let mut ret = smallvec::SmallVec::<_, 2>::new();
        let mut curr = (&*self.solutions, self.parent_solutions);
        loop {
            for v in curr.0 {
                if let BoundedValue::Solved(tm) = &v.solution
                    && freevars.iter().any(|f| f.name() == v.name.as_ref())
                {
                    ret.push((v.name.to_string(), tm.clone()));
                }
            }
            if let Some(Ancestor { p, gp }) = curr.1 {
                curr = (p, gp.copied());
            } else {
                break;
            }
        }
        drop(freevars);
        term / ret.as_slice()
    }
}
