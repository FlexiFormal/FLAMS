use immt_utils::{parsing::{ParseSource, StringOrStr}, sourcerefs::SourceRange};

use super::{rules::{DynEnv, DynMacro}, AnyEnv, AnyMacro, Environment, EnvironmentResult, FromLaTeXToken, LaTeXParser, Macro, MacroResult, ParserState};


pub fn verbcmd<'a,
  Pa: ParseSource<'a>,
  T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
  Err:FnMut(String,SourceRange<Pa::Pos>),
  State: ParserState<'a,Pa,T,Err>
>(parser: &mut LaTeXParser<'a,Pa,T,Err,State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    parser.add_macro_rule(args, Some(AnyMacro::Ptr(super::rules::lstinline as _)));
  }
}

pub fn verbenv<'a,
  Pa: ParseSource<'a>,
  T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
  Err:FnMut(String,SourceRange<Pa::Pos>),
  State: ParserState<'a,Pa,T,Err>
>(parser: &mut LaTeXParser<'a,Pa, T, Err, State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    parser.add_environment_rule(args, Some(AnyEnv::Ptr((super::rules::general_listing_open as _, super::rules::general_listing_close as _))));
  }
}

pub fn macro_dir<'a,
  Pa: ParseSource<'a>,
  T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
  Err:FnMut(String,SourceRange<Pa::Pos>),
  State: ParserState<'a,Pa,T,Err>
>(parser: &mut LaTeXParser<'a,Pa, T, Err, State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    if let Some((m,_)) = args.as_ref().split_once(' ') {
      let len = m.len();
      let (m,mut spec) = args.split_n(len);
      spec.trim_ws();
      parser.add_macro_rule(m, Some(AnyMacro::Str(DynMacro {
        ptr:do_macro_dir as _,
        arg:spec
      })));
    }
  }
}

fn do_macro_dir<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(arg:&Pa::Str,
    mut m:Macro<'a, Pa::Pos, Pa::Str>,
    parser: &mut LaTeXParser<'a,Pa, T, Err, State>
) -> MacroResult<'a, Pa::Pos, Pa::Str, T> {
    let arg = arg.as_ref();
    do_spec(arg,&mut m,parser);
    MacroResult::Simple(m)
}

#[inline]
fn do_spec<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(spec:&str,
    m:&mut Macro<'a, Pa::Pos, Pa::Str>,
    parser: &mut LaTeXParser<'a,Pa, T, Err, State>
) {
    for c in spec.as_bytes() { match *c {
        b'v' => parser.skip_arg(m),
        _ => parser.tokenizer.problem(m.range.start, format!("Unknown arg spec {c}")),
    }}
}

pub fn env_dir<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(parser: &mut LaTeXParser<'a,Pa, T, Err, State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    if let Some((m,_)) = args.as_ref().split_once(' ') {
      let len = m.len();
      let (m,mut spec) = args.split_n(len);
      spec.trim_ws();
      parser.add_environment_rule(m, Some(AnyEnv::Str(DynEnv {
        open:do_env_dir as _,
        close: do_env_dir_close as _,
        arg:spec
      })));
    }
  }
}


fn do_env_dir<'a,'b,'c,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(arg:&Pa::Str,
    e:&'b mut Environment<'a, Pa::Pos, Pa::Str, T>,
    parser: &'c mut LaTeXParser<'a,Pa, T, Err, State>) {
  let arg = arg.as_ref();
  do_spec(arg, &mut e.begin, parser);
}

fn do_env_dir_close<'a,'b,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(
    e:Environment<'a, Pa::Pos, Pa::Str, T>,
    _: &'b mut LaTeXParser<'a,Pa,T,Err,State>
) -> EnvironmentResult<'a, Pa::Pos, Pa::Str, T> {
  EnvironmentResult::Simple(e)
}

pub fn nolint<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(parser: &mut LaTeXParser<'a,Pa, T, Err, State>, _:Pa::Str) {
  parser.tokenizer.reader.read_until_str("%%STEXIDE dolint");
}

#[inline]
pub fn dolint<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>),
    State: ParserState<'a,Pa,T,Err>
>(_: &mut LaTeXParser<'a,Pa, T, Err, State>, _:Pa::Str) {}