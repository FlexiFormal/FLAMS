//! Converter from generator's TaskMap to buildsystem's TaskMap
//!
//! Bridges the gap between the two different TaskMap implementations to enable
//! testing the buildsystem with generator-created cyclic dependencies.

use buildsystem::taskmap::{
    TaskMap as BuildsystemTaskMap,
    buildtask::{
        BuildTask as BuildsystemBuildTask,
        BuildStep as BuildsystemBuildStep,
        buildstep::BuildStepI,
        buildtaskid::BuildTaskId as BuildsystemBuildTaskId,
        taskstate::{TaskState as BuildsystemTaskState, AtomicTaskState},
        dependency::Dependency as BuildsystemDependency,
    },
};
use flams_utils::vecmap::VecSet;
use flams_utils::triomphe::Arc;
use flams_utils::prelude::HMap;
use ftml_uris::{ArchiveId, ArchiveUri, UriPath};
use parking_lot::RwLock;
use std::num::NonZeroU32;
use either::Either;
use std::str::FromStr;
use super::fake_targets;

/// Convert generator's TaskMap to buildsystem's TaskMap
///
/// Creates synthetic archive/document URIs for each task, converts step
/// states and dependencies to the buildsystem format using fake BuildTargetIds.
pub fn convert_generator_taskmap(
    gen_taskmap: &generator::taskm::TaskMap,
) -> BuildsystemTaskMap {
    let mut map: HMap<(ArchiveId, UriPath), BuildsystemBuildTask> = HMap::default();
    let mut dependents = HMap::default();
    let mut counter = NonZeroU32::new(1).unwrap();
    let mut total_steps = 0usize;

    // First pass: create all BuildTasks without cross-task dependencies
    for gen_task in &gen_taskmap.map {
        let task_id = gen_task.id.give();

        // Create synthetic URIs for this task
        let archive_id = ArchiveId::from_str("test/cyclic").expect("valid archive id");
        let rel_path = UriPath::try_new(format!("task_{}.fake", task_id))
            .expect("valid path");

        // Create archive URI
        let archive_uri = ArchiveUri::from_str("test/cyclic")
            .expect("valid archive uri");

        let buildsystem_id = BuildsystemBuildTaskId(counter);
        counter = counter.saturating_add(1);

        // Convert steps
        let mut steps = Vec::new();
        for gen_step in gen_task.steps.iter() {
            let buildsystem_step = convert_step(gen_step);
            steps.push(buildsystem_step);
            total_steps += 1;
        }

        // Create BuildTask
        let buildsystem_task = BuildsystemBuildTask::new(
            buildsystem_id,
            archive_uri.clone(),
            steps.into_boxed_slice(),
            Either::Right(format!("task_{}", task_id)),
            rel_path.clone(),
        ).expect("valid BuildTask");

        map.insert((archive_id, rel_path), buildsystem_task);
    }

    // Second pass: add cross-task dependencies
    // Map generator task positions to buildsystem tasks
    let gen_id_to_buildsys: HMap<usize, BuildsystemBuildTask> = map
        .values()
        .enumerate()
        .map(|(idx, task)| (idx, task.clone()))
        .collect();

    // Now add dependencies: iterate through generator tasks and connect them
    for (task_idx, gen_task) in gen_taskmap.map.iter().enumerate() {
        if let Some(buildsys_task) = gen_id_to_buildsys.get(&task_idx) {
            for (step_idx, gen_step) in gen_task.steps.iter().enumerate() {
                for gen_dep in &gen_step.requires {
                    let dep_gen_id = gen_dep.id.give();

                    // Find the target task in buildsystem map
                    if let Some(dep_task) = gen_id_to_buildsys.get(&dep_gen_id) {
                        // Get the current step
                        if let Some(current_step) = buildsys_task.steps().get(step_idx) {
                            // Get the dependency task's step (use first step as placeholder)
                            if let Some(dep_step) = dep_task.steps().first() {
                                let resolved_dep = BuildsystemDependency::Resolved {
                                    task: dep_task.clone(),
                                    step: dep_step.0.target,
                                    strict: true,
                                };

                                current_step.add_dependency(resolved_dep);
                            }
                        }
                    }
                }
            }
        }
    }

    BuildsystemTaskMap {
        map,
        dependents,
        counter,
        total: total_steps,
    }
}

/// Convert a generator BuildStep to buildsystem BuildStep
///
/// Maps generator's StepTask types to fake BuildTargetIds with matching names:
/// - StepTask::PDFLATEX1 → FAKE_PDFLATEX1_ID
/// - StepTask::BIBTEX → FAKE_BIBTEX_ID
/// - StepTask::PDFLATEX2 → FAKE_PDFLATEX2_ID
///
/// These fake targets are sufficient for testing the scheduler's
/// graph logic without needing real build target implementations.
fn convert_step(gen_step: &generator::taskm::BuildStep) -> BuildsystemBuildStep {
    let state = match gen_step.state {
        generator::taskm::TaskState::Running => BuildsystemTaskState::Running,
        generator::taskm::TaskState::Blocked => BuildsystemTaskState::Blocked,
        generator::taskm::TaskState::Queued => BuildsystemTaskState::Queued,
        generator::taskm::TaskState::Failed => BuildsystemTaskState::Failed,
        generator::taskm::TaskState::Done => BuildsystemTaskState::Done,
        generator::taskm::TaskState::None => BuildsystemTaskState::None,
    };

    // Map generator step type to fake BuildTargetId with matching name
    let target = match gen_step.task {
        generator::taskm::StepTask::PDFLATEX1 => fake_targets::FAKE_PDFLATEX1_ID,
        generator::taskm::StepTask::BIBTEX => fake_targets::FAKE_BIBTEX_ID,
        generator::taskm::StepTask::PDFLATEX2 => fake_targets::FAKE_PDFLATEX2_ID,
    };

    let step_i = BuildStepI {
        target,
        state: AtomicTaskState::new(state),
        requires: RwLock::new(VecSet::new()),
        dependents: RwLock::new(Vec::new()),
    };

    BuildsystemBuildStep(Arc::new(step_i))
}
