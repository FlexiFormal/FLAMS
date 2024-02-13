pub mod rules;

use std::convert::Into;
use std::marker::PhantomData;
use immt_api::utils::HMap;
use immt_api::utils::problems::ProblemHandler;
use crate::quickparse::tokenizer::{TeXTokenizer, TokenizerGroup};
use crate::quickparse::tokens::TeXToken;
use immt_system::utils::parse::{ParseSource, StringOrStr};
use immt_system::utils::sourcerefs::{SourcePos, SourceRange};

pub trait FromLaTeXToken<'a,S:StringOrStr<'a>,P:SourcePos>:Sized {
    fn from_comment(r:SourceRange<P>) -> Option<Self>;
    fn from_group(r:SourceRange<P>,children:Vec<Self>) -> Option<Self>;
    fn from_math(display:bool,r:SourceRange<P>,children:Vec<Self>) -> Option<Self>;
    fn from_control_sequence(start:P,name:S) -> Option<Self>;
    fn from_text(r:SourceRange<P>,text:S) -> Option<Self>;
    fn from_macro_application(m:Macro<'a,S,P,Self>) -> Option<Self>;
    fn from_environment(e:Environment<'a,S,P,Self>) -> Option<Self>;
}

pub enum LaTeXToken<'a,S:StringOrStr<'a>,P:SourcePos> {
    Comment(SourceRange<P>),
    Group {
        range:SourceRange<P>,
        children:Vec<Self>
    },
    Math {
        display:bool,
        range:SourceRange<P>,
        children:Vec<Self>
    },
    ControlSequence{start:P,name:S},
    Text{range:SourceRange<P>,text:S},
    MacroApplication(Macro<'a,S,P,Self>),
    Environment(Environment<'a,S,P,Self>)
}
impl<'a,S:StringOrStr<'a>,P:SourcePos> FromLaTeXToken<'a,S,P> for LaTeXToken<'a,S,P> {
    fn from_comment(r: SourceRange<P>) -> Option<Self> {
        Some(LaTeXToken::Comment(r))
    }
    fn from_group(r: SourceRange<P>, children: Vec<Self>) -> Option<Self> {
        Some(LaTeXToken::Group{range:r,children})
    }
    fn from_math(display: bool, r: SourceRange<P>, children: Vec<Self>) -> Option<Self> {
        Some(LaTeXToken::Math{display,range:r,children})
    }
    fn from_control_sequence(start: P, name: S) -> Option<Self> {
        Some(LaTeXToken::ControlSequence{start,name})
    }
    fn from_text(range: SourceRange<P>,text:S) -> Option<Self> {
        Some(LaTeXToken::Text{range,text})
    }
    fn from_macro_application(m: Macro<'a,S, P, Self>) -> Option<Self> {
        Some(LaTeXToken::MacroApplication(m))
    }
    fn from_environment(e: Environment<'a,S, P, Self>) -> Option<Self> {
        Some(LaTeXToken::Environment(e))
    }
}

pub struct Macro<'a,S:StringOrStr<'a>,P:SourcePos,T:FromLaTeXToken<'a,S,P>> {
    range:SourceRange<P>,
    name:S,
    args:Vec<T>,
    phantom:PhantomData<&'a T>
}
pub struct Environment<'a,S:StringOrStr<'a>,P:SourcePos,T:FromLaTeXToken<'a,S,P>> {
    begin:Macro<'a,S,P,T>,
    end:Option<Macro<'a,S,P,T>>,
    name:S,
    args:Vec<T>,
    children:Vec<T>,
    phantom:PhantomData<&'a T>
}

pub enum MacroResult<'a,S:StringOrStr<'a>,P:SourcePos,T:FromLaTeXToken<'a,S,P>> {
    Success(T),
    Simple(Macro<'a,S,P,T>),
    Other(Vec<T>)
}
pub enum EnvironmentResult<'a,S:StringOrStr<'a>,P:SourcePos,T:FromLaTeXToken<'a,S,P>> {
    Success(T),
    Simple(Environment<'a,S,P,T>),
    Other(Vec<T>)
}

pub type MacroRule<'a,Pa:ParseSource<'a>,Pr,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> = fn(Macro<'a,Pa::Str,Pa::Pos,T>,&mut LaTeXParser<'a,Pa,Pr,T>) -> MacroResult<'a,Pa::Str,Pa::Pos,T>;
pub type EnvironmentRule<'a,Pa:ParseSource<'a>,Pr,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> = (
    fn(&mut Environment<'a,Pa::Str,Pa::Pos,T>,&mut LaTeXParser<'a,Pa,Pr,T>),
    fn(Environment<'a,Pa::Str,Pa::Pos,T>,&mut LaTeXParser<'a,Pa,Pr,T>) -> EnvironmentResult<'a,Pa::Str,Pa::Pos,T>
);


pub struct RuleGroup<'a,Pa:ParseSource<'a>,Pr:ProblemHandler,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> {
    previous_letters:Option<String>,
    macro_rule_changes:HMap<Pa::Str,Option<MacroRule<'a,Pa,Pr,T>>>,
    environment_rule_changes:HMap<Pa::Str,Option<EnvironmentRule<'a,Pa,Pr,T>>>,
    environment:Option<&'a str>
}
impl<'a,Pa:ParseSource<'a>,Pr:ProblemHandler,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> TokenizerGroup<'a> for RuleGroup<'a,Pa,Pr,T> {
    fn new<Pa2:ParseSource<'a>,Pr2:ProblemHandler>(_: &mut TeXTokenizer<'a, Pa2, Pr2, Self>) -> Self {
        RuleGroup {
            previous_letters:None,
            macro_rule_changes:HMap::default(),
            environment_rule_changes:HMap::default(),
            environment:None
        }
    }
    fn close<Pa2:ParseSource<'a>,Pr2:ProblemHandler>(self, tokenizer: &mut TeXTokenizer<'a, Pa2, Pr2, Self>) {
        if let Some(l) = self.previous_letters {
            tokenizer.letters = l
        }
    }
    fn letter_change(&mut self, old: &str) {
        if self.previous_letters.is_none() {
            self.previous_letters = Some(old.to_string());
        }
    }
}

pub struct LaTeXParser<'a,Pa:ParseSource<'a>,Pr:ProblemHandler = (),T:FromLaTeXToken<'a,Pa::Str,Pa::Pos> = LaTeXToken<'a,<Pa as ParseSource<'a>>::Pos,<Pa as ParseSource<'a>>::Str>> {
    tokenizer:super::tokenizer::TeXTokenizer<'a,Pa,Pr,RuleGroup<'a,Pa,Pr,T>>,
    macro_rules:HMap<Pa::Str,MacroRule<'a,Pa,Pr,T>>,
    environment_rules:HMap<Pa::Str,EnvironmentRule<'a,Pa,Pr,T>>,
    directives:HMap<Pa::Str,fn(&mut Self)>,
    buf:Vec<T>
}
macro_rules! count {
    () => (0usize);
    ( $e:expr; $($n:expr;)* ) => (1usize + count!($($n;)*));
}

macro_rules! default_rules {
    ($( $($name:ident)? $(($l:literal,$lname:ident))? ),*) => {
        pub fn default_rules() -> [(Pa::Str,MacroRule<'a,Pa,Pr,T>);count!($( $($name;)? $($lname;)? )*)] {[
            $($((stringify!($name).into(),rules::$name))?$(($l.into(),rules::$lname))?),*
        ]}
    }
}

macro_rules! default_envs {
    ($( $($name:ident)? $(($l:literal,$lname:ident))? ),*) => {
        pub fn default_env_rules() -> [(Pa::Str,EnvironmentRule<'a,Pa,Pr,T>);count!($( $($name;)? $($lname;)? )*)] {[
            $(paste::paste!(
                $((stringify!($name).into(),(rules::[<$name _open>],rules::[<$name _close>])))?
                $(($l.into(),(rules::$lname,rules::rules::[<$lname _close>])))?
            )),*
        ]}
    }
}

impl<'a,Pa:ParseSource<'a>,Pr:ProblemHandler,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> LaTeXParser<'a,Pa,Pr,T> {
    pub fn new(input:Pa,source_file:Option<&'a std::path::Path>,handler:&'a Pr) -> Self {
        let mut macro_rules = HMap::default();
        let mut environment_rules = HMap::default();
        for (k,v) in Self::default_rules() {
            macro_rules.insert(k,v);
        }
        for (k,v) in Self::default_env_rules() {
            environment_rules.insert(k,v);
        }
        LaTeXParser {
            tokenizer:super::tokenizer::TeXTokenizer::new(input,source_file,handler),
            macro_rules,
            environment_rules,
            directives:HMap::default(),
            buf:Vec::new()
        }
    }
    pub fn with_rules(input:Pa,source_file:Option<&'a std::path::Path>,handler:&'a Pr,rules:impl Iterator<Item=(Pa::Str,MacroRule<'a,Pa,Pr,T>)>) -> Self {
        let mut macro_rules = HMap::default();
        for (k,v) in rules {
            macro_rules.insert(k,v);
        }
        LaTeXParser {
            tokenizer:super::tokenizer::TeXTokenizer::new(input,source_file,handler),
            macro_rules,
            environment_rules:HMap::default(),
            directives:HMap::default(),
            buf:Vec::new()
        }
    }

    default_rules!(
        begin,end,begingroup,endgroup,makeatletter,makeatother,ExplSyntaxOn,ExplSyntaxOff,lstinline,
        verb,stexcodeinline,newcommand,renewcommand,providecommand,newenvironment,renewenvironment,
        provideenvironment,NewDocumentCommand,DeclareDocumentCommand,DeclareRobustCommand,
        NewDocumentEnvironment,DeclareDocumentEnvironment,("ref",r#ref),label,cite,
        includegraphics,url,lstdefinelanguage,hbox,vbox,fbox,text,texttt,textrm,ensuremath,scalebox
    );

    default_envs!(
        document,verbatim,lstlisting,stexcode
    );

    #[inline]
    pub fn curr_pos(&self) -> &Pa::Pos { self.tokenizer.reader.curr_pos() }

    fn default(&mut self,t:TeXToken<Pa::Pos,Pa::Str>) -> Option<T> {
         match t {
            TeXToken::Comment(r) => T::from_comment(r),
            TeXToken::Text{range,text} => T::from_text(range,text),
            TeXToken::BeginGroupChar(start) => {
                let children = self.group();
                T::from_group(SourceRange{start,end:self.tokenizer.reader.curr_pos().clone()}, children)
            }
            TeXToken::BeginMath {display,start} => {
                let children = self.math(display);
                T::from_math(display, SourceRange{start,end:self.tokenizer.reader.curr_pos().clone()}, children)
            }
            TeXToken::Directive(s) => { self.directive(s);None },
            TeXToken::EndGroupChar(_) => {
                self.tokenizer.problem("Unmatched close group");
                None
            } TeXToken::EndMath {..} =>  {
                self.tokenizer.problem("Unmatched math close");
                None
            }
             TeXToken::ControlSequence{start,name} => self.cs(name,start)
        }
    }

    fn cs(&mut self,name:Pa::Str,start:Pa::Pos) -> Option<T> {
        match self.macro_rules.get(&name) {
            Some(r) => {
                let r#macro = Macro {
                    range:SourceRange{start,end:self.curr_pos().clone()},
                    name,
                    args:Vec::new(),
                    phantom:PhantomData
                };
                match r(r#macro,self) {
                    MacroResult::Success(t) => Some(t),
                    MacroResult::Simple(m) => T::from_macro_application(m),
                    MacroResult::Other(v) => {
                        self.buf.extend(v.into_iter().rev());
                        self.buf.pop()
                    }
                }
            }
            None => {
                T::from_control_sequence(start,name)
            }
        }
    }

    pub(in crate::quickparse) fn environment(&mut self,begin:Macro<'a,Pa::Str,Pa::Pos,T>,name:Pa::Str) -> EnvironmentResult<'a,Pa::Str,Pa::Pos,T> {
        let mut env = Environment {
            begin,end:None,
            name:name.clone(),
            args:Vec::new(),
            children:Vec::new(),
            phantom:PhantomData
        };
        let close = match self.environment_rules.get(&name) {
            Some((open,close)) => {
                let close = *close;
                open(&mut env,self);
                Some(close)
            }
            _ => None
        };
        self.tokenizer.open_group();
        while let Some(next) = self.tokenizer.next() {
            if let TeXToken::ControlSequence{start,name:endname} = &next {
                if endname.as_ref() == "end" {
                    let mut end = Macro {
                        range: SourceRange { start:start.clone(), end: self.curr_pos().clone() },
                        name:name.clone(),
                        args: Vec::new(),
                        phantom: PhantomData
                    };
                    match self.read_name(&mut end) {
                        Some(n) if n == name => {
                            env.end = Some(end);
                            match close {
                                Some(close) => {
                                    let ret = close(env, self);
                                    self.tokenizer.close_group();
                                    return ret
                                },
                                None => {
                                    self.tokenizer.close_group();
                                    return EnvironmentResult::Simple(env)
                                }
                            }
                        }
                        Some(n) => {
                            self.tokenizer.problem(format!("Expected \\end{{{}}}, found \\end{{{}}}", name.as_ref(), n.as_ref()));
                            break
                        }
                        None => {
                            self.tokenizer.problem("Expected environment name after \\end");
                            break
                        }
                    }
                }
            }
            if let Some(n) = self.default(next) {
                env.children.push(n)
            }
        }
        self.tokenizer.close_group();
        self.tokenizer.problem("Unclosed environment");
        EnvironmentResult::Simple(env)
    }

    fn directive(&mut self,s:Pa::Str) {
        if let Some(d) = self.directives.get(&s) {
            d(self)
        } else {
            self.tokenizer.problem(&format!("Unknown directive {}",s.as_ref()))
        }
    }

    fn math(&mut self,_display:bool) -> Vec<T> {
        let mut v = Vec::new();
        while let Some(next) = self.tokenizer.next() {
            if matches!(next,TeXToken::EndMath{..}) {
                return v
            }
            if let Some(n) = self.default(next) {
                v.push(n)
            }
        }
        self.tokenizer.problem("Unclosed group");
        v
    }

    fn group(&mut self) -> Vec<T> {
        let mut v = Vec::new();
        while let Some(next) = self.tokenizer.next() {
            if matches!(next,TeXToken::EndGroupChar(_)) {
                return v
            }
            if let Some(n) = self.default(next) {
                v.push(n)
            }
        }
        self.tokenizer.problem("Unclosed group");
        v
    }

    fn group_i(&mut self) -> Vec<T> {
        let mut v = Vec::new();
        while !self.tokenizer.reader.starts_with('}') {
            let next = match self.tokenizer.next() {
                Some(t) => t,
                _ => {
                    self.tokenizer.problem("Unclosed group");
                    return v
                }
            };
            if let Some(n) = self.default(next) {
                v.push(n)
            }
        }
        if self.tokenizer.reader.starts_with('}') {
            self.tokenizer.reader.pop_head();
        } else {
            self.tokenizer.problem("Unclosed group");
        }
        v
    }

    pub fn read_argument(&mut self,in_macro:&mut Macro<'a,Pa::Str,Pa::Pos,T>) {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            let start = self.curr_pos().clone();
            self.tokenizer.reader.pop_head();
            let v = self.group_i();
            if let Some(g) = T::from_group(SourceRange{start,end:self.curr_pos().clone()},v) {
                in_macro.args.push(g);
            }
        } else if self.tokenizer.reader.starts_with('\\') {
            let t = self.tokenizer.next().unwrap();
            if let Some(t) = self.default(t) {
                in_macro.args.push(t)
            }
        }
        else {
            let _ = self.tokenizer.next();
        }
    }

    pub fn read_name(&mut self,r#in:&mut Macro<'a,Pa::Str,Pa::Pos,T>) -> Option<Pa::Str> {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            let gstart = self.curr_pos().clone();
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            let tstart = self.curr_pos().clone();
            let s = self.tokenizer.reader.read_until_with_brackets::<'{','}'>(|c| c == '}');
            let text = T::from_text(SourceRange{start:tstart,end:self.curr_pos().clone()},s.clone());
            self.tokenizer.reader.pop_head();
            let v = match text {
                Some(t) => vec![t],
                None => Vec::new()
            };
            if let Some(g) = T::from_group(SourceRange{start:gstart,end:self.curr_pos().clone()},v) {
                r#in.args.push(g);
            }
            r#in.range.end = self.curr_pos().clone();
            Some(s)
        } else {
            None
        }
    }

    pub fn skip_opt(&mut self,in_macro:&mut Macro<'a,Pa::Str,Pa::Pos,T>) -> bool {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('[') {
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            let tstart = self.curr_pos().clone();
            let s = self.tokenizer.reader.read_until_with_brackets::<'{','}'>(|c| c == ']');
            let text = T::from_text(SourceRange{start:tstart,end:self.curr_pos().clone()},s.clone());
            self.tokenizer.reader.pop_head();
            if let Some(t) = text {
                in_macro.args.push(t)
            }
            in_macro.range.end = self.curr_pos().clone();
            true
        } else {false}
    }
    pub fn skip_arg(&mut self,in_macro:&mut Macro<'a,Pa::Str,Pa::Pos,T>) {
        self.tokenizer.reader.trim_start();
        if self.tokenizer.reader.starts_with('{') {
            self.tokenizer.reader.pop_head();
            self.tokenizer.reader.trim_start();
            let tstart = self.curr_pos().clone();
            let s = self.tokenizer.reader.read_until_with_brackets::<'{','}'>(|c| c == '}');
            let text = T::from_text(SourceRange{start:tstart,end:self.curr_pos().clone()},s.clone());
            self.tokenizer.reader.pop_head();
            if let Some(t) = text {
                in_macro.args.push(t);
            }
        } else {
            let _ = self.tokenizer.next();
        }
        in_macro.range.end = self.curr_pos().clone();
    }
}

impl<'a,Pa:ParseSource<'a>,Pr:ProblemHandler,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>> Iterator for LaTeXParser<'a,Pa,Pr,T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if let Some(t) = self.buf.pop() { return Some(t) }
        while let Some(t) = self.tokenizer.next() {
            if let Some(n) = self.default(t) {
                return Some(n)
            }
        }
        None
    }
}