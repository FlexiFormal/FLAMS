use crate::quickparse::{latex::{
    FromLaTeXToken, LaTeXParser, Macro
}, stex::DiagnosticLevel};
use flams_utils::{parsing::{ParseSource, StringOrStr}, sourcerefs::{SourcePos, SourceRange}};

use super::{Environment, ParserState};


#[derive(Debug)]
pub enum MacroResult<'a, 
    Pos:SourcePos, 
    Str:StringOrStr<'a>, 
    T:FromLaTeXToken<'a,Pos, Str>
> {
    Success(T),
    Simple(Macro<'a, Pos, Str>),
    Other(Vec<T>),
}

#[derive(Debug)]
pub enum EnvironmentResult<'a, 
    Pos:SourcePos, 
    Str:StringOrStr<'a>, 
    T:FromLaTeXToken<'a,Pos, Str>
> {
    Success(T),
    Simple(Environment<'a, Pos, Str, T>),
    Other(Vec<T>),
}

pub type MacroRule<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> =
    fn(
        Macro<'a, Pa::Pos, Pa::Str>,
        &mut LaTeXParser<'a, Pa,T,Err,State>
    ) -> MacroResult<'a, Pa::Pos,Pa::Str,T>;

pub type EnvOpenRule<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> = for<'b, 'c> fn(
    &'b mut Environment<'a, Pa::Pos,Pa::Str,T>, 
    &'c mut LaTeXParser<'a, Pa,T,Err,State>
);

pub type EnvCloseRule<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> =
    for<'b> fn(
        Environment<'a, Pa::Pos,Pa::Str,T>,
        &'b mut LaTeXParser<'a, Pa,T,Err,State>
    ) -> EnvironmentResult<'a, Pa::Pos,Pa::Str,T>;

pub type EnvironmentRule<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> = (EnvOpenRule<'a, Pa,T,Err,State>, EnvCloseRule<'a, Pa,T,Err,State>);


#[allow(clippy::type_complexity)]
pub struct DynMacro<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>,
    Arg
> {
    pub ptr:fn(
        &Arg,
        Macro<'a, Pa::Pos, Pa::Str>,
        &mut LaTeXParser<'a, Pa, T, Err,State>
    ) -> MacroResult<'a, Pa::Pos, Pa::Str, T>,
    pub arg:Arg
}

#[allow(clippy::type_complexity)]
pub struct DynEnv<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>,
    Arg
> {
    pub open:for<'b, 'c> fn(
        &Arg,
        &'b mut Environment<'a, Pa::Pos, Pa::Str, T>, 
        &'c mut LaTeXParser<'a, Pa, T, Err,State>
    ),
    pub close:for<'b> fn(
        Environment<'a, Pa::Pos, Pa::Str, T>,
        &'b mut LaTeXParser<'a, Pa, T, Err, State>
    ) -> EnvironmentResult<'a, Pa::Pos, Pa::Str, T>,
    pub arg:Arg
}

pub enum AnyMacro<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> {
    Ptr(MacroRule<'a,Pa,T,Err,State>),
    Str(DynMacro<'a,Pa,T,Err,State,Pa::Str>),
    Ext(DynMacro<'a,Pa,T,Err,State,State::MacroArg>)
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> AnyMacro<'a,Pa,T,Err,State> {
    pub fn call(&self,
        m:Macro<'a, Pa::Pos, Pa::Str>,
        p:&mut LaTeXParser<'a, Pa, T, Err, State>
    ) -> MacroResult<'a, Pa::Pos, Pa::Str, T> {
        match self {
            Self::Ptr(ptr) => ptr(m,p),
            Self::Str(str) => (str.ptr)(&str.arg,m,p),
            Self::Ext(ext) => (ext.ptr)(&ext.arg,m,p)
        }
    }
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> Clone for AnyMacro<'a,Pa,T,Err,State> {
    fn clone(&self) -> Self {
        match self {
            Self::Ptr(ptr) => Self::Ptr(*ptr),
            Self::Str(str) => Self::Str(
                DynMacro {
                    ptr:str.ptr,
                    arg:str.arg.clone()
                }
            ),
            Self::Ext(ext) => Self::Ext(
                DynMacro {
                    ptr:ext.ptr,
                    arg:ext.arg.clone()
                }
            )
        }
    }
}

pub enum AnyEnv<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> {
    Ptr(EnvironmentRule<'a,Pa,T,Err,State>),
    Str(DynEnv<'a,Pa,T,Err,State,Pa::Str>),
    Ext(DynEnv<'a,Pa,T,Err,State,State::MacroArg>)
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> AnyEnv<'a,Pa,T,Err,State> {
    pub fn open<'b, 'c>(&self,
        e:&'b mut Environment<'a, Pa::Pos,Pa::Str,T>, 
        p:&'c mut LaTeXParser<'a, Pa, T, Err, State>
    ) {
        match self {
            Self::Ptr((ptr,_)) => ptr(e,p),
            Self::Str(str) => (str.open)(&str.arg,e,p),
            Self::Ext(ext) => (ext.open)(&ext.arg,e,p)
        }
    }
    pub fn close(self) -> EnvCloseRule<'a, Pa,T,Err,State> {
        match self {
            Self::Ptr((_,close)) => close,
            Self::Str(str) => str.close,
            Self::Ext(ext) => ext.close
        }
    }
}

impl<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
> Clone for AnyEnv<'a,Pa,T,Err,State> {
    fn clone(&self) -> Self {
        match self {
            Self::Ptr(ptr) => Self::Ptr(*ptr),
            Self::Str(str) => Self::Str(
                DynEnv {
                    open:str.open,
                    close:str.close,
                    arg:str.arg.clone()
                }
            ),
            Self::Ext(ext) => Self::Ext(
                DynEnv {
                    open:ext.open,
                    close:ext.close,
                    arg:ext.arg.clone()
                }
            )
        }
    }
}


pub fn read_verbatim_char<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
>(
    mac: &mut Macro<'a, Pa::Pos, Pa::Str>,
    p: &mut LaTeXParser<'a, Pa, T, Err, State>,
    end: char,
) {
    //let tstart = p.curr_pos();
    let _t = p.tokenizer.reader.read_until(|c| c == end);
    /*if let Some(text) = T::from_text(
        SourceRange {
            start: tstart,
            end: p.curr_pos(),
        },
        t,
    ) {
        mac.args.push(text);
    }*/
    if let Some(h2) = p.tokenizer.reader.pop_head() {
        if h2 != end {
            p.tokenizer.problem(mac.range.start,"Expected end of verbatim",DiagnosticLevel::Error);
        }
    } else {
        p.tokenizer.problem(mac.range.start,"Expected end of verbatim",DiagnosticLevel::Error);
    }
}

pub fn read_verbatim_str<'a,
    Pa: ParseSource<'a>,
    T: FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
    Err:FnMut(String,SourceRange<Pa::Pos>,DiagnosticLevel),
    State: ParserState<'a,Pa,T,Err>
>(
    _env: &mut Environment<'a, Pa::Pos, Pa::Str, T>,
    p: &mut LaTeXParser<'a, Pa, T, Err, State>,
    end_str: &str,
) {
    //let tstart = p.curr_pos();
    let _t = p.tokenizer.reader.read_until_str(end_str);
    /*if let Some(text) = T::from_text(
        SourceRange {
            start: tstart,
            end: p.curr_pos(),
        },
        t,
    ) {
        env.args.push(text);
    }*/
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
    ($p:ident => $name:ident$($args:tt)*) => {
        #[allow(unused_mut,non_snake_case)]
        pub fn $name<'a,
            Pa: ::flams_utils::parsing::ParseSource<'a>,
            T: $crate::quickparse::latex::FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
            Err:FnMut(String,::flams_utils::sourcerefs::SourceRange<Pa::Pos>,DiagnosticLevel),
            State: $crate::quickparse::latex::ParserState<'a,Pa,T,Err>
        >(
            mut $name:$crate::quickparse::latex::Macro<'a,Pa::Pos,Pa::Str>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<'a, Pa, T, Err, State>
        ) -> $crate::quickparse::latex::rules::MacroResult<'a, Pa::Pos, Pa::Str,T> {
            tex!{@args $p:$name$($args)*}
        }
    };

    ($p:ident => @begin{$name:ident}$( ($($args:tt)* ) )? {$($start:tt)*} $($end:tt)*) => {paste::paste!(
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _open>]<'a,
            Pa: ::flams_utils::parsing::ParseSource<'a>,
            T: $crate::quickparse::latex::FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
            Err:FnMut(String,::flams_utils::sourcerefs::SourceRange<Pa::Pos>,DiagnosticLevel),
            State: $crate::quickparse::latex::ParserState<'a,Pa,T,Err>
        >(
            $name:&mut $crate::quickparse::latex::Environment<'a, Pa::Pos, Pa::Str, T>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<'a, Pa, T, Err, State>
        ) {
            $( tex!{@envargs $p:$name $($args)* } )?
            $($start)*
        }
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _close>]<'a,
            Pa: ::flams_utils::parsing::ParseSource<'a>,
            T: $crate::quickparse::latex::FromLaTeXToken<'a, Pa::Pos, Pa::Str>,
            Err:FnMut(String,::flams_utils::sourcerefs::SourceRange<Pa::Pos>,DiagnosticLevel),
            State: $crate::quickparse::latex::ParserState<'a,Pa,T,Err>
        >(
            mut $name:$crate::quickparse::latex::Environment<'a,Pa::Pos, Pa::Str, T>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<'a, Pa, T, Err, State>
        ) -> $crate::quickparse::latex::rules::EnvironmentResult<'a,Pa::Pos,Pa::Str,T> {
            tex!{@end $name $($end)*}
        }
    );};

    (<{$($tks:tt)+} M{$($mtks:tt)+} P{$($ptks:tt)+} R{$($rtks:tt)+}> $p:ident => $name:ident $($args:tt)*) => {
        #[allow(unused_mut,non_snake_case)]
        pub fn $name<$($tks)*>(
            mut $name:$crate::quickparse::latex::Macro<$($mtks)*>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<$($ptks)*>
        ) -> $crate::quickparse::latex::rules::MacroResult<$($rtks)*> {
            tex!{@args $p:$name$($args)*}
        }
    };

    (<{$($tks:tt)+} E{$($mtks:tt)+} P{$($ptks:tt)+} R{$($rtks:tt)+}> $p:ident => @begin{$name:ident}$( ($($args:tt)* ) )? {$($start:tt)*} $($end:tt)*) => {paste::paste!(
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _open>]<$($tks)*>(
            $name:&mut $crate::quickparse::latex::Environment<$($mtks)*>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<$($ptks)*>
        ) {
            $( tex!{@envargs $p:$name $($args)* } )?
            $($start)*
        }
        #[allow(unused,unused_mut,non_snake_case)]
        pub fn [<$name _close>]<$($tks)*>(
            mut $name:$crate::quickparse::latex::Environment<$($mtks)*>,
            $p:&mut $crate::quickparse::latex::LaTeXParser<$($ptks)*>
        ) -> $crate::quickparse::latex::rules::EnvironmentResult<$($rtks)*> {
            tex!{@end $name $($end)*}
        }
    );};

    (@end $name:ident $b:block !) => {
        $b
        $crate::quickparse::latex::rules::EnvironmentResult::Simple($name)
    };
    (@end $name:ident !) => {
        $crate::quickparse::latex::rules::EnvironmentResult::Simple($name)
    };
    (@end $name:ident $b:block) => {$b};

    (@envargs $p:ident:$name:ident{$arg:ident:name}$($args:tt)*) => {
        let Some($arg) = $p.read_name(&mut $name.begin) else {
            $p.tokenizer.problem($name.begin.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return;
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:!name}$($args:tt)*) => {
        let Some($arg) = $p.read_name_normalized(&mut $name.begin) else {
            $p.tokenizer.problem($name.begin.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return;
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:name+}$($args:tt)*) => {
        let $arg = $p.read_names(&mut $name.begin);
        if $arg.is_empty() {
            $p.tokenizer.problem($name.begin.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return;
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:!name+}$($args:tt)*) => {
        let $arg = $p.read_names_normalized(&mut $name.begin);
        if $arg.is_empty() {
            $p.tokenizer.problem($name.begin.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return;
        };
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident{$arg:ident:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        let $arg = $p.get_argument(&mut $name.begin);
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
            $p.get_argument(&mut $name.begin)
        } else {
            $p.tokenizer.open_math(false);
            let r = $p.get_argument(&mut $name.begin);
            $p.tokenizer.close_math();
            r
        };
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
    (@envargs $p:ident:$name:ident[$opt:ident:!name]$($args:tt)*) => {
        let $opt = $p.read_opt_name_normalized(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[$opt:ident]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[mut $opt:ident:Map]$($args:tt)*) => {
        let mut $opt = $p.read_opt_map(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[$opt:ident:Map]$($args:tt)*) => {
        let $opt = $p.read_opt_map(&mut $name.begin);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident[$opt:ident:type $tp:ty]$($args:tt)*) => {
        let $opt = <Vec<$tp> as crate::quickparse::latex::KeyValValues<_,_,_,_>>::parse_opt($p);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident V:C($c:expr) $($args:tt)*) => {
        $crate::quickparse::latex::rules::read_verbatim_char(&mut $name.begin,$p,$c);
        tex!{@envargs $p:$name $($args)*}
    };
    (@envargs $p:ident:$name:ident V) => {
        $crate::quickparse::latex::rules::read_verbatim_str($name,$p,concat!("\\end{",stringify!($name),"}"));
    };
    (@envargs $p:ident:$name:ident V!) => {
        $crate::quickparse::latex::rules::read_verbatim_str($name,$p,&format!("\\end{{{}}}",$name.name));
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
            $p.tokenizer.problem("Expected character",DiagnosticLevel::Error);
        }
    };
    (@envargs $p:ident:$name:ident => $b:block) => {$b};
    (@envargs $p:ident:$name:ident) => {};

    (@args $p:ident:$name:ident{$arg:ident:name}$($args:tt)*) => {
        let Some($arg) = $p.read_name(&mut $name) else {
            $p.tokenizer.problem($name.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return $crate::quickparse::latex::rules::MacroResult::Simple($name);
        };
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{$arg:ident:!name}$($args:tt)*) => {
        let Some($arg) = $p.read_name_normalized(&mut $name) else {
            $p.tokenizer.problem($name.range.start,concat!("Expected { after \\",stringify!($name)),DiagnosticLevel::Error);
            return $crate::quickparse::latex::rules::MacroResult::Simple($name);
        };
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{$arg:ident}$($args:tt)*) => {
        let $arg = $p.get_argument(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{$arg:ident:T}$($args:tt)*) => {
        let mode = $p.tokenizer.mode;
        $p.open_group();
        $p.tokenizer.mode = $crate::quickparse::tokenizer::Mode::Text;
        let $arg = $p.get_argument(&mut $name);
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
            $p.get_argument(&mut $name)
        } else {
            $p.tokenizer.open_math(false);
            let r = $p.get_argument(&mut $name);
            $p.tokenizer.close_math();
            r
        };
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident{_:M}$($args:tt)*) => {
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
    (@args $p:ident:$name:ident[$opt:ident:!name]$($args:tt)*) => {
        let $opt = $p.read_opt_name_normalized(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[$opt:ident]$($args:tt)*) => {
        let $opt = $p.read_opt_str(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[mut $opt:ident:Map]$($args:tt)*) => {
        let mut $opt = $p.read_opt_map(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[$opt:ident:Map]$($args:tt)*) => {
        let $opt = $p.read_opt_map(&mut $name);
        tex!{@args $p:$name $($args)*}
    };
    (@args $p:ident:$name:ident[$opt:ident:type $tp:ty]$($args:tt)*) => {
        let $opt = <Vec<$tp> as crate::quickparse::latex::KeyValValues<_,_,_,_>>::parse_opt($p);
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
            $p.tokenizer.problem($name.range.start,"Expected character",DiagnosticLevel::Error);
            $crate::quickparse::latex::rules::MacroResult::Simple($name)
        }
    };
    (@args $p:ident:$name:ident !) => {
        $crate::quickparse::latex::rules::MacroResult::Simple($name)
    };
    (@args $p:ident:$name:ident => $b:block !) => {
        $b;
        $crate::quickparse::latex::rules::MacroResult::Simple($name)
    };
    (@args $p:ident:$name:ident => $b:block) => {$b};
}

tex!(p => begin{n:name} => {
    match p.environment(begin,n.0,n.1) {
        EnvironmentResult::Success(e) => MacroResult::Success(e),
        EnvironmentResult::Other(v) => MacroResult::Other(v),
        EnvironmentResult::Simple(e) => T::from_environment(e).map_or_else(
            || MacroResult::Other(Vec::new()),
            MacroResult::Success
        )
    }
});

tex!(p => end{n:name} => {
    p.tokenizer.problem(end.range.start,format!("environment {} not open",n.0.as_ref()),DiagnosticLevel::Error);
}!);

tex!(p => lstinline[_](c)V:C(c)!);
tex!(p => verb[_](c)V:C(c)!);
tex!(p => stexcodeinline[_](c)V:C(c)!);
tex!(p => stexinline[_](c)V:C(c)!);
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

tex!(p => hbox{t:T} => { MacroResult::Other(t.1) });
tex!(p => vbox{t:T} => { MacroResult::Other(t.1) });
tex!(p => fbox{t:T} => { MacroResult::Other(t.1) });
tex!(p => mvbox{t:T} => { MacroResult::Other(t.1) });
tex!(p => text{t:T} => { MacroResult::Other(t.1) });
tex!(p => texttt{t:T} => { MacroResult::Other(t.1) });
tex!(p => textrm{t:T} => { MacroResult::Other(t.1) });
tex!(p => textbf{t:T} => { MacroResult::Other(t.1) });
tex!(p => scalebox{_}{t:T} => { MacroResult::Other(t.1) });
tex!(p => raisebox{_}{t:T} => { MacroResult::Other(t.1) });
tex!(p => ensuremath{t:M} => { MacroResult::Other(t.1) });


tex!(p => def => {
    p.tokenizer.reader.read_until(|c| c == '{');
    p.skip_arg(&mut def);
}!);
tex!(p => edef => {def(edef,p)});
tex!(p => gdef => {def(gdef,p)});
tex!(p => xdef => {def(xdef,p)});

tex!(p => @begin{document} {}{
    let _start = p.curr_pos();
    let _rest = p.tokenizer.reader.read_until_str("this string should never occur FOOBARBAZ BLA BLA asdk<ösndkf.k<asfb.mdv <sdasdjn");
}!);
tex!(p => @begin{verbatim}(V) {}{}!);
tex!(p => @begin{lstlisting}(V) {}{}!);
tex!(p => @begin{stexcode}(V) {}{}!);

tex!(p => @begin{general_listing}(V!) {}{}!);