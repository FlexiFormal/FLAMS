//! Turns a [`TaskMap`] into a `petgraph` dependency graph suitable for
//! `buildsystem::scheduler::Scheduler::from_graph` - the adapter between
//! FLAMS's real build-task model and the generic scheduler.

use std::collections::HashMap;

use flams_math_archives::formats::BuildTargetId;
use petgraph::graph::{DiGraph, NodeIndex};

use super::{queue::TaskMap, BuildTaskId, Dependency, TaskState};

/// One node in the scheduling graph: a single build step of a single build
/// task. This is the `T` that `Scheduler<T>`/`Executor<T>` get instantiated
/// with for FLAMS.
pub type StepId = (BuildTaskId, BuildTargetId);

// we now know the logic, so if a node in a graph is still have zero edges in degree then just use

fn node(
    graph: &mut DiGraph<StepId, ()>,
    nodes: &mut HashMap<StepId, NodeIndex>,
    id: StepId,
) -> NodeIndex {
    *nodes.entry(id).or_insert_with(|| graph.add_node(id))
}

/// Build the dependency graph for everything in `map` that still needs to
/// run. An edge `dependency -> dependent` is added for:
///
///  - every resolved cross-task dependency (`Dependency::Resolved`) - the
///    only variant that carries a concrete task+step to depend on. This is
///    the same limitation `queueing.rs::sort` and `tests::find_cycles`
///    already have: `Physical`/`Logical` dependencies aren't resolved into
///    edges here either.
///  - the implicit in-task pipeline order between a task's own steps (they
///    are stored in pipeline order in `BuildTask::steps()`, e.g. pdflatex
///    before bibtex before the final check) - nothing else encodes that
///    ordering, so this adapter adds it explicitly as a synthetic edge.
///
/// A step already `Done` is left out entirely (nothing to schedule, and it
/// can't block anything - readiness is checked by state, not by presence in
/// this graph). A step that is `Failed`, or that depends - directly or via
/// the in-task pipeline order - on a step that's already `Failed`, is *also*
/// left out, for a more fundamental reason: `Scheduler` has no notion of a
/// pre-failed node (see the open "failure propagation" item), so the only
/// sound thing this adapter can do today is never let such a step reach the
/// graph at all. It can never legitimately become ready.
///
/// NOTE: `Dependency::Resolved` also carries a `strict` flag that the
/// existing scheduler (`queueing.rs::sort`) treats specially - a
/// non-strict dependency is only honored on its first, "weak" pass, then
/// dropped if it would otherwise stall things. This adapter does not
/// replicate that distinction yet: every `Resolved` dependency, strict or
/// not, becomes a hard graph edge. `Scheduler`'s SCC-based cycle-breaking
/// already forces progress on a genuine cycle, so a non-strict edge isn't
/// needed purely as a "give up" escape valve the way it is today - but if
/// `strict = false` is also meant as "purely advisory, never block on this
/// at all" (not just "breakable if it deadlocks"), that's a distinct
/// behavior this adapter doesn't provide yet.
#[must_use]
pub fn build_graph(map: &TaskMap) -> DiGraph<StepId, ()> {
    let mut graph = DiGraph::new();
    let mut nodes: HashMap<StepId, NodeIndex> = HashMap::new();

    for task in map.map.values() {
        let mut prev_idx: Option<NodeIndex> = None;
        for step in task.steps() {
            match step.state.get() {
                TaskState::Done => {
                    // Already finished: no node, and not a blocking
                    // predecessor for the next step either.
                    prev_idx = None;
                    continue;
                }
                TaskState::Failed => {
                    // Already failed: no node, and everything after it in
                    // this task's pipeline can never legitimately run
                    // either (they need this step's output), so stop
                    // walking this task's steps entirely.
                    break;
                }
                TaskState::Running | TaskState::Queued | TaskState::Blocked | TaskState::None => {}
            }

            let here = (task.get_id(), step.target);

            // Resolve cross-task dependencies *before* creating a node for
            // `here`, so an already-failed dependency can veto it outright.
            let mut dep_failed = false;
            let mut dep_edges = Vec::new();
            for dep in step.requires.read().iter() {
                if let Dependency::Resolved {
                    task: dep_task,
                    step: dep_step,
                    ..
                } = dep
                {
                    match dep_task.get_step(*dep_step).map(|s| s.state.get()) {
                        Some(TaskState::Done) => {} // already satisfied, no edge needed
                        Some(TaskState::Failed) => {
                            dep_failed = true;
                            break;
                        }
                        _ => dep_edges.push((dep_task.get_id(), *dep_step)),
                    }
                }
            }
            if dep_failed {
                break; // this and every later step in the task is unrunnable
            }

            let here_idx = node(&mut graph, &mut nodes, here);
            if let Some(p_idx) = prev_idx {
                graph.update_edge(p_idx, here_idx, ());
            }
            for dep_id in dep_edges {
                let dep_idx = node(&mut graph, &mut nodes, dep_id);
                graph.update_edge(dep_idx, here_idx, ());
            }
            prev_idx = Some(here_idx);
        }
    }
    graph
}
