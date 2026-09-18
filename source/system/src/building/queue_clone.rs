//! Two alternative `Queue` methods, kept entirely out of `queue.rs`/
//! `queueing.rs`/`graphmap.rs` so nothing there gets touched - both operate
//! on the *real* `Queue`/`RunningQueue`, not a shadow copy, since
//! `QueueI.state` and `Queue::listener()` are already `pub`.
//!
//!  - `sort_by_indegree`: seeds the initial ready set directly off the
//!    dependency graph's in-degree ("zero incoming edges" = no unmet
//!    dependency), instead of `sort`/`sort_graph`'s per-task state-machine
//!    scan. Same signature shape as `sort`/`mysort`/`sort_graph`, so it's a
//!    drop-in alternative - but nothing currently calls it: `Queue::start`
//!    (in `queue.rs`, not touched) still hardcodes `Self::sort_graph`.
//!    Same situation `mysort` is already in.
//!
//!  - `get_next_reactive`: a genuinely async alternative to
//!    `get_next_async` (which polls `get_next_i_graph` on a 100ms
//!    sleep-and-repeat loop). This suspends on `Queue::listener()` instead
//!    - the *same* broadcast channel `run_task`/`get_next_i_graph` already
//!    send `QueueMessage::TaskSuccess`/`TaskFailed`/`TaskBlocked`/... on
//!    for every real state change - so it resumes exactly when real
//!    progress that could matter has happened, not on a fixed timer. This
//!    one is usable right now, independent of which sort function seeded
//!    the queue.

use std::collections::{HashMap, HashSet};

use flams_math_archives::formats::BuildTargetId;
use petgraph::{
    algo::kosaraju_scc,
    graph::NodeIndex,
    visit::NodeFiltered,
    Direction::{Incoming, Outgoing},
};

use super::{
    graphmap::{self, GraphScope},
    queue::{Queue, QueueState, RunningQueue, TaskMap},
    BuildTask, BuildTaskId, TaskState,
};

impl Queue {
    /// Builds directly into `state.dep_graph` (the persistent field - no
    /// throwaway local graph) using `GraphScope::All`, so the graph always
    /// contains every step regardless of current state; nothing here ever
    /// rebuilds it.
    ///
    /// `kosaraju_scc` is *not* run unconditionally the way `sort_graph`
    /// does it - the cheap "does anything have zero incoming edges at
    /// all" check runs first, and kosaraju only gets paid for if that
    /// finds nothing (a fully acyclic graph, by far the common case,
    /// never touches it). When it does run, it's over a `NodeFiltered`
    /// view of `dep_graph` that excludes anything already in `done` -
    /// filtered at query time, rather than physically removing nodes or
    /// rebuilding the graph.
    ///
    /// The zero-in-degree check itself is *not* done-filtered - safe only
    /// because `done` is guaranteed empty at the point this runs (the
    /// one-time initial seed, right after `Queue::start()`). If this ever
    /// gets called again later in a queue's life, that check would also
    /// need the same done-filtering the kosaraju fallback already has.
    pub fn sort_by_indegree(map: &TaskMap, state: &mut RunningQueue) {
        let RunningQueue {
            queue,
            blocked,
            done,
            dep_graph,
            in_degree_store,
            ..
        } = state;

        let store = graphmap::build_graph(map, GraphScope::All, dep_graph, in_degree_store);

        let by_id: HashMap<BuildTaskId, BuildTask> =
            map.map.values().map(|t| (t.get_id(), t.clone())).collect();

        let is_ready = |idx: NodeIndex| dep_graph.neighbors_directed(idx, Incoming).count() == 0;

        let mut forced_ready: HashSet<NodeIndex> = HashSet::new();
        if !in_degree_store.is_empty() && !in_degree_store.values().any(|&idx| is_ready(idx)) {
            tracing::info!(
                target: "buildqueue",
                "sort_by_indegree: nothing naturally ready ({} steps) - running kosaraju_scc to break a cycle",
                in_degree_store.len()
            );
            let done_ids: HashSet<BuildTaskId> = done.iter().map(BuildTask::get_id).collect();
            let filtered = NodeFiltered::from_fn(&*dep_graph, |idx: NodeIndex| {
                !done_ids.contains(&dep_graph[idx].0)
            });
            let all_sccs = kosaraju_scc(&filtered);
            for scc in &all_sccs {
                if scc.len() < 2 || scc.iter().any(|&n| is_ready(n)) {
                    continue;
                }
                if let Some(&n) = scc
                    .iter()
                    .max_by_key(|&&n| dep_graph.neighbors_directed(n, Outgoing).count())
                {
                    let (tid, target) = dep_graph[n];
                    if let Some(task) = by_id.get(&tid) {
                        tracing::info!(
                            target: "buildqueue",
                            "sort_by_indegree: breaking cycle (SCC size {}) by force-queuing [{}]{{{}}} :: {target}",
                            scc.len(), task.archive(), task.rel_path()
                        );
                    }
                    forced_ready.insert(n);
                }
            }
        }

        for (&(tid, target), &idx) in in_degree_store.iter() {
            let Some(task) = by_id.get(&tid) else {
                continue;
            };
            let Some(step) = task.get_step(target) else {
                continue;
            };
            if is_ready(idx) || forced_ready.contains(&idx) {
                step.state.set(TaskState::Queued);
                queue.push_back(task.clone());
            } else {
                step.state.set(TaskState::Blocked);
                blocked.push(task.clone());
            }
        }
        // This is reduntant
        // for task in map.map.values() {
        //     if task
        //         .steps()
        //         .iter()
        //         .all(|s| s.state.get() == TaskState::Done)
        //     {
        //         done.push(task.clone());
        //     }
        // }

        tracing::info!(
            target: "buildqueue",
            "sort_by_indegree: {} queued, {} blocked, {} done",
            queue.len(), blocked.len(), done.len()
        );
    }

    /// See module docs. The listener is created *before* the first
    /// dispatch check, not after, so a message broadcast in between is
    /// still queued for it to receive - `async_broadcast`'s per-receiver
    /// buffering means nothing sent after subscription is lost, even if
    /// this task hasn't called `.await` yet.
    #[cfg(feature = "tokio")]
    pub async fn get_next_reactive(&self) -> Option<(BuildTask, BuildTargetId)> {
        let mut listener = self.listener();
        loop {
            match self.try_dispatch() {
                DispatchAttempt::Dispatched(task, target) => return Some((task, target)),
                DispatchAttempt::Finished => return None,
                DispatchAttempt::Pending => {
                    if listener.read().await.is_none() {
                        // sender dropped - the queue itself is gone.
                        return None;
                    }
                }
            }
        }
    }

    /// One non-blocking attempt at dispatching, against the real
    /// `RunningQueue`. Fast path: pop from `queue` (`get_next_i_graph`'s
    /// Stage 2 - doesn't reproduce `can_be_next`'s strict-dependency race
    /// check, a private method this file can't call).
    ///
    /// Stuck fallback (nothing in `queue`/`running`, but `blocked` isn't
    /// empty): reuses `state.dep_graph` as-is - never rebuilt, same as
    /// `sort_by_indegree`. Readiness here can't mean "zero incoming
    /// edges" the way it does at initial-seed time, since edges in this
    /// `All`-scoped graph never disappear - it means "every incoming
    /// neighbor's task is already `Done`", checked against a `done`-based
    /// filter. Cheap pass first (is anything in `blocked` already ready
    /// this way); only if that finds nothing does `kosaraju_scc` run, over
    /// a `NodeFiltered` view of `dep_graph` (still no rebuild/mutation) -
    /// same lazy-kosaraju philosophy as `sort_by_indegree`, just re-tested
    /// against later, live state instead of the empty-`done` startup case.
    fn try_dispatch(&self) -> DispatchAttempt {
        let mut state = self.0.state.write();
        let QueueState::Running(RunningQueue {
            queue,
            blocked,
            running,
            done,
            dep_graph,
            in_degree_store,
            ..
        }) = &mut *state
        else {
            return DispatchAttempt::Finished;
        };

        if let Some(task) = queue.pop_front() {
            let Some(step) = task
                .steps()
                .iter()
                .find(|s| s.state.get() == TaskState::Queued)
            else {
                return DispatchAttempt::Pending;
            };
            step.state.set(TaskState::Running);
            let target = step.target;
            tracing::info!(
                target: "buildqueue",
                "dispatch (ready, indegree) [{}]{{{}}} :: {target}",
                task.archive(), task.rel_path()
            );
            running.push(task.clone());
            return DispatchAttempt::Dispatched(task, target);
        }

        if !running.is_empty() {
            return DispatchAttempt::Pending;
        }

        if blocked.is_empty() {
            return DispatchAttempt::Finished;
        }

        let done_ids: HashSet<BuildTaskId> = done.iter().map(BuildTask::get_id).collect();
        let mut node_of: HashMap<(BuildTaskId, BuildTargetId), NodeIndex> = HashMap::new();
        for idx in dep_graph.node_indices() {
            node_of.insert(dep_graph[idx], idx);
        }
        let is_ready_live = |idx: NodeIndex| {
            dep_graph
                .neighbors_directed(idx, Incoming)
                .all(|n| done_ids.contains(&dep_graph[n].0))
        };

        // Cheap pass first: did anything in `blocked` actually become
        // ready since it was last checked? Only fall to kosaraju if
        // nothing did.
        if let Some(pos) = blocked.iter().position(|t| {
            t.steps()
                .iter()
                .find(|s| s.state.get() == TaskState::Blocked)
                .and_then(|s| node_of.get(&(t.get_id(), s.target)))
                .is_some_and(|&idx| is_ready_live(idx))
        }) {
            let task = blocked.remove(pos);
            for s in task.steps() {
                s.state.set_if_is(TaskState::Blocked, TaskState::Queued);
            }

            let Some(step) = task
                .steps()
                .iter()
                .find(|s| s.state.get() == TaskState::Queued)
            else {
                return DispatchAttempt::Pending;
            };
            step.state.set(TaskState::Running);
            let target = step.target;
            tracing::info!(
                target: "buildqueue",
                "dispatch (unblocked, indegree) [{}]{{{}}} :: {target}",
                task.archive(), task.rel_path()
            );
            running.push(task.clone());
            return DispatchAttempt::Dispatched(task, target);
        }

        // Nothing in `blocked` is naturally ready either - only now is
        // kosaraju worth paying for.
        tracing::info!(
            target: "buildqueue",
            "try_dispatch: still nothing ready among {} blocked tasks - running kosaraju_scc",
            blocked.len()
        );
        let filtered = NodeFiltered::from_fn(&*dep_graph, |idx: NodeIndex| {
            !done_ids.contains(&dep_graph[idx].0)
        });
        let all_sccs = kosaraju_scc(&filtered);

        let by_id: HashMap<BuildTaskId, &BuildTask> =
            blocked.iter().map(|t| (t.get_id(), t)).collect();
        let forced = all_sccs.iter().rev().find_map(|scc| {
            if scc.len() < 2 {
                return None;
            }
            scc.iter()
                .filter(|&&n| by_id.contains_key(&dep_graph[n].0))
                .max_by_key(|&&n| dep_graph.neighbors_directed(n, Outgoing).count())
                .copied()
        });

        if let Some(node) = forced {
            let (tid, target) = dep_graph[node];
            let i = blocked
                .iter()
                .position(|t| t.get_id() == tid)
                .unwrap_or_else(|| unreachable!());
            let task = blocked.remove(i);
            for s in task.steps() {
                s.state.set_if_is(TaskState::Blocked, TaskState::Queued);
            }
            let Some(step) = task.get_step(target) else {
                return DispatchAttempt::Pending;
            };
            step.state.set(TaskState::Running);
            tracing::info!(
                target: "buildqueue",
                "dispatch (FORCED - breaking cycle, indegree) [{}]{{{}}} :: {target}",
                task.archive(), task.rel_path()
            );
            running.push(task.clone());
            return DispatchAttempt::Dispatched(task, target);
        }

        // Not a cycle - genuinely unresolvable. Fail everything left.
        tracing::info!(
            target: "buildqueue",
            "try_dispatch: not a cycle - failing {} remaining blocked task(s)",
            blocked.len()
        );
        while let Some(t) = blocked.pop() {
            for s in t.steps() {
                if s.state.get() != TaskState::Done {
                    s.state.set(TaskState::Failed);
                }
            }
        }
        DispatchAttempt::Finished
    }
}

enum DispatchAttempt {
    Dispatched(BuildTask, BuildTargetId),
    Finished,
    Pending,
}
