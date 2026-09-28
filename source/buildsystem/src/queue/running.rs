use std::collections::{HashMap, VecDeque};

use flams_math_archives::formats::BuildTargetId;
use ftml_ontology::utils::time::{Eta, Timestamp};
use petgraph::{adj::NodeIndex, graph::DiGraph};

use crate::taskmap::buildtask::{BuildTask, buildtaskid::BuildTaskId};

#[derive(Debug)]
pub struct RunningQueue {
    pub(crate) queue: VecDeque<BuildTask>,
    pub(crate) blocked: Vec<BuildTask>,
    pub(crate) done: Vec<BuildTask>,
    pub(super) failed: Vec<BuildTask>,
    pub(crate) running: Vec<BuildTask>,
    timer: Timer,
    /// Cached `kosaraju_scc` result from `sort_graph`/`get_next_i_graph`'s
    /// cycle-breaking fallback, keyed by the stable `StepId` rather than
    /// `NodeIndex` - `graph::build_graph` returns a fresh `DiGraph` on every
    /// call, so `NodeIndex` values aren't stable across rebuilds the way
    /// they are in `buildsystem::Scheduler`'s persistent `StableDiGraph`.
    /// The `usize` is the total known-step count at computation time, used
    /// to detect a graph that's grown (e.g. a fresh `enqueue_archive` on an
    /// already-running queue) and force a recompute rather than silently
    /// returning a stale decomposition.
    pub(super) sccs: Option<(usize, Vec<Vec<(BuildTaskId, BuildTargetId)>>)>,
    pub(super) dep_graph: DiGraph<(BuildTaskId, BuildTargetId), bool>,
    pub(super) in_degree_store: HashMap<(BuildTaskId, BuildTargetId), NodeIndex>,
}
impl RunningQueue {
    fn new(total: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            failed: Vec::new(),
            blocked: Vec::new(),
            done: Vec::new(),
            running: Vec::new(),
            timer: Timer::new(total),
            sccs: None,
            dep_graph: DiGraph::new(),
            in_degree_store: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct FinishedQueue {
    pub(super) done: Vec<BuildTask>,
    pub(super) failed: Vec<BuildTask>,
}

#[derive(Debug)]
struct Timer {
    started: Timestamp,
    steps: usize,
    done: usize,
}
impl Timer {
    fn new(total: usize) -> Self {
        Self {
            started: Timestamp::now(),
            steps: total,
            done: 0,
        }
    }
    #[allow(clippy::cast_precision_loss)]
    fn update(&mut self, dones: u8) -> Eta {
        self.done += dones as usize;
        let avg = self.started.since_now() * (1.0 / (self.done as f64));
        let time_left = avg * ((self.steps - self.done) as f64);
        Eta {
            time_left,
            done: self.done,
            total: self.steps,
        }
    }
}
