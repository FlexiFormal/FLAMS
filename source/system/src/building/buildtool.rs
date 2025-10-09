use super::{
    queue::{RunningQueue, TaskMap},
    Queue,
};

mod tests {
    use crate::building::{BuildTask, TaskState};

    use super::*;

    pub fn test_buildtool(map: &TaskMap, state: &mut RunningQueue) {
        let RunningQueue {
            queue,
            running,
            blocked,
            done,
            failed,
            ..
        } = state;
        let tasks = map.map.values().collect::<Vec<_>>();
        while !tasks.is_empty() {
            for i in &tasks {
                if let Some(x) = i.steps().iter().any(|st| {
                    let ans = st.0.state.read();
                    *ans == TaskState::Failed
                }) {
                    failed.push(i.clone());
                }
            }
        }
        // now in each build task there is some kind of dependency with other build task
        // there is no failure assumption
    }
}
