use super::{
    queue::{RunningQueue, TaskMap},
    Queue,
};

impl Queue {
    #[allow(clippy::significant_drop_in_scrutinee)]
    pub(crate) fn my_sort(map: &TaskMap, state: &mut RunningQueue) {
        let tasks = map.map.values().collect::<Vec<_>>();
        let buildask = tasks[0];
        let steps = buildask.steps();
    }
}
