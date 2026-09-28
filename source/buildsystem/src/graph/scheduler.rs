use std::collections::HashMap;

use flams_math_archives::formats::BuildTargetId;
use ftml_uris::{ArchiveId, UriPath};
use petgraph::{
    Direction::Incoming,
    graph::{DiGraph, NodeIndex},
};

use crate::{
    graph::scheduler::SchedulerState::{AllDone, Pending},
    queue::running::RunningQueue,
    taskmap::{
        TaskMap,
        buildtask::{buildtaskid::BuildTaskId, dependency::Dependency, taskstate::TaskState},
    },
};

// A scheduler with a reference to Running queue and TaskMap , which owns a Digraph which should be
// populated during initialization we make new function not public but a seperated .init() function
// for full initialization and populating the Digraph
pub struct Scheduler<'a> {
    queue: &'a mut RunningQueue,
    map: &'a TaskMap,
    graph: DiGraph<(BuildTaskId, BuildTargetId), bool>,
    reverse_map: HashMap<BuildTaskId, (ArchiveId, UriPath)>,
}

impl<'a> Scheduler<'a> {
    fn new(queue: &'a mut RunningQueue, map: &'a TaskMap) -> Self {
        let reverse_map = map
            .map
            .iter()
            .map(|(k, v)| (v.get_id(), k.clone()))
            .collect();
        Self {
            queue,
            map,
            graph: DiGraph::new(),
            reverse_map,
        }
    }
    // this initialized the queue
    pub fn init(queue: &'a mut RunningQueue, map: &'a TaskMap) -> Self {
        let mut scheduler = Self::new(queue, map);
        let mut node_map = HashMap::new();
        scheduler.build_graph(GraphScope::All, &mut node_map);
        while let Ok(x) = scheduler.get_next_maybe() {
            let k = scheduler.reverse_map.get(&x.0).expect("impossible");
            let taks = scheduler.map.map.get(k).unwrap();
            taks.get_step(x.1)
                .expect("not possible")
                .state
                .set(TaskState::Queued);
            scheduler.queue.queue.push_back(taks.clone());
        }
        for i in map.map.values() {
            if !scheduler.queue.queue.contains(i) {
                scheduler.queue.blocked.push(i.clone());
            }
        }
        scheduler
    }

    fn build_graph(
        &mut self,
        scope: GraphScope,
        store: &mut HashMap<(BuildTaskId, BuildTargetId), NodeIndex>,
    ) {
        let to_skip = matches!(scope, GraphScope::PendingOnly);
        for i in self.map.map.values() {
            let mut prev_step: Option<(BuildTaskId, BuildTargetId)> = None;
            for step in i.steps() {
                if to_skip {
                    match step.state.get() {
                        TaskState::Done => {
                            prev_step = None;
                            continue;
                        }
                        TaskState::Failed => {
                            tracing::debug!(
                                "the task failed to add to build graph task : {} target : {}",
                                i.archive(),
                                step.target.name
                            );
                            break;
                        }
                        _ => {}
                    }
                }
                tracing::debug!(
                    "step successfully added to build graph task : {} target : {} ",
                    i.archive(),
                    step.target.name
                );
                let mut dep_edges = Vec::new();
                let mut dep_failed = false;

                // here requires gives you dep edge from req to task
                for i in step.requires.read().iter() {
                    match i {
                        Dependency::Resolved { task, step, strict } => {
                            if to_skip {
                                let st = task.get_step(*step).expect("impossible");
                                match st.state.get() {
                                    TaskState::Done => {
                                        continue;
                                    }
                                    TaskState::Failed => {
                                        tracing::debug!(
                                            "dependency for task failed not adding task to the graph task: {},target : {}",
                                            task.archive(),
                                            step.name
                                        );
                                        dep_failed = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            let dep_step = (task.get_id(), *step);
                            dep_edges.push((dep_step, *strict));
                        }
                        _ => (),
                    }
                }
                if dep_failed {
                    break;
                }
                let node = (i.get_id(), step.target);
                let idx = self.graph.add_node(node);
                store.insert(node, idx);
                if let Some(x) = prev_step {
                    let prev_node = store.get(&x).expect("impossible");
                    self.graph.add_edge(*prev_node, idx, true);
                }

                prev_step = Some(node);

                for (dep_step, strict) in dep_edges {
                    let idx2 = store
                        .entry(dep_step)
                        .or_insert_with(|| self.graph.add_node(dep_step));
                    self.graph.update_edge(*idx2, idx, strict);
                }
            }
        }
    }

    fn get_node_idx(
        &mut self,
        node_map: &mut HashMap<(BuildTaskId, BuildTargetId), NodeIndex>,
        tasks: (BuildTaskId, BuildTargetId),
    ) -> NodeIndex {
        *node_map
            .entry(tasks.clone())
            .or_insert_with(|| self.graph.add_node(tasks))
    }

    // This function operates on the DiGraph from petgraph
    // Here assumption is that the task is always in the blocked queue ?
    pub fn get_next_maybe(&self) -> Result<(BuildTaskId, BuildTargetId), SchedulerState> {
        let mut count = 0;
        for i in self.graph.node_indices().filter(|f| {
            !matches!(
                self.is_task(&self.graph[*f]),
                TaskState::Done | TaskState::Failed | TaskState::Running | TaskState::Queued
            )
        }) {
            count += 1;
            let mut incoming = self.graph.neighbors_directed(i, Incoming);
            if incoming.all(|n| self.is_task(&self.graph[n]) == TaskState::Done) {
                return Ok(self.graph[i]);
            }
        }
        Err(if count == 0 {
            SchedulerState::AllDone
        } else {
            SchedulerState::Pending
        })
    }
    // This should be called continuously Since this function should actually manupulate the Queue
    // this handle gives the task which is then moved to run_task function
    pub fn schedule(&mut self) {
        let next = self.get_next_maybe();
        match next {
            Ok(t) => {
                if let Some(x) = self.queue.blocked.iter().position(|x| x.get_id() == t.0) {
                    let r = self.queue.blocked.remove(x);
                    r.get_step(t.1)
                        .expect("not possible")
                        .state
                        .set(TaskState::Queued);
                    self.queue.queue.push_back(r);
                }
            }
            Err(AllDone) => {
                // TODO here we just log saying everything ran
            }
            // TODO How do we filter ?
            Err(Pending) => todo!("Here we need to run kosaraju"),
        }
    }

    pub fn is_task(&self, task: &(BuildTaskId, BuildTargetId)) -> TaskState {
        let key = self.reverse_map.get(&task.0).expect("key not found ?");
        let task1 = self.map.map.get(key).expect("not possible");
        task1
            .get_step(task.1)
            .map(|f| f.state.get())
            .expect("impossible")
    }
}

// This is to determine whether to run kosaraju or not
#[derive(Debug)]
pub enum SchedulerState {
    AllDone,
    Pending,
}

#[derive(Debug)]
pub enum GraphScope {
    All,
    PendingOnly,
}
