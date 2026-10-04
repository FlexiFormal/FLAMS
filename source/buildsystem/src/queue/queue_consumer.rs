/// Async consumer: waits for tasks to become available in queue, pops them,
/// returns them for execution. When queue is empty, waits for a notification
/// before retrying.
use crate::{
    queue::{Queue, QueueState},
    taskmap::buildtask::{BuildTask, buildtaskid::BuildTaskId},
};
use flams_math_archives::formats::BuildTargetId;
use tracing::instrument;

impl Queue {
    /// Pop the next task from the queue, or wait until one becomes available.
    /// Returns `Some((task, target_id))` if a task is ready to run, or `None`
    /// when the queue is finished and no more work is coming.
    ///
    /// This is the async consumer that drives the execution loop. It works
    /// alongside `schedule_newly_ready()`: after task completion, the scheduler
    /// promotes blocked tasks to queue and broadcasts a message. This function
    /// wakes up, checks if anything new is in the queue, and pops it.
    #[instrument(level = "debug", parent = &self.0.span, skip_all, name = "get_next_async")]
    pub async fn get_next_async(&self) -> Option<(BuildTask, BuildTargetId)> {
        let mut listener = self.listener();

        loop {
            {
                let mut state = self.0.state.write();
                let QueueState::Running(running) = &mut *state else {
                    // Not running, or finished — no more work
                    return None;
                };

                // Try to pop from queue (non-blocking)
                if let Some(task) = running.queue.pop_front() {
                    // Got a task; mark it as Running and add to running bucket
                    running.running.push(task.clone());

                    // Extract the target ID — for now, assume the first non-Done step
                    // In a production system, you'd track "current step" per task
                    let target = task
                        .steps()
                        .iter()
                        .find(|s| !matches!(s.0.state.get(), crate::taskmap::buildtask::taskstate::TaskState::Done))
                        .map(|s| s.0.target)
                        .unwrap_or_else(|| {
                            // All steps are done (shouldn't happen if queued, but defensive)
                            task.steps()[0].0.target
                        });

                    return Some((task, target));
                }

                // Queue is empty. Check if we're done entirely.
                if running.blocked.is_empty() && running.running.is_empty() {
                    // No pending work left; transition to finished
                    return None;
                }

                // Otherwise, drop lock and wait for notification
                drop(state);
            }

            // Wait for a message (from schedule_newly_ready or task completion)
            // This is the async part — polling would block the executor here,
            // but .await gives up the thread so other tasks can run.
            let _ = listener.read().await;

            // Loop back, reacquire lock, and try queue again
        }
    }
}
