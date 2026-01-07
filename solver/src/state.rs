use std::borrow::Borrow;

use ftml_ontology::terms::Term;
use ftml_uris::Id;

use crate::SmallSet;

pub trait TermExtSolvable {
    fn is_solvable(&self) -> Option<&Id>;
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
