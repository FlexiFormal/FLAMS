use std::collections::HashMap;

use flams_math_archives::formats::BuildTargetId;
use petgraph::Direction::Incoming;

use crate::building::{
    graphmap::{self, GraphScope::All},
    queue::{QueueState, RunningQueue, TaskMap},
    BuildTask, BuildTaskId, Queue, TaskState,
};

impl Queue {
    pub fn sorting_my(map: &TaskMap, queue: &mut RunningQueue) {
        // this function sorts the tasks into the queues to begin with
        let RunningQueue {
            queue,
            blocked,
            dep_graph,
            in_degree_store,
            ..
        } = queue;
        // here it update the build graph stored in the queue

        graphmap::build_graph(map, All, dep_graph, in_degree_store);
        let map_task: HashMap<BuildTaskId, BuildTask> = map
            .map
            .iter()
            .map(|(_, v)| (v.get_id(), v.clone()))
            .collect();
        // for i in dep_graph.node_indices() {
        //     let incoming = dep_graph
        //         .edges_directed(i, Incoming)
        //         .filter(|e| *e.weight())
        //         .count();
        //     in_degree_store.insert(i, incoming);
        // }
        // for i in in_degree_store.iter() {
        //     let task = dep_graph
        //         .node_weight(*i.0)
        //         .expect("graph must contain the task");
        //     let task_queue = map_task.get(&task.0).expect("all tasks should exist");
        //     let target = task_queue
        //         .get_step(task.1)
        //         .expect("failed to get build step during graph generation");
        //     if *i.1 == 0 {
        //         target.0.state.set(TaskState::Queued);
        //         queue.push_back(task_queue.clone());
        //     } else {
        //         target.0.state.set(TaskState::Blocked);
        //         blocked.push(task_queue.clone());
        //     }
        // }
    }

    pub fn get_next_async_i(&self) -> Result<Option<(BuildTask, BuildTargetId)>, ()> {
        let mut state = self.0.state.write();
        let QueueState::Running(RunningQueue {
            queue,
            blocked,
            done,
            failed,
            running,
            sccs,
            dep_graph,
            in_degree_store,
            ..
        }) = &mut *state
        else {
            unreachable!()
        };

        if queue.is_empty() || blocked.is_empty() || running.is_empty() {
            return Ok(None);
        }

        if let Some(x) = queue.pop_back() {}

        Ok(None)
    }
}
