use immt_utils::{parsing::{ParseSource, StringOrStr}, sourcerefs::SourceRange};

use super::{AnyEnv, AnyMacro, DynEnv, DynMacro, Environment, EnvironmentResult, FromLaTeXToken, LaTeXParser, Macro, MacroResult};


pub fn verbcmd<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(parser: &mut LaTeXParser<'a,Pa,Err,T,State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    parser.add_macro_rule(args, Some(AnyMacro::Ptr(super::rules::lstinline as _)));
  }
}

pub fn verbenv<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(parser: &mut LaTeXParser<'a,Pa,Err,T,State>,args:Pa::Str) {
  if !args.as_ref().is_empty() {
    parser.add_environment_rule(args, Some(AnyEnv::Ptr((super::rules::general_listing_open as _, super::rules::general_listing_close as _))));
  }
}

pub fn macro_dir<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(parser: &mut LaTeXParser<'a,Pa,Err,T,State>,args:Pa::Str) {
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

fn do_macro_dir<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(arg:&Pa::Str,mut m:Macro<'a, Pa::Str, Pa::Pos, T>,parser: &mut LaTeXParser<'a,Pa,Err,T,State>) -> MacroResult<'a, Pa::Str, Pa::Pos, T> {
  let arg = arg.as_ref();
  do_spec(arg,&mut m,parser);
  MacroResult::Simple(m)
}

#[inline]
fn do_spec<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(spec:&str,m:&mut Macro<'a, Pa::Str, Pa::Pos, T>,parser: &mut LaTeXParser<'a,Pa,Err,T,State>) {
  for c in spec.as_bytes() { match *c {
    b'v' => parser.skip_arg(m),
    _ => parser.tokenizer.problem(m.range.start, format!("Unknown arg spec {c}")),
  }}

}

pub fn env_dir<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(parser: &mut LaTeXParser<'a,Pa,Err,T,State>,args:Pa::Str) {
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


fn do_env_dir<'a,'b,'c, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(arg:&Pa::Str,e:&'b mut Environment<'a, Pa::Str, Pa::Pos, T>,parser: &'c mut LaTeXParser<'a,Pa,Err,T,State>) {
  let arg = arg.as_ref();
  do_spec(arg, &mut e.begin, parser);
}

fn do_env_dir_close<'a,'b, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(e:Environment<'a, Pa::Str, Pa::Pos, T>,parser: &'b mut LaTeXParser<'a,Pa,Err,T,State>) -> EnvironmentResult<'a, Pa::Str, Pa::Pos, T> {
  EnvironmentResult::Simple(e)
}

pub fn nolint<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(parser: &mut LaTeXParser<'a,Pa,Err,T,State>,_:Pa::Str) {
  parser.tokenizer.reader.read_until_str("%%STEXIDE dolint");
}

#[inline]
pub fn dolint<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>,State,Err:FnMut(String,SourceRange<Pa::Pos>)>(_: &mut LaTeXParser<'a,Pa,Err,T,State>,_:Pa::Str) {}