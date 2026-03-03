pub mod results;

use ftml_ontology::terms::{ComponentVar, Term, Variable};
use ftml_uris::{FtmlUri, Uri};
#[cfg(feature = "colors")]
use owo_colors::OwoColorize;
use std::borrow::Cow;
use std::{fmt::Write, marker::PhantomData};

#[cfg(feature = "full")]
pub trait CheckerRule: std::fmt::Debug + Send + Sync + std::any::Any {
    fn priority(&self) -> isize {
        0
    }
    fn display(&self) -> Vec<Displayable>;
    fn as_box_dyn(&self) -> Box<dyn CheckerRule>;
    fn as_dyn(&self) -> &dyn CheckerRule;
    fn as_any(&self) -> &dyn std::any::Any;
    fn eq(&self, o: &dyn CheckerRule) -> bool;
}

#[cfg(feature = "full")]
pub trait SizedSolverRule:
    std::fmt::Debug + Send + Sync + std::any::Any + Clone + Sized + PartialEq + Eq
{
    fn priority(&self) -> isize {
        0
    }
    fn display(&self) -> Vec<Displayable>;
}

#[cfg(feature = "full")]
impl<T: SizedSolverRule> CheckerRule for T {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn priority(&self) -> isize {
        <Self as SizedSolverRule>::priority(self)
    }

    #[inline]
    fn display(
        &self,
        //f: &mut std::fmt::Formatter,
    ) -> Vec<Displayable> {
        <T as SizedSolverRule>::display(self)
    }

    fn as_box_dyn(&self) -> Box<dyn CheckerRule> {
        Box::new(self.clone()) as _
    }
    fn as_dyn(&self) -> &dyn CheckerRule {
        self as _
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
    fn eq(&self, o: &dyn CheckerRule) -> bool {
        o.as_any().downcast_ref::<T>().is_some_and(|v| v == self)
    }
}

#[cfg(feature = "full")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageLevel {
    Failure,
    Comment,
    Header,
    Emph,
}

#[cfg(feature = "full")]
#[derive(Clone, Copy, Default)]
pub struct Indent(pub usize);
#[cfg(feature = "full")]
impl Indent {
    pub const fn increase(&mut self) {
        self.0 += 1;
    }
    pub const fn decrease(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}
#[cfg(feature = "full")]
impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            return Ok(());
        }
        for _ in 0..self.0 - 1 {
            f.write_str("  │")?;
        }
        f.write_str("  ├─")?;
        Ok(())
    }
}

#[cfg(feature = "full")]
#[derive(Debug)]
pub enum CheckLogCow<'t> {
    Owned(PreCheckLog),
    Borrowed(RefCheckLog<'t>),
}
#[cfg(feature = "full")]
impl<'t> From<RefCheckLog<'t>> for CheckLogCow<'t> {
    #[inline]
    fn from(value: RefCheckLog<'t>) -> Self {
        Self::Borrowed(value)
    }
}
#[cfg(feature = "full")]
impl From<PreCheckLog> for CheckLogCow<'_> {
    #[inline]
    fn from(value: PreCheckLog) -> Self {
        Self::Owned(value)
    }
}

macro_rules! tasks {
    (
        $(
            $name:ident($($field:ident : $tp:ident),*) => $res:tt
        ),* $(,)?
    ) => {

        #[cfg(feature = "full")]
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
                success:bool
            },
            Msg(Cow<'static, str>, MessageLevel),
            //Dyn(&'t dyn CheckTraceDisplayable)
            //Interpolated(Box<[DisplayableElem]>, MessageLevel),
        }
        #[cfg(feature = "full")]
        impl RefCheckLog<'_> {
            pub fn into_owned(self,term:&impl Fn(Term) -> Term) -> PreCheckLog {
                match self {
                    $(
                        Self::$name{$($field,)* steps,context,result} => PreCheckLog::$name{
                            $($field:tasks!(@CONV $tp $field term),)*
                            steps: steps.into_iter().map(|t| CheckLogCow::into_owned(t,term)).collect(),
                            context: context.into_iter().map(Cow::into_owned).collect(),
                            result,

                        },
                    )*
                    Self::Msg(txt,lvl) => PreCheckLog::Msg(txt,lvl),
                    Self::Rule{rule,steps} => PreCheckLog::Rule{
                        rule:rule.as_box_dyn(),
                        steps: steps.into_iter().map(|t| CheckLogCow::into_owned(t,term)).collect(),
                    },
                    Self::Strategy{name,steps,success} => PreCheckLog::Strategy{
                        name,
                        steps: steps.into_iter().map(|t| CheckLogCow::into_owned(t,term)).collect(),
                        success
                    }
                }
            }
        }
        #[cfg(feature = "full")]
        #[derive(Debug)]
        pub enum PreCheckLog {
            $(
                $name {
                    $($field: tasks!(@TPOWN $tp),)*
                    steps:Box<[Self]>,
                    context:Box<[ComponentVar]>,
                    result:Option<$res>
                },
            )*
            Rule{
                rule:Box<dyn CheckerRule>,
                steps:Box<[Self]>,
            },
            Strategy{
                name: &'static str,
                steps:Box<[Self]>,
                success:bool
            },
            //Dyn(Box<dyn CheckTraceDisplayable>)
            Msg(Cow<'static, str>, MessageLevel),
            Count(&'static str,usize)
            //Interpolated(Box<[DisplayableElem]>, MessageLevel),
        }

        #[cfg(feature = "full")]
        impl CheckLog {
            pub fn from_pre(v:PreCheckLog,terms:&mut impl FnMut(Term) -> Term) -> Self {
                use PreCheckLog as P;
                match v {
                    $(
                        P::$name{
                            $($field,)*steps,context,result
                        } => Self::$name{
                            $( $field:tasks!(@FROMPRE $tp $field terms),)*
                            context,result:result.map(|r| tasks!(@FROMPRE $res r terms)),
                            steps:steps.into_iter().map(|e| Self::from_pre(e,terms)).collect()
                        },
                    )*
                    P::Rule{ rule, steps } => Self::Rule {
                        header:Displayable::map(rule.display(),terms),
                        steps:steps.into_iter().map(|e| Self::from_pre(e,terms)).collect()
                    },
                    P::Strategy{ name, steps, success } => Self::Strategy {
                        name:name.to_string(),
                        steps:steps.into_iter().map(|e| Self::from_pre(e,terms)).collect(),
                        success
                    },
                    P::Msg(s, MessageLevel::Comment) => Self::Comment(s.into_owned()),
                    P::Msg(s, MessageLevel::Emph) => Self::Emph(s.into_owned()),
                    P::Msg(s, MessageLevel::Header) => Self::Header(s.into_owned()),
                    P::Msg(s, MessageLevel::Failure) => Self::Fail(s.into_owned()),
                    P::Count(s, u) =>
                        Self::Comment(format!("{s} {u}"))
                }
            }
        }
        #[cfg(feature = "full")]
        impl<'t> CheckingTask<'t> {
            pub fn close<R:Clone>(self,res:Option<&R>,steps:Box<[CheckLogCow<'t>]>,context:&[Cow<'t,ComponentVar>]) -> RefCheckLog<'t> {
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
                    Self::Strategy(name) => RefCheckLog::Strategy {
                        name,
                        steps,success:res.is_some_and(|v| {
                            if std::mem::size_of::<R>() == 1 {
                                // SAFETY: => boolean
                                unsafe{
                                    std::mem::transmute_copy::<R,bool>(v)
                                }
                            } else {
                                true
                            }
                        })
                    },
                    Self::Rule(rule) => RefCheckLog::Rule { rule, steps }
                }
            }
        }
        #[cfg(feature = "full")]
        impl<'t> CheckLogCow<'t> {
            pub fn into_owned(self,term:&impl Fn(Term) -> Term) -> PreCheckLog {
                match self {
                    Self::Owned(o) => o,
                    Self::Borrowed(b) => b.into_owned(term)
                }
            }
        }

        #[cfg(feature = "full")]
        #[derive(Copy,Clone,Debug)]
        pub enum CheckingTask<'t> {
            $(
                $name($(tasks!(@TPBORROW $tp)),*)
            ),*,
            Rule(&'t dyn CheckerRule),
            Strategy(&'static str),
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        pub enum CheckLog {
            $(
                $name {
                    $($field: tasks!(@TPOWN $tp),)*
                    steps:Box<[Self]>,
                    context:Box<[ComponentVar]>,
                    result:Option<$res>
                },
            )*
            Comment(String),
            Emph(String),
            Header(String),
            Fail(String),
            Strategy {
                name: String,
                steps: Vec<Self>,
                success: bool,
            },
            Rule {
                header: Vec<Displayable>,
                steps: Vec<Self>,
            },
        }
        #[cfg(feature = "full")]
        impl CheckLog {
            pub(crate) fn display_i(&self,displayer:&mut impl TraceDisplay) -> std::fmt::Result {
                let mut curr = std::slice::from_ref(self).iter();
                let mut stack = Vec::new();
                let mut indent = Indent::default();
                loop {
                    while let Some(next) = curr.next() {
                        if displayer.line(next,indent)? == std::ops::ControlFlow::Continue(()) {
                            match next {
                                $(
                                    Self::$name{ $($field,)* steps, context,result } => {
                                        displayer.task(CheckingTask::$name($($field),*),context,result.is_some())?;
                                        tasks!(@DISPL result displayer $res);
                                        indent.increase();
                                        stack.push(std::mem::replace(&mut curr,steps.iter()));
                                    }
                                )*
                                Self::Rule{header,steps} => {
                                    for e in header {
                                        displayer.displayable(e,None)?;
                                    }
                                    indent.increase();
                                    stack.push(std::mem::replace(&mut curr,steps.iter()));
                                }
                                Self::Strategy{name,steps,success} => {
                                    displayer.strategy(name,&[],*success)?;
                                    indent.increase();
                                    stack.push(std::mem::replace(&mut curr,steps.iter()));
                                }
                                Self::Comment(s) => {
                                    displayer.string(&s,Some(MessageLevel::Comment))?;
                                }
                                Self::Emph(s) => {
                                    displayer.string(&s,Some(MessageLevel::Emph))?;
                                }
                                Self::Header(s) => {
                                    displayer.string(&s,Some(MessageLevel::Header))?;
                                }
                                Self::Fail(s) => {
                                    displayer.string(&s,Some(MessageLevel::Failure))?;
                                }
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

    };
    (@DISPL $res:ident $disp:ident Term) => {
        if let Some(t) = $res {
            $disp.string(": ",None)?;
            $disp.term(t,None)?;
        }
    };
    (@DISPL $res:ident $disp:ident bool) => {};
    (@TPBORROW Term) => {&'t Term};
    (@TPOWN Term) => {Term};
    (@TPBORROW str) => {&'t str};
    (@TPOWN str) => {Box<str>};
    (@FROMPRE Term $name:ident $f:ident) => {$f($name)};
    (@FROMPRE str $name:ident $f:ident) => {$name};
    (@FROMPRE bool $name:ident $f:ident) => {$name};
    (@CONV Term $name:ident $f:ident) => {$name.clone()};//{ $f($name.clone()) };
    (@CONV str $name:ident $f:ident) => { $name.to_string().into_boxed_str() };
    //(@TPBORROW SolverRule) => {&'t dyn SolverRule};
    //(@TPOWN SolverRule) => {Box<dyn SolverRule>};
    (@CONV SolverRule $name:ident $f:ident) => { $name.as_box_dyn() };
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum Displayable {
    //Log(CheckLog),
    Num(i128),
    Space,
    String(String),
    Term(Term),
    Uri(Uri),
    Var(Variable),
}
impl Displayable {
    fn map(v: Vec<Self>, terms: &mut impl FnMut(Term) -> Term) -> Vec<Self> {
        v.into_iter()
            .map(|e| {
                if let Self::Term(t) = e {
                    Self::Term(terms(t))
                } else {
                    e
                }
            })
            .collect()
    }
}

tasks! {
    Simplify(term:Term) => Term,
    Inference(term: Term) => Term,
    VariableInference(var: str) => Term,
    //Simplify(term:Term) => Term,
    Inhabitable(term: Term) => bool,
    Universe(term:Term) => bool,
    Subtype(sub:Term,sup:Term) => bool,
    HasType(tm:Term,tp:Term) => bool,
    Equality(lhs:Term,rhs:Term) => bool,
}

#[cfg(feature = "full")]
impl CheckLog {
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        TraceDisplayer::<'_, D> {
            trace: self,
            d: PhantomData,
        }
    }
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        TraceDisplayer {
            d: PhantomData::<ColorDisplay>,
            trace: self,
        }
    }
}
#[cfg(feature = "full")]
impl std::fmt::Display for CheckLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display::<()>().fmt(f)
    }
}

#[cfg(feature = "full")]
pub trait FmtTraceDisplay {
    fn new(f: &mut std::fmt::Formatter<'_>) -> impl TraceDisplay;
}

#[cfg(feature = "full")]
pub trait TraceDisplay {
    // /// ### Errors
    //fn rule(&mut self, rule: &dyn CheckerRule) -> std::fmt::Result;

    /// ### Errors
    fn displayable(&mut self, d: &Displayable, lvl: Option<MessageLevel>) -> std::fmt::Result {
        match d {
            Displayable::Num(i) => self.num(*i, lvl),
            Displayable::Space => self.space(),
            Displayable::String(s) => self.string(s, lvl),
            Displayable::Term(t) => self.term(t, lvl),
            Displayable::Uri(u) => self.uri(u.as_uri(), lvl),
            Displayable::Var(v) => self.variable(v, lvl),
        }
    }

    /// ### Errors
    fn line(
        &mut self,
        _: &CheckLog,
        indent: Indent,
    ) -> Result<std::ops::ControlFlow<()>, std::fmt::Error>; /* {
    f.write_char('\n')?;
    self.indent(indent, None, f)?;
    Ok(std::ops::ControlFlow::Continue(()))
    }*/

    /// ### Errors
    fn task(
        &mut self,
        task: CheckingTask<'_>,
        context: &[ComponentVar],
        success: bool,
    ) -> std::fmt::Result;

    /// ### Errors
    fn strategy(&mut self, name: &str, context: &[ComponentVar], success: bool)
    -> std::fmt::Result;

    /// ### Errors
    fn uri(&mut self, uri: ftml_uris::UriRef, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn term(&mut self, term: &Term, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn string(&mut self, s: &str, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn variable(&mut self, var: &Variable, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn num(&mut self, num: i128, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn indent(&mut self, indent: Indent, lvl: Option<MessageLevel>) -> std::fmt::Result;

    /// ### Errors
    fn space(&mut self) -> std::fmt::Result;
}
#[cfg(feature = "full")]
impl FmtTraceDisplay for () {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn new(f: &mut std::fmt::Formatter<'_>) -> impl TraceDisplay {
        f
    }
}
#[cfg(feature = "full")]
impl TraceDisplay for &mut std::fmt::Formatter<'_> {
    fn line(
        &mut self,
        _: &CheckLog,
        indent: Indent,
        //f: &mut std::fmt::Formatter<'_>,
    ) -> Result<std::ops::ControlFlow<()>, std::fmt::Error> {
        self.write_char('\n')?;
        self.indent(indent, None)?;
        Ok(std::ops::ControlFlow::Continue(()))
    }
    fn space(&mut self) -> std::fmt::Result {
        self.write_char(' ')
    }
    /*
    fn rule(&mut self, rule: &dyn CheckerRule) -> std::fmt::Result {
        self.write_str("Using rule: ")?;
        rule.display(self, None)
    }
     */

    fn strategy(&mut self, name: &str, _: &[ComponentVar], _: bool) -> std::fmt::Result {
        write!(self, "Strategy: {name}")
    }
    fn task(
        &mut self,
        task: CheckingTask<'_>,
        context: &[ComponentVar],
        success: bool,
        //f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        fn do_context(
            context: &[ComponentVar],
            mut f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            if context.is_empty() {
                return Ok(());
            }
            f.write_str("{... ")?;
            let mut first = true;
            for ComponentVar { var, tp, df } in context {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }
                f.variable(var, None)?;
                if let Some(tp) = tp {
                    f.write_str(" : ")?;
                    f.term(tp, None)?;
                }
                if let Some(df) = df {
                    f.write_str(" : ")?;
                    f.term(df, None)?;
                }
            }
            f.write_str(" } ")
        }
        if success {
            self.write_str("[SUCCESS] ")?;
        } else {
            self.write_str("[FAILED] ")?;
        }
        match task {
            CheckingTask::Simplify(t) => {
                self.write_str("Simplifying ")?;
                do_context(context, self)?;
                self.term(t, None)
            }
            CheckingTask::Inference(t) => {
                self.write_str("Inferring type of ")?;
                do_context(context, self)?;
                self.term(t, None)
            }
            CheckingTask::VariableInference(t) => {
                self.write_str("Inferring type of variable ")?;
                do_context(context, self)?;
                self.write_str(t)
            }
            CheckingTask::Inhabitable(tm) => {
                self.write_str("Checking inhabitability ")?;
                do_context(context, self)?;
                self.write_str("⊢ INH ")?;
                self.term(tm, None)
            }
            CheckingTask::Universe(tm) => {
                self.write_str("Checking universe ")?;
                do_context(context, self)?;
                self.write_str("⊢ UNIV ")?;
                self.term(tm, None)
            }
            CheckingTask::Subtype(sub, sup) => {
                self.write_str("Checking subtyping ")?;
                do_context(context, self)?;
                self.write_str("⊢ ")?;
                self.term(sub, None)?;
                self.write_str(" <: ")?;
                self.term(sup, None)
            }
            CheckingTask::HasType(tm, tp) => {
                self.write_str("Checking typing ")?;
                do_context(context, self)?;
                self.write_str("⊢ ")?;
                self.term(tm, None)?;
                self.write_str(" : ")?;
                self.term(tp, None)
            }
            CheckingTask::Equality(lhs, rhs) => {
                self.write_str("Checking equality ")?;
                do_context(context, self)?;
                self.write_str("⊢ ")?;
                self.term(lhs, None)?;
                self.write_str(" == ")?;
                self.term(rhs, None)
            }
            CheckingTask::Rule(rl) => {
                for e in rl.display() {
                    self.displayable(&e, None)?;
                }
                Ok(())
            }
            CheckingTask::Strategy(s) => self.strategy(s, context, success),
        }
    }
    fn uri(&mut self, uri: ftml_uris::UriRef, _: Option<MessageLevel>) -> std::fmt::Result {
        match uri {
            ftml_uris::UriRef::Symbol(s) => {
                std::fmt::Display::fmt(&s.module.name, self)?;
                self.write_char('?')?;
                std::fmt::Display::fmt(&s.name, self)
            }
            _ => std::fmt::Display::fmt(&uri, self),
        }
    }
    fn indent(&mut self, indent: Indent, _: Option<MessageLevel>) -> std::fmt::Result {
        write!(self, "{indent} ")
    }
    fn term(&mut self, term: &Term, _: Option<MessageLevel>) -> std::fmt::Result {
        <_ as std::fmt::Debug>::fmt(&term.debug_short(), self)
    }
    fn string(&mut self, s: &str, _: Option<MessageLevel>) -> std::fmt::Result {
        self.write_str(s)
    }
    fn variable(&mut self, var: &Variable, _: Option<MessageLevel>) -> std::fmt::Result {
        self.write_str(var.name())
    }
    fn num(&mut self, num: i128, _: Option<MessageLevel>) -> std::fmt::Result {
        <i128 as std::fmt::Display>::fmt(&num, self)
    }
}

#[cfg(feature = "colors")]
pub struct ColorDisplay<'a, 'b>(&'a mut std::fmt::Formatter<'b>);
#[cfg(feature = "colors")]
impl FmtTraceDisplay for ColorDisplay<'_, '_> {
    fn new(f: &mut std::fmt::Formatter<'_>) -> impl TraceDisplay {
        ColorDisplay(f)
    }
}
#[cfg(feature = "colors")]
impl TraceDisplay for ColorDisplay<'_, '_> {
    fn line(
        &mut self,
        _: &CheckLog,
        indent: Indent,
    ) -> Result<std::ops::ControlFlow<()>, std::fmt::Error> {
        self.0.write_char('\n')?;
        self.indent(indent, None)?;
        Ok(std::ops::ControlFlow::Continue(()))
    }

    fn space(&mut self) -> std::fmt::Result {
        self.0.write_char(' ')
    }

    /*
    fn rule(&mut self, rule: &dyn CheckerRule) -> std::fmt::Result {
        write!(self.0, "{} ", "Using rule: ".italic())?;
        rule.display(self, None)
    }
     */

    fn strategy(&mut self, name: &str, _: &[ComponentVar], _: bool) -> std::fmt::Result {
        write!(self.0, "Strategy: {}", name.italic())
    }

    fn task(
        &mut self,
        task: CheckingTask<'_>,
        context: &[ComponentVar],
        success: bool,
    ) -> std::fmt::Result {
        fn do_context(context: &[ComponentVar], f: &mut ColorDisplay<'_, '_>) -> std::fmt::Result {
            if context.is_empty() {
                return Ok(());
            }
            f.0.write_str("{... ")?;
            let mut first = true;
            for ComponentVar { var, tp, df } in context {
                if first {
                    first = false;
                } else {
                    f.0.write_str(", ")?;
                }
                f.variable(var, None)?;
                if let Some(tp) = tp {
                    f.0.write_str(" : ")?;
                    f.term(tp, None)?;
                }
                if let Some(df) = df {
                    f.0.write_str(" : ")?;
                    f.term(df, None)?;
                }
            }
            f.0.write_str(" } ")
        }
        if success {
            write!(self.0, "{} ", "[SUCCESS]".green())?;
        } else {
            write!(self.0, "{} ", "[FAILED]".red())?;
        }
        match task {
            CheckingTask::Simplify(t) => {
                write!(self.0, "{} ", "Simplifying".bright_white().bold())?;
                do_context(context, self)?;
                self.term(t, None)
            }
            CheckingTask::Inference(t) => {
                write!(self.0, "{} ", "Inferring type of".bright_white().bold())?;
                do_context(context, self)?;
                self.term(t, None)
            }
            CheckingTask::VariableInference(t) => {
                write!(
                    self.0,
                    "{} ",
                    "Inferring type of variable".bright_white().bold()
                )?;
                do_context(context, self)?;
                self.0.write_str(t)
            }
            CheckingTask::Inhabitable(tm) => {
                write!(
                    self.0,
                    "{} ",
                    "Checking inhabitability".bright_white().bold()
                )?;
                do_context(context, self)?;
                write!(self.0, "{} ", "⊢ INH".bright_white().bold())?;
                self.term(tm, None)
            }
            CheckingTask::Universe(tm) => {
                write!(self.0, "{} ", "Checking universe".bright_white().bold())?;
                do_context(context, self)?;
                write!(self.0, "{} ", "⊢ UNIV".bright_white().bold())?;
                self.term(tm, None)
            }
            CheckingTask::Subtype(sub, sup) => {
                write!(self.0, "{} ", "Checking subtyping".bright_white().bold())?;
                do_context(context, self)?;
                write!(self.0, "{} ", "⊢".bright_white().bold())?;
                self.term(sub, None)?;
                write!(self.0, " {} ", "<:".bright_white().bold())?;
                self.term(sup, None)
            }
            CheckingTask::HasType(tm, tp) => {
                write!(self.0, "{} ", "Checking typing".bright_white().bold())?;
                do_context(context, self)?;
                write!(self.0, "{} ", "⊢".bright_white().bold())?;
                self.term(tm, None)?;
                write!(self.0, " {} ", ":".bright_white().bold())?;
                self.term(tp, None)
            }
            CheckingTask::Equality(lhs, rhs) => {
                write!(self.0, "{} ", "Checking equality".bright_white().bold())?;
                do_context(context, self)?;
                write!(self.0, "{} ", "⊢".bright_white().bold())?;
                self.term(lhs, None)?;
                write!(self.0, " {} ", "==".bright_white().bold())?;
                self.term(rhs, None)
            }
            CheckingTask::Rule(rl) => {
                for e in rl.display() {
                    self.displayable(&e, None)?;
                }
                Ok(())
            }
            CheckingTask::Strategy(s) => self.strategy(s, context, success),
        }
    }
    fn uri(&mut self, uri: ftml_uris::UriRef, _: Option<MessageLevel>) -> std::fmt::Result {
        match uri {
            ftml_uris::UriRef::Symbol(s) => {
                std::fmt::Display::fmt(&s.module.name, self.0)?;
                self.0.write_char('?')?;
                std::fmt::Display::fmt(&s.name, self.0)
            }
            _ => std::fmt::Display::fmt(&uri, self.0),
        }
    }
    fn indent(&mut self, indent: Indent, _: Option<MessageLevel>) -> std::fmt::Result {
        write!(self.0, "{} ", indent.blue())
    }
    fn term(&mut self, term: &Term, _: Option<MessageLevel>) -> std::fmt::Result {
        write!(self.0, "{:?}", term.debug_short().yellow())
    }
    fn string(&mut self, s: &str, _: Option<MessageLevel>) -> std::fmt::Result {
        write!(self.0, "{}", s.bright_black())
    }
    fn variable(&mut self, var: &Variable, _: Option<MessageLevel>) -> std::fmt::Result {
        self.0.write_str(var.name())
    }
    fn num(&mut self, num: i128, _: Option<MessageLevel>) -> std::fmt::Result {
        <i128 as std::fmt::Display>::fmt(&num, self.0)
    }
}

#[cfg(feature = "full")]
struct TraceDisplayer<'d, D: FmtTraceDisplay> {
    trace: &'d CheckLog,
    d: PhantomData<D>,
}
#[cfg(feature = "full")]
impl<D: FmtTraceDisplay> std::fmt::Display for TraceDisplayer<'_, D> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.trace.display_i(&mut D::new(f))
    }
}

impl<T: FtmlUri> From<&T> for Displayable {
    fn from(value: &T) -> Self {
        Self::Uri(value.as_uri().owned())
    }
}
impl From<&str> for Displayable {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<String> for Displayable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[cfg(feature = "full")]
#[macro_export]
macro_rules! trace {
    ($($e:expr),* $(,)? ) => {
        {vec![$(
            $e.into()
        ),*]
        }
    }
}
