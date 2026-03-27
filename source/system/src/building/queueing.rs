use either::Either;
use flams_math_archives::{
    backend::AnyBackend,
    formats::{BuildTargetId, FormatOrTargets},
    source_files::{FileState, SourceFile},
    Archive, MathArchive,
};
use flams_utils::{triomphe::Arc, vecmap::VecSet};
use ftml_ontology::utils::time::Eta;
use ftml_uris::UriWithArchive;
use parking_lot::RwLock;
use std::collections::hash_map::Entry;

use crate::building::{AtomicTaskState, QueueMessage};

use super::{
    queue::{Queue, QueueState, RunningQueue, TaskMap},
    BuildStep, BuildStepI, BuildTask, BuildTaskId, Dependency, TaskState,
};

impl Queue {
    #[allow(clippy::significant_drop_in_scrutinee)]
    pub(super) fn sort(map: &TaskMap, state: &mut RunningQueue) {
        let RunningQueue {
            queue,
            done,
            blocked,
            failed,
            ..
        } = state;
        let mut tasks = map.map.values().cloned().collect::<Vec<_>>();
        let mut weak = true;
        while !tasks.is_empty() {
            let mut changed = false;
            for t in &tasks {
                let mut has_failed = false;
                let Some(step) = t.steps().iter().find(|s| {
                    let state = s.0.state.get();
                    if state == TaskState::Failed {
                        has_failed = true;
                        return false;
                    }
                    !matches!(state, TaskState::Done)
                }) else {
                    if has_failed {
                        failed.push(t.clone());
                    } else {
                        done.push(t.clone());
                    }
                    continue;
                };
                let mut newstate = TaskState::Queued;
                for d in step.0.requires.read().iter() {
                    match d {
                        Dependency::Resolved { task, strict, step } if *strict || weak => {
                            match task
                                .get_step(*step)
                                .unwrap_or_else(|| unreachable!())
                                .0
                                .state
                                .get()
                            {
                                TaskState::Done
                                | TaskState::Queued
                                | TaskState::Failed
                                | TaskState::Running => (),
                                TaskState::Blocked => {
                                    newstate = TaskState::Blocked;
                                }
                                TaskState::None => {
                                    newstate = TaskState::None;
                                    break;
                                }
                            }
                        }
                        _ => (),
                    }
                }
                let mut found = false;
                if newstate == TaskState::None {
                    continue;
                }
                changed = true;
                for s in t.steps() {
                    if s == step {
                        found = true;
                        s.0.state.set(newstate);
                    } else if found {
                        s.0.state.set(TaskState::Blocked);
                    }
                }
                match newstate {
                    TaskState::Blocked => blocked.push(t.clone()),
                    TaskState::Queued => queue.push_back(t.clone()),
                    _ => (),
                }
            }
            if changed {
                tasks.retain(|t| t.steps().iter().any(|s| s.0.state.get() == TaskState::None));
            } else if weak {
                weak = false;
            } else {
                let tasks = std::mem::take(&mut tasks);
                for t in tasks {
                    for s in t.steps() {
                        s.0.state.set_if_is(TaskState::None, TaskState::Blocked);
                    }
                    blocked.push(t);
                }
            }
        }
    }

    pub(super) fn get_next(&self) -> Option<(BuildTask, BuildTargetId)> {
        loop {
            match self.get_next_i() {
                Ok(r) => return r,
                Err(()) => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }
    }

    #[cfg(feature = "tokio")]
    pub(super) async fn get_next_async(&self) -> Option<(BuildTask, BuildTargetId)> {
        loop {
            match self.get_next_i() {
                Ok(r) => return r,
                Err(()) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
    }

    fn get_next_i(&self) -> Result<Option<(BuildTask, BuildTargetId)>, ()> {
        let mut state = self.0.state.write();
        let QueueState::Running(RunningQueue {
            queue,
            blocked,
            running,
            failed,
            ..
        }) = &mut *state
        else {
            unreachable!()
        };
        if queue.is_empty() && blocked.is_empty() && running.is_empty() {
            return Ok(None);
        }
        if let Some((i, target)) = queue
            .iter()
            .enumerate()
            .find_map(|(next, e)| Self::can_be_next(e).map(|t| (next, t)))
        {
            let Some(task) = queue.remove(i) else {
                unreachable!()
            };
            task.get_step(target)
                .unwrap_or_else(|| unreachable!())
                .0
                .state
                .set(TaskState::Running);
            running.push(task.clone());
            return Ok(Some((task, target)));
        }
        if !running.is_empty() {
            drop(state);
            Err(())
            //std::thread::sleep(std::time::Duration::from_secs(1));
        } else if !blocked.is_empty() {
            if let Some((i, target)) = blocked
                .iter()
                .enumerate()
                .find_map(|(next, e)| Self::can_be_next(e).map(|t| (next, t)))
            {
                let task = blocked.remove(i);
                for s in task.steps() {
                    s.0.state.set_if_is(TaskState::Blocked, TaskState::Queued);
                }
                task.get_step(target)
                    .unwrap_or_else(|| unreachable!())
                    .0
                    .state
                    .set(TaskState::Running);
                running.push(task.clone());
                return Ok(Some((task, target)));
            }
            while let Some(t) = blocked.pop() {
                for s in t.steps() {
                    let state = s.0.state.get();
                    if state != TaskState::Done {
                        s.0.state.set(TaskState::Failed);
                    }
                }
                self.0.sender.lazy_send(|| QueueMessage::TaskFailed {
                    id: t.0.id,
                    target: t.steps().last().expect("???").0.target,
                    eta: Eta::default(),
                });
                failed.push(t);
            }
            Ok(None)
        } else {
            Ok(None)
        }
    }

    fn can_be_next(e: &BuildTask) -> Option<BuildTargetId> {
        let step =
            e.0.steps.iter().find(|step| {
                matches!(step.0.state.get(), TaskState::Queued | TaskState::Blocked)
            })?;
        for d in &step.0.requires.read().0 {
            if let Dependency::Resolved { task, step, strict } = d {
                if *strict
                    && task
                        .get_step(*step)
                        .unwrap_or_else(|| unreachable!())
                        .0
                        .state
                        .get()
                        == TaskState::Running
                {
                    return None;
                }
            }
        }
        Some(step.0.target)
    }

    #[deprecated(note = "assumes local archives")]
    pub(super) fn enqueue<'a, I: Iterator<Item = &'a SourceFile>>(
        map: &mut TaskMap,
        backend: &AnyBackend,
        archive: &Archive,
        target: FormatOrTargets,
        stale_only: bool,
        files: I,
    ) -> usize {
        let targets = match target {
            FormatOrTargets::Format(f) => f.targets,
            FormatOrTargets::Targets(t) => t,
        };
        let has_target = |f: &SourceFile, tgt: BuildTargetId| {
            f.target_state
                .iter()
                .find_map(|(k, v)| if *k == tgt { Some(v) } else { None })
                .is_some_and(|t| !stale_only || matches!(t, FileState::Stale(_) | FileState::New))
        };
        let should_queue = |f: &SourceFile| targets.iter().any(|t| has_target(f, *t));
        let mut count = 0;

        for f in files.filter(|f| should_queue(f)) {
            let key = (archive.id().clone(), f.relative_path.clone());
            let task = match map.map.entry(key) {
                Entry::Vacant(e) => {
                    count += 1;
                    let steps = targets
                        .iter()
                        .filter(|t| has_target(f, **t))
                        .map(|t| {
                            BuildStep(Arc::new(BuildStepI {
                                target: *t,
                                state: AtomicTaskState::new(TaskState::None),
                                //yields:RwLock::new(Vec::new()),
                                requires: RwLock::new(VecSet::default()),
                                dependents: RwLock::new(Vec::new()),
                            }))
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    map.total += steps.len();
                    let id = map.counter;
                    map.counter = map.counter.saturating_add(1);
                    let task = BuildTask::new(
                        BuildTaskId(id),
                        archive.uri().clone(),
                        steps,
                        match archive {
                            Archive::Local(archive) => {
                                Either::Left(archive.source_dir().join(f.relative_path.as_ref()))
                            }
                            Archive::Ext(..) => todo!("foreign archives"),
                        },
                        f.relative_path.clone(),
                    )
                    .expect("this is a bug");
                    e.insert(task.clone());
                    task
                }
                Entry::Occupied(o) => {
                    count += 1;
                    for s in o.get().steps() {
                        s.0.state.set(TaskState::None);
                    }
                    continue;
                }
            };
            let spec = task.as_build_spec(backend);
            if let FormatOrTargets::Format(fmt) = target {
                for (f, d) in (fmt.dependencies)(spec) {
                    if let Some(step) = task.get_step(f) {
                        step.add_dependency(d.into());
                    }
                }
                Self::process_dependencies(&task, map);
            }
        }
        count
    }

    fn process_dependencies(task: &BuildTask, map: &mut TaskMap) {
        for s in task.steps() {
            let key = task.as_task_ref(s.0.target);
            if let Some(v) = map.dependents.remove(&key) {
                for (d, i) in v {
                    if let Some(t) = d.get_step(s.0.target) {
                        let mut deps = t.0.requires.write();
                        for d in &mut deps.0 {
                            if let Dependency::Physical {
                                task: ref t,
                                strict,
                            } = d
                            {
                                if *t == key {
                                    *d = Dependency::Resolved {
                                        task: task.clone(),
                                        step: i,
                                        strict: *strict,
                                    };
                                }
                            }
                        }
                    }
                }
            }
            for dep in &mut s.0.requires.write().0 {
                if let Dependency::Physical {
                    task: ref deptask,
                    strict,
                } = dep
                {
                    if deptask.archive == *task.0.uri.archive_id()
                        && deptask.rel_path == task.0.rel_path
                    {
                        continue;
                        // TODO check for more
                    }
                    let key = (deptask.archive.clone(), deptask.rel_path.clone());
                    if let Some(deptasks) = map.map.get(&key) {
                        if let Some(step) =
                            deptasks.steps().iter().find(|bt| bt.0.target == s.0.target)
                        {
                            step.0.dependents.write().push((task.0.id, s.0.target));
                            *dep = Dependency::Resolved {
                                task: deptasks.clone(),
                                step: s.0.target,
                                strict: *strict,
                            };
                            continue;
                        }
                        map.dependents
                            .entry(deptask.clone())
                            .or_default()
                            .push((task.clone(), s.0.target));
                    }
                }
            }
        }
    }
}
