use std::hint::unreachable_unchecked;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum TaskState {
    Running = 0,
    Queued = 1,
    Blocked = 2,
    Done = 3,
    Failed = 4,
    None = 5,
}
// I don't Understand this complicated atomictaskstate?
// read that this is useful for sharing across threads but in this case why ?
#[derive(Debug)]
pub struct AtomicTaskState(std::sync::atomic::AtomicU8);
impl AtomicTaskState {
    pub fn new(state: TaskState) -> Self {
        Self(std::sync::atomic::AtomicU8::new(state as _))
    }
    /// Sets the state to `state`, but only if it's currently `if_is` -
    /// otherwise a no-op. Callers use this to reset *some* of a task's
    /// steps (e.g. "every step that's still Blocked, back to Queued")
    /// without disturbing steps already past that point (Done/Failed/
    /// Running/etc.) - the CAS failing just means this particular step
    /// wasn't the one being targeted, not an error.
    pub fn set_if_is(&self, if_is: TaskState, state: TaskState) {
        let _ = self.0.compare_exchange(
            if_is as _,
            state as _,
            std::sync::atomic::Ordering::Release,
            std::sync::atomic::Ordering::Acquire,
        );
    }
    pub fn get(&self) -> TaskState {
        let b = self.0.load(std::sync::atomic::Ordering::Acquire);
        match b {
            0 => TaskState::Running,
            1 => TaskState::Queued,
            2 => TaskState::Blocked,
            3 => TaskState::Done,
            4 => TaskState::Failed,
            5 => TaskState::None,
            // SAFETY: impossible b< construction
            _ => unsafe { unreachable_unchecked() },
        }
    }
    pub fn set(&self, state: TaskState) {
        self.0
            .store(state as _, std::sync::atomic::Ordering::Release);
    }
}
