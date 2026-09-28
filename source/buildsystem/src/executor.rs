use std::sync::Arc;

use tokio::sync::{
    Semaphore,
    mpsc::{Receiver, Sender},
};

pub struct Executor<T> {
    // The executor also know about the actual task states
    // This is sempahore which says how many tasks can be executed
    permits: Arc<Semaphore>,
    rec: Receiver<T>,
    sender: Sender<State<T>>,
}

impl<T: Clone + Send + 'static> Executor<T> {
    pub fn new(permits: usize, rec: Receiver<T>, sender: Sender<State<T>>) -> Self {
        Self {
            permits: Arc::new(Semaphore::const_new(permits)),
            rec,
            sender,
        }
    }

    // Consume tasks from the scheduler, run at most `permits` of them at a time,
    // and report each task's state back to the scheduler.
    //
    // `run_task` is the actual (blocking) work: run the task and return Ok(())
    // on success. It is executed via `spawn_blocking`, i.e. on tokio's dedicated
    // blocking thread pool, so the async worker threads are never stalled by a
    // long-running build step. The loop ends when the scheduler closes the task
    // channel.
    pub async fn run<F>(mut self, run_task: F)
    where
        F: Fn(T) -> Result<(), TaskFailed> + Send + Sync + 'static,
    {
        let run_task = Arc::new(run_task);
        while let Some(task) = self.rec.recv().await {
            // Back-pressure: do not pull the next task off the channel until a
            // permit is free. The owned permit is moved into the child task, so
            // the slot is released automatically when that task finishes.
            let permit = self
                .permits
                .clone()
                .acquire_owned()
                .await
                .expect("executor semaphore closed");
            let sender = self.sender.clone();
            let run_task = run_task.clone();
            let report = task.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let _ = sender.send(State::running(&report)).await;
                // run the actual work on the blocking pool, not a worker thread
                let state = match tokio::task::spawn_blocking(move || run_task(task)).await {
                    Ok(Ok(())) => State::finished(&report),
                    Ok(Err(_)) | Err(_) => State::failed(&report),
                };
                let _ = sender.send(state).await;
            });
        }
    }
}

// A task that failed to execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskFailed;

pub struct State<T> {
    task: T,
    taskstate: ExecutorState,
}

impl<T: Clone> State<T> {
    pub fn running(task: &T) -> Self {
        Self {
            task: task.clone(),
            taskstate: ExecutorState::Running,
        }
    }

    pub fn finished(task: &T) -> Self {
        Self {
            task: task.clone(),
            taskstate: ExecutorState::Finished,
        }
    }

    pub fn failed(task: &T) -> Self {
        Self {
            task: task.clone(),
            taskstate: ExecutorState::Failed,
        }
    }

    pub fn task(&self) -> &T {
        &self.task
    }

    pub fn taskstate(&self) -> ExecutorState {
        self.taskstate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorState {
    Running,
    Queued,
    Failed,
    None,
    Finished,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test(flavor = "multi_thread")]
    async fn executor_runs_and_reports_states() {
        let (tx, rx) = tokio::sync::mpsc::channel::<u64>(16);
        let (stx, mut srx) = tokio::sync::mpsc::channel::<State<u64>>(16);

        let executor = Executor::new(2, rx, stx);
        let ran = std::sync::Arc::new(AtomicUsize::new(0));
        let ran_inner = ran.clone();
        let handle = tokio::spawn(async move {
            executor
                .run(move |_t: u64| {
                    let ran = ran_inner.clone();
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    ran.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await;
        });

        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
        drop(tx);

        // Draining until the channel closes waits for the *last* task to report,
        // so this doubles as the join for the spawned children.
        let mut running = 0;
        let mut finished = 0;
        while let Some(s) = srx.recv().await {
            match s.taskstate() {
                ExecutorState::Running => running += 1,
                ExecutorState::Finished => finished += 1,
                other => panic!("unexpected state {other:?}"),
            }
        }
        handle.await.unwrap();

        assert_eq!(ran.load(Ordering::SeqCst), 10);
        assert_eq!(running, 10);
        assert_eq!(finished, 10);
    }
}
