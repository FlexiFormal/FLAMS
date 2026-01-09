use std::borrow::Borrow;

use crate::{
    SmallSet,
    split::SplitStrategy,
    test::{Ancestor, CheckRef},
};
use ftml_ontology::terms::Term;
use ftml_uris::Id;

pub trait TermExtSolvable {
    fn is_solvable(&self) -> Option<&Id>;
}

impl<Split: SplitStrategy> CheckRef<'_, '_, Split> {
    pub(crate) fn merge_solutions(&mut self, solutions: rustc_hash::FxHashSet<Solvable>) {
        for s in solutions.into_iter() {
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

#[derive(Default)]
pub(crate) struct Solutions<'a> {
    branch: rustc_hash::FxHashSet<Solvable>,
    parent: Option<&'a Self>,
}
impl Solutions<'_> {
    pub fn derive(&self) -> Solutions<'_> {
        Solutions {
            branch: rustc_hash::FxHashSet::default(),
            parent: Some(self),
        }
    }
    pub fn add(&mut self, name: Id) {
        self.branch.insert(Solvable {
            name,
            solution: BoundedValue::None,
            tp: BoundedValue::None,
        });
    }
    pub fn get(&self, name: &Id) -> Option<&Solvable> {
        self.branch
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.get(name)))
    }
}

#[derive(Default)]
pub struct SolverState<'a> {
    variables: SmallSet<Solvable>,
    parent: Option<&'a Self>,
}
impl SolverState<'_> {
    pub fn add(&self, name: Id) {
        self.variables.write().insert(Solvable {
            name,
            solution: BoundedValue::None,
            tp: BoundedValue::None,
        });
    }
    pub fn solve(&self, name: Id, solution: Term) {
        let solution = self.subst(solution);
        let tp = self.get(&name).map_or(BoundedValue::None, |e| e.tp.clone());
        let mut v = self.variables.write();
        v.remove(&name);
        let ne = Solvable {
            name,
            solution: BoundedValue::Solved(solution),
            tp,
        };
        v.insert(ne);
    }
    pub fn subst(&self, term: Term) -> Term {
        let freevars = term.free_variables();
        if freevars.is_empty() {
            drop(freevars);
            return term;
        }
        let mut ret = smallvec::SmallVec::<_, 2>::new();
        let mut curr = self;
        loop {
            for v in curr.variables.read().iter() {
                if let BoundedValue::Solved(tm) = &v.solution
                    && freevars.iter().any(|f| f.name() == v.name.as_ref())
                {
                    ret.push((v.name.to_string(), tm.clone()));
                }
            }
            if let Some(p) = curr.parent {
                curr = p;
            } else {
                break;
            }
        }
        drop(freevars);
        term / ret.as_slice()
    }
    pub fn solve_type(&self, name: Id, tp: Term) {
        let solution = self
            .get(&name)
            .map_or(BoundedValue::None, |e| e.solution.clone());
        let ne = Solvable {
            name,
            solution,
            tp: BoundedValue::Solved(tp),
        };
        self.variables.write().insert(ne);
    }
    pub(crate) fn get(&self, name: &Id) -> Option<impl std::ops::Deref<Target = Solvable>> {
        if let Ok(v) = parking_lot::RwLockReadGuard::try_map(self.variables.read(), |m| m.get(name))
        {
            return Some(v);
        }
        self.parent.and_then(|p| p.get(name))
    }
}

#[derive(Clone, Debug)]
pub enum BoundedValue {
    None,
    Solved(Term),
    Bounded(Option<Term>, Option<Term>),
}

#[derive(Clone)]
pub struct Solvable {
    name: Id,
    pub(crate) solution: BoundedValue,
    pub(crate) tp: BoundedValue,
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
