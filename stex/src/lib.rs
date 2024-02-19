pub mod quickparse;

mod dependencies;
mod tasks;
#[cfg(test)]
#[doc(hidden)]
mod test;

use crate::dependencies::STeXDependency;
use crate::tasks::{PdfLaTeX, RusTeX};
use async_trait::async_trait;
use immt_api::archives::ArchiveId;
use immt_api::formats::building::{Backend, BuildData};
use immt_api::formats::building::{
    BuildInfo, BuildResult, BuildStep, BuildStepKind, Dependency, TaskStep,
};
use immt_api::formats::{Format, FormatExtension, Id};
use immt_api::CloneStr;
use immt_system::controller::ControllerBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const ID: Id = Id::new_unchecked(*b"sTeX");
pub const EXTENSIONS: &[&str] = &["tex", "ltx"];

pub fn register(controller: &mut ControllerBuilder) {
    immt_shtml::register(controller);
    let format = immt_api::formats::Format::new(ID, EXTENSIONS, Box::new(STeXExtension));
    controller.register_format(format);
}

pub struct STeXExtension;

impl FormatExtension for STeXExtension {
    fn get_task(&self, info: &mut BuildInfo, backend: &Backend<'_>) -> Vec<BuildStep> {
        let deps = dependencies::get_deps(info.source().unwrap(), info.path().unwrap());
        let mut pdfdeps = Vec::new();
        let mut contentdeps = vec![Dependency::Physical {
            id: "sHTML",
            archive: info.archive_id.clone(),
            filepath: info.rel_path.clone(),
            strong: true,
        }];
        let mut narrationdeps = vec![Dependency::Physical {
            id: "content",
            archive: info.archive_id.clone(),
            filepath: info.rel_path.clone(),
            strong: true,
        }];
        for d in deps {
            match d {
                STeXDependency::ImportModule { archive, module } => {
                    let archive = archive
                        .map(ArchiveId::new)
                        .unwrap_or_else(|| info.archive_id.clone());
                    if let Some((a, p)) = info
                        .path()
                        .and_then(|p| to_file_path(p, archive, module, backend))
                    {
                        pdfdeps.push(Dependency::Physical {
                            id: "pdfLaTeX",
                            archive: a.clone(),
                            filepath: p.clone(),
                            strong: false,
                        });
                        contentdeps.push(Dependency::Physical {
                            id: "content",
                            archive: a,
                            filepath: p,
                            strong: true,
                        });
                    }
                }
                STeXDependency::UseModule { archive, module } => {
                    let archive = archive
                        .map(ArchiveId::new)
                        .unwrap_or_else(|| info.archive_id.clone());
                    if let Some((a, p)) = info
                        .path()
                        .and_then(|p| to_file_path(p, archive, module, backend))
                    {
                        pdfdeps.push(Dependency::Physical {
                            id: "pdfLaTeX",
                            archive: a.clone(),
                            filepath: p.clone(),
                            strong: false,
                        });
                        narrationdeps.push(Dependency::Physical {
                            id: "content",
                            archive: a,
                            filepath: p,
                            strong: true,
                        });
                    }
                }
                STeXDependency::Inputref { archive, filepath } => {
                    let archive = archive
                        .map(ArchiveId::new)
                        .unwrap_or_else(|| info.archive_id.clone());
                    if let Some((a, p)) = info
                        .path()
                        .and_then(|p| to_file_path_ref(archive, filepath, backend))
                    {
                        pdfdeps.push(Dependency::Physical {
                            id: "pdfLaTeX",
                            archive: a.clone(),
                            filepath: p.clone(),
                            strong: false,
                        });
                    }
                }
            }
        }
        vec![
            BuildStep {
                kind: BuildStepKind::Source(Arc::new(PdfLaTeX)),
                id: "pdfLaTeX",
                dependencies: pdfdeps,
            },
            BuildStep {
                kind: BuildStepKind::Source(Arc::new(RusTeX)),
                id: "RusTeX",
                dependencies: vec![Dependency::Physical {
                    id: "pdfLaTeX",
                    archive: info.archive_id.clone(),
                    filepath: info.rel_path.clone(),
                    strong: true,
                }],
            },
            BuildStep {
                kind: BuildStepKind::Source(Arc::new(immt_shtml::SHMLTaskStep)),
                id: "sHTML",
                dependencies: vec![Dependency::Physical {
                    id: "RusTeX",
                    archive: info.archive_id.clone(),
                    filepath: info.rel_path.clone(),
                    strong: true,
                }],
            },
            BuildStep {
                kind: BuildStepKind::Check,
                id: "content",
                dependencies: contentdeps,
            },
            BuildStep {
                kind: BuildStepKind::Check,
                id: "narration",
                dependencies: narrationdeps,
            },
        ]
    }
}

fn to_file_path(
    current: &Path,
    id: ArchiveId,
    module: &str,
    backend: &Backend<'_>,
) -> Option<(ArchiveId, CloneStr)> {
    let lang = if current.extension().and_then(|s| s.to_str()) == Some("tex") {
        match current.file_stem().and_then(|s| s.to_str()) {
            Some(s) if s.ends_with(".ru") => "ru",
            Some(s) if s.ends_with(".de") => "de",
            Some(s) if s.ends_with(".fr") => "fr",
            // TODO etc
            _ => "en",
        }
    } else {
        "en"
    };
    let archive_path = backend.get_path(&id)?;
    let (path, module) = module.split_once('?')?;
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.{lang}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some((id, format!("/{path}/{module}.{lang}.tex").into()));
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.en.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some((id, format!("/{path}/{module}.en.tex").into()));
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some((id, format!("/{path}/{module}.tex").into()));
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}.{lang}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some((id, format!("/{path}.{lang}.tex").into()));
    }
    let p = PathBuf::from(format!("{}/source/{path}.en.tex", archive_path.display()));
    if p.exists() {
        return Some((id, format!("/{path}.en.tex").into()));
    }
    let p = PathBuf::from(format!("{}/source/{path}.tex", archive_path.display()));
    if p.exists() {
        return Some((id, format!("/{path}.tex").into()));
    }
    None
}

fn to_file_path_ref(
    id: ArchiveId,
    path: &str,
    backend: &Backend<'_>,
) -> Option<(ArchiveId, CloneStr)> {
    let archive_path = backend.get_path(&id)?;
    if path.ends_with(".tex") {
        Some((id, format!("/{path}").into()))
    } else {
        Some((id, format!("/{path}.tex").into()))
    }
}
