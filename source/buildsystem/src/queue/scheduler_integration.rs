/// After a task completes (Done/Failed/Blocked), call the scheduler to promote
/// any newly-ready tasks from blocked → queue, then broadcast so waiting
/// get_next_async() wakes up and tries again.
use crate::{
    graph::scheduler::Scheduler,
    queue::{Queue, QueueState},
};
use tracing::instrument;

impl Queue {
    /// Reschedule after task completion: call Scheduler::schedule() to promote
    /// newly-ready tasks from blocked to queue, then broadcast notification.
    ///
    /// **Precondition**: caller must hold `self.0.state.write()` lock.
    /// The lock is dropped at the end of this function.
    #[instrument(level = "debug", parent = &self.0.span, skip_all, name = "reschedule")]
    pub fn schedule_newly_ready(&self, state_lock: &mut flams_utils::parking_lot::RwLockWriteGuard<crate::queue::QueueState>) {
        let QueueState::Running(running) = &mut **state_lock else {
            return;
        };

        let map = self.0.map.read();

        // Build scheduler from current state. Since we've already mutated
        // running.queue via completion logic, passing &mut running to init
        // will call schedule() again internally to find any newly-ready work.
        let _scheduler = Scheduler::init(&mut *running, &map);

        drop(map);
        drop(state_lock); // release lock before broadcasting

        // Broadcast a "state changed" message to wake any waiting get_next_async().
        // The listener will retry the queue pop. Use lazy_send to only broadcast
        // if someone's actually listening.
        self.0.sender.lazy_send(|| {
            // Send a synthetic message indicating work may be newly available
            // Listeners respond by looping back to check queue again
            crate::queue::entry::QueueMessage::Started {
                running: Vec::new(),
                queue: Vec::new(),
                blocked: Vec::new(),
                failed: Vec::new(),
                done: Vec::new(),
            }
        });
    }
}
