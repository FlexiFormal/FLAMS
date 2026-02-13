pub mod context;
mod impls;
pub mod results {
    pub use ftml_solver_trace::results::*;
}
pub mod rules;
pub mod split;
pub mod trace {
    pub use ftml_solver_trace::*;
}

use crate::{
    impls::{
        ContextWrap,
        solving::{Ancestor, Solvable},
    },
    results::{
        CheckResult, ContentCheckResult, DocumentCheckResult, SymbolCheckResult, TypeCheckResult,
    },
    rules::RuleSet,
    split::{CancelToken, SingleThreadedSplit, SplitStrategy},
    trace::{CheckLogCow, CheckingTask, PreCheckLog},
};
use flams_math_archives::{
    artifacts::{ContentUpdate, FileOrString},
    backend::{AnyBackend, LocalBackend},
    formats::BuildResult,
    utils::errors::BackendError,
};
use ftml_ontology::{
    domain::{
        declarations::{AnyDeclarationRef, symbols::Symbol},
        modules::{Module, ModuleData, ModuleLike},
    },
    narrative::{
        documents::{Document, DocumentData},
        elements::{DocumentElementRef, VariableDeclaration},
    },
    terms::{ComponentVar, Term, termpaths::TermPath},
    utils::RefTree,
};
use ftml_solver_trace::CheckLog;
use ftml_uris::ModuleUri;
pub(crate) use impls::backend::TermExtSeq;
use smallvec::SmallVec;
use std::{hint::unreachable_unchecked, marker::PhantomData};

flams_math_archives::build_target!(CHECK {
    name: "typecheck",
    description: "check the validity of formal/complex expressions",
    run: check
});

#[allow(clippy::needless_pass_by_value)]
fn check(task: flams_math_archives::formats::BuildSpec) -> BuildResult {
    let d = match task.backend.get_document(task.uri) {
        Ok(d) => d,
        Err(e) => {
            return BuildResult {
                log: FileOrString::Str(format!("Document not found: {e}").into_boxed_str()),
                result: Err(Vec::new()),
            };
        }
    };
    let mut checker = Checker::<SingleThreadedSplit>::new(task.backend.clone());
    let (logs, modules) = checker.check_document(&d);
    let log = serde_json::to_string(&logs).unwrap_or_else(|s| s.to_string());
    BuildResult {
        log: FileOrString::Str(log.into_boxed_str()),
        result: Ok(Some(Box::new(ContentUpdate {
            document: Some(d),
            modules,
        }) as _)),
    }
}

type BigSet<T> = dashmap::DashSet<T, rustc_hash::FxBuildHasher>;

/*
static MINIMAL_STACK: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(usize::MAX - 10);
pub fn minimal_stack() -> usize {
    MINIMAL_STACK.load(std::sync::atomic::Ordering::Acquire)
}
pub(crate) fn update_stack() {
    let curr = MINIMAL_STACK.load(std::sync::atomic::Ordering::Acquire);
    let Some(rem) = stacker::remaining_stack() else {
        return;
    };
    let min = curr.min(rem);
    if min < curr {
        MINIMAL_STACK.store(min, std::sync::atomic::Ordering::Release);
    }
}
 */

pub struct SubtermCheckResult {
    pub simplified: Term,
    pub inferred_type: Option<Term>,
    pub context: Vec<ComponentVar>,
    pub log: CheckLog,
}

pub struct Checker<Split: SplitStrategy> {
    backend: AnyBackend,
    rules: RuleSet<Split>,
    modules: BigSet<Module>,
    documents: BigSet<Document>,
    implicits: std::sync::atomic::AtomicUsize,
    current_module: Option<ModuleData>,
    current_document: Option<DocumentData>,
    __phantom: PhantomData<Split>,
}

pub struct CheckRef<'c, 'i, Split: SplitStrategy> {
    pub(crate) top: &'c Checker<Split>,
    pub(crate) context: ContextWrap<'c, 'i>,
    pub(crate) solutions: &'i mut rustc_hash::FxHashSet<Solvable>,
    pub(crate) parent_solutions: Option<Ancestor<'i>>,
    messages: &'i mut SmallVec<CheckLogCow<'c>, 2>,
    pub(crate) cancel: &'i CancelToken<'i, Split::CancelToken>,
    added: u8,
    traced: bool,
}

impl<Split: SplitStrategy> Checker<Split> {
    #[must_use]
    pub fn new(backend: AnyBackend) -> Self {
        Self {
            backend,
            modules: BigSet::default(),
            documents: BigSet::default(),
            rules: RuleSet::default(),
            implicits: std::sync::atomic::AtomicUsize::new(1),
            current_module: None,
            current_document: None,
            __phantom: PhantomData,
        }
    }

    pub fn check_document(&mut self, d: &Document) -> (DocumentCheckResult, Vec<Module>) {
        /*tracing::error!(
            "Current: {}",
            bytesize::ByteSize::b(stacker::remaining_stack().expect("wut") as _)
                .display()
                .iec_short()
        );*/
        let mut all = Vec::new();
        let mut modules = Vec::new();
        let mut results = Vec::new();
        for e in d.dfs() {
            match e {
                DocumentElementRef::UseModule { uri: module, .. } => all.push(module.clone()),
                DocumentElementRef::Module { module, .. } => {
                    all.push(module.clone());
                    modules.push(module.clone());
                }
                _ => (),
            }
        }
        let _ = self.set_context_i(all);
        let modules = modules
            .into_iter()
            .filter_map(|uri| match self.backend.get_module(&uri) {
                Ok(ModuleLike::Module(m)) => Some(m),
                _ => None,
            })
            .collect();
        tracing::debug!("Rules: {:?}", self.rules);
        for e in d.dfs() {
            match e {
                DocumentElementRef::Module { module, .. } => {
                    if let Ok(module) = self.get_module(module) {
                        results.push(CheckResult::Module {
                            uri: module.uri.clone(),
                            checks: self.check_module(&module).into_boxed_slice(),
                        });
                    } else {
                        results.push(CheckResult::Missing(module.clone()));
                    }
                }
                DocumentElementRef::VariableDeclaration(v) => {
                    if let Some(r) = self.check_variable(v) {
                        results.push(CheckResult::Variable(v.uri.clone(), r));
                    }
                }
                DocumentElementRef::Term(top) => {
                    tracing::debug!("Checking term {:?}", top.get_parsed().debug_short());
                    let tm = self.prepare(top.get_parsed().clone());
                    let (t, _, log) = self.infer_type(&tm);
                    let t = t.map(|t| self.revert_prepare(t));
                    if let Some(t) = &t {
                        top.set_type(t.clone());
                    }
                    results.push(CheckResult::Term {
                        uri: top.uri.clone(),
                        inferred: t,
                        log: CheckLog::from_pre(log, &mut |t| self.revert_prepare(t)),
                    });
                }
                _ => (),
            }
        }
        (
            DocumentCheckResult {
                uri: d.uri.clone(),
                checks: results.into_boxed_slice(),
            },
            modules,
        )
    }

    // TODO return checked Module
    pub fn check_module(&mut self, m: &Module) -> Vec<ContentCheckResult> {
        let mut all = Vec::new();
        self.load_context(m, &mut all);
        self.modules.insert(m.clone());
        if !all.is_empty() {
            let _ = self.set_context_i(all);
        }
        let mut ret = Vec::new();
        for d in m.dfs().filter_map(|e| {
            if let AnyDeclarationRef::Symbol(s) = e {
                Some(s)
            } else {
                None
            }
        }) {
            if let Some(r) = self.check_symbol(d) {
                ret.push(ContentCheckResult::Symbol(d.uri.clone(), r));
            }
        }
        ret
    }

    /*
    pub fn check_document(
        &mut self,
        d: &mut DocumentData,
        modules: &mut [ModuleData],
    ) -> Vec<TraceLine> {
        self.current_document = Some(d.clone());
        let mut all = Vec::new();
        for e in d.dfs() {
            match e {
                DocumentElementRef::UseModule(module) => all.push(module.clone()),
                _ => (),
            }
        }
        let _ = self.set_context_i(all);
        let mut ret = vec![TraceLine::Msg(
            format!("Checking document {}", d.uri).into(),
        )];
        for e in d.dfs() {
            match e {
                DocumentElementRef::Module { module, .. } => {
                    if let Some(m) = modules.iter_mut().find(|m| m.uri == *module) {
                        ret.extend(self.check_module(m));
                    }
                }
                DocumentElementRef::VariableDeclaration(v) => {
                    ret.push(TraceLine::Msg(
                        format!("Checking variable {}", v.uri).into(),
                    ));
                    ret.extend(self.check_variable(v).1);
                }
                DocumentElementRef::Term(t) => {
                    ret.push(TraceLine::Msg(format!("Checking term {}", t.uri).into()));
                    ret.push(self.infer_type(&t.term).1);
                }
                _ => (),
            }
        }
        ret
    }

    // TODO return checked Module
    pub fn check_module(&mut self, m: &mut ModuleData) -> Vec<TraceLine> {
        let mut all = Vec::new();
        self.current_module = Some(m.clone());
        self.load_context(m, &mut all);
        if !all.is_empty() {
            let _ = self.set_context_i(all);
        }
        let mut ret = vec![TraceLine::Msg(format!("Checking module {}", m.uri).into())];
        for d in m.dfs().filter_map(|e| {
            if let AnyDeclarationRef::Symbol(s) = e {
                Some(s)
            } else {
                None
            }
        }) {
            ret.push(TraceLine::Msg(format!("Checking symbol {}", d.uri).into()));
            ret.extend(self.check_symbol(d).1);
        }
        ret
    }
    */

    pub fn add_modules(&mut self, modules: Vec<Module>) -> Result<(), BackendError> {
        let uris = modules
            .into_iter()
            .map(|m| {
                let uri = m.uri.clone();
                self.modules.insert(m);
                uri
            })
            .collect();
        self.set_context_i(uris)
    }

    pub fn set_context(&mut self, m: Vec<ModuleUri>) -> Result<(), BackendError> {
        self.set_context_i(m)
    }

    pub fn check_subterm(&mut self, term: Term, mut path: TermPath) -> Option<SubtermCheckResult> {
        let nterm = self.wrap_none(|slf| slf.prepare(term, Some(&mut path)));
        let (ctx, t) = nterm.subterm_at_path(&path)?;
        let mut nt = t.clone();
        //ctx.reverse();
        let mut ctx = ctx.into_iter().cloned().rev().collect::<Vec<_>>();
        let (r, s, log) = self.wrap_task(CheckingTask::Inference(t), |mut slf| {
            let allvars = t.free_variables();
            for v in allvars {
                if !ctx.iter().any(|cv| cv.var == *v) {
                    let tp = slf.infer_var_type_i(v);
                    ctx.push(ComponentVar {
                        var: v.clone(),
                        tp,
                        df: None,
                    });
                }
            }
            let mut i = 0;
            while let Some(vd) = ctx.get(i) {
                i += 1;
                let tp = vd.tp.clone();
                let df = vd.df.clone();
                if let Some(t) = tp {
                    let allvars = t
                        .free_variables()
                        .into_iter()
                        .filter(|v| !ctx.iter().any(|cv| cv.var == **v))
                        .cloned()
                        .collect::<smallvec::SmallVec<_, 2>>();
                    for v in allvars {
                        let tp = slf.infer_var_type_i(&v);
                        ctx.push(ComponentVar {
                            var: v,
                            tp,
                            df: None,
                        });
                    }
                }
                if let Some(t) = df {
                    let allvars = t
                        .free_variables()
                        .into_iter()
                        .filter(|v| !ctx.iter().any(|cv| cv.var == **v))
                        .cloned()
                        .collect::<smallvec::SmallVec<_, 2>>();
                    for v in allvars {
                        let tp = slf.infer_var_type_i(&v);
                        ctx.push(ComponentVar {
                            var: v,
                            tp,
                            df: None,
                        });
                    }
                }
            }
            ctx.reverse();
            for c in &ctx {
                slf.extend_context(c);
            }
            nt = slf.revert_prepare(t.clone());
            slf.infer_type(t).map(|t| slf.revert_prepare(t))
        });
        /*
        let mut frees = nt.free_variables();
        for v in r.as_ref().map(|t| t.free_variables()).unwrap_or_default() {
            if !frees.contains(&v) {
                frees.push(v);
            }
        }
         */
        /*let context = ctx
        .into_iter()
        //.filter(|v| frees.iter().any(|f| f.name() == v.var.name()))
        .cloned()
        .collect();*/
        let log = self.wrap_none(|slf| {
            for c in &mut ctx {
                if let Some(tp) = c.tp.take() {
                    c.tp = Some(slf.revert_prepare(tp));
                }
                if let Some(df) = c.df.take() {
                    c.df = Some(slf.revert_prepare(df));
                }
            }
            CheckLog::from_pre(log, &mut |t| slf.revert_prepare(t))
        });
        //drop(frees);
        Some(SubtermCheckResult {
            simplified: nt,
            inferred_type: r,
            context: ctx,
            log,
        })
    }

    fn set_context_i(&mut self, mut all: Vec<ModuleUri>) -> Result<(), BackendError> {
        while let Some(uri) = all.pop() {
            if uri.is_top() && !self.modules.contains(&uri) {
                let ModuleLike::Module(m) = self.backend.get_module(&uri)? else {
                    // SAFETY: uri.is_top()
                    unsafe { unreachable_unchecked() };
                };
                self.load_context(&m, &mut all);
                self.modules.insert(m);
            }
        }
        Ok(())
    }
    fn load_context(&mut self, m: &Module, todos: &mut Vec<ModuleUri>) {
        if let Some(uri) = m.meta_module.as_ref() {
            if uri.is_top() {
                if !self.modules.contains(uri) {
                    todos.push(uri.clone());
                }
            } else {
                let uri = !uri.clone();
                if !self.modules.contains(&uri) {
                    todos.push(uri);
                }
            }
        }
        for d in m.dfs() {
            match d {
                AnyDeclarationRef::Import { uri: m, .. } if !self.modules.contains(m) => {
                    if m.is_top() {
                        todos.push(m.clone());
                    } else {
                        let uri = !m.clone();
                        if !self.modules.contains(&uri) {
                            todos.push(uri);
                        }
                    }
                }
                AnyDeclarationRef::Symbol(s) => {
                    for e in Split::SYMBOL_EXTRACTORS {
                        e(s, &mut self.rules);
                    }
                }
                AnyDeclarationRef::Rule { id, parameters, .. } => {
                    tracing::debug!("Rule: {id}");
                    if let Some(rule) = Split::RULE_EXTRACTORS
                        .iter()
                        .find_map(|(s, f)| if id.as_ref() == *s { Some(f) } else { None })
                    {
                        rule(parameters, &mut self.rules);
                    }
                }
                _ => (),
            }
        }
    }

    // TODO return checked term
    pub fn check_symbol(&mut self, s: &Symbol) -> Option<SymbolCheckResult> {
        tracing::debug!("Checking Symbol {s:?}");
        match (s.data.tp.get_parsed(), s.data.df.get_parsed()) {
            (Some(tp), None) => {
                tracing::trace!("Checking Type");
                let tp = self.prepare(tp.clone());
                let (b, _, l) = self.check_inhabitable(&tp);
                s.data.tp.set_checked(tp);
                Some(SymbolCheckResult::TypeOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                })
            }
            (None, Some(df)) => {
                tracing::trace!("Checking Definiens");
                let df = self.prepare(df.clone());
                let (tp, _, l) = self.infer_type(&df);

                s.data.df.set_checked(df);

                if let Some(tp) = tp {
                    s.data.tp.set_checked(tp.clone());
                    let tp = self.revert_prepare(tp);
                    s.data.tp.set_presentation(tp.clone());
                    Some(SymbolCheckResult::DefiniensOnly {
                        inferred: Some(tp),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    })
                } else {
                    Some(SymbolCheckResult::DefiniensOnly {
                        inferred: None,
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    })
                }
            }
            (Some(tp), Some(df)) => {
                tracing::trace!("Checking Type and Definiens");
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, _, l) = self.check_inhabitable(&tp);

                if b.is_some_and(|b| !b) {
                    s.data.tp.set_checked(tp);
                    return Some(SymbolCheckResult::Both {
                        inhabitable: TypeCheckResult {
                            success: false,
                            log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                        },
                        matches: None,
                    });
                }
                let (b, _, l2) = self.check_type(&df, &tp);
                s.data.tp.set_checked(tp);
                s.data.df.set_checked(df);
                Some(SymbolCheckResult::Both {
                    inhabitable: TypeCheckResult {
                        success: true,
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                    matches: Some(TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l2, &mut |t| self.revert_prepare(t)),
                    }),
                })
            }
            (None, None) => None,
        }
    }

    // TODO return checked term
    pub fn check_variable(&mut self, s: &VariableDeclaration) -> Option<SymbolCheckResult> {
        match (s.data.tp.get_parsed(), s.data.df.get_parsed()) {
            (Some(tp), None) => {
                let tp = self.prepare(tp.clone());
                let (b, _, l) = self.check_inhabitable(&tp);
                s.data.tp.set_checked(tp);
                Some(SymbolCheckResult::TypeOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                })
            }
            (None, Some(df)) => {
                let df = self.prepare(df.clone());
                let (tp, _, l) = self.infer_type(&df);
                s.data.df.set_checked(df);
                if let Some(tp) = tp {
                    s.data.tp.set_checked(tp.clone());
                    let tp = self.revert_prepare(tp);
                    s.data.tp.set_presentation(tp.clone());
                    Some(SymbolCheckResult::DefiniensOnly {
                        inferred: Some(tp),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    })
                } else {
                    Some(SymbolCheckResult::DefiniensOnly {
                        inferred: None,
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    })
                }
            }
            (Some(tp), Some(df)) => {
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, _, l) = self.check_inhabitable(&tp);
                if b.is_some_and(|b| !b) {
                    s.data.tp.set_checked(tp);
                    return Some(SymbolCheckResult::Both {
                        inhabitable: TypeCheckResult {
                            success: false,
                            log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                        },
                        matches: None,
                    });
                }
                let (b, _, l2) = self.check_type(&df, &tp);

                s.data.df.set_checked(df);
                s.data.tp.set_checked(tp);
                Some(SymbolCheckResult::Both {
                    inhabitable: TypeCheckResult {
                        success: true,
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                    matches: Some(TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l2, &mut |t| self.revert_prepare(t)),
                    }),
                })
            }
            (None, None) => None,
        }
    }

    pub fn check_type(
        &self,
        tm: &Term,
        tp: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::HasType(tm, tp), |mut slf| {
            slf.check_type_i(tm, tp)
        })
    }

    pub fn check_subtype(
        &self,
        sub: &Term,
        sup: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Subtype(sub, sup), |mut slf| {
            slf.check_subtype_i(sub, sup)
        })
    }

    pub fn infer_type(
        &self,
        t: &Term,
    ) -> (Option<Term>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Inference(t), |mut slf| slf.infer_type_i(t))
    }

    pub fn check_inhabitable(
        &self,
        t: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Inhabitable(t), |mut slf| {
            slf.check_inhabitable_i(t)
        })
    }

    fn prepare(&self, t: Term) -> Term {
        self.wrap_none(|slf| slf.prepare(t, None))
    }
    fn revert_prepare(&self, t: Term) -> Term {
        self.wrap_none(|slf| slf.revert_prepare(t))
    }
}

#[allow(clippy::useless_let_if_seq)]
fn topo_sort(
    new: &mut Vec<ModuleUri>,
    sorted: &mut Vec<ModuleUri>,
    get: impl Fn(&ModuleUri) -> Option<Module>,
) -> usize {
    let mut added = 0;
    while let Some(uri) = new.last() {
        if sorted.contains(uri) {
            let _ = new.pop();
            continue;
        }
        let Some(m) = get(uri) else {
            // SAFETY: uris.last() == Some(uri)
            sorted.push(unsafe { new.pop().unwrap_unchecked() });
            added += 1;
            continue;
        };
        let curr = new.len();

        let mut changed = false;
        if let Some(e) = m.meta_module.as_ref()
            && !sorted.contains(e)
        {
            new.insert(curr, e.clone());
            changed = true;
        }
        for e in m.dfs() {
            let AnyDeclarationRef::Import { uri, .. } = e else {
                continue;
            };
            if sorted.contains(uri) {
                continue;
            }
            new.insert(curr, uri.clone());
            changed = true;
        }
        if !changed {
            // SAFETY: uris.last() == Some(uri)
            sorted.push(unsafe { new.pop().unwrap_unchecked() });
            added += 1;
        }
    }
    added
}
