//! Fake/synthetic build targets for testing without flams-stex dependency
//!
//! This module creates mock BuildTargets and SourceFormat that are sufficient
//! for testing the scheduler's graph logic. The scheduler only cares about
//! task dependencies and readiness, not the actual build execution.

use flams_math_archives::formats::{BuildTarget, BuildResult, SourceFormat};
use flams_math_archives::backend::AnyBackend;
use std::path::{Path, PathBuf};
use either::Either;

/// Fake build function that always succeeds
fn fake_build_success(_spec: BuildSpec) -> BuildResult {
    BuildResult {
        log: Either::Left("Fake build (success)".to_string()),
        result: Ok(None),
    }
}

/// Fake build function that always fails
fn fake_build_failure(_spec: BuildSpec) -> BuildResult {
    BuildResult {
        log: Either::Left("Fake build (failure)".to_string()),
        result: Err(Vec::new()),
    }
}

// ============================================================================
// Fake BuildTargets for the three pipeline stages
// Named to match the StepTask types they correspond to
// ============================================================================

pub static FAKE_PDFLATEX1: BuildTarget = BuildTarget {
    id: "fake_pdflatex1",
    run: fake_build_success,
};

pub static FAKE_BIBTEX: BuildTarget = BuildTarget {
    id: "fake_bibtex",
    run: fake_build_success,
};

pub static FAKE_PDFLATEX2: BuildTarget = BuildTarget {
    id: "fake_pdflatex2",
    run: fake_build_success,
};

// BuildTargetIds are references to static BuildTargets
pub static FAKE_PDFLATEX1_ID: flams_math_archives::formats::BuildTargetId =
    flams_math_archives::formats::BuildTargetId(&FAKE_PDFLATEX1);

pub static FAKE_BIBTEX_ID: flams_math_archives::formats::BuildTargetId =
    flams_math_archives::formats::BuildTargetId(&FAKE_BIBTEX);

pub static FAKE_PDFLATEX2_ID: flams_math_archives::formats::BuildTargetId =
    flams_math_archives::formats::BuildTargetId(&FAKE_PDFLATEX2);

// ============================================================================
// Fake SourceFormat for the test
// ============================================================================

pub static FAKE_SOURCE_FORMAT: SourceFormat = SourceFormat {
    id: "fake_format",
    name: "Fake Format",
    description: "Fake format for testing",
    targets: &[
        &FAKE_PDFLATEX1,
        &FAKE_BIBTEX,
        &FAKE_PDFLATEX2,
    ],
    file_extensions: &["fake"],
    dependencies: fake_get_deps,
};

/// Fake dependency resolver - always returns empty
fn fake_get_deps(_uri: &flams_math_archives::formats::DocumentUri) -> Vec<flams_math_archives::formats::TaskDependency> {
    Vec::new()
}

// ============================================================================
// Stub types for BuildSpec (required by the fake build functions)
// ============================================================================

/// Minimal BuildSpec for fake builds
#[derive(Debug, Clone)]
pub struct BuildSpec {
    // We don't need a full spec for fake builds
}
