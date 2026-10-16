use ftml_ontology::terms::{Argument, BoundArgument, MaybeSequence, Term};
use ftml_solver_trace::SizedSolverRule;
use ftml_uris::SymbolUri;

use crate::{
    CheckRef,
    rules::{InhabitableRule, MarkerRule, UniverseRule, operators::pi::PiExtensionRule},
    split::SplitStrategy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntersectionTypeInhabitable(pub SymbolUri);
impl SizedSolverRule for IntersectionTypeInhabitable {
    fn display(&self) -> Vec<crate::trace::Displayable> {
        ftml_solver_trace::trace!("intersection types inherit inhabitability and universality")
    }
}
impl<Split: SplitStrategy> InhabitableRule<Split> for IntersectionTypeInhabitable {
    fn applicable(&self, term: &Term) -> bool {
        if let Term::Application(app) = term
            && let Term::Symbol { uri, .. } = &app.head
            && *uri == self.0
            && let [Argument::Sequence(_)] = &*app.arguments
        {
            true
        } else {
            false
        }
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        let Term::Application(app) = term else {
            return None;
        };
        let [Argument::Sequence(MaybeSequence::Seq(tps))] = &*app.arguments else {
            return None;
        };
        for t in tps {
            if !checker.check_inhabitable(t)? {
                return None;
            }
        }
        Some(true)
    }
}
impl<Split: SplitStrategy> UniverseRule<Split> for IntersectionTypeInhabitable {
    fn applicable(&self, term: &Term) -> bool {
        <Self as InhabitableRule<Split>>::applicable(&self, term)
    }
    fn apply<'t>(&self, mut checker: CheckRef<'t, '_, Split>, term: &'t Term) -> Option<bool> {
        let Term::Application(app) = term else {
            return None;
        };
        let [Argument::Sequence(MaybeSequence::Seq(tps))] = &*app.arguments else {
            return None;
        };
        for t in tps {
            if !checker.check_universe(t)? {
                return None;
            }
        }
        Some(true)
    }
}

pub fn intersect_pi_extension<Split: SplitStrategy>(
    intersect: SymbolUri,
    pi: SymbolUri,
) -> super::pi::PiExtensionRule<Split> {
    super::pi::PiExtensionRule {
        extension: intersect,
        pi,
        applicable: |slf, tp, _| {
            if let Term::Application(app) = tp
                && let Term::Symbol { uri, .. } = &app.head
                && *uri == slf.extension
                && let [Argument::Sequence(MaybeSequence::Seq(tps))] = &*app.arguments
            {
                tps.iter().all(|t| {
                    if let Term::Bound(b) = t {
                        matches!(&b.head,Term::Symbol { uri, .. } if *uri == slf.pi)
                            && b.arguments.len() == 2
                            && matches!(&b.arguments[1], BoundArgument::Simple(_))
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        },
        infer: |slf, pi, checker, tp, args, index| {
            let Argument::Simple(arg) = &args[*index] else {
                return None;
            };
            let Term::Application(app) = tp else {
                return None;
            };
            let [Argument::Sequence(MaybeSequence::Seq(tps))] = &*app.arguments else {
                return None;
            };
            for tp in tps {
                let Ok(b) = super::pi::PiInferenceRule::deconstruct_tp(&pi.0, checker, tp.clone())
                else {
                    continue;
                };
                let [_, BoundArgument::Simple(body)] = &*b.arguments else {
                    // SAFETY: invariant of deconstruct_tp
                    unsafe { std::hint::unreachable_unchecked() }
                };
                if let Some(r) = super::pi::PiInferenceRule::simple_apply(checker, &b, arg, body) {
                    *index += 1;
                    return Some(r);
                }
            }
            None
        },
    }
}
