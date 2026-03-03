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
        HasDeclarations,
        declarations::{AnyDeclarationRef, morphisms::Morphism, symbols::Symbol},
        modules::{Module, ModuleLike},
    },
    narrative::{
        documents::Document,
        elements::{
            DocumentElementRef, LogicalParagraph, VariableDeclaration, paragraphs::ParagraphKind,
        },
    },
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, Term, TermContainer,
        Variable, termpaths::TermPath,
    },
    utils::RefTree,
};
use ftml_solver_trace::CheckLog;
use ftml_uris::{Id, IsDomainUri, ModuleUri, SymbolUri};
pub(crate) use rules::sequences::TermExtSeq;
use smallvec::SmallVec;
use std::{borrow::Cow, hint::unreachable_unchecked, marker::PhantomData};

pub static DUMMY: std::sync::LazyLock<Id> =
    // SAFETY: "DUMMY" is a valid ID
    std::sync::LazyLock::new(|| unsafe { "DUMMY".parse().unwrap_unchecked() });

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
    /*println!("Result: {d:#?}");
    for m in &modules {
        println!("{m:#?}");
    }*/
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
        self.documents.insert(d.clone());
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
        let modules: Vec<_> = modules
            .into_iter()
            .filter_map(|uri| self.get_module(&uri).ok())
            .collect();
        tracing::trace!("Rules: {:?}", self.rules);
        for e in d.dfs() {
            match e {
                DocumentElementRef::Module { module, .. } => {
                    if self.get_module(module).is_err() {
                        results.push(CheckResult::Missing(module.clone()));
                    }
                }
                DocumentElementRef::Morphism { morphism, .. } => {
                    let top = morphism.clone().simple_module();
                    let Some(m) = self
                        .get_module(top.module_uri())
                        .ok()
                        .and_then(|m| m.get_as::<Morphism>(top.name()))
                    else {
                        results.push(CheckResult::Missing(morphism.module.clone()));
                        continue;
                    };
                    m.initialize(&mut |uri| self.get_module_like(uri));
                    if let Some(r) = self.check_morphism(&m) {
                        results.extend(r.into_iter());
                    }
                }
                DocumentElementRef::VariableDeclaration(v) => {
                    if let Some(r) = self.check_variable(v) {
                        results.push(CheckResult::Variable(v.uri.clone(), r));
                    }
                }
                DocumentElementRef::SymbolDeclaration(uri) => {
                    let top = uri.clone().simple_module();
                    if let Some(s) = modules
                        .iter()
                        .find(|m| m.uri == top.module)
                        .and_then(|m| m.find::<Symbol>(top.name.steps()))
                    {
                        if let Some(r) = self.check_symbol(s) {
                            results.push(CheckResult::Content(ContentCheckResult::Symbol(
                                uri.clone(),
                                r,
                            )));
                        }
                    } else {
                        results.push(CheckResult::Missing(uri.module.clone()));
                    }
                }
                DocumentElementRef::Term(top) => {
                    tracing::debug!("Checking term {:?}", top.get_parsed().debug_short());
                    let (unks, tm) = self.prepare(None, top.get_parsed().clone());
                    let (t, _, log) = self.infer_type(Some(unks), &tm);
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
                DocumentElementRef::Paragraph(
                    p @ LogicalParagraph {
                        kind: ParagraphKind::Proof,
                        fors,
                        ..
                    },
                ) if fors.len() == 1 => {
                    if let Some(r) = self.check_proof(p) {
                        results.push(r);
                    }
                }
                DocumentElementRef::Paragraph(
                    p @ LogicalParagraph {
                        kind: ParagraphKind::Assertion,
                        fors,
                        ..
                    },
                ) if p.fors.iter().any(|(_, t)| t.is_some()) => {
                    if let Some(r) = self.check_assertion(p) {
                        results.extend(r.into_iter());
                    }
                }
                DocumentElementRef::Paragraph(
                    p @ LogicalParagraph {
                        kind, fors, styles, ..
                    },
                ) if kind.is_definition_like(styles) && p.fors.iter().any(|(_, t)| t.is_some()) => {
                    if let Some(r) = self.check_definition(p) {
                        results.push(r);
                    }
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

    /// #### Errors
    pub fn add_modules(&mut self, modules: Vec<Module>) -> Result<(), BackendError> {
        let mut todos = Vec::new();
        for m in modules {
            if !self.modules.contains(&m.uri) {
                self.modules.insert(m.clone());
                self.load_context(&m, &mut todos);
                if !todos.is_empty() {
                    self.set_context_i(std::mem::take(&mut todos))?;
                }
            }
        }
        Ok(())
    }

    /// #### Errors
    pub fn set_context(&mut self, m: Vec<ModuleUri>) -> Result<(), BackendError> {
        self.set_context_i(m)
    }

    pub fn check_subterm(&mut self, term: Term, mut path: TermPath) -> Option<SubtermCheckResult> {
        let (unks, nterm) = self.wrap_none(None, |mut slf| {
            let (s, r) = slf.prepare(term, Some(&mut path));
            slf.merge_solutions(s);
            r
        });
        let (ctx, t) = nterm.subterm_at_path(&path)?;
        let mut nt = t.clone();
        //ctx.reverse();
        let mut ctx = ctx.into_iter().cloned().rev().collect::<Vec<_>>();
        let (r, s, log) = self.wrap_task(CheckingTask::Inference(t), Some(unks), |mut slf| {
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
            let simp = slf.simplify_full(true, t).unwrap_or_else(|| t.clone());
            nt = slf.revert_prepare(slf.subst(simp));
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
        let (_, log) = self.wrap_none(None, |slf| {
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
        tracing::trace!("Context: {all:?}");
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
        tracing::trace!("Loading: {:?}", m.uri);
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

    fn check_components(
        &self,
        tpc: &TermContainer,
        dfc: &TermContainer,
    ) -> Option<SymbolCheckResult> {
        match (tpc.get_parsed(), dfc.get_parsed()) {
            (Some(tp), None) => {
                tracing::trace!("Checking Type");
                let (unks, tp) = self.prepare(None, tp.clone());
                let (b, _, l) = self.check_inhabitable(Some(unks), &tp);
                tpc.set_checked(tp);
                Some(SymbolCheckResult::TypeOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                })
            }
            (None, Some(df)) => {
                tracing::trace!("Checking Definiens");
                let (unks, df) = self.prepare(None, df.clone());
                let (tp, _, l) = self.infer_type(Some(unks), &df);

                dfc.set_checked(df);

                if let Some(tp) = tp {
                    tpc.set_checked(tp.clone());
                    let tp = self.revert_prepare(tp);
                    tpc.set_presentation(tp.clone());
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
                let (tunks, tp) = self.prepare(None, tp.clone());
                let (b, tunks, l) = self.check_inhabitable(Some(tunks), &tp);
                if b.is_some_and(|b| !b) {
                    tpc.set_checked(tp);
                    return Some(SymbolCheckResult::Both {
                        inhabitable: TypeCheckResult {
                            success: false,
                            log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                        },
                        matches: None,
                    });
                }
                let (dunks, df) = self.prepare(Some(tunks), df.clone());

                let (b, _, l2) = self.check_type(Some(dunks), &df, &tp);
                tpc.set_checked(tp);
                dfc.set_checked(df);
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
    pub fn check_symbol(&mut self, s: &Symbol) -> Option<SymbolCheckResult> {
        tracing::debug!("Checking Symbol {s:?}");
        self.check_components(&s.data.tp, &s.data.df)
    }

    // TODO return checked term
    pub fn check_variable(&mut self, s: &VariableDeclaration) -> Option<SymbolCheckResult> {
        self.check_components(&s.data.tp, &s.data.df)
    }

    pub fn check_type(
        &self,
        unknowns: Option<rustc_hash::FxHashSet<Solvable>>,
        tm: &Term,
        tp: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::HasType(tm, tp), unknowns, |mut slf| {
            slf.check_type_i(tm, tp)
        })
    }

    pub fn check_subtype(
        &self,
        unknowns: Option<rustc_hash::FxHashSet<Solvable>>,
        sub: &Term,
        sup: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Subtype(sub, sup), unknowns, |mut slf| {
            slf.check_subtype_i(sub, sup)
        })
    }

    pub fn infer_type(
        &self,
        unknowns: Option<rustc_hash::FxHashSet<Solvable>>,
        t: &Term,
    ) -> (Option<Term>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Inference(t), unknowns, |mut slf| {
            slf.infer_type_i(t)
        })
    }

    pub fn check_inhabitable(
        &self,
        unknowns: Option<rustc_hash::FxHashSet<Solvable>>,
        t: &Term,
    ) -> (Option<bool>, rustc_hash::FxHashSet<Solvable>, PreCheckLog) {
        self.wrap_task(CheckingTask::Inhabitable(t), unknowns, |mut slf| {
            slf.check_inhabitable_i(t)
        })
    }

    fn prepare(
        &self,
        unks: Option<rustc_hash::FxHashSet<Solvable>>,
        t: Term,
    ) -> (rustc_hash::FxHashSet<Solvable>, Term) {
        self.wrap_none(unks, |mut slf| {
            let (sols, r) = slf.prepare(t, None);
            slf.merge_solutions(sols);
            r
        })
    }
    fn revert_prepare(&self, t: Term) -> Term {
        self.wrap_none(None, |slf| slf.revert_prepare(t)).1
    }

    pub fn check_morphism(&mut self, m: &Morphism) -> Option<Vec<CheckResult>> {
        let mut ret = Vec::new();
        for d in m.declarations() {
            match d {
                AnyDeclarationRef::Symbol(s) => {
                    // TODO:
                    // - check that a *refined type* is a subtype of the original type's translation
                    // - check that an *assigned definies* is ???? the original definiens
                    if let Some(r) = self.check_components(&s.data.tp, &s.data.df) {
                        ret.push(CheckResult::Content(ContentCheckResult::Symbol(
                            s.uri.clone(),
                            r,
                        )));
                    }
                }
                _ => (),
            }
            //println!("Here! Proof {}", p.uri);
        }
        Some(ret)
    }

    pub fn check_proof(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here! Proof {}", p.uri);
        None
    }

    pub fn check_definition(&mut self, p: &LogicalParagraph) -> Option<CheckResult> {
        //println!("Here! Definition {}", p.uri);
        None
    }

    pub fn check_assertion(&mut self, p: &LogicalParagraph) -> Option<Vec<CheckResult>> {
        let judgment = self.rules.marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<rules::IsJudgmentRule>()
                .map(|rl| rl.0.clone())
        });
        let (lambda, pi, apply) = self.rules.marker().iter().rev().find_map(|rl| {
            rl.as_any()
                .downcast_ref::<rules::HOASRule>()
                .map(|rl| (rl.lambda.clone(), rl.pi.clone(), rl.apply.clone()))
        })?;
        let mut ret = Vec::new();
        for (target, term) in &p.fors {
            let Ok(target) = self.get_symbol(target, |t| t) else {
                continue;
            };
            let Some(term) = term else { continue };
            let Some(term) = term.get_parsed() else {
                continue;
            };
            let wrapped = wrap_premises(term, &p.premises, judgment.as_ref(), apply.as_ref(), &pi);
            let (unks, tp) = self.prepare(None, wrapped.into_owned());

            tracing::trace!("Checking assertion for {}", target.uri);
            let (b, _, l) = self.check_inhabitable(Some(unks), &tp);
            target
                .data
                .tp
                .set_presentation(self.revert_prepare(tp.clone()));
            target.data.tp.set_checked(tp);
            ret.push(CheckResult::Content(ContentCheckResult::Symbol(
                target.uri.clone(),
                SymbolCheckResult::TypeOnly {
                    result: TypeCheckResult {
                        success: b.unwrap_or(false),
                        log: CheckLog::from_pre(l, &mut |t| self.revert_prepare(t)),
                    },
                },
            )));
        }
        Some(ret)
    }
}

fn wrap_premises<'c>(
    conclusion: &'c Term,
    premises: &[Term],
    judgment: Option<&SymbolUri>,
    apply: Option<&SymbolUri>,
    pi: &SymbolUri,
) -> Cow<'c, Term> {
    let conclusion = judgment.map_or_else(
        || Cow::Borrowed(conclusion),
        |j| {
            Cow::Owned(apply.map_or_else(
                || {
                    Term::Application(ApplicationTerm::new(
                        Term::Symbol {
                            uri: j.clone(),
                            presentation: None,
                        },
                        Box::new([Argument::Simple(conclusion.clone())]),
                        None,
                    ))
                },
                |app| {
                    Term::Application(ApplicationTerm::new(
                        Term::Symbol {
                            uri: app.clone(),
                            presentation: None,
                        },
                        Box::new([
                            Argument::Simple(Term::Symbol {
                                uri: j.clone(),
                                presentation: None,
                            }),
                            Argument::Simple(conclusion.clone()),
                        ]),
                        None,
                    ))
                },
            ))
        },
    );
    premises.iter().fold(conclusion, |c, p| {
        let premise = judgment.map_or_else(
            || p.clone(),
            |j| {
                apply.map_or_else(
                    || {
                        Term::Application(ApplicationTerm::new(
                            Term::Symbol {
                                uri: j.clone(),
                                presentation: None,
                            },
                            Box::new([Argument::Simple(p.clone())]),
                            None,
                        ))
                    },
                    |app| {
                        Term::Application(ApplicationTerm::new(
                            Term::Symbol {
                                uri: app.clone(),
                                presentation: None,
                            },
                            Box::new([
                                Argument::Simple(Term::Symbol {
                                    uri: j.clone(),
                                    presentation: None,
                                }),
                                Argument::Simple(p.clone()),
                            ]),
                            None,
                        ))
                    },
                )
            },
        );
        Cow::Owned(Term::Bound(BindingTerm::new(
            Term::Symbol {
                uri: pi.clone(),
                presentation: None,
            },
            Box::new([
                BoundArgument::Bound(ComponentVar {
                    var: Variable::Name {
                        name: crate::DUMMY.clone(),
                        notated: None,
                    },
                    tp: Some(premise),
                    df: None,
                }),
                BoundArgument::Simple(c.into_owned()),
            ]),
            None,
        )))
    })
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
