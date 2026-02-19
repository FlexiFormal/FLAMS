use ftml_ontology::terms::{ApplicationTerm, Argument, ComponentVar, MaybeSequence, Term};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{rules::InhabitableRule, split::SplitStrategy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapInhabitableRule(pub SymbolUri);
impl SizedSolverRule for MapInhabitableRule {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("{ x:A, INH f(x), s:A* } ⊢ INH ", &self.0, "(s,f)")
    }
}

impl<Split: SplitStrategy> InhabitableRule<Split> for MapInhabitableRule {
    fn applicable(&self, term: &ftml_ontology::terms::Term) -> bool {
        if let Term::Application(app) = term
            && let Term::Symbol { uri, .. } = &app.head
        {
            *uri == self.0 && app.arguments.len() == 2
        } else {
            false
        }
    }
    fn apply<'t>(
        &self,
        mut checker: crate::CheckRef<'t, '_, Split>,
        term: &'t ftml_ontology::terms::Term,
    ) -> Option<bool> {
        let Term::Application(app) = term else {
            return None;
        };
        let [Argument::Sequence(seq), Argument::Simple(f)] = &*app.arguments else {
            checker.failure("arguments don't match");
            return None;
        };
        let seqtp = match seq {
            MaybeSequence::One(t) => checker.infer_type(t)?,
            MaybeSequence::Seq(ts) => {
                let mut curr = None;
                for t in ts {
                    if !checker.scoped::<Option<bool>>(|checker| {
                        let r = checker.infer_type(t)?;
                        if let Some(c) = &curr {
                            if !checker.scoped(|checker| checker.check_equality(c, &r))? {
                                return None;
                            };
                        } else {
                            curr = Some(r);
                        }
                        Some(true)
                    })? {
                        return None;
                    }
                }
                curr?
            }
        };
        let (v, _) = f.fresh_variable(&crate::DUMMY, None);
        checker.extend_context(ComponentVar {
            var: v.clone(),
            tp: Some(seqtp),
            df: None,
        });
        let nt = Term::Application(ApplicationTerm::new(
            f.clone(),
            Box::new([Argument::Simple(Term::Var {
                variable: v,
                presentation: None,
            })]),
            None,
        ));
        checker.scoped(|checker| checker.check_inhabitable(&nt))
    }
}
