pub mod context;
mod impls;
pub mod rules;
pub mod split;
pub mod trace;

use crate::{
    impls::{
        ContextWrap,
        solving::{Ancestor, Solvable},
    },
    rules::RuleSet,
    split::{CancelToken, SplitStrategy},
    trace::{CheckLog, CheckLogCow, CheckingTask},
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
pub(crate) use impls::backend::TermExtSeq;
use smallvec::SmallVec;
use std::{hint::unreachable_unchecked, marker::PhantomData};

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

    pub fn check_document(&mut self, d: &Document) -> Vec<CheckLog> {
        /*tracing::error!(
            "Current: {}",
            bytesize::ByteSize::b(stacker::remaining_stack().expect("wut") as _)
                .display()
                .iec_short()
        );*/
        let mut all = Vec::new();
        for e in d.dfs() {
            match e {
                DocumentElementRef::UseModule { uri: module, .. }
                | DocumentElementRef::Module { module, .. } => all.push(module.clone()),
                _ => (),
            }
        }
        let _ = self.set_context_i(all);
        let mut ret = vec![CheckLog::Msg(
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
                    ret.push(CheckLog::Msg(
                        format!("Checking variable {}", v.uri).into(),
                        trace::MessageLevel::Header,
                    ));
                    ret.extend(self.check_variable(v).1);
                }
                DocumentElementRef::Term(t) => {
                    ret.push(CheckLog::Msg(
                        format!("Checking term {}", t.uri).into(),
                        trace::MessageLevel::Header,
                    ));
                    tracing::debug!("Checking term {:?}", t.parsed().debug_short());
                    let tm = self.prepare(t.parsed().clone());
                    ret.push(self.infer_type(&tm).2);
                }
                _ => (),
            }
        }
        ret
    }

    // TODO return checked Module
    pub fn check_module(&mut self, m: &Module) -> Vec<CheckLog> {
        let mut all = Vec::new();
        self.load_context(m, &mut all);
        self.modules.insert(m.clone());
        if !all.is_empty() {
            let _ = self.set_context_i(all);
        }
        let mut ret = vec![CheckLog::Msg(
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
            ret.push(CheckLog::Msg(
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
    pub fn check_symbol(&mut self, s: &Symbol) -> (bool, SmallVec<CheckLog, 2>) {
        tracing::debug!("Checking Symbol {s:?}");
        match (s.data.tp.parsed(), s.data.df.parsed()) {
            (Some(tp), None) => {
                tracing::trace!("Checking Type");
                let tp = self.prepare(tp.clone());
                let (b, _, l) = self.check_inhabitable(&tp);

                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), smallvec::smallvec![l])
            }
            (None, Some(df)) => {
                tracing::trace!("Checking Definiens");
                let df = self.prepare(df.clone());
                let (tp, _, l) = self.infer_type(&df);

                s.data.df.set_checked(df);

                if let Some(tp) = tp {
                    //let (b, l2) = self.check_type(&df, &tp);

                    s.data.tp.set_checked(tp);

                    (true, smallvec::smallvec![l])
                } else {
                    (false, smallvec::smallvec![l])
                }
            }
            (Some(tp), Some(df)) => {
                tracing::trace!("Checking Type and Definiens");
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, _, l) = self.check_inhabitable(&tp);

                if b.is_some_and(|b| !b) {
                    return (false, smallvec::smallvec![l]);
                }
                let (b, _, l2) = self.check_type(&df, &tp);

                // TODO solve variables in tp; check against solved type

                s.data.tp.set_checked(tp);
                s.data.df.set_checked(df);

                (b.unwrap_or(false), smallvec::smallvec![l, l2])
            }
            (None, None) => (false, smallvec::smallvec![]),
        }
    }

    // TODO return checked term
    pub fn check_variable(
        &mut self,
        s: &VariableDeclaration,
    ) -> (bool, smallvec::SmallVec<CheckLog, 2>) {
        match (s.data.tp.parsed(), s.data.df.parsed()) {
            (Some(tp), None) => {
                let tp = self.prepare(tp.clone());
                let (b, _, l) = self.check_inhabitable(&tp);

                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), smallvec::smallvec![l])
            }
            (None, Some(df)) => {
                let df = self.prepare(df.clone());
                let (tp, _, l) = self.infer_type(&df);
                if let Some(tp) = tp {
                    let (b, _, l2) = self.check_type(&df, &tp);

                    s.data.df.set_checked(df);
                    s.data.tp.set_checked(tp);

                    (b.unwrap_or(false), smallvec::smallvec![l, l2])
                } else {
                    s.data.df.set_checked(df);

                    (false, smallvec::smallvec![l])
                }
            }
            (Some(tp), Some(df)) => {
                let tp = self.prepare(tp.clone());
                let df = self.prepare(df.clone());
                let (b, _, l) = self.check_inhabitable(&tp);
                if b.is_some_and(|b| !b) {
                    return (false, smallvec::smallvec![l]);
                }
                let (b, _, l2) = self.check_type(&df, &tp);

                s.data.df.set_checked(df);
                s.data.tp.set_checked(tp);

                (b.unwrap_or(false), smallvec::smallvec![l, l2])
            }
            (None, None) => (false, SmallVec::new()),
        }
    }

    pub fn check_type(
        &self,
        tm: &Term,
        tp: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, CheckLog) {
        self.wrap_task(CheckingTask::HasType(tm, tp), |mut slf| {
            slf.check_type_i(tm, tp)
        })
    }

    pub fn check_subtype(
        &self,
        sub: &Term,
        sup: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, CheckLog) {
        self.wrap_task(CheckingTask::Subtype(sub, sup), |mut slf| {
            slf.check_subtype_i(sub, sup)
        })
    }

    pub fn infer_type(
        &self,
        t: &Term,
    ) -> (Option<Term>, rustc_hash::FxHashSet<Solvable>, CheckLog) {
        self.wrap_task(CheckingTask::Inference(t), |mut slf| slf.infer_type_i(t))
    }

    pub fn check_inhabitable(
        &self,
        t: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, CheckLog) {
        self.wrap_task(CheckingTask::Inhabitable(t), |mut slf| {
            slf.check_inhabitable_i(t)
        })
    }

    fn prepare(&self, t: Term) -> Term {
        self.wrap_none(|slf| slf.prepare(t))
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
