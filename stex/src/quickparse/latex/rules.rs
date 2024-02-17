use immt_system::utils::parse::ParseSource;
use immt_system::utils::sourcerefs::{SourcePos, SourceRange};
use crate::quickparse::latex::{EnvironmentResult, FromLaTeXToken, LaTeXParser, Macro, MacroResult};
use crate::quickparse::tokenizer::Mode;

#[macro_export]
macro_rules! csrule {
    ($name:ident,$m:ident,$p:ident => $c:block) => {
        #[allow(unused_mut,non_snake_case)]
        pub fn $name<'a,
            Pa:ParseSource<'a>,
            T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >(mut $m:Macro<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) -> MacroResult<'a,Pa::Str,Pa::Pos,T> $c
    };
}

#[macro_export]
macro_rules! simple {
    ($name:ident,$m:ident,$p:ident => $c:block) => {
        csrule!($name,$m,$p => {
            $c;
            MacroResult::Simple($m)
        });
    }
}

csrule!(begin, r#macro, parser => {
    match parser.read_name(&mut r#macro) {
        None => {
            parser.tokenizer.problem("Expected { after \\begin");
            MacroResult::Simple(r#macro)
        }
        Some(s) => {
            match parser.environment(r#macro,s) {
                EnvironmentResult::Success(e) => MacroResult::Success(e),
                EnvironmentResult::Other(v) => MacroResult::Other(v),
                EnvironmentResult::Simple(e) => match T::from_environment(e) {
                    Some(t) => MacroResult::Success(t),
                    None => MacroResult::Other(vec!())
                }
            }
        }
    }
});

csrule!(end, r#macro, parser => {
    match parser.read_name(&mut r#macro) {
        None => {
            parser.tokenizer.problem("Expected { after \\end");
        }
        Some(s) => {
            parser.tokenizer.problem(format!("environment {} not open",s.as_ref()));
        }
    }
    MacroResult::Simple(r#macro)
});

simple!(inline_verbatim,m,p => {
    p.skip_opt(&mut m);
    if let Some(h) = p.tokenizer.reader.pop_head() {
        let tstart = p.curr_pos().clone();
        let t = p.tokenizer.reader.read_until(|c| c == h);
        if let Some(text) = T::from_text(SourceRange{start:tstart,end:p.curr_pos().clone()},t) {
            m.args.push(text);
        }
        if let Some(h2) = p.tokenizer.reader.pop_head() {
            if h2 != h {
                p.tokenizer.problem("Expected end of verbatim");
            }
        } else {
            p.tokenizer.problem("Expected end of verbatim");
        }
    } else {
        p.tokenizer.problem("Expected character");
    }
});

pub enum ArgType {
    Normal,Optional
}

pub fn skip_args<'a,
    Pa:ParseSource<'a>,
    T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
>(args:&[ArgType],mut m:Macro<'a,Pa::Str,Pa::Pos,T>,p:&mut LaTeXParser<'a,Pa,T>) -> MacroResult<'a,Pa::Str,Pa::Pos,T> {
    for a in args {
        match a {
            ArgType::Normal => p.skip_arg(&mut m),
            ArgType::Optional => {p.skip_opt(&mut m);}
        }
    }
    MacroResult::Simple(m)
}

const N : ArgType = ArgType::Normal;
const O : ArgType = ArgType::Optional;

simple!(begingroup,m,p => { p.open_group() });
simple!(endgroup,m,p => { p.close_group() });
simple!(makeatletter,m,p => { p.add_letters("@") });
simple!(makeatother,m,p => { p.remove_letters("@") });
simple!(ExplSyntaxOn,m,p => { p.add_letters(":_") });
simple!(ExplSyntaxOff,m,p => { p.remove_letters(":_") });
csrule!(lstinline,m,p => {inline_verbatim(m,p)});
csrule!(verb,m,p => {inline_verbatim(m,p)});
csrule!(stexcodeinline,m,p => {inline_verbatim(m,p)});
csrule!(newcommand,m,p => {skip_args(&[N,O,O,N],m,p)});
csrule!(providecommand,m,p => {skip_args(&[N,O,O,N],m,p)});
csrule!(renewcommand,m,p => {skip_args(&[N,O,O,N],m,p)});
csrule!(newenvironment,m,p => {skip_args(&[N,O,O,N,N],m,p)});
csrule!(provideenvironment,m,p => {skip_args(&[N,O,O,N,N],m,p)});
csrule!(renewenvironment,m,p => {skip_args(&[N,O,O,N,N],m,p)});
csrule!(NewDocumentCommand,m,p => {skip_args(&[N,N,N],m,p)});
csrule!(DeclareDocumentCommand,m,p => {skip_args(&[N,N,N],m,p)});
csrule!(DeclareRobustCommand,m,p => {skip_args(&[N,N],m,p)});
csrule!(NewDocumentEnvironment,m,p => {skip_args(&[N,N,N,N],m,p)});
csrule!(DeclareDocumentEnvironment,m,p => {skip_args(&[N,N,N,N],m,p)});
csrule!(r#ref,m,p => {skip_args(&[N],m,p)});
csrule!(label,m,p => {skip_args(&[N],m,p)});
csrule!(cite,m,p => {skip_args(&[N],m,p)});
csrule!(includegraphics,m,p => {skip_args(&[O,N],m,p)});
csrule!(url,m,p => {skip_args(&[O,N],m,p)});
simple!(lstdefinelanguage,m,p => {
    p.skip_arg(&mut m);
    if p.skip_opt(&mut m) {
        p.skip_arg(&mut m);
    }
    p.skip_arg(&mut m);
});

#[macro_export]
macro_rules! switch_mode {
    ($i:ident,$m:expr) => {
        simple!($i,m,p => {
            let mode = p.tokenizer.mode;
            p.open_group();
            p.tokenizer.mode = $m;
            p.read_argument(&mut m);
            p.tokenizer.mode = mode;
            p.close_group();
        });
    }
}

switch_mode!(hbox,Mode::Text);
switch_mode!(vbox,Mode::Text);
switch_mode!(fbox,Mode::Text);
switch_mode!(text,Mode::Text);
switch_mode!(texttt,Mode::Text);
switch_mode!(textrm,Mode::Text);
simple!(ensuremath,m,p => {
    if matches!(p.tokenizer.mode,Mode::Math{..}) {
        p.read_argument(&mut m);
    } else {
        p.tokenizer.open_math(false);
        p.read_argument(&mut m);
        p.tokenizer.close_math();
    }
});
simple!(scalebox,m,p => {
    let mode = p.tokenizer.mode;
    p.open_group();
    p.skip_arg(&mut m);
    p.read_argument(&mut m);
    p.tokenizer.mode = mode;
    p.close_group();
});

use super::Environment;

#[macro_export]
macro_rules! envrule {
    ($name:ident,$e:ident,$p:ident => $open:block $close:block) => {paste::paste!(
        #[allow(unused_mut,non_snake_case)]
        pub fn [<$name _open>]<'a,
            Pa:ParseSource<'a>,
            T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >($e:&mut Environment<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) $open
        #[allow(unused_mut,non_snake_case)]
        pub fn [<$name _close>]<'a,
            Pa:ParseSource<'a>,
            T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >(mut $e:Environment<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) -> EnvironmentResult<'a,Pa::Str,Pa::Pos,T> $close
    );};
}

envrule!(document,e,p => {} {
    let start = p.curr_pos().clone();
    let rest = p.tokenizer.reader.read_until_str("this string should never occur FOOBARBAZ BLA BLA asdk<ösndkf.k<asfb.mdv <sdasdjn");
    EnvironmentResult::Simple(e)
});

macro_rules! simple_env {
    ($name:ident,$e:ident,$p:ident => $open:block) => {
        envrule!($name,$e,$p => {
            $open;
        } {
            EnvironmentResult::Simple($e)
        });
    };
}

simple_env!(verbatim,e,p => {
    let start = p.curr_pos().clone();
    let s = p.tokenizer.reader.read_until_str("\\end{verbatim}");
    if let Some(t) = T::from_text(SourceRange{start,end:p.curr_pos().clone()},s) {
        e.children.push(t)
    }
});
simple_env!(lstlisting,e,p => {
    let start = p.curr_pos().clone();
    let s = p.tokenizer.reader.read_until_str("\\end{lstlisting}");
    if let Some(t) = T::from_text(SourceRange{start,end:p.curr_pos().clone()},s) {
        e.children.push(t)
    }
});
simple_env!(stexcode,e,p => {
    let start = p.curr_pos().clone();
    let s = p.tokenizer.reader.read_until_str("\\end{stexcode}");
    if let Some(t) = T::from_text(SourceRange{start,end:p.curr_pos().clone()},s) {
        e.children.push(t)
    }
});