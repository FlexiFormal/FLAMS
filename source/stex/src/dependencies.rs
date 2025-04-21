use crate::quickparse::stex::rules;
use crate::{
    quickparse::{
        latex::LaTeXParser,
        stex::{
            structs::{ModuleReference, STeXParseState, STeXToken},
            DiagnosticLevel,
        },
    },
    PDFLATEX_FIRST,
};
use either::Either;
use flams_ontology::{
    languages::Language,
    uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, DocumentURI},
};
use flams_system::{
    backend::AnyBackend,
    building::{BuildTask, Dependency, TaskRef},
    formats::CHECK,
};
use flams_utils::{parsing::ParseStr, sourcerefs::SourceRange};
use std::path::Path;

pub enum STeXDependency {
    ImportModule {
        archive: ArchiveId,
        module: std::sync::Arc<str>,
    },
    UseModule {
        archive: ArchiveId,
        module: std::sync::Arc<str>,
    },
    Inputref {
        archive: Option<ArchiveId>,
        filepath: std::sync::Arc<str>,
    },
    Module {
        //uri:ModuleURI,
        sig: Option<Language>,
        meta: Option<(ArchiveId, std::sync::Arc<str>)>,
    },
}

#[allow(clippy::type_complexity)]
pub struct DepParser<'a> {
    parser: LaTeXParser<
        'a,
        ParseStr<'a, ()>,
        STeXToken<()>,
        fn(String, SourceRange<()>, DiagnosticLevel),
        STeXParseState<'a, (), ()>,
    >,
    stack: Vec<std::vec::IntoIter<STeXToken<()>>>,
    curr: Option<std::vec::IntoIter<STeXToken<()>>>,
}

fn parse_deps<'a>(
    source: &'a str,
    path: &'a Path,
    archive: ArchiveURIRef<'a>,
    doc: &'a DocumentURI,
    backend: &'a AnyBackend,
) -> impl Iterator<Item = STeXDependency> + use<'a> {
    const NOERR: fn(String, SourceRange<()>, DiagnosticLevel) = |_, _, _| {};
    let parser = LaTeXParser::with_rules(
        ParseStr::new(source),
        STeXParseState::<(), ()>::new(Some(archive), Some(path), doc, backend, ()),
        NOERR,
        LaTeXParser::default_rules().into_iter().chain([
            ("importmodule", rules::importmodule_deps as _),
            ("setmetatheory", rules::setmetatheory as _),
            ("usemodule", rules::usemodule_deps as _),
            ("inputref", rules::inputref as _),
            ("mhinput", rules::inputref as _),
            ("stexstyleassertion", rules::stexstyleassertion as _),
            ("stexstyledefinition", rules::stexstyledefinition as _),
            ("stexstyleparagraph", rules::stexstyleparagraph as _),
        ]),
        LaTeXParser::default_env_rules().into_iter().chain([(
            "smodule",
            (
                rules::smodule_deps_open as _,
                rules::smodule_deps_close as _,
            ),
        )]),
    );
    DepParser {
        parser,
        stack: Vec::new(),
        curr: None,
    }
}

impl DepParser<'_> {
    fn convert(&mut self, t: STeXToken<()>) -> Option<STeXDependency> {
        match t {
            STeXToken::ImportModule {
                module:
                    ModuleReference {
                        uri,
                        rel_path: Some(rel_path),
                        ..
                    },
                ..
            }
            | STeXToken::SetMetatheory {
                module:
                    ModuleReference {
                        uri,
                        rel_path: Some(rel_path),
                        ..
                    },
                ..
            } => Some(STeXDependency::ImportModule {
                archive: uri.archive_id().clone(),
                module: rel_path,
            }),
            STeXToken::UseModule {
                module:
                    ModuleReference {
                        uri,
                        rel_path: Some(rel_path),
                        ..
                    },
                ..
            } => Some(STeXDependency::UseModule {
                archive: uri.archive_id().clone(),
                module: rel_path,
            }),
            STeXToken::Module {
                /*uri,*/ sig,
                children,
                meta_theory,
                ..
            } => {
                let old = std::mem::replace(&mut self.curr, Some(children.into_iter()));
                if let Some(old) = old {
                    self.stack.push(old);
                }
                Some(STeXDependency::Module {
                    /*uri,*/ sig,
                    meta: meta_theory
                        .and_then(|m| m.rel_path.map(|p| (m.uri.archive_id().clone(), p))),
                })
            }
            STeXToken::Inputref {
                archive,
                filepath: module,
                ..
            } => Some(STeXDependency::Inputref {
                archive: archive.map(|(a, _)| a),
                filepath: module.0,
            }),
            STeXToken::Vec(v) => {
                let old = std::mem::replace(&mut self.curr, Some(v.into_iter()));
                if let Some(old) = old {
                    self.stack.push(old);
                }
                None
            }
            _ => None,
        }
    }
}

impl Iterator for DepParser<'_> {
    type Item = STeXDependency;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(curr) = &mut self.curr {
                if let Some(t) = curr.next() {
                    if let Some(t) = self.convert(t) {
                        return Some(t);
                    }
                } else {
                    self.curr = self.stack.pop();
                }
            } else if let Some(t) = self.parser.next() {
                if let Some(t) = self.convert(t) {
                    return Some(t);
                }
            } else {
                return None;
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn get_deps(backend: &AnyBackend, task: &BuildTask) {
    let Either::Left(path) = task.source() else {
        return;
    };
    let Ok(uri) = task.document_uri() else { return };
    let source = std::fs::read_to_string(path);
    if let Ok(source) = source {
        //let mut yields = Vec::new();
        for d in parse_deps(&source, path, task.archive(), &uri, backend) {
            match d {
                STeXDependency::ImportModule { archive, module }
                | STeXDependency::UseModule { archive, module } => {
                    if let Some(step) = task.get_step(PDFLATEX_FIRST) {
                        step.add_dependency(Dependency::Physical {
                            strict: false,
                            task: TaskRef {
                                archive: archive.clone(),
                                rel_path: module.clone(),
                                target: PDFLATEX_FIRST,
                            },
                        });
                    }
                    if let Some(step) = task.get_step(CHECK) {
                        step.add_dependency(Dependency::Physical {
                            strict: true,
                            task: TaskRef {
                                archive,
                                rel_path: module,
                                target: CHECK,
                            },
                        });
                    }
                }
                STeXDependency::Inputref { archive, filepath } => {
                    if let Some(step) = task.get_step(PDFLATEX_FIRST) {
                        step.add_dependency(Dependency::Physical {
                            strict: false,
                            task: TaskRef {
                                archive: archive.unwrap_or_else(|| task.archive().id().clone()),
                                rel_path: filepath,
                                target: PDFLATEX_FIRST,
                            },
                        });
                    }
                }
                STeXDependency::Module {
                    /*uri:_,*/ sig,
                    meta,
                } => {
                    //yields.push(uri);
                    if let Some(lang) = sig {
                        let archive = task.archive().id().clone();
                        let Some(rel_path) = task.rel_path().rsplit_once('.').and_then(|(a, _)| {
                            a.rsplit_once('.').map(|(a, _)| format!("{a}.{lang}.tex"))
                        }) else {
                            continue;
                        };
                        //tracing::debug!("Adding dependency: {:?}", rf);
                        if let Some(step) = task.get_step(PDFLATEX_FIRST) {
                            step.add_dependency(Dependency::Physical {
                                strict: false,
                                task: TaskRef {
                                    archive: archive.clone(),
                                    rel_path: rel_path.clone().into(),
                                    target: PDFLATEX_FIRST,
                                },
                            });
                        }
                        if let Some(step) = task.get_step(CHECK) {
                            step.add_dependency(Dependency::Physical {
                                strict: true,
                                task: TaskRef {
                                    archive,
                                    rel_path: rel_path.into(),
                                    target: CHECK,
                                },
                            });
                        }
                    }
                    if let Some((archive, module)) = meta {
                        if let Some(step) = task.get_step(PDFLATEX_FIRST) {
                            step.add_dependency(Dependency::Physical {
                                strict: false,
                                task: TaskRef {
                                    archive: archive.clone(),
                                    rel_path: module.clone(),
                                    target: PDFLATEX_FIRST,
                                },
                            });
                        }
                        if let Some(step) = task.get_step(CHECK) {
                            step.add_dependency(Dependency::Physical {
                                strict: true,
                                task: TaskRef {
                                    archive,
                                    rel_path: module,
                                    target: CHECK,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
}

/*
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn file_path_from_archive(
    current: &Path,
    id: &ArchiveId,
    module: &str,
    backend: &AnyBackend,
    yields: &[&str],
) -> Option<std::sync::Arc<str>> {
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
    let archive_path = backend.get_base_path(id)?;
    let (path, mut module) = if let Some((a, b)) = module.split_once('?') {
        (a, b)
    } else {
        ("", module)
    };
    module = module.split('/').next().unwrap_or(module);
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.{lang}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some(format!("{path}/{module}.{lang}.tex").into());
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.en.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some(format!("{path}/{module}.en.tex").into());
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}/{module}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some(format!("{path}/{module}.tex").into());
    }
    let p = PathBuf::from(format!(
        "{}/source/{path}.{lang}.tex",
        archive_path.display()
    ));
    if p.exists() {
        return Some(format!("{path}.{lang}.tex").into());
    }
    let p = PathBuf::from(format!("{}/source/{path}.en.tex", archive_path.display()));
    if p.exists() {
        return Some(format!("{path}.en.tex").into());
    }
    let p = PathBuf::from(format!("{}/source/{path}.tex", archive_path.display()));
    if p.exists() {
        return Some(format!("{path}.tex").into());
    }
    if yields.contains(&module) {
        return None;
    }
    None
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn to_file_path_ref(path: &str) -> std::sync::Arc<str> {
    if path.ends_with(".tex") {
        path.into()
    } else {
        format!("{path}.tex").into()
    }
}
*/
