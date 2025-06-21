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
    uris::{ArchiveId, ArchiveURITrait, DocumentURI},
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
    Img {
        archive: Option<ArchiveId>,
        filepath: std::sync::Arc<str>,
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

pub(super) fn parse_deps<'a>(
    source: &'a str,
    path: &'a Path,
    doc: &'a DocumentURI,
    backend: &'a AnyBackend,
) -> impl Iterator<Item = STeXDependency> + use<'a> {
    const NOERR: fn(String, SourceRange<()>, DiagnosticLevel) = |_, _, _| {};
    let archive = doc.archive_uri();
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
            ("mhgraphics", rules::mhgraphics as _),
            ("cmhgraphics", rules::mhgraphics as _),
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
                archive, filepath, ..
            } => Some(STeXDependency::Inputref {
                archive: archive.map(|(a, _)| a),
                filepath: filepath.0,
            }),
            STeXToken::Vec(v) => {
                let old = std::mem::replace(&mut self.curr, Some(v.into_iter()));
                if let Some(old) = old {
                    self.stack.push(old);
                }
                None
            }
            STeXToken::MHGraphics {
                filepath, archive, ..
            } => Some(STeXDependency::Img {
                archive: archive.map(|(a, _)| a),
                filepath: filepath.0,
            }),
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
        for d in parse_deps(&source, path, &uri, backend) {
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
                STeXDependency::Img { .. } => (),
            }
        }
    }
}
