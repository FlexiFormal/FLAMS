use crate::quickparse::latex::{
    EnvironmentResult, FromLaTeXToken, LaTeXParser, Macro, MacroResult,
};
use crate::quickparse::tokenizer::Mode;
use immt_utils::{sourcerefs::{SourcePos, SourceRange},parsing::ParseSource};

pub fn read_verbatim_char<'a,Pa:ParseSource<'a>,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>>
    (mac:&mut Macro<'a,Pa::Str,Pa::Pos,T>,p:&mut LaTeXParser<'a,Pa,T>,end:char) {

    let tstart = p.curr_pos().clone();
    let t = p.tokenizer.reader.read_until(|c| c == end);
    if let Some(text) = T::from_text(SourceRange{start:tstart,end:p.curr_pos().clone()},t) {
        mac.args.push(text);
    }
    if let Some(h2) = p.tokenizer.reader.pop_head() {
        if h2 != end {
            p.tokenizer.problem("Expected end of verbatim");
        }
    } else {
        p.tokenizer.problem("Expected end of verbatim");
    }
}

pub fn read_verbatim_str<'a,Pa:ParseSource<'a>,T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>>
(env:&mut Environment<'a,Pa::Str,Pa::Pos,T>,p:&mut LaTeXParser<'a,Pa,T>,end:&str) {
    let tstart = p.curr_pos().clone();
    let t = p.tokenizer.reader.read_until_str(end);
    if let Some(text) = T::from_text(SourceRange{start:tstart,end:p.curr_pos().clone()},t) {
        env.args.push(text);
    }
}

#[macro_export]
macro_rules! texrules {
    ($name:ident <= $(($($rl:tt)*))*) => {
        $(
            $crate::tex!($($rl)*)
        )*
        paste!{
            pub fn [<$name _macros>]<'a, Pa: ParseSource<'a>, T: FromLaTeXToken<'a, Pa::Str, Pa::Pos>>() ->
            [(Pa::Str,MacroRule<'a,Pa,T>);texrules!( $( ($($rl)*) )* )] {[
                todo!()
            ]}
        }
    };
    (@count ) => (0usize);
    (@count ($($i:tt)*) $($r:tt)* ) => {
        (1usize + texrules!(@count $($r)*))
    }
}

#[macro_export]
macro_rules! tex {
    ($p:ident => @begin{$name:ident}$( ($($args:tt)* ) )? {$($start:tt)*} $($end:tt)*) => {paste::paste!(
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _open>]<'a,
            Pa:ParseSource<'a>,
            T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >($name:&mut Environment<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) {
            $( tex!{@envargs $p:$name $($args)* } )?
            $($start)*
        }
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _close>]<'a,
            Pa:ParseSource<'a>,
            T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >(mut $name:Environment<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) -> EnvironmentResult<'a,Pa::Str,Pa::Pos,T> {
            tex!{@end $name $($end)*}
        }
    );};
    (<l=$l:lifetime,Str=$str:ty,Pa=$pa:ty,Pos=$pos:ty,T=$t:ty> $p:ident => @begin{$name:ident}$( ($($args:tt)* ) )? {$($start:tt)*} $($end:tt)*) => {paste::paste!(
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _open>]<$l>($name:&mut Environment<$l,$str,$pos,$t>,$p:&mut LaTeXParser<$l,$pa,$t>) {
            $( tex!{@envargs $p:$name $($args)* } )?
            $($start)*
        }
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _close>]<$l>(mut $name:Environment<$l,$str,$pos,$t>,$p:&mut LaTeXParser<$l,$pa,$t>) -> EnvironmentResult<$l,$str,$pos,$t> {
            tex!{@end $name $($end)*}
        }
    );};
    (@end $name:ident $b:block !) => {
        $b
        EnvironmentResult::Simple($name)
    };
    (@end $name:ident !) => {
        EnvironmentResult::Simple($name)
    };
    (@end $name:ident $b:block) => {$b};

    (<l=$l:lifetime,Str=$str:ty,Pa=$pa:ty,Pos=$pos:ty,T=$t:ty> $p:ident => $name:ident $($args:tt)*) => {
        #[allow(unused_mut,non_snake_case)]
        pub fn $name<$l>
        (mut $name:Macro<$l,$str,$pos,$t>,$p:&mut LaTeXParser<$l,$pa,$t>) -> MacroResult<$l,$str,$pos,$t> {
            tex!{@args $p:$name$($args)*}
        }
    };
    ($p:ident => $name:ident$($args:tt)*) => {
        #[allow(unused_mut,non_snake_case)]
        pub fn $name<'a, Pa:ParseSource<'a>, T:FromLaTeXToken<'a,Pa::Str,Pa::Pos>
        >(mut $name:Macro<'a,Pa::Str,Pa::Pos,T>,$p:&mut LaTeXParser<'a,Pa,T>) -> MacroResult<'a,Pa::Str,Pa::Pos,T> {
            tex!{@args $p:$name$($args)*}
        }
    };


    (@envargs $p:ident:$name:ident{$arg:ident:name}$($args:tt)*) => {
        let $arg = if let Some(n) = $p.read_name(&mut $name.begin) { n } else {
            $p.tokenizer.problem(concat!("Expected { after \\",stringify!($name)));
            return;
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        let $arg = $p.read_argument(&mut $name.begin);
        $p.tokenizer.mode = mode;
        $p.close_group();
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{_:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        $p.read_argument(&mut $name.begin);
        $p.tokenizer.mode = mode;
        $p.close_group();
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:M}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        let $arg = if matches!($p.tokenizer.mode,$crate::quickparse::tokenizer::Mode::Math{..}) {
            $p.read_argument(&mut $name.begin);
        } else {
            $p.tokenizer.open_math(false);
            let r = $p.read_argument(&mut $name.begin);
            $p.tokenizer.close_math();
            r
        }
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{_:M}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        if matches!($p.tokenizer.mode,$crate::quickparse::tokenizer::Mode::Math{..}) {
            $p.read_argument(&mut $name.begin);
        } else {
            $p.tokenizer.open_math(false);
            $p.read_argument(&mut $name.begin);
            $p.tokenizer.close_math();
        }
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{_}$($args:tt)*) => {
        $p.skip_arg(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[_?$opt:ident]$($args:tt)*) => {
        let $opt = $p.skip_opt(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[_]$($args:tt)*) => {
        $p.skip_opt(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[$opt:ident:str]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name.begin).into_name();
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[$opt:ident]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident V:C($c:expr) $($args:tt)*) => {
        $crate::quickparse::latex::rules::read_verbatim_char(&mut $name.begin,$p,$c);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident V) => {
        $crate::quickparse::latex::rules::read_verbatim_str($name,$p,concat!("\\end{",stringify!($name),"}"));
    };
    (@envargs $p:ident:$name:ident($c:literal?$t:ident)$($args:tt)*) => {
        let $t = $p.tokenizer.reader.starts_with($c) && {
            $p.tokenizer.reader.pop_head();true
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident($t:ident)$($args:tt)*) => {
        if let Some($t) = $p.tokenizer.reader.pop_head() {
            tex!{@envargs $p:$name $($args)*}
        } else {
            $p.tokenizer.problem("Expected character");
        }
    };
    (@envargs $p:ident:$name:ident => $b:block) => {$b};
    (@envargs $p:ident:$name:ident) => {};




    (@args $p:ident:$name:ident{$arg:ident:name}$($args:tt)*) => {
        let $arg = if let Some(n) = $p.read_name(&mut $name) { n } else {
            $p.tokenizer.problem(concat!("Expected { after \\",stringify!($name)));
            return MacroResult::Simple($name);
        };
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{$arg:ident:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        let $arg = $p.read_argument(&mut $name);
        $p.tokenizer.mode = mode;
        $p.close_group();
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{_:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        $p.read_argument(&mut $name);
        $p.tokenizer.mode = mode;
        $p.close_group();
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{$arg:ident:M}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        let $arg = if matches!($p.tokenizer.mode,$crate::quickparse::tokenizer::Mode::Math{..}) {
            $p.read_argument(&mut $name);
        } else {
            $p.tokenizer.open_math(false);
            let r = $p.read_argument(&mut $name);
            $p.tokenizer.close_math();
            r
        }
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{_:M}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        if matches!($p.tokenizer.mode,$crate::quickparse::tokenizer::Mode::Math{..}) {
            $p.read_argument(&mut $name);
        } else {
            $p.tokenizer.open_math(false);
            $p.read_argument(&mut $name);
            $p.tokenizer.close_math();
        }
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{_}$($args:tt)*) => {
        $p.skip_arg(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[_?$opt:ident]$($args:tt)*) => {
        let $opt = $p.skip_opt(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[_]$($args:tt)*) => {
        $p.skip_opt(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[$opt:ident:str]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name).into_name();
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[$opt:ident]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident V:C($c:expr) $($args:tt)*) => {
        $crate::quickparse::latex::rules::read_verbatim_char(&mut $name,$p,$c);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident($c:literal?$t:ident)$($args:tt)*) => {
        let $t = $p.tokenizer.reader.starts_with($c) && {
            $p.tokenizer.reader.pop_head();true
        };
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident($t:ident)$($args:tt)*) => {
        if let Some($t) = $p.tokenizer.reader.pop_head() {
            tex!{@args $p:$name $($args)*}
        } else {
            $p.tokenizer.problem("Expected character");
            MacroResult::Simple($name)
        }
    };
    (@args $p:ident:$name:ident !) => {
        MacroResult::Simple($name)
    };
    (@args $p:ident:$name:ident => $b:block !) => {
        $b;MacroResult::Simple($name)
    };
    (@args $p:ident:$name:ident => $b:block) => {$b};
}


tex!(p => begin{n:name} => {
    match p.environment(begin,n) {
        EnvironmentResult::Success(e) => MacroResult::Success(e),
        EnvironmentResult::Other(v) => MacroResult::Other(v),
        EnvironmentResult::Simple(e) => match T::from_environment(e) {
            Some(t) => MacroResult::Success(t),
            None => MacroResult::Other(vec!())
        }
    }
});

tex!(p => end{n:name} => {
    p.tokenizer.problem(format!("environment {} not open",n.as_ref()));
}!);

tex!(p => lstinline[_](c)V:C(c)!);
tex!(p => verb[_](c)V:C(c)!);
tex!(p => stexcodeinline[_](c)V:C(c)!);
tex!(p => begingroup => { p.open_group() }!);
tex!(p => endgroup => { p.close_group() }!);
tex!(p => makeatletter => { p.add_letters("@") }!);
tex!(p => makeatother => { p.remove_letters("@") }!);
tex!(p => ExplSyntaxOn => { p.add_letters(":_") }!);
tex!(p => ExplSyntaxOff => { p.remove_letters(":_") }!);
tex!(p => lstdefinelanguage{_}[_?o]{_} => {
    if o {p.skip_arg(&mut lstdefinelanguage);}
}!);
tex!(p => r#ref{_}!);
tex!(p => label{_}!);
tex!(p => cite{_}!);
tex!(p => includegraphics[_]{_}!);
tex!(p => url[_]{_}!);

tex!(p => newcommand{_}[_][_]{_}!);
tex!(p => providecommand{_}[_][_]{_}!);
tex!(p => renewcommand{_}[_][_]{_}!);
tex!(p => NewDocumentCommand{_}{_}{_}!);
tex!(p => DeclareDocumentCommand{_}{_}{_}!);
tex!(p => DeclareRobustCommand{_}{_}{_}!);
tex!(p => newenvironment{_}[_][_]{_}{_}!);
tex!(p => renewenvironment{_}[_][_]{_}{_}!);
tex!(p => provideenvironment{_}[_][_]{_}{_}!);
tex!(p => NewDocumentEnvironment{_}{_}{_}{_}!);
tex!(p => DeclareDocumentEnvironment{_}{_}{_}{_}!);

tex!(p => hbox{_:T}!);
tex!(p => vbox{_:T}!);
tex!(p => fbox{_:T}!);
tex!(p => text{_:T}!);
tex!(p => texttt{_:T}!);
tex!(p => textrm{_:T}!);
tex!(p => ensuremath{_:M}!);
tex!(p => scalebox{_}{_:T}!);

use super::Environment;

tex!(p => @begin{document} {}{
    let _start = p.curr_pos().clone();
    let _rest = p.tokenizer.reader.read_until_str("this string should never occur FOOBARBAZ BLA BLA asdk<ösndkf.k<asfb.mdv <sdasdjn");
}!);
tex!(p => @begin{verbatim}(V) {}{}!);
tex!(p => @begin{lstlisting}(V) {}{}!);
tex!(p => @begin{stexcode}(V) {}{}!);
