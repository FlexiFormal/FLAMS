pub mod rules;
pub mod directives;

use crate::quickparse::tokens::TeXToken;
use immt_utils::{
    parsing::{ParseSource, ParseStr, StringOrStr},
    prelude::*,
    sourcerefs::{SourcePos, SourceRange},
};
use rules::{AnyEnv, AnyMacro, EnvironmentResult, EnvironmentRule, MacroResult, MacroRule};
use std::collections::hash_map::Entry;
use std::convert::Into;
use std::marker::PhantomData;
use tex_engine::utils::HMap;


pub trait FromLaTeXToken<'a, Pos:SourcePos, Str:StringOrStr<'a>>: Sized + std::fmt::Debug {
    fn from_comment(r: SourceRange<Pos>) -> Option<Self>;
    fn from_group(r: SourceRange<Pos>, children: Vec<Self>) -> Option<Self>;
    fn from_math(display: bool, r: SourceRange<Pos>, children: Vec<Self>) -> Option<Self>;
    fn from_control_sequence(start: Pos, name: Str) -> Option<Self>;
    fn from_text(r: SourceRange<Pos>, text: Str) -> Option<Self>;
    fn from_macro_application(m: Macro<'a, Pos, Str>) -> Option<Self>;
    fn from_environment(e: Environment<'a, Pos, Str,Self>) -> Option<Self>;
}

#[derive(Debug)]
pub enum LaTeXToken<'a, 
    Pos:SourcePos, 
    Str:StringOrStr<'a>
> {
    Comment(SourceRange<Pos>),
    Group {
        range: SourceRange<Pos>,
        children: Vec<Self>,
    },
    Math {
        display: bool,
        range: SourceRange<Pos>,
        children: Vec<Self>,
    },
    ControlSequence {
        start: Pos,
        name: Str,
    },
    Text {
        range: SourceRange<Pos>,
        text: Str,
    },
    MacroApplication(Macro<'a, Pos,Str>),
    Environment(Environment<'a, Pos, Str, Self>),
}

impl<'a, Pos:SourcePos, Str:StringOrStr<'a>> FromLaTeXToken<'a, Pos, Str> for LaTeXToken<'a, Pos, Str> {
    #[inline]
    fn from_comment(r: SourceRange<Pos>) -> Option<Self> {
        Some(LaTeXToken::Comment(r))
    }
    #[inline]
    fn from_group(r: SourceRange<Pos>, children: Vec<Self>) -> Option<Self> {
        Some(LaTeXToken::Group { range: r, children })
    }
    #[inline]
    fn from_math(display: bool, r: SourceRange<Pos>, children: Vec<Self>) -> Option<Self> {
        Some(LaTeXToken::Math {
            display,
            range: r,
            children,
        })
    }
    #[inline]
    fn from_control_sequence(start: Pos, name: Str) -> Option<Self> {
        Some(LaTeXToken::ControlSequence { start, name })
    }
    #[inline]
    fn from_text(range: SourceRange<Pos>, text: Str) -> Option<Self> {
        Some(LaTeXToken::Text { range, text })
    }
    #[inline]
    fn from_macro_application(m: Macro<'a, Pos, Str>) -> Option<Self> {
        Some(LaTeXToken::MacroApplication(m))
    }
    #[inline]
    fn from_environment(e: Environment<'a, Pos, Str, Self>) -> Option<Self> {
        Some(LaTeXToken::Environment(e))
    }
}

#[derive(Debug)]
pub struct Macro<'a, Pos:SourcePos,Str:StringOrStr<'a>> {
    pub token_range:SourceRange<Pos>,
    pub range: SourceRange<Pos>,
    pub name: Str,
    //pub args: Vec<T>,
    phantom: PhantomData<&'a str>,
}

#[derive(Debug)]
pub struct Environment<'a, Pos:SourcePos, Str:StringOrStr<'a>, T:FromLaTeXToken<'a,Pos, Str>> {
    pub begin: Macro<'a, Pos, Str>,
    pub end: Option<Macro<'a, Pos, Str>>,
    pub name: Str,
    pub name_range: SourceRange<Pos>,
    //pub args: Vec<T>,
    pub children: Vec<T>,
    //phantom:PhantomData<&'a T>
}

pub struct OptArg<'a, Pos:SourcePos, Str:StringOrStr<'a>> {
    inner: Option<Str>,
    range:SourceRange<Pos>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, Pos:SourcePos, Str:StringOrStr<'a>> OptArg<'a, Pos,Str> {
    #[inline]
    pub const fn is_some(&self) -> bool {
        self.inner.is_some()
    }
    pub fn into_name(self) -> Option<(Str,SourceRange<Pos>)> {
        self.inner.map(|i| (i,self.range))
    }
    pub fn as_keyvals(&'a self) -> VecMap<&'a str, OptVal<'a,Pos>> {
        let mut map = VecMap::default();
        if let Some(s) = &self.inner {
            let mut curr = self.range;
            for e in s.split_noparens::<'{', '}'>(',') {
                if let Some((a, b)) = e.split_once('=') {
                    curr.end.update_str_maybe_newline(a);
                    let key_range = curr;
                    curr.end.update('=');
                    curr.start = curr.end;
                    curr.end.update_str_maybe_newline(b);
                    let val_range = curr;
                    curr.end.update(',');
                    curr.start = curr.end;
                    let a = a.trim();
                    map.insert(a, OptVal {
                        key:a,
                        key_range,
                        val:b.trim(),
                        val_range
                    });
                } else {
                    curr.end.update_str_maybe_newline(e);
                    let key_range = curr;
                    curr.end.update(',');
                    curr.start = curr.end;
                    map.insert(e.trim(), OptVal {
                        key:e,
                        key_range,
                        val:"",
                        val_range:curr
                    });
                }
            }
        }
        map
    }
}

pub struct OptVal<'a,Pos:SourcePos> {
    pub key:&'a str,
    pub key_range: SourceRange<Pos>,
    pub val:&'a str,
    pub val_range: SourceRange<Pos>,
}

pub struct OptMapVal<'a,
    Pos:SourcePos,
    Str:StringOrStr<'a>,
    T:FromLaTeXToken<'a,Pos,Str>
> {
    pub key_range: SourceRange<Pos>,
    pub val_range: SourceRange<Pos>,
    pub val: Vec<T>,
    pub str:&'a str,
    phantom:PhantomData<Str>
}

pub struct OptMap<'a,
    Pos:SourcePos,
    Str:StringOrStr<'a>,
    T:FromLaTeXToken<'a,Pos,Str>
> {
    pub inner:VecMap<&'a str,OptMapVal<'a,Pos, Str, T>>,
    phantom:PhantomData<&'a Str>
}

pub struct Group<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> {
    previous_letters: Option<String>,
    #[allow(clippy::type_complexity)]
    macro_rule_changes: HMap<Pa::Str, Option<AnyMacro<'a, Pa, T, Err, State>>>,
    #[allow(clippy::type_complexity)]
    environment_rule_changes: HMap<Pa::Str, Option<AnyEnv<'a, Pa, T, Err, State>>>
}

pub trait GroupState<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> {
    fn new(parent:Option<&mut Self>) -> Self;
    fn inner(&self) -> &Group<'a, Pa, T, Err, State>;
    fn inner_mut(&mut self) -> &mut Group<'a,Pa,T,Err,State>;
    fn close(self, parser: &mut LaTeXParser<'a, Pa, T, Err,State>);
    fn add_macro_rule(&mut self, name: Pa::Str, old: Option<AnyMacro<'a, Pa, T, Err, State>>);
    fn add_environment_rule(&mut self, name: Pa::Str, old: Option<AnyEnv<'a, Pa, T, Err, State>>);
    fn letter_change(&mut self, old: &str);
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> GroupState<'a,Pa,T,Err,State> for Group<'a, Pa, T, Err,State> {
    fn new(_:Option<&mut Self>) -> Self {
        Group {
            previous_letters: None,
            macro_rule_changes: HMap::default(),
            environment_rule_changes: HMap::default()
        }
    }
    fn inner(&self) -> &Self {
        self
    }
    fn inner_mut(&mut self) -> &mut Self {
        self
    }

    fn add_macro_rule(&mut self, name: Pa::Str, old: Option<AnyMacro<'a, Pa, T, Err, State>>) {
        if let Entry::Vacant(e) = self.macro_rule_changes.entry(name) {
            e.insert(old);
        }
    }
    fn add_environment_rule(&mut self, name: Pa::Str, old: Option<AnyEnv<'a, Pa, T, Err, State>>) {
        if let Entry::Vacant(e) = self.environment_rule_changes.entry(name) {
            e.insert(old);
        }
    }

    fn letter_change(&mut self, old: &str) {
        if self.previous_letters.is_none() {
            self.previous_letters = Some(old.to_string());
        }
    }

    fn close(self, parser: &mut LaTeXParser<'a, Pa, T, Err, State>) {
        if let Some(l) = self.previous_letters {
            parser.tokenizer.letters = l;
        }
        for (n, r) in self.macro_rule_changes {
            if let Some(r) = r {
                parser.macro_rules.insert(n, r);
            } else {
                parser.macro_rules.remove(&n);
            }
        }
    }
}

pub trait ParserState<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>)
>:Sized {
    type Group:GroupState<'a,Pa,T,Err,Self>;
    type MacroArg:Clone;
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>)
> ParserState<'a,Pa,T,Err> for () {
    type Group=Group<'a,Pa,T,Err,Self>;
    type MacroArg = ();
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> Group<'a, Pa, T, Err, State> {

}

pub struct LaTeXParser<
    'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> {
    pub tokenizer: super::tokenizer::TeXTokenizer<'a, Pa,Err>,
    macro_rules: HMap<Pa::Str, AnyMacro<'a, Pa, T, Err, State>>,
    pub groups: Vec<State::Group>,
    environment_rules: HMap<Pa::Str, AnyEnv<'a, Pa, T, Err, State>>,
    directives: HMap<&'a str, fn(&mut Self,Pa::Str)>,
    buf: Vec<T>,
    pub state:State
}

macro_rules! count {
    () => (0usize);
    ( $e:expr; $($n:expr;)* ) => (1usize + count!($($n;)*));
}

macro_rules! default_rules {
    ($( $($name:ident)? $(($l:literal,$lname:ident))? ),*) => {
        #[must_use]
        pub fn default_rules() -> [(Pa::Str,MacroRule<'a,Pa, T, Err, State>);count!($( $($name;)? $($lname;)? )*)] {[
            $($((stringify!($name).into(),rules::$name))?$(($l.into(),rules::$lname))?),*
        ]}
    }
}

macro_rules! default_envs {
    ($( $($name:ident)? $(($l:literal,$lname:ident))? ),*) => {
        #[must_use]
        pub fn default_env_rules() -> [(Pa::Str,EnvironmentRule<'a,Pa, T, Err, State>);count!($( $($name;)? $($lname;)? )*)] {[
            $(paste::paste!(
                $((stringify!($name).into(),(rules::[<$name _open>],rules::[<$name _close>])))?
                $(($l.into(),(rules::$lname,rules::rules::[<$lname _close>])))?
            )),*
        ]}
    }
}

pub struct Groups<'a,'b,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> {
    pub groups:&'b mut Vec<State::Group>,
    pub rules:&'b mut HMap<Pa::Str,AnyMacro<'a,Pa, T, Err, State>>,
    pub environment_rules:&'b mut HMap<Pa::Str,AnyEnv<'a,Pa, T, Err, State>>,
    pub tokenizer: &'b mut super::tokenizer::TeXTokenizer<'a, Pa, Err>,
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> LaTeXParser<'a, Pa, T, Err,State> {
    pub fn new(input: Pa, state:State,err:Err) -> Self {
        Self::with_rules(
            input,
            state,
            err,
            Self::default_rules().into_iter(),
            Self::default_env_rules().into_iter()
        )
    }

    pub fn with_rules(
        input: Pa,
        state:State,
        err:Err,
        rules: impl Iterator<Item = (Pa::Str, MacroRule<'a, Pa, T, Err, State>)>,
        envs: impl Iterator<Item = (Pa::Str, EnvironmentRule<'a, Pa, T, Err, State>)>,
    ) -> Self {
        let mut macro_rules = HMap::default();
        let mut environment_rules = HMap::default();
        for (k, v) in rules {
            macro_rules.insert(k, AnyMacro::Ptr(v));
        }
        for (k, v) in envs {
            environment_rules.insert(k, AnyEnv::Ptr(v));
        }
        let mut directives = HMap::default();
        directives.insert("verbcmd",directives::verbcmd as _);
        directives.insert("verbenv",directives::verbenv as _);
        directives.insert("nolint",directives::nolint as _);
        directives.insert("dolint",directives::dolint as _);
        directives.insert("macro",directives::macro_dir as _);
        directives.insert("env",directives::env_dir as _);

        LaTeXParser {
            tokenizer: super::tokenizer::TeXTokenizer::new(input, err),
            macro_rules,
            groups: Vec::new(),
            environment_rules,
            directives,
            buf: Vec::new(),
            state
        }
    }

    #[inline]
    pub fn split<'b>(&'b mut self) -> (&'b mut State,Groups<'a,'b,Pa, T, Err, State>) {
        (&mut self.state,Groups {
            groups: &mut self.groups,
            rules: &mut self.macro_rules,
            environment_rules: &mut self.environment_rules,
            tokenizer: &mut self.tokenizer
        })
    }

    pub fn add_macro_rule(&mut self, name: Pa::Str, rule: Option<AnyMacro<'a, Pa, T, Err, State>>) {
        let old = if let Some(rule) = rule {
            self.macro_rules.insert(name.clone(), rule)
        } else {
            self.macro_rules.remove(&name)
        };
        if let Some(g) = self.groups.last_mut(){
            g.add_macro_rule(name,old);
        }
    }

    pub fn add_environment_rule(&mut self, name: Pa::Str, rule: Option<AnyEnv<'a, Pa, T, Err, State>>) {
        let old = if let Some(rule) = rule {
            self.environment_rules.insert(name.clone(), rule)
        } else {
            self.environment_rules.remove(&name)
        };
        if let Some(g) = self.groups.last_mut(){
            g.add_environment_rule(name,old);
        }
    }

    default_rules!(
        begin,
        end,
        begingroup,
        endgroup,
        makeatletter,
        makeatother,
        ExplSyntaxOn,
        ExplSyntaxOff,
        lstinline,
        verb,
        stexcodeinline,stexinline,
        newcommand,
        renewcommand,
        providecommand,
        newenvironment,
        renewenvironment,
        provideenvironment,
        NewDocumentCommand,
        DeclareDocumentCommand,
        DeclareRobustCommand,
        NewDocumentEnvironment,
        DeclareDocumentEnvironment,
        ("ref", r#ref),
        label,
        cite,
        includegraphics,
        url,
        lstdefinelanguage,
        hbox,
        vbox,
        fbox,
        mvbox,
        text,
        texttt,
        textrm,
        textbf,
        ensuremath,
        scalebox,
        def,edef,gdef,xdef
    );

    default_envs!(document, verbatim, lstlisting, stexcode);

    #[inline]
    pub fn curr_pos(&self) -> Pa::Pos {
        self.tokenizer.reader.curr_pos()
    }

    fn default(&mut self, t: TeXToken<Pa::Pos, Pa::Str>) -> Option<T> {
        match t {
            TeXToken::Comment(r) => T::from_comment(r),
            TeXToken::Text { range, text } => T::from_text(range, text),
            TeXToken::BeginGroupChar(start) => {
                let children = self.group();
                T::from_group(
                    SourceRange {
                        start,
                        end: self.tokenizer.reader.curr_pos(),
                    },
                    children,
                )
            }
            TeXToken::BeginMath { display, start } => {
                let children = self.math(display);
                T::from_math(
                    display,
                    SourceRange {
                        start,
                        end: self.tokenizer.reader.curr_pos(),
                    },
                    children,
                )
            }
            TeXToken::Directive(s) => {
                self.directive(s);
                None
            }
            TeXToken::EndGroupChar(p) => {
                self.tokenizer.problem(p,"Unmatched close group");
                None
            }
            TeXToken::EndMath { start,.. } => {
                self.tokenizer.problem(start,"Unmatched math close");
                None
            }
            TeXToken::ControlSequence { start, name } => self.cs(name, start),
        }
    }

    pub fn open_group(&mut self) {
        let g = State::Group::new(self.groups.last_mut());
        self.groups.push(g);
    }

    pub fn close_group(&mut self) {
        match self.groups.pop() {
            None => self.tokenizer.problem(self.curr_pos(),"Unmatched }"),
            Some(g) => g.close(self),
        }
    }
    pub fn add_letters(&mut self, s: &str) {
        if let Some(g) = self.groups.last_mut() {
            g.letter_change(&self.tokenizer.letters);
        }
        self.tokenizer.letters.push_str(s);
    }
    pub fn remove_letters(&mut self, s: &str) {
        if let Some(g) = self.groups.last_mut() {
            g.letter_change(&self.tokenizer.letters);
        }
        self.tokenizer.letters.retain(|x| !s.contains(x));
    }

    fn cs(&mut self, name: Pa::Str, start: Pa::Pos) -> Option<T> {
        match self.macro_rules.get(&name).cloned() {
            Some(r) => {
                let r#macro = Macro {
                    range: SourceRange {
                        start,
                        end: self.curr_pos(),
                    },
                    token_range: SourceRange {
                        start,
                        end: self.curr_pos(),
                    },
                    name,
                    //args: Vec::new(),
                    phantom: PhantomData,
                };
                match r.call(r#macro, self) {
                    MacroResult::Success(t) => Some(t),
                    MacroResult::Simple(m) => T::from_macro_application(m),
                    MacroResult::Other(v) => {
                        self.buf.extend(v.into_iter().rev());
                        self.buf.pop()
                    }
                }
            }
            None => T::from_control_sequence(start, name),
        }
    }

    pub(in crate::quickparse) fn environment(
        &mut self,
        begin: Macro<'a, Pa::Pos, Pa::Str>,
        name: Pa::Str,
        name_range:SourceRange<Pa::Pos>,
    ) -> EnvironmentResult<'a, Pa::Pos, Pa::Str, T> {
        let mut env = Environment {
            begin,
            end: None,
            name,name_range,
            //args: Vec::new(),
            children: Vec::new(),
            //phantom:PhantomData
        };
        self.open_group();
        let close = self.environment_rules.get(&env.name).cloned().map(|e|{
            e.open(&mut env, self);
            let close = e.close();
            close
        });
        while let Some(next) = self.tokenizer.next() {
            if let TeXToken::ControlSequence {
                start,
                name: endname,
            } = &next
            {
                if endname.as_ref() == "end" {
                    let mut end_macro = Macro {
                        range: SourceRange {
                            start:*start,
                            end: self.curr_pos(),
                        },
                        token_range: SourceRange {
                            start:*start,
                            end: self.curr_pos(),
                        },
                        name: env.name.clone(),
                        //args: Vec::new(),
                        phantom: PhantomData,
                    };
                    match self.read_name(&mut end_macro).map(|(n,_)| n) {
                        Some(n) if n == env.name => {
                            env.end = Some(end_macro);
                            return if let Some(close) = close {
                                let ret = close(env, self);
                                self.close_group();
                                ret
                            } else {
                                self.close_group();
                                EnvironmentResult::Simple(env)
                            };
                        }
                        Some(n) => {
                            self.tokenizer.problem(end_macro.range.start,format!(
                                "Expected \\end{{{}}}, found \\end{{{}}}",
                                env.name.as_ref(),
                                n.as_ref()
                            ));
                            break;
                        }
                        None => {
                            self.tokenizer
                                .problem(end_macro.range.start,"Expected environment name after \\end");
                            break;
                        }
                    }
                }
            }
            if let Some(n) = self.default(next) {
                env.children.push(n);
            }
        }
        self.close_group();
        self.tokenizer.problem(env.begin.range.start,"Unclosed environment");
        EnvironmentResult::Simple(env)
    }

    fn directive(&mut self, s: Pa::Str) {
        let mut str = s.as_ref().trim();
        if let Some(i) = str.find(|c:char| c.is_ascii_whitespace()) {
            str = &str[..i];
        }
        if let Some(d) = self.directives.get(str) {
            let len = str.len();
            let (_,mut args) = s.split_n(len);
            args.trim_ws();
            d(self,args);
        } else {
            self.tokenizer
                .problem(self.curr_pos(),format!("Unknown directive {s}"));
        }
    }

    fn math(&mut self, _display: bool) -> Vec<T> {
        let start = self.curr_pos();
        self.open_group();
        let mut v = Vec::new();
        while let Some(next) = self.tokenizer.next() {
            if matches!(next, TeXToken::EndMath { .. }) {
                self.close_group();
                return v;
            }
            if let Some(n) = self.default(next) {
                v.push(n);
            }
        }
        self.tokenizer.problem(start,"Unclosed math group");
        self.close_group();
        v
    }

    fn group(&mut self) -> Vec<T> {
        let start = self.curr_pos();
        self.open_group();
        let mut v = Vec::new();
        while let Some(next) = self.tokenizer.next() {
            if matches!(next, TeXToken::EndGroupChar(_)) {
                self.close_group();
                return v;
            }
            if let Some(n) = self.default(next) {
                v.push(n);
            }
        }
        self.tokenizer.problem(start,"Unclosed group");
        v
    }

    fn group_i(&mut self) -> Vec<T> {
        let start = self.curr_pos();
        let mut v = Vec::new();
        while !self.tokenizer.reader.starts_with('}') {
            let Some(next) = self.tokenizer.next() else {
                self.tokenizer.problem(start,"Unclosed group");
                return v;
            };
            if matches!(next,TeXToken::EndGroupChar(_)) {
                return v;
            }
            if let Some(n) = self.default(next) {
                v.push(n);
            }
        }
        if self.tokenizer.reader.starts_with('}') {
            self.tokenizer.reader.pop_head();
        } else {
            self.tokenizer.problem(start,"Unclosed group");
        }
        v
    }

    pub fn read_argument(&mut self, in_macro: &mut Macro<'a, Pa::Pos, Pa::Str>) {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            //let start = self.curr_pos();
            self.tokenizer.reader.pop_head();
            let _v = self.group_i();
            /*if let Some(g) = T::from_group(
                SourceRange {
                    start,
                    end: self.curr_pos(),
                },
                v,
            ) {
                in_macro.args.push(g);
            }*/
        } else if self.tokenizer.reader.starts_with('\\') {
            let _t = self.tokenizer.next().unwrap_or_else(|| unreachable!());
            /*if let Some(t) = self.default(t) {
                in_macro.args.push(t);
            }*/
        } else {
            let _ = self.tokenizer.next();
        }
        in_macro.range.end = self.curr_pos();
    }

    pub fn read_opt_str(
        &mut self,
        in_macro: &mut Macro<'a, Pa::Pos, Pa::Str>,
    ) -> OptArg<'a, Pa::Pos, Pa::Str> {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('[') {
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            let tstart = self.curr_pos();
            let s = self
                .tokenizer
                .reader
                .read_until_with_brackets::<'{', '}'>(|c| c == ']');
            let range = SourceRange {
                start: tstart,
                end: self.curr_pos(),
            };
            /*let text = Cfg::Token::from_text(
                range,
                s.clone(),
            );*/
            self.tokenizer.reader.pop_head();
            /*if let Some(t) = text {
                in_macro.args.push(t);
            }*/
            in_macro.range.end = self.curr_pos();
            OptArg {
                inner: Some(s),
                range,
                phantom: PhantomData,
            }
        } else {            
            let range = SourceRange {
                start: self.curr_pos(),
                end: self.curr_pos(),
            };
            OptArg {
                inner: None,
                range,
                phantom: PhantomData,
            }
        }
    }

    pub fn read_name(&mut self, r#in: &mut Macro<'a, Pa::Pos, Pa::Str>) -> Option<(Pa::Str,SourceRange<Pa::Pos>)> {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            //let gstart = self.curr_pos();
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            let tstart = self.curr_pos();
            let s = self
                .tokenizer
                .reader
                .read_until_with_brackets::<'{', '}'>(|c| c == '}');
            let range = SourceRange { start: tstart, end: self.curr_pos() };
            //let text = Cfg::Token::from_text(range,s.clone());
            self.tokenizer.reader.pop_head();
            //let v = text.map_or_else(|| Vec::new(), |t| vec![t]);
            /*if let Some(g) = T::from_group(
                SourceRange {
                    start: gstart,
                    end: self.curr_pos(),
                },
                v,
            ) {
                r#in.args.push(g);
            }*/
            r#in.range.end = self.curr_pos();
            Some((s,range))
        } else {
            None
        }
    }

    pub fn skip_opt(&mut self, in_macro: &mut Macro<'a, Pa::Pos, Pa::Str>) -> bool {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('[') {
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            //let tstart = self.curr_pos();
            let _s = self
                .tokenizer
                .reader
                .read_until_with_brackets::<'{', '}'>(|c| c == ']');
            /*let text = Cfg::Token::from_text(
                SourceRange {
                    start: tstart,
                    end: self.curr_pos(),
                },
                s.clone(),
            );*/
            self.tokenizer.reader.pop_head();
            /*if let Some(t) = text {
                in_macro.args.push(t);
            }*/
            in_macro.range.end = self.curr_pos();
            true
        } else {
            false
        }
    }
    pub fn skip_arg(&mut self, in_macro: &mut Macro<'a, Pa::Pos, Pa::Str>) {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            //let tstart = self.curr_pos();
            let _s = self
                .tokenizer
                .reader
                .read_until_with_brackets::<'{', '}'>(|c| c == '}');
            /*let text = Cfg::Token::from_text(
                SourceRange {
                    start: tstart,
                    end: self.curr_pos(),
                },
                s.clone(),
            );*/
            self.tokenizer.reader.pop_head();
            /*if let Some(t) = text {
                in_macro.args.push(t);
            }*/
        } else {
            let _ = self.tokenizer.next();
        }
        in_macro.range.end = self.curr_pos();
    }

    fn skip_comments(&mut self) {
        self.tokenizer.reader.trim_start();
        while self.tokenizer.reader.starts_with('%') {
            let _ = self.tokenizer.next();
            self.tokenizer.reader.trim_start();
        }
    }
}


impl<'a,
    Pos:SourcePos,
    T: FromLaTeXToken<'a, Pos, &'a str>,
    Err:FnMut(String,SourceRange<Pos>),
    State: ParserState<'a,ParseStr<'a,Pos>,T,Err>
> LaTeXParser<'a, ParseStr<'a,Pos>,T,Err,State> {
    pub fn read_opt_map(
        &mut self,
        in_macro: &mut Macro<'a, Pos, &'a str>,
    ) -> OptMap<'a, Pos, &'a str, T> {
        self.skip_comments();
        if self.tokenizer.reader.starts_with('[') {
            self.tokenizer.reader.pop_head();
            let mut map = VecMap::new();
            loop {
                self.skip_comments();
                let key_start = self.curr_pos();
                let key = self.tokenizer.reader.read_until(|c| c == ']' || c == ',' || c == '=' || c == '%').trim();
                let key_end = self.curr_pos();
                self.skip_comments();
                match self.tokenizer.reader.pop_head() {
                    Some(']') => {
                        if !key.is_empty() {
                            map.insert(key,OptMapVal {
                                key_range: SourceRange {start:key_start,end:key_end},
                                val_range: SourceRange {start:self.curr_pos(),end:self.curr_pos()},
                                val: Vec::new(),
                                str:"",
                                phantom: PhantomData
                            });
                        }
                        break
                    }
                    Some(',') if !key.is_empty() => {
                        map.insert(key,OptMapVal {
                            key_range: SourceRange {start:key_start,end:key_end},
                            val_range: SourceRange {start:self.curr_pos(),end:self.curr_pos()},
                            val: Vec::new(),
                            str:"",
                            phantom: PhantomData
                        });
                    }
                    Some(',') => (),
                    Some('=') => {
                        self.skip_comments();
                        let value_start = self.curr_pos();
                        let str = self.tokenizer.reader.read_until_with_brackets::<'{','}'>(|c| c == ']' || c == ',');
                        let mut new = ParseStr::new(str);
                        new.pos = value_start;
                        let mut old = std::mem::replace(&mut self.tokenizer.reader,new);
                        let mut val = Vec::new();
                        while self.tokenizer.reader.peek_head().is_some() {
                            let Some(next) = self.tokenizer.next() else {
                                self.tokenizer.problem(value_start,"Unclosed optional argument");
                                break;
                            };
                            if let Some(n) = self.default(next) {
                                val.push(n);
                            }
                            self.tokenizer.reader.trim_start();
                        }
                        old.pos = self.curr_pos();
                        self.tokenizer.reader = old;
                        map.insert(key,OptMapVal {
                            key_range: SourceRange {start:key_start,end:key_end},
                            val_range: SourceRange {start:value_start,end:self.curr_pos()},
                            val,str,
                            phantom: PhantomData
                        });
                    }
                    _ => {
                        self.tokenizer.problem(key_start, 
                            format!("value for key \"{key}\" in {} ended unexpectedly",in_macro.name)
                        );
                        break
                    }
                }
            }
            OptMap {
                inner: map,
                phantom: PhantomData,
            }
        } else {            
            /*let range = SourceRange {
                start: self.curr_pos(),
                end: self.curr_pos(),
            };*/
            OptMap {
                inner: VecMap::new(),
                phantom: PhantomData,
            }
        }
    }
}


impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
> Iterator
    for LaTeXParser<'a, Pa, T, Err, State>
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if let Some(t) = self.buf.pop() {
            return Some(t);
        }
        while let Some(t) = self.tokenizer.next() {
            if let Some(n) = self.default(t) {
                return Some(n);
            }
        }
        None
    }
}

/*
#[test]
fn test() {
    use std::path::PathBuf;
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    );
    let path = PathBuf::from("/home/jazzpirate/work/MathHub/courses/FAU/IWGS/problems/source/regex/prob/regex_scientific.de.tex");
    let str = std::fs::read_to_string(&path).unwrap();
    let reader = immt_utils::parsing::ParseStr::<immt_utils::sourcerefs::LSPLineCol>::new(&str);
    let parser = LaTeXParser::<'_,_,_,LaTeXToken<'_,_,_>,_>::new(reader, Some(&path),(),|e,p| tracing::error!("Error {e} ({p:?})"));
    for tk in parser {
        tracing::info!("{tk:?}");
    }
}
*/