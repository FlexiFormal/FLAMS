use std::borrow::Cow;

use flams_utils::{
    parsing::{ParseSource, StringOrStr},
    sourcerefs::SourcePos,
};

use crate::quickparse::stex::DiagnosticLevel;

use super::{
    rules::{DynEnv, DynMacro},
    AnyEnv, AnyMacro, Environment, EnvironmentResult, FromLaTeXToken, LaTeXParser, Macro,
    MacroResult, ParserState,
};

#[allow(clippy::needless_pass_by_value)]
pub fn copycmd<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    args: &'a str,
) {
    if let Some((a, b)) = args.split_once(' ') {
        let a = a.trim();
        let b = b.trim();
        let old = parser.macro_rules.get(b);
        parser.add_macro_rule(Cow::Owned(a.to_string()), old.cloned());
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn verbcmd<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    args: &'a str,
) {
    if !args.is_empty() {
        parser.add_macro_rule(
            args.trim().as_cow(),
            Some(AnyMacro::Ptr(super::rules::lstinline as _)),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn verbenv<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    args: &'a str,
) {
    if !args.is_empty() {
        parser.add_environment_rule(
            args.trim().as_cow(),
            Some(AnyEnv::Ptr((
                super::rules::general_listing_open as _,
                super::rules::general_listing_close as _,
            ))),
        );
    }
}

pub fn macro_dir<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    args: &'a str,
) {
    if !args.is_empty() {
        if let Some((m, _)) = args.split_once(' ') {
            let len = m.len();
            let (m, mut spec) = args.split_n(len);
            spec.trim_ws();
            parser.add_macro_rule(
                m.trim().as_cow(),
                Some(AnyMacro::Str(DynMacro {
                    ptr: do_macro_dir as _,
                    arg: spec,
                })),
            );
        }
    }
}

fn do_macro_dir<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    arg: &&'a str,
    mut m: Macro<'a, Pos>,
    parser: &mut LaTeXParser<'a, Pos, T, State>,
) -> MacroResult<'a, Pos, T> {
    do_spec(arg, &mut m, parser);
    MacroResult::Simple(m)
}

#[inline]
fn do_spec<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    spec: &str,
    m: &mut Macro<'a, Pos>,
    parser: &mut LaTeXParser<'a, Pos, T, State>,
) {
    for c in spec.as_bytes() {
        match *c {
            b'v' => parser.skip_arg(m),
            _ => parser.tokenizer.problem(
                m.range.start,
                format!("Unknown arg spec {c}"),
                DiagnosticLevel::Error,
            ),
        }
    }
}

pub fn env_dir<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    args: &'a str,
) {
    if !args.is_empty() {
        if let Some((m, _)) = args.split_once(' ') {
            let len = m.len();
            let (m, mut spec) = args.split_n(len);
            spec.trim_ws();
            parser.add_environment_rule(
                m.as_cow(),
                Some(AnyEnv::Str(DynEnv {
                    open: do_env_dir as _,
                    close: do_env_dir_close as _,
                    arg: spec,
                })),
            );
        }
    }
}

fn do_env_dir<
    'a,
    'b,
    'c,
    Pos: SourcePos,
    T: FromLaTeXToken<'a, Pos>,
    State: ParserState<'a, Pos, T>,
>(
    arg: &&'a str,
    e: &'b mut Environment<'a, Pos, T>,
    parser: &'c mut LaTeXParser<'a, Pos, T, State>,
) {
    let arg = arg.as_ref();
    do_spec(arg, &mut e.begin, parser);
}

const fn do_env_dir_close<
    'a,
    'b,
    Pos: SourcePos,
    T: FromLaTeXToken<'a, Pos>,
    State: ParserState<'a, Pos, T>,
>(
    e: Environment<'a, Pos, T>,
    _: &'b mut LaTeXParser<'a, Pos, T, State>,
) -> EnvironmentResult<'a, Pos, T> {
    EnvironmentResult::Simple(e)
}

pub fn nolint<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    parser: &mut LaTeXParser<'a, Pos, T, State>,
    _: &'a str,
) {
    parser.tokenizer.reader.read_until_str("%%STEXIDE dolint");
}

#[inline]
pub fn dolint<'a, Pos: SourcePos, T: FromLaTeXToken<'a, Pos>, State: ParserState<'a, Pos, T>>(
    _: &mut LaTeXParser<'a, Pos, T, State>,
    _: &'a str,
) {
}
