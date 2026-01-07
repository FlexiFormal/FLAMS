use std::{borrow::Cow, fmt::Write};

use ftml_ontology::terms::{ComponentVar, Term, Variable};
use owo_colors::OwoColorize;
use smallvec::SmallVec;

use crate::{context::Context, rules::SolverRule};

#[derive(Debug)]
pub struct SolverTrace<'s> {
    cancelled: std::sync::atomic::AtomicBool,
    task: SolverTask<'s>,
    messages: smallvec::SmallVec<TraceLine, 2>,
    parent: Option<&'s Self>,
}

impl<'s> SolverTrace<'s> {
    pub fn add_line(&mut self, line: TraceLine) {
        self.messages.push(line);
    }
    pub fn comment(&mut self, s: impl Into<Cow<'static, str>>) {
        self.messages
            .push(TraceLine::Msg(s.into(), MessageLevel::Info));
    }
    pub fn failure(&mut self, s: impl Into<Cow<'static, str>>) {
        self.messages
            .push(TraceLine::Msg(s.into(), MessageLevel::Failure));
    }
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }

    //invariant: result matches the trace's task!!
    pub(crate) fn destroy<R: std::fmt::Debug>(
        self,
        result: Option<&R>,
        context: &Context,
    ) -> TraceLine {
        macro_rules! cast {
            ($r:ident : $tp:ty) => {
                unsafe { &*std::ptr::from_ref($r).cast::<$tp>() }
            };
        }
        let steps = self.messages.into_vec();
        match self.task {
            SolverTask::Infer(term) => TraceLine::InferenceTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:Term)).cloned(),
            },
            SolverTask::VarInfer(variable) => TraceLine::VarInferenceTask {
                variable: variable.name().to_string(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:Term)).cloned(),
            },
            SolverTask::Inhabitable(term) => TraceLine::InhabitableTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            SolverTask::Universe(term) => TraceLine::UniverseTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            SolverTask::Rule(rule) => TraceLine::Rule {
                rule: rule.as_box_dyn(),
                steps,
                success: result.map(|r| {
                    if std::mem::size_of::<R>() == std::mem::size_of::<bool>() {
                        *cast!(r:bool)
                    } else {
                        true
                    }
                }),
            },
            SolverTask::Strategy(name) => TraceLine::Strategy { name, steps },
            SolverTask::HasType(term, tp) => TraceLine::HasTypeTask {
                term: term.clone(),
                tp: tp.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            SolverTask::Equality(lhs, rhs) => TraceLine::Equality {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            SolverTask::Subtype(sub, sup) => TraceLine::SubtypeTask {
                sub: sub.clone(),
                sup: sup.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
        }
    }
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
            || self.parent.as_ref().is_some_and(|tk| {
                tk.is_cancelled() && {
                    self.cancelled
                        .store(true, std::sync::atomic::Ordering::Release);
                    true
                }
            })
    }

    pub fn derived<'t, R: std::fmt::Debug>(
        &self,
        task: SolverTask,
        mut context: Context<'t, '_>,
        then: impl FnOnce(&mut SolverTrace, Context<'t, '_>) -> Option<R>,
    ) -> (Option<R>, TraceLine) {
        let mut new = SolverTrace {
            task,
            cancelled: std::sync::atomic::AtomicBool::new(false),
            messages: SmallVec::default(),
            parent: Some(self),
        };
        let ret = then(&mut new, context.branch());
        let line = new.destroy(ret.as_ref(), &context);
        (ret, line)
    }

    pub fn derive<'t>(&'t self, task: SolverTask<'t>) -> SolverTrace<'t> {
        SolverTrace {
            task,
            cancelled: std::sync::atomic::AtomicBool::new(false),
            messages: SmallVec::default(),
            parent: Some(self),
        }
    }

    #[must_use]
    pub const fn new(task: SolverTask<'s>) -> Self {
        Self {
            task,
            cancelled: std::sync::atomic::AtomicBool::new(false),
            messages: SmallVec::new(),
            parent: None,
        }
    }
}

#[derive(Debug)]
pub enum SolverTask<'t> {
    Infer(&'t Term),
    VarInfer(&'t Variable),
    Inhabitable(&'t Term),
    Universe(&'t Term),
    Rule(&'t dyn SolverRule),
    Subtype(&'t Term, &'t Term),
    HasType(&'t Term, &'t Term),
    Equality(&'t Term, &'t Term),
    Strategy(&'static str),
}

#[derive(Debug)]
pub enum TraceLine {
    InferenceTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<Term>,
    },
    VarInferenceTask {
        variable: String,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<Term>,
    },
    InhabitableTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    UniverseTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    HasTypeTask {
        term: Term,
        tp: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    Equality {
        lhs: Term,
        rhs: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    SubtypeTask {
        sub: Term,
        sup: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    Rule {
        rule: Box<dyn SolverRule>,
        steps: Vec<Self>,
        success: Option<bool>,
    },
    NoRuleApplicable,
    Strategy {
        name: &'static str,
        steps: Vec<Self>,
    },
    Msg(Cow<'static, str>, MessageLevel),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageLevel {
    Failure,
    Info,
    Header,
}

// -----------------------------------------------------------------------------------

impl TraceLine {
    pub fn display(&self) -> TraceLineDisplay<'_> {
        TraceLineDisplay {
            line: self,
            styles: Box::default(),
            stream: None,
        }
    }
}

pub struct TraceLineDisplay<'a> {
    line: &'a TraceLine,
    styles: Box<TraceLineStyles>,
    stream: Option<owo_colors::Stream>,
}
impl std::fmt::Display for TraceLineDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = smallvec::SmallVec::<_, 2>::new();
        let mut children = self.prettyprint_header(f, self.line)?.iter();
        let mut indent = Indent(1);
        loop {
            while let Some(ch) = children.next() {
                self.maybe(&format_args!("{indent}─ "), self.styles.indent)
                    .fmt(f)?;
                let next = self.prettyprint_header(f, ch)?;
                if !next.is_empty() {
                    let old = std::mem::replace(&mut children, next.iter());
                    stack.push(old);
                    indent.0 += 1;
                }
            }
            indent.0 -= 1;
            if let Some(next) = stack.pop() {
                children = next;
            } else {
                return Ok(());
            }
        }
    }
}
impl TraceLineDisplay<'_> {
    #[must_use]
    pub const fn colorize(mut self, stream: owo_colors::Stream) -> Self {
        self.stream = Some(stream);
        self.styles.normal = owo_colors::Style::new().black().bold();
        self.styles.indent = owo_colors::Style::new().blue();
        self.styles.success = owo_colors::Style::new().green();
        self.styles.failed = owo_colors::Style::new().red();
        self.styles.emph = owo_colors::Style::new().bright_white().bold();
        self.styles.term = owo_colors::Style::new().yellow();
        self.styles.name_text = owo_colors::Style::new().italic();
        self.styles.variables = owo_colors::Style::new().cyan();
        self
    }
    #[inline]
    #[must_use]
    pub const fn colorize_stdout(self) -> Self {
        self.colorize(owo_colors::Stream::Stdout)
    }
    #[inline]
    #[must_use]
    pub const fn colorize_stderr(self) -> Self {
        self.colorize(owo_colors::Stream::Stderr)
    }
    fn prettyprint_context<'s>(
        &'s self,
        vars: &'s [ComponentVar],
    ) -> impl std::fmt::Display + use<'s> + 's {
        struct Ctx<'s>(&'s TraceLineDisplay<'s>, &'s [ComponentVar]);
        impl std::fmt::Display for Ctx<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let slf = self.0;
                let vars = self.1;
                write!(f, "{}", slf.emph(&"{ "))?;
                let mut first = true;
                for ComponentVar { var, tp, df } in vars {
                    if !first {
                        write!(f, "{}", slf.emph(&", "))?;
                    }
                    first = false;
                    match var {
                        Variable::Name { name, .. } => {
                            write!(f, "{}", slf.maybe(name, slf.styles.variables))?;
                        }
                        Variable::Ref { declaration, .. } => {
                            write!(f, "{}", slf.maybe(declaration.name(), slf.styles.variables))?;
                        }
                    }
                    if let Some(tp) = tp {
                        write!(f, " {} {:?}", slf.emph(&':'), slf.term(&tp.debug_short()))?;
                    }
                    if let Some(df) = df {
                        write!(f, " {} {:?}", slf.emph(&":="), slf.term(&df.debug_short()))?;
                    }
                }
                write!(f, "{}", slf.emph(&" }"))
            }
        }
        Ctx(self, vars)
    }
    fn prettyprint_header<'a>(
        &self,
        f: &mut std::fmt::Formatter,
        line: &'a TraceLine,
    ) -> Result<&'a [TraceLine], std::fmt::Error> {
        match line {
            TraceLine::HasTypeTask {
                term,
                tp,
                steps,
                result,
                context,
            } => {
                self.prefix(*result, f)?;
                writeln!(
                    f,
                    "{} {} {} {:?} {} {:?}",
                    self.normal(&"Checking typing"),
                    self.prettyprint_context(context),
                    self.emph(&"⊢"),
                    self.term(&term.debug_short()),
                    self.emph(&":"),
                    self.term(&tp.debug_short())
                )?;
                Ok(steps)
            }

            TraceLine::Equality {
                lhs: tm_a,
                rhs: tm_b,
                steps,
                result,
                context,
            } => {
                self.prefix(*result, f)?;
                writeln!(
                    f,
                    "{} {} {} {:?} {} {:?}",
                    self.normal(&"Checking equality"),
                    self.prettyprint_context(context),
                    self.emph(&"⊢"),
                    self.term(&tm_a.debug_short()),
                    self.emph(&"=="),
                    self.term(&tm_b.debug_short())
                )?;
                Ok(steps)
            }

            TraceLine::InferenceTask {
                term,
                steps,
                result,
                context,
            } => {
                let (s, st) = if result.is_some() {
                    ("SUCCESS", self.styles.success)
                } else {
                    ("FAILED", self.styles.failed)
                };

                write!(
                    f,
                    "{} {} {:?} {} {}",
                    self.maybe(&format_args!("[{s}]"), st),
                    self.normal(&"Inferring type of"),
                    self.term(&term.debug_short()),
                    self.normal(&"in context"),
                    self.prettyprint_context(context),
                )?;
                if let Some(tp) = result.as_ref() {
                    writeln!(
                        f,
                        "{} {:?}",
                        self.success(&":"),
                        self.term(&tp.debug_short())
                    )?;
                } else {
                    f.write_char('\n')?;
                }
                Ok(steps)
            }

            TraceLine::VarInferenceTask {
                variable,
                steps,
                result,
                context,
            } => {
                let (s, st) = if result.is_some() {
                    ("SUCCESS", self.styles.success)
                } else {
                    ("FAILED", self.styles.failed)
                };

                write!(
                    f,
                    "{} {} {} {} {}",
                    self.maybe(&format_args!("[{s}]"), st),
                    self.normal(&"Inferring type of variable"),
                    self.maybe(&variable, self.styles.variables),
                    self.normal(&"in context"),
                    self.prettyprint_context(context),
                )?;
                if let Some(tp) = result.as_ref() {
                    writeln!(
                        f,
                        "{} {:?}",
                        self.success(&":"),
                        self.term(&tp.debug_short())
                    )?;
                } else {
                    f.write_char('\n')?;
                }
                Ok(steps)
            }

            TraceLine::InhabitableTask {
                term,
                steps,
                result,
                context,
            } => {
                self.prefix(*result, f)?;
                writeln!(
                    f,
                    "{} {} {} {:?}",
                    self.normal(&"Checking Inhabitability"),
                    self.prettyprint_context(context),
                    self.emph(&"⊢ INH"),
                    self.term(&term.debug_short())
                )?;
                Ok(steps)
            }

            TraceLine::UniverseTask {
                term,
                steps,
                result,
                context,
            } => {
                self.prefix(*result, f)?;
                writeln!(
                    f,
                    "{} {} {} {:?}",
                    self.normal(&"Checking Universe"),
                    self.prettyprint_context(context),
                    self.emph(&"⊢ UNIV"),
                    self.term(&term.debug_short())
                )?;
                Ok(steps)
            }

            TraceLine::SubtypeTask {
                sub,
                sup,
                steps,
                result,
                context,
            } => {
                self.prefix(*result, f)?;
                writeln!(
                    f,
                    "{} {} {} {:?} {} {:?}",
                    self.normal(&"Checking Subtyping"),
                    self.prettyprint_context(context),
                    self.emph(&"⊢"),
                    self.term(&sub.debug_short()),
                    self.emph(&"<:"),
                    self.term(&sup.debug_short())
                )?;
                Ok(steps)
            }

            TraceLine::Strategy { name, steps } => {
                writeln!(
                    f,
                    "{} {}",
                    self.maybe(&"Strategy:", self.styles.emph),
                    self.maybe(&name, owo_colors::style().bright_white())
                )?;
                Ok(steps)
            }

            TraceLine::Rule {
                rule,
                steps,
                success,
            } => {
                self.prefix(*success, f)?;
                writeln!(
                    f,
                    "{} {}",
                    self.normal(&"trying rule"),
                    self.name_text(&rule)
                )?;
                Ok(steps)
            }

            TraceLine::Msg(s, MessageLevel::Info) => {
                use std::fmt::Display;
                self.maybe(&format_args!("{s}\n"), self.styles.variables.italic())
                    .fmt(f)?;
                Ok(&[])
            }
            TraceLine::Msg(s, MessageLevel::Failure) => {
                use std::fmt::Display;
                self.maybe(&format_args!("{s}\n"), self.styles.failed.italic())
                    .fmt(f)?;
                Ok(&[])
            }
            TraceLine::Msg(s, MessageLevel::Header) => {
                use std::fmt::Display;
                self.maybe(
                    &format_args!("{s}\n"),
                    self.styles.emph.italic().bold().underline(),
                )
                .fmt(f)?;
                Ok(&[])
            }

            TraceLine::NoRuleApplicable => {
                use std::fmt::Display;
                self.maybe(&"No applicable rule found!\n", self.styles.failed)
                    .fmt(f)?;
                Ok(&[])
            }
        }
    }

    fn prefix(&self, opt: Option<bool>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (s, st) = match opt {
            Some(true) => ("SUCCESS", self.styles.success),
            Some(false) => ("DISPROVEN", self.styles.failed),
            None => ("FAILED", self.styles.failed),
        };
        write!(f, "{} ", self.maybe(&format_args!("[{s}]"), st))
    }

    fn normal<'a, D: std::fmt::Display>(&'a self, v: &'a D) -> impl std::fmt::Display + use<'a, D> {
        self.maybe(v, self.styles.normal)
    }

    fn name_text<'a, D: std::fmt::Display>(
        &'a self,
        v: &'a D,
    ) -> impl std::fmt::Display + use<'a, D> {
        self.maybe(v, self.styles.name_text)
    }

    fn emph<'a, D: std::fmt::Display>(&'a self, v: &'a D) -> impl std::fmt::Display + use<'a, D> {
        self.maybe(v, self.styles.emph)
    }

    fn success<'a, D: std::fmt::Display>(
        &'a self,
        v: &'a D,
    ) -> impl std::fmt::Display + use<'a, D> {
        self.maybe(v, self.styles.success)
    }

    fn term<'a, D: std::fmt::Debug>(&'a self, v: &'a D) -> impl std::fmt::Debug + use<'a, D> {
        self.maybe_db(v, self.styles.term)
    }

    fn maybe_db<'a, D: std::fmt::Debug + 'a>(
        &'a self,
        v: &'a D,
        style: owo_colors::Style,
    ) -> impl std::fmt::Debug {
        if let Some(s) = self.stream.as_ref() {
            SupportsColorMaybe::Sc(v.if_supports_color(*s, move |v| v.style(style)))
        } else {
            SupportsColorMaybe::None(v)
        }
    }

    fn maybe<'a, D: std::fmt::Display + 'a>(
        &'a self,
        v: &'a D,
        style: owo_colors::Style,
    ) -> impl std::fmt::Display {
        if let Some(s) = self.stream.as_ref() {
            SupportsColorMaybe::Sc(v.if_supports_color(*s, move |v| v.style(style)))
        } else {
            SupportsColorMaybe::None(v)
        }
    }
}

enum SupportsColorMaybe<'a, InVal, F>
where
    F: Fn(&'a InVal) -> owo_colors::Styled<&'a InVal>,
{
    Sc(owo_colors::SupportsColorsDisplay<'a, InVal, owo_colors::Styled<&'a InVal>, F>),
    None(&'a InVal),
}
impl<'a, InVal, F> std::fmt::Display for SupportsColorMaybe<'a, InVal, F>
where
    InVal: std::fmt::Display,
    F: Fn(&'a InVal) -> owo_colors::Styled<&'a InVal>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sc(v) => v.fmt(f),
            Self::None(v) => v.fmt(f),
        }
    }
}

impl<'a, InVal, F> std::fmt::Debug for SupportsColorMaybe<'a, InVal, F>
where
    InVal: std::fmt::Debug,
    F: Fn(&'a InVal) -> owo_colors::Styled<&'a InVal>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sc(v) => v.fmt(f),
            Self::None(v) => v.fmt(f),
        }
    }
}

#[derive(Default)]
pub struct TraceLineStyles {
    normal: owo_colors::Style,
    indent: owo_colors::Style,
    success: owo_colors::Style,
    failed: owo_colors::Style,
    emph: owo_colors::Style,
    term: owo_colors::Style,
    name_text: owo_colors::Style,
    variables: owo_colors::Style,
}

impl std::fmt::Display for TraceLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display().fmt(f)
    }
}

struct Indent(usize);
impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            return Ok(());
        }
        for _ in 0..self.0 - 1 {
            f.write_str("  │")?;
        }
        f.write_str("  ├")?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------

pub enum TraceLineB<'t> {
    NoRuleApplicable,
    Inference {
        term: &'t Term,
        steps: Vec<TraceLineCow<'t>>,
    },
}
impl<'t> TraceLineB<'t> {
    pub(crate) fn from_task(task: SolverTask<'t>, steps: SmallVec<TraceLineCow<'t>, 2>) -> Self {
        todo!()
    }
}

pub enum TraceLineCow<'t> {
    Owned(TraceLineOwned),
    Borrowed(TraceLineB<'t>),
}
impl<'t> From<TraceLineB<'t>> for TraceLineCow<'t> {
    fn from(value: TraceLineB<'t>) -> Self {
        Self::Borrowed(value)
    }
}

pub enum TraceLineOwned {
    Inference {
        term: Term,
        steps: Box<[Self]>,
        context: Box<[ComponentVar]>,
        result: Option<Term>,
    },
}

/*
#[derive(Clone, Debug)]
pub enum Displayable {
    Text(Cow<'static, str>),
    Term(Term),
    Var(Variable),
    Num(i128),
}
impl From<&'static str> for Displayable {
    fn from(value: &'static str) -> Self {
        Self::Text(Cow::Borrowed(value))
    }
}
impl From<String> for Displayable {
    fn from(value: String) -> Self {
        Self::Text(Cow::Owned(value))
    }
}
impl From<Term> for Displayable {
    fn from(value: Term) -> Self {
        Self::Term(value)
    }
}
impl From<&Term> for Displayable {
    fn from(value: &Term) -> Self {
        Self::Term(value.clone())
    }
}
impl From<Variable> for Displayable {
    fn from(value: Variable) -> Self {
        Self::Var(value)
    }
}
impl From<&Variable> for Displayable {
    fn from(value: &Variable) -> Self {
        Self::Var(value.clone())
    }
}
macro_rules! impl_num {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Displayable {
                fn from(value:$t) -> Self {
                    #[allow(clippy::cast_lossless)]
                    Self::Num(value as _)
                }
            }
        )*
    }
}
impl_num!(u8, i8, u16, i16, u32, i32, u64, i64, i128, usize, isize);

pub trait TraceDisplay {}
impl TraceDisplay for () {}

#[derive(Debug)]
pub enum TraceLineB {
    InferenceTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<Term>,
    },
    VarInferenceTask {
        variable: String,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<Term>,
    },
    InhabitableTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    UniverseTask {
        term: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    HasTypeTask {
        term: Term,
        tp: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    Equality {
        lhs: Term,
        rhs: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    SubtypeTask {
        sub: Term,
        sup: Term,
        steps: Vec<Self>,
        context: Box<[ComponentVar]>,
        result: Option<bool>,
    },
    Rule {
        rule: Box<dyn SolverRule>,
        steps: Vec<Self>,
        success: Option<bool>,
    },
    NoRuleApplicable,
    Strategy {
        name: &'static str,
        steps: Vec<Self>,
    },
    Msg(Cow<'static, str>, MessageLevel),
    Interpolated(SmallVec<Displayable, 2>, MessageLevel),
}

#[macro_export]
macro_rules! traceline {
    (FAIL $($e:expr),* $(,)?) => {
        $crate::traceline!(@WRAP
            $($e),*;
            $crate::trace::MessageLevel::Failure
        )
    };
    (# $($e:expr),* $(,)?) => {
        $crate::traceline!(@WRAP
            $($e),*;
            $crate::trace::MessageLevel::Header
        )
    };
    ($($e:expr),* $(,)?) => {
        $crate::traceline!(@WRAP
            $($e),*;
            $crate::trace::MessageLevel::Info
        )
    };
    (@WRAP $($e:expr),*;$lvl:expr) => {
        $crate::trace::TraceLineB::Interpolated(
            [$($e.into()),*].into_iter().collect(),
            $lvl
        )
    }
}

fn test() -> TraceLineB {
    let tm = Term::Number(Numeric::Int(15));
    traceline!(
        "Foo",
        3,
        Term::Number(Numeric::Float((3.24).into())),
        "and also",
        &tm
    )
}
 */
