pub mod context;
mod impls;
pub mod rules;
pub mod split;
mod state;
mod test;
pub mod trace;

use crate::{
    context::Context,
    rules::RuleSet,
    split::SplitStrategy,
    state::SolverState,
    trace::{CheckingTask, SolverTrace, TraceLine},
};
use flams_math_archives::{
    backend::{AnyBackend, LocalBackend},
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
    terms::Term,
    utils::RefTree,
};
use ftml_uris::ModuleUri;
use std::{hint::unreachable_unchecked, marker::PhantomData};
pub(crate) use {impls::backend::TermExtSeq, state::TermExtSolvable};

type BigSet<T> = dashmap::DashSet<T, rustc_hash::FxBuildHasher>;
type SmallSet<T> = parking_lot::RwLock<rustc_hash::FxHashSet<T>>;

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

#[derive(Copy, Clone)]
pub struct SolverRef<'s, Split: SplitStrategy> {
    top: &'s Checker<Split>,
    state: &'s SolverState<'s>,
}

impl<Split: SplitStrategy> Checker<Split> {
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

    pub fn check_document(&mut self, d: &Document) -> Vec<TraceLine> {
        /*tracing::error!(
            "Current: {}",
            bytesize::ByteSize::b(stacker::remaining_stack().expect("wut") as _)
                .display()
                .iec_short()
        );*/
        let mut all = Vec::new();
        for e in d.dfs() {
            match e {
                DocumentElementRef::UseModule(module)
                | DocumentElementRef::Module { module, .. } => all.push(module.clone()),
                _ => (),
            }
        }
        let _ = self.set_context_i(all);
        let mut ret = vec![TraceLine::Msg(
            format!("Checking document {}", d.uri).into(),
            trace::MessageLevel::Header,
        )];
        tracing::debug!("Rules: {:?}", self.rules);
        for e in d.dfs() {
            match e {
                DocumentElementRef::Module { module, .. } => {
                    if let Ok(module) = self.get_module(module) {
                        ret.extend(self.check_module(&module));
                    }
                }
                DocumentElementRef::VariableDeclaration(v) => {
                    ret.push(TraceLine::Msg(
                        format!("Checking variable {}", v.uri).into(),
                        trace::MessageLevel::Header,
                    ));
                    ret.extend(self.check_variable(v).1);
                }
                DocumentElementRef::Term(t) => {
                    ret.push(TraceLine::Msg(
                        format!("Checking term {}", t.uri).into(),
                        trace::MessageLevel::Header,
                    ));
                    tracing::debug!("Checking term {:?}", t.parsed().debug_short());
                    let tm = self.prepare(t.parsed().clone());
                    ret.push(self.infer_type(&tm).1);
                }
                _ => (),
            }
        }
        ret
    }

    // TODO return checked Module
    pub fn check_module(&mut self, m: &Module) -> Vec<TraceLine> {
        let mut all = Vec::new();
        self.load_context(m, &mut all);
        self.modules.insert(m.clone());
        if !all.is_empty() {
            let _ = self.set_context_i(all);
        }
        let mut ret = vec![TraceLine::Msg(
            format!("Checking module {}", m.uri).into(),
            trace::MessageLevel::Header,
        )];
        for d in m.dfs().filter_map(|e| {
            if let AnyDeclarationRef::Symbol(s) = e {
                Some(s)
            } else {
                None
            }
        }) {
            ret.push(TraceLine::Msg(
                format!("Checking symbol {}", d.uri).into(),
                trace::MessageLevel::Header,
            ));
            ret.extend(self.check_symbol(d).1);
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

    pub fn set_context(&mut self, m: ModuleUri) -> Result<(), BackendError> {
        self.set_context_i(vec![!m])
    }

    fn set_context_i(&mut self, mut all: Vec<ModuleUri>) -> Result<(), BackendError> {
        while let Some(uri) = all.pop() {
            if !self.modules.contains(&uri) {
                let ModuleLike::Module(m) = self.backend.get_module(&uri)? else {
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
                AnyDeclarationRef::Import(m) if !self.modules.contains(m) => {
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
                AnyDeclarationRef::Rule { id, parameters } => {
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
    pub fn check_symbol(&mut self, s: &Symbol) -> (bool, Vec<TraceLine>) {
        tracing::debug!("Checking Symbol {s:?}");
        match (s.data.tp.parsed(), s.data.df.parsed()) {
            (Some(tp), None) => {
                tracing::trace!("Checking Type");
                let tp = self.prepare(tp.clone());
                let (b, l) = self.check_inhabitable(&tp);

                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), vec![l])
            }
            (None, Some(df)) => {
                tracing::trace!("Checking Definiens");
                let df = self.prepare(df.clone());
                let (tp, l) = self.infer_type(&df);
                if let Some(tp) = tp {
                    //let (b, l2) = self.check_type(&df, &tp);

                    s.data.df.set_checked(df);
                    s.data.tp.set_checked(tp);

                    (true, vec![l])
                } else {
                    s.data.df.set_checked(df);

                    (false, vec![l])
                }
            }
            (Some(tp), Some(df)) => {
                tracing::trace!("Checking Type and Definiens");
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, l) = self.check_inhabitable(&tp);
                if b.is_some_and(|b| !b) {
                    return (false, vec![l]);
                }
                let (b, l2) = self.check_type(&df, &tp);

                s.data.df.set_checked(df);
                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), vec![l, l2])
            }
            (None, None) => (false, Vec::new()),
        }
    }

    // TODO return checked term
    pub fn check_variable(&mut self, s: &VariableDeclaration) -> (bool, Vec<TraceLine>) {
        match (s.data.tp.parsed(), s.data.df.parsed()) {
            (Some(tp), None) => {
                let tp = self.prepare(tp.clone());
                let (b, l) = self.check_inhabitable(&tp);

                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), vec![l])
            }
            (None, Some(df)) => {
                let df = self.prepare(df.clone());
                let (tp, l) = self.infer_type(&df);
                if let Some(tp) = tp {
                    let (b, l2) = self.check_type(&df, &tp);

                    s.data.df.set_checked(df);
                    s.data.tp.set_checked(tp);

                    (b.unwrap_or(false), vec![l, l2])
                } else {
                    s.data.df.set_checked(df);

                    (false, vec![l])
                }
            }
            (Some(tp), Some(df)) => {
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, l) = self.check_inhabitable(&tp);
                if b.is_some_and(|b| !b) {
                    return (false, vec![l]);
                }
                let (b, l2) = self.check_type(&df, &tp);

                s.data.df.set_checked(df);
                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), vec![l, l2])
            }
            (None, None) => (false, Vec::new()),
        }
    }

    pub fn check_type(&self, tm: &Term, tp: &Term) -> (Option<bool>, TraceLine) {
        self.wrap(CheckingTask::HasType(tm, tp), |slf, trace, context| {
            slf.check_type_i(trace, context, tm, tp)
        })
    }

    pub fn check_subtype(&self, sub: &Term, sup: &Term) -> (Option<bool>, TraceLine) {
        self.wrap(CheckingTask::Subtype(sub, sup), |slf, trace, context| {
            slf.check_subtype_i(trace, context, sub, sup)
        })
    }

    pub fn infer_type(&self, t: &Term) -> (Option<Term>, TraceLine) {
        self.wrap(CheckingTask::Inference(t), |slf, trace, context| {
            slf.infer_type_i(trace, context, t)
        })
    }

    pub fn check_inhabitable(&self, t: &Term) -> (Option<bool>, TraceLine) {
        self.wrap(CheckingTask::Inhabitable(t), |slf, trace, context| {
            slf.check_inhabitable_i(trace, context, t)
        })
    }

    fn prepare(&self, t: Term) -> Term {
        let st = SolverState::default();
        SolverRef {
            top: self,
            state: &st,
        }
        .prepare(t)
    }

    /*
    fn prepare_and_bind(&self, t: Term) -> Term {
        let s = SolverRef { top: self };
        s.bind_implicits(s.prepare(t))
    }
     */

    fn wrap<'t, R: std::fmt::Debug + 'static>(
        &self,
        task: CheckingTask,
        then: impl FnOnce(SolverRef<Split>, &mut SolverTrace, Context<'t, '_>) -> Option<R>,
    ) -> (Option<R>, TraceLine) {
        let mut trace = SolverTrace::new(task);
        let mut top = Context::new_top();
        let state = SolverState::default();
        let rf = SolverRef {
            top: self,
            state: &state,
        };
        let r = then(rf, &mut trace, top.build());
        let l = trace.destroy(r.as_ref(), &top.build());
        // TODO check state
        (r, l)
    }
}
