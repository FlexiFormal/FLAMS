#[macro_export]
macro_rules! oms {
    (shtml:$i:ident) => {
        $crate::content::terms::Term::OMID($crate::uris::ContentURI::Symbol($crate::shtml!($i)))
    };
    (=shtml:$i:ident) => {
        $crate::content::terms::Term::OMID($crate::uris::ContentURI::Symbol($crate::metatheory::$i))
    };
    ($s:expr) => {
        $crate::content::terms::Term::OMID($crate::uris::ContentURI::Symbol($s))
    };
}

#[macro_export]
macro_rules! omsp {
    ($p:pat) => {
        $crate::content::terms::Term::OMID($crate::uris::ContentURI::Symbol($p))
    };
}

#[macro_export]
macro_rules! omfp {
    (($e:expr).($f:expr) = ($o:expr)) => {
        $crate::content::terms::Term::Field {
            record: Box::new($e),
            key: $f,
            owner: Some(Box::new($o)),
        }
    };
}

#[macro_export]
macro_rules! shtml {
    ($name:ident) => {
        ($crate::metatheory::$name).clone()
    };
}
#[macro_export]
macro_rules! oma {
    //matches!(self,oma!(omsp!(fp),[N:_,N:_]) if *fp == *crate::metatheory::FIELD_PROJECTION)
    ($head:expr,[$({$($tt:tt)*}),+]) => {
        $crate::content::terms::Term::OMA{
            head:Box::new($head),
            args:Box::new([$crate::$(oma!(@ARGS $($tt)*)),+])
        }
    };
    ($head:pat,[$($i:ident:$p:pat),+]) => {
        $crate::content::terms::Term::OMA{
            head:box $head,
            args:box [$($crate::oma!(@ARGSPAT $i:$p )),+]
        }
    };
    ($head:pat,$args:pat) => {
        $crate::content::terms::Term::OMA{
            head:box $head,
            args:$args
        }
    };
    ($head:expr,I@$($args:tt)*) => {
        $crate::content::terms::Term::OMA{
            head:Box::new($head),
            args:$crate::oma!(@ARGSITER $($args)*)
        }
    };
    (@ARGSITER $mode:ident:$args:expr) => {
        $args.map(|term| $crate::content::terms::Arg{term,mode:$crate::oma!(@MODE $mode)})
            .collect::<Box<[_]>>()
    };
    (@ARGSPAT $mode:ident:$p:pat) => {
        $crate::content::terms::Arg{term:$p,mode:$crate::oma!(@MODE $mode)}
    };
    (@ARGS $mode:ident:$a:expr) => {
        $crate::content::terms::Arg{term:$a,mode:$crate::oma!(@MODE $mode)}
    };

    (@MODE N) => {$crate::content::terms::ArgMode::Normal};
}

#[macro_export]
macro_rules! oml {
    ($name:literal $(:$tp:expr;)? $(:=$df:expr)?) => {
        $crate::content::terms::Term::OML{
            name:$name.into(),
            tp:$crate::oml!(@TPDF $($tp)?),
            df:$crate::oml!(@TPDF $($df)?),
        }
    };
    (@TPDF $tpdf:expr) => {Some(Box::new($tpdf))};
    (@TPDF) => {None};
}
#[macro_export]
macro_rules! omv {
    ($name:literal) => {
        $crate::content::terms::Term::OMV($crate::content::terms::Var::Name($name.into()))
    };
}
