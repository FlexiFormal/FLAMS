use crate::quickparse::latex::{
    EnvCloseRule, EnvOpenRule, Environment, EnvironmentResult, EnvironmentRule, FromLaTeXToken,
    LaTeXParser, Macro, MacroResult, MacroRule,
};
use immt_api::core::utils::sourcerefs::SourceRange;
use immt_api::core::utils::parse::{ParseSource, ParseStr};
use std::path::Path;
use immt_api::building::targets::BuildTarget;
use immt_api::building::tasks::{BuildTask, Dependency, TaskRef};
use immt_api::controller::Controller;
use immt_api::core::narration::Language;
use immt_api::core::uris::archives::ArchiveId;
use crate::{dependencies, PDFLATEX_FIRST, tex, to_file_path_ref};
use crate::utils::file_path_from_archive;

pub(crate) enum DepToken<'a> {
    ImportModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    UseModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    Inputref {
        archive: Option<&'a str>,
        filepath: &'a str,
    },
    YieldModule(&'a str),
    Vec(Vec<DepToken<'a>>),
    Signature(Language),
}

pub(crate) enum STeXDependency<'a> {
    ImportModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    UseModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    Inputref {
        archive: Option<&'a str>,
        filepath: &'a str,
    },
    YieldModule(&'a str),
    Signature(Language),
}
pub(crate) enum STeXDep<'a> {
    ImportModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    UseModule {
        archive: Option<&'a str>,
        module: &'a str,
    },
    Inputref {
        archive: Option<&'a str>,
        filepath: &'a str,
    },
    Signature(Language),
}

pub(crate) struct DepParser<'a> {
    parser: LaTeXParser<'a, ParseStr<'a, ()>, DepToken<'a>>,
    stack: Vec<std::vec::IntoIter<DepToken<'a>>>,
    curr: Option<std::vec::IntoIter<DepToken<'a>>>,
}

fn parse_deps<'a>(source: &'a str, path: &'a Path) -> impl Iterator<Item = STeXDependency<'a>> {
    let parser = LaTeXParser::with_rules(
        ParseStr::new(source),
        Some(path),
        LaTeXParser::default_rules().into_iter().chain(vec![
            ("importmodule", importmodule as MacroRule<'a, _, _>),
            ("setmetatheory", setmetatheory as MacroRule<'a, _, _>),
            ("usemodule", usemodule as MacroRule<'a, _, _>),
            ("inputref", inputref as MacroRule<'a, _, _>),
        ]),
        LaTeXParser::default_env_rules().into_iter().chain(vec![(
            "smodule",
            (
                smodule_open as EnvOpenRule<'a, _, _>,
                smodule_close as EnvCloseRule<'a, _, _>,
            ),
        )]),
    );
    DepParser {
        parser,
        stack: Vec::new(),
        curr: None,
    }
}

impl<'a> Iterator for DepParser<'a> {
    type Item = STeXDependency<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(curr) = &mut self.curr {
                if let Some(t) = curr.next() {
                    match t {
                        DepToken::ImportModule { archive, module } => {
                            return Some(STeXDependency::ImportModule { archive, module })
                        }
                        DepToken::UseModule { archive, module } => {
                            return Some(STeXDependency::UseModule { archive, module })
                        }
                        DepToken::YieldModule(name) => return Some(STeXDependency::YieldModule(name)),
                        DepToken::Signature(lang) => return Some(STeXDependency::Signature(lang)),
                        DepToken::Inputref {
                            archive,
                            filepath: module,
                        } => {
                            return Some(STeXDependency::Inputref {
                                archive,
                                filepath: module,
                            })
                        }
                        DepToken::Vec(v) => {
                            let old = std::mem::replace(&mut self.curr, Some(v.into_iter()));
                            if let Some(old) = old {
                                self.stack.push(old);
                            }
                        }
                    }
                } else {
                    self.curr = self.stack.pop();
                }
            } else {
                match self.parser.next() {
                    Some(DepToken::ImportModule { archive, module }) => {
                        return Some(STeXDependency::ImportModule { archive, module })
                    }
                    Some(DepToken::UseModule { archive, module }) => {
                        return Some(STeXDependency::UseModule { archive, module })
                    }
                    Some(DepToken::Signature(lang)) => {
                        return Some(STeXDependency::Signature(lang))
                    }
                    Some(DepToken::YieldModule(name)) => {
                        return Some(STeXDependency::YieldModule(name))
                    }
                    Some(DepToken::Inputref {
                        archive,
                        filepath: module,
                    }) => {
                        return Some(STeXDependency::Inputref {
                            archive,
                            filepath: module,
                        })
                    }
                    Some(DepToken::Vec(v)) => {
                        self.curr = Some(v.into_iter());
                    }
                    None => return None,
                }
            }
        }
    }
}

impl<'a> FromLaTeXToken<'a, &'a str, ()> for DepToken<'a> {
    fn from_comment(_: SourceRange<()>) -> Option<Self> {
        None
    }
    fn from_group(_: SourceRange<()>, v: Vec<Self>) -> Option<Self> {
        Some(DepToken::Vec(v))
    }
    fn from_math(_: bool, _: SourceRange<()>, v: Vec<Self>) -> Option<Self> {
        Some(DepToken::Vec(v))
    }
    fn from_control_sequence(_: (), _: &'a str) -> Option<Self> {
        None
    }
    fn from_text(_: SourceRange<()>, _: &'a str) -> Option<Self> {
        None
    }
    fn from_macro_application(_: Macro<'a, &'a str, (), Self>) -> Option<Self> {
        None
    }
    fn from_environment(e: Environment<'a, &'a str, (), Self>) -> Option<Self> {
        Some(DepToken::Vec(e.children))
    }
}

tex!(<l='a,Str=&'a str,Pa=ParseStr<'a,()>,Pos=(),T=DepToken<'a>>
    p => importmodule[archive:str]{module:name} => {
        MacroResult::Success(DepToken::ImportModule { archive, module })
    }
);
tex!(<l='a,Str=&'a str,Pa=ParseStr<'a,()>,Pos=(),T=DepToken<'a>>
    p => setmetatheory[archive:str]{module:name} => {
        MacroResult::Success(DepToken::ImportModule { archive, module })
    }
);
tex!(<l='a,Str=&'a str,Pa=ParseStr<'a,()>,Pos=(),T=DepToken<'a>>
    p => usemodule[archive:str]{module:name} => {
        MacroResult::Success(DepToken::UseModule { archive, module })
    }
);
tex!(<l='a,Str=&'a str,Pa=ParseStr<'a,()>,Pos=(),T=DepToken<'a>>
    p => inputref('*'?_s)[archive:str]{filepath:name} => {
        MacroResult::Success(DepToken::Inputref { archive, filepath })
    }
);

tex!(<l='a,Str=&'a str,Pa=ParseStr<'a,()>,Pos=(),T=DepToken<'a>>
    p => @begin{smodule}([opt]{name:name}){
        if let Some(v) = opt.as_keyvals().get(&"sig") {
            if let Ok(l) = (*v).try_into() {
                smodule.children.push(DepToken::Signature(l))
            }
        }
        smodule.children.push(DepToken::YieldModule(name));
        match opt.as_keyvals().get(&"meta").copied() {
            None => smodule.children.push(DepToken::ImportModule {
                archive: Some("sTeX/meta-inf"),
                module: "Metatheory",
            }),
            Some(""|"{}") => (),
            Some(o) => todo!("Metatheory: {o}")
        }
    }{}!
);


pub(crate) fn get_deps(ctrl:&dyn Controller,task: &BuildTask) {
    let source = std::fs::read_to_string(task.path());
    if let Ok(source) = source {
        let mut deps = Vec::new();
        let mut yields = Vec::new();
        for d in parse_deps(&source,task.path()) { match d {
            STeXDependency::ImportModule {archive,module} => deps.push(STeXDep::ImportModule {archive,module}),
            STeXDependency::UseModule {archive,module} => deps.push(STeXDep::UseModule {archive,module}),
            STeXDependency::Inputref {archive,filepath} => deps.push(STeXDep::Inputref {archive,filepath}),
            STeXDependency::Signature(lang) => deps.push(STeXDep::Signature(lang)),
            STeXDependency::YieldModule(name) => yields.push(name),
        }};
        for d in deps {
            match d {
                STeXDep::ImportModule { archive, module} | STeXDep::UseModule { archive, module} => {
                    let archive = archive.map(|s| ArchiveId::new(s)).unwrap_or(task.archive().id().clone());
                    if let Some(rel_path) = ctrl.archives().with_tree(|tree|
                        file_path_from_archive(task.path(),archive,module,tree,yields.as_slice())
                    ) {
                        if archive == task.archive().id() && rel_path.as_ref() == task.rel_path() {
                            continue
                        }
                        let mut rf = TaskRef {
                            archive,rel_path,
                            target: PDFLATEX_FIRST.id
                        };
                        //tracing::debug!("Adding dependency: {:?}", rf);
                        if let Some(step) = task.find_step(PDFLATEX_FIRST.id) {
                            step.push_dependency(Dependency::Physical {
                                strict:false,
                                task:rf.clone()
                            })
                        }
                        if let Some(step) = task.find_step(BuildTarget::CHECK.id) {
                            rf.target = BuildTarget::CHECK.id;
                            step.push_dependency(Dependency::Physical {
                                strict:true,
                                task:rf
                            })
                        }
                    }
                }
                STeXDep::Inputref { archive, filepath } => {
                    let archive = archive.map(|s| ArchiveId::new(s)).unwrap_or(task.archive().id().clone());
                    let rel_path = to_file_path_ref(filepath);
                    if archive == task.archive().id() && rel_path.as_ref() == task.rel_path() {
                        continue
                    }
                    let rf = TaskRef {
                        archive,rel_path,
                        target: PDFLATEX_FIRST.id
                    };
                    //tracing::debug!("Adding dependency: {:?}", rf);
                    if let Some(step) = task.find_step(PDFLATEX_FIRST.id) {
                        step.push_dependency(Dependency::Physical {
                            strict:false, task:rf
                        })
                    }
                },
                STeXDep::Signature(lang) => {
                    let archive = task.archive().id().clone();
                    let rel_path = task.rel_path()
                        .rsplit_once('.')
                        .and_then(|(a, _)| {
                            a.rsplit_once('.').map(|(a, _)| format!("{a}.{lang}.tex"))
                        })
                        .unwrap().into();
                    let mut rf = TaskRef {
                        archive,rel_path,
                        target: PDFLATEX_FIRST.id
                    };
                    //tracing::debug!("Adding dependency: {:?}", rf);
                    if let Some(step) = task.find_step(PDFLATEX_FIRST.id) {
                        step.push_dependency(Dependency::Physical {
                            strict:false,
                            task:rf.clone()
                        })
                    }
                    if let Some(step) = task.find_step(BuildTarget::CHECK.id) {
                        rf.target = BuildTarget::CHECK.id;
                        step.push_dependency(Dependency::Physical {
                            strict:true,
                            task:rf
                        })
                    }
                }
            }
        }
    }
}