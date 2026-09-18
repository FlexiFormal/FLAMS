use std::collections::HashMap;

use flams_math_archives::formats::BuildTargetId;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    Graph,
};

use tracing::instrument;
#[derive(Debug)]
pub enum GraphScope {
    All,
    PendingOnly,
}

use crate::building::{queue::TaskMap, BuildTaskId, TaskState};
pub type Step = (BuildTaskId, BuildTargetId);
#[instrument(name = "dependency graph", level = "debug")]
pub fn build_graph(
    map: &TaskMap,
    scope: GraphScope,
    g: &mut DiGraph<Step, bool>,
    store: &mut HashMap<Step, NodeIndex>,
) {
    let to_skip = matches!(scope, GraphScope::PendingOnly);
    for i in map.map.values() {
        let mut prev_step: Option<Step> = None;
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
                    super::Dependency::Resolved { task, step, strict } => {
                        if to_skip {
                            let st = task.get_step(*step).expect("impossible");
                            match st.state.get() {
                                TaskState::Done => {
                                    continue;
                                }
                                TaskState::Failed => {
                                    tracing::debug!("dependency for task failed not adding task to the graph task: {},target : {}",task.archive(),step.name);
                                    dep_failed = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        let dep_step: Step = (task.get_id(), *step);
                        dep_edges.push((dep_step, *strict));
                    }
                    _ => (),
                }
            }
            if dep_failed {
                break;
            }
            let node: Step = (i.get_id(), step.target);
            let idx = g.add_node(node);
            store.insert(node, idx);
            if let Some(x) = prev_step {
                let prev_node = store.get(&x).expect("impossible");
                g.add_edge(*prev_node, idx, true);
            }

            prev_step = Some(node);

            for (dep_step, strict) in dep_edges {
                let idx2 = store
                    .entry(dep_step)
                    .or_insert_with(|| g.add_node(dep_step));
                g.update_edge(*idx2, idx, strict);
            }

            // for j in step.dependents.read().iter() {
            //     let idx2 = store.entry(*j).or_insert_with(|| g.add_node(*j));
            //     g.update_edge(idx, *idx2, true);
            // }
        }
    }
}
