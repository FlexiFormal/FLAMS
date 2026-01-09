use std::{borrow::Cow, fmt::Write};

use ftml_ontology::terms::{ComponentVar, Term, Variable};
use ftml_uris::FtmlUri;
use owo_colors::OwoColorize;
use smallvec::SmallVec;

use crate::{context::Context, rules::CheckerRule};

#[derive(Debug)]
pub struct SolverTrace<'s> {
    cancelled: std::sync::atomic::AtomicBool,
    task: CheckingTask<'s>,
    messages: smallvec::SmallVec<TraceLine, 2>,
    parent: Option<&'s Self>,
}

impl<'s> SolverTrace<'s> {
    pub fn add_line(&mut self, line: TraceLine) {
        self.messages.push(line);
    }
    pub fn comment(&mut self, s: impl Into<Cow<'static, str>>) {
        self.messages
            .push(TraceLine::Msg(s.into(), MessageLevel::Comment));
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
            CheckingTask::Inference(term) => TraceLine::InferenceTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:Term)).cloned(),
            },
            CheckingTask::VariableInference(variable) => TraceLine::VarInferenceTask {
                variable: variable.to_string(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:Term)).cloned(),
            },
            CheckingTask::Inhabitable(term) => TraceLine::InhabitableTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            CheckingTask::Universe(term) => TraceLine::UniverseTask {
                term: term.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            CheckingTask::Rule(rule) => TraceLine::Rule {
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
            CheckingTask::Strategy(name) => TraceLine::Strategy { name, steps },
            CheckingTask::HasType(term, tp) => TraceLine::HasTypeTask {
                term: term.clone(),
                tp: tp.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            CheckingTask::Equality(lhs, rhs) => TraceLine::Equality {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
                steps,
                context: context.to_boxed(),
                result: result.map(|r| cast!(r:bool)).copied(),
            },
            CheckingTask::Subtype(sub, sup) => TraceLine::SubtypeTask {
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
        task: CheckingTask,
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

    pub fn derive<'t>(&'t self, task: CheckingTask<'t>) -> SolverTrace<'t> {
        SolverTrace {
            task,
            cancelled: std::sync::atomic::AtomicBool::new(false),
            messages: SmallVec::default(),
            parent: Some(self),
        }
    }

    #[must_use]
    pub const fn new(task: CheckingTask<'s>) -> Self {
        Self {
            task,
            cancelled: std::sync::atomic::AtomicBool::new(false),
            messages: SmallVec::new(),
            parent: None,
        }
    }
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
        rule: Box<dyn CheckerRule>,
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
    Comment,
    Header,
    Emph,
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

            TraceLine::Msg(s, MessageLevel::Comment) => {
                use std::fmt::Display;
                self.maybe(&format_args!("{s}\n"), self.styles.variables.italic())
                    .fmt(f)?;
                Ok(&[])
            }
            TraceLine::Msg(s, MessageLevel::Emph) => {
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

#[derive(Clone, Copy, Default)]
pub struct Indent(pub usize);
impl Indent {
    pub const fn increase(&mut self) {
        self.0 += 1;
    }
    pub const fn decrease(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}
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

#[derive(Debug)]
pub enum CheckLogCow<'t> {
    Owned(CheckLog),
    Borrowed(RefCheckLog<'t>),
}
impl<'t> From<RefCheckLog<'t>> for CheckLogCow<'t> {
    #[inline]
    fn from(value: RefCheckLog<'t>) -> Self {
        Self::Borrowed(value)
    }
}
impl From<CheckLog> for CheckLogCow<'_> {
    #[inline]
    fn from(value: CheckLog) -> Self {
        Self::Owned(value)
    }
}

macro_rules! tasks {
    (
        $(
            $name:ident($($field:ident : $tp:ident),*) => $res:tt
        ),* $(,)?
    ) => {
        #[derive(Copy,Clone,Debug)]
        pub enum CheckingTask<'t> {
            $(
                $name($(tasks!(@TPBORROW $tp)),*)
            ),*,
            Rule(&'t dyn CheckerRule),
            Strategy(&'static str)
        }
        #[derive(Debug)]
        pub enum RefCheckLog<'t> {
            $(
                $name {
                    $($field: tasks!(@TPBORROW $tp),)*
                    steps:Box<[CheckLogCow<'t>]>,
                    context: Box<[Cow<'t, ComponentVar>]>,
                    result: Option<$res>,
                },
            )*
            Rule{
                rule: &'t dyn CheckerRule,
                steps:Box<[CheckLogCow<'t>]>,
            },
            Strategy{
                name: &'static str,
                steps:Box<[CheckLogCow<'t>]>,
            },
            Msg(Cow<'static, str>, MessageLevel),
            //Dyn(&'t dyn CheckTraceDisplayable)
            //Interpolated(Box<[DisplayableElem]>, MessageLevel),
        }
        #[derive(Debug)]
        pub enum CheckLog {
            $(
                $name {
                    $($field: tasks!(@TPOWN $tp),)*
                    steps:Box<[Self]>,
                    context:Box<[ComponentVar]>,
                    result:Option<$res>
                },
            )*
            Rule{
                rule: Box<dyn CheckerRule>,
                steps:Box<[Self]>,
            },
            Strategy{
                name: &'static str,
                steps:Box<[Self]>,
            },
            //Dyn(Box<dyn CheckTraceDisplayable>)
            Msg(Cow<'static, str>, MessageLevel),
            //Interpolated(Box<[DisplayableElem]>, MessageLevel),
        }
        impl CheckLog {
            pub(crate) fn display_i(&self,displayer:&impl TraceDisplay,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut curr = std::slice::from_ref(self).iter();
                let mut stack = Vec::new();
                let mut indent = Indent::default();
                loop {
                    while let Some(next) = curr.next() {
                        if displayer.line(next,indent,f)? == std::ops::ControlFlow::Continue(()) {
                            match next {
                                Self::Strategy{name,steps} => {
                                    displayer.string(*name,Some(MessageLevel::Header),f)?;
                                    indent.increase();
                                    stack.push(std::mem::replace(&mut curr,steps.iter()));
                                }
                                Self::Msg(s,lvl) => {
                                    displayer.string(&**s,Some(*lvl),f)?;
                                }
                                /*
                                Self::Dyn(d) => d.display(displayer,None,f)?,
                                Self::Interpolated(v,lvl) => {
                                    let mut first = true;
                                    for e in v {
                                        if !first {
                                            displayer.space(f)?;
                                        }
                                        first = false;
                                        e.display(displayer,*lvl,f)?;
                                    }
                                } */
                                _ => todo!()
                            }
                        }
                    }
                    if let Some(next) = stack.pop() {
                        indent.decrease();
                        curr = next;
                    } else {
                        break
                    }
                }
                Ok(())
            }
        }
        impl<'t> CheckingTask<'t> {
            pub(crate) fn close<R:Clone>(self,res:Option<&R>,steps:Box<[CheckLogCow<'t>]>,context:&[Cow<'t,ComponentVar>]) -> RefCheckLog<'t> {
                let context = context.iter().map(Cow::clone).collect();
                match self {
                    $(
                        Self::$name( $($field),* ) => RefCheckLog::$name {
                            $($field,)*
                            steps,
                            context,
                            result: res.map(|r| unsafe{&*std::ptr::from_ref(r).cast::<$res>()}.clone() )
                        },
                    )*
                    Self::Strategy(name) => RefCheckLog::Strategy { name, steps },
                    Self::Rule(rule) => RefCheckLog::Rule { rule, steps }
                }
            }
        }
        impl<'t> CheckLogCow<'t> {
            pub fn into_owned(self) -> CheckLog {
                match self {
                    Self::Owned(o) => o,
                    $(
                        Self::Borrowed(RefCheckLog::$name{$($field,)* steps,context,result}) => CheckLog::$name{
                            $($field:tasks!(@CONV $tp $field),)*
                            steps: steps.into_iter().map(Self::into_owned).collect(),
                            context: context.into_iter().map(Cow::into_owned).collect(),
                            result,

                        },
                    )*
                    Self::Borrowed(RefCheckLog::Msg(txt,lvl)) => CheckLog::Msg(txt,lvl),
                    /*
                    Self::Borrowed(RefCheckLog::Interpolated(v, lvl)) => CheckLog::Interpolated(v,lvl),
                    */
                    Self::Borrowed(RefCheckLog::Rule{rule,steps}) => CheckLog::Rule{
                        rule:rule.as_box_dyn(),
                        steps: steps.into_iter().map(Self::into_owned).collect(),
                    },
                    Self::Borrowed(RefCheckLog::Strategy{name,steps}) => CheckLog::Strategy{
                        name,
                        steps: steps.into_iter().map(Self::into_owned).collect(),
                    },
                }
            }
        }
    };
    (@TPBORROW Term) => {&'t Term};
    (@TPOWN Term) => {Term};
    (@CONV Term $name:ident) => { $name.clone() };
    (@TPBORROW str) => {&'t str};
    (@TPOWN str) => {Box<str>};
    (@CONV str $name:ident) => { $name.to_string().into_boxed_str() };
    (@TPBORROW SolverRule) => {&'t dyn SolverRule};
    (@TPOWN SolverRule) => {Box<dyn SolverRule>};
    (@CONV SolverRule $name:ident) => { $name.as_box_dyn() };
}

tasks! {
    Inference(term: Term) => Term,
    VariableInference(var: str) => Term,
    Inhabitable(term: Term) => bool,
    Universe(term:Term) => bool,
    Subtype(sub:Term,sup:Term) => bool,
    HasType(tm:Term,tp:Term) => bool,
    Equality(lhs:Term,rhs:Term) => bool,
}

impl CheckLog {
    pub fn display<'d, D: TraceDisplay>(
        &'d self,
        displayer: &'d D,
    ) -> impl std::fmt::Display + use<'d, D> {
        TraceDisplayer {
            d: displayer,
            trace: self,
        }
    }
}

pub trait CheckTraceDisplayable: std::fmt::Debug {
    /// ### Errors
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
    //fn as_owned(&self) -> CheckTraceCow<'static>;
}
#[derive(Debug)]
pub enum CheckTraceCow<'t> {
    Borrowed(&'t dyn CheckTraceDisplayable),
    Owned(Box<dyn CheckTraceDisplayable>),
}
impl CheckTraceDisplayable for CheckTraceCow<'_> {
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match self {
            Self::Borrowed(v) => v.display(displayer, lvl, f),
            Self::Owned(v) => v.display(displayer, lvl, f),
        }
    }
    /*
    fn as_owned(&self) -> CheckTraceCow<'static> {
        match self {
            Self::Borrowed(b) => b.as_owned(),
            Self::Owned(b) => b.as_owned(),
        }
    } */
}
impl<CD: CheckTraceDisplayable> CheckTraceDisplayable for &CD {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        CD::display(*self, displayer, lvl, f)
    }
    /*
    #[inline]
    fn as_owned(&self) -> CheckTraceCow<'static> {
        CD::as_owned(*self)
    } */
}

impl<CD: CheckTraceDisplayable> CheckTraceDisplayable for Box<[CD]> {
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for e in self {
            e.display(displayer, lvl, f)?;
        }
        Ok(())
    }
    /*
    fn as_owned(&self) -> CheckTraceCow<'static> {
        CheckTraceCow::Owned(Box::new(
            self.iter().map(|v| v.as_owned()).collect::<Box<[_]>>(),
        ))
    } */
}
impl<CD: CheckTraceDisplayable, const N: usize> CheckTraceDisplayable for [CD; N] {
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for e in self {
            e.display(displayer, lvl, f)?;
        }
        Ok(())
    }
    /*
    fn as_owned(&self) -> CheckTraceCow<'static> {
        CheckTraceCow::Owned(Box::new(self.iter().map(|v| v.as_owned()).collect::<Box<[_]>>()) as _)
    } */
}

impl CheckTraceDisplayable for &'static str {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.string(self, lvl, f)
    }
}
impl CheckTraceDisplayable for Term {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.term(self, lvl, f)
    }
}
impl CheckTraceDisplayable for Variable {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.variable(self, lvl, f)
    }
}
impl CheckTraceDisplayable for ftml_uris::Uri {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.uri(self.as_uri(), lvl, f)
    }
}
impl CheckTraceDisplayable for ftml_uris::UriRef<'_> {
    #[inline]
    fn display(
        &self,
        displayer: &dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.uri(*self, lvl, f)
    }
}

#[derive(Clone, Debug)]
pub enum DisplayableElem {
    Text(Cow<'static, str>),
    Term(Term),
    Var(Variable),
    Num(i128),
    Uri(ftml_uris::Uri),
}
impl DisplayableElem {
    fn display(
        &self,
        displayer: &impl TraceDisplay,
        lvl: MessageLevel,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Text(txt) => displayer.string(txt, Some(lvl), f),
            Self::Term(term) => displayer.term(term, Some(lvl), f),
            Self::Var(variable) => displayer.variable(variable, Some(lvl), f),
            Self::Num(i) => displayer.num(*i, Some(lvl), f),
            Self::Uri(uri) => displayer.uri(uri.as_uri(), Some(lvl), f),
        }
    }
}
/*
#[derive(Clone, Debug)]
pub enum DisplayableRef<'a> {
    Text(Cow<'static, str>),
    Term(&'a Term),
    Var(&'a Variable),
    Num(i128),
    Uri(ftml_uris::UriRef<'a>),
}
impl DisplayableRef<'_> {
    fn into_owned(self) -> DisplayableElem {
        match self {
            DisplayableRef::Num(i) => DisplayableElem::Num(i),
            DisplayableRef::Term(t) => DisplayableElem::Term(t.clone()),
            DisplayableRef::Text(s) => DisplayableElem::Text(s),
            DisplayableRef::Var(v) => DisplayableElem::Var(v.clone()),
            DisplayableRef::Uri(u) => DisplayableElem::Uri(u.owned()),
        }
    }
    fn display(
        &self,
        displayer: &impl TraceDisplay,
        lvl: MessageLevel,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Text(txt) => displayer.string(txt, Some(lvl), f),
            Self::Term(term) => displayer.term(term, Some(lvl), f),
            Self::Var(variable) => displayer.variable(variable, Some(lvl), f),
            Self::Num(i) => displayer.num(*i, Some(lvl), f),
            Self::Uri(uri) => displayer.uri(*uri, Some(lvl), f),
        }
    }
}
 */

impl From<&'static str> for DisplayableElem {
    fn from(value: &'static str) -> Self {
        Self::Text(Cow::Borrowed(value))
    }
}
impl From<String> for DisplayableElem {
    fn from(value: String) -> Self {
        Self::Text(Cow::Owned(value))
    }
}
impl From<Term> for DisplayableElem {
    fn from(value: Term) -> Self {
        Self::Term(value)
    }
}
impl From<&Term> for DisplayableElem {
    fn from(value: &Term) -> Self {
        Self::Term(value.clone())
    }
}
impl From<Variable> for DisplayableElem {
    fn from(value: Variable) -> Self {
        Self::Var(value)
    }
}
impl From<&Variable> for DisplayableElem {
    fn from(value: &Variable) -> Self {
        Self::Var(value.clone())
    }
}
macro_rules! impl_num {
    ($($t:ty),* $(,)?) => {
        $(
            impl CheckTraceDisplayable for $t {
                #[inline]
                fn display(
                    &self,
                    displayer: &dyn TraceDisplay,
                    lvl: Option<MessageLevel>,
                    f: &mut std::fmt::Formatter,
                ) -> std::fmt::Result {
                    #[allow(clippy::cast_lossless)]
                    displayer.num(*self as _, lvl, f)
                }
            }
            impl From<$t> for DisplayableElem {
                fn from(value:$t) -> Self {
                    #[allow(clippy::cast_lossless)]
                    Self::Num(value as _)
                }
            }
        )*
    }
}
impl_num!(u8, i8, u16, i16, u32, i32, u64, i64, i128, usize, isize);

pub trait TraceDisplay {
    /// ### Errors
    fn line(
        &self,
        _: &CheckLog,
        indent: Indent,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<std::ops::ControlFlow<()>, std::fmt::Error> {
        f.write_char('\n')?;
        self.indent(indent, None, f)?;
        Ok(std::ops::ControlFlow::Continue(()))
    }

    /// ### Errors
    fn uri(
        &self,
        uri: ftml_uris::UriRef,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn term(
        &self,
        term: &Term,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn string(
        &self,
        s: &str,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn variable(
        &self,
        var: &Variable,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn num(
        &self,
        num: i128,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn indent(
        &self,
        indent: Indent,
        lvl: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;

    /// ### Errors
    fn space(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
impl TraceDisplay for () {
    fn space(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(' ')
    }
    fn uri(
        &self,
        uri: ftml_uris::UriRef,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        std::fmt::Display::fmt(&uri, f)
    }
    fn indent(
        &self,
        indent: Indent,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{indent} ")
    }
    fn term(
        &self,
        term: &Term,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        <_ as std::fmt::Debug>::fmt(&term.debug_short(), f)
    }
    fn string(
        &self,
        s: &str,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(s)
    }
    fn variable(
        &self,
        var: &Variable,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(var.name())
    }
    fn num(
        &self,
        num: i128,
        _: Option<MessageLevel>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        <i128 as std::fmt::Display>::fmt(&num, f)
    }
}

struct TraceDisplayer<'d, D: TraceDisplay> {
    d: &'d D,
    trace: &'d CheckLog,
}
impl<D: TraceDisplay> std::fmt::Display for TraceDisplayer<'_, D> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.trace.display_i(self.d, f)
    }
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
    (_ $($e:expr),* $(,)?) => {
        $crate::traceline!(@WRAP
            $($e),*;
            $crate::trace::MessageLevel::Emph
        )
    };
    ($($e:expr),* $(,)?) => {
        $crate::traceline!(@WRAP
            $($e),*;
            $crate::trace::MessageLevel::Comment
        )
    };
    (@WRAP $l:literal;$lvl:expr) => {
        $crate::trace::RefCheckLog::Msg(std::borrow::Cow::Borrowed($l),$lvl)
    };
    (@WRAP $($e:expr),*;$lvl:expr) => {
        $crate::trace::RefCheckLog::Interpolated(
            Box::new([$($e.into()),*]),
            $lvl
        )
    }
}

/*
fn test() -> RefCheckLog<'static> {
    use ftml_ontology::terms::Numeric;
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
