#[macro_export]
macro_rules! asyncs {
    (@macros_sync $($f:tt)*) => {
        macro_rules! switch {
            (($sync:expr)($async:expr)) => {$sync};
        }
        macro_rules! read_dir{
            ($e:expr) => {std::fs::read_dir($e)};
        }
        macro_rules! next_file{
            ($e:expr) => {$e.next()};
        }
        macro_rules! wait {
            ($e:expr) => {$e};
        }
        macro_rules! read_file {
            ($e:expr) => {std::fs::read($e)};
        }
        macro_rules! read_lines {
            ($l:ident <- $e:expr) => {
                let reader = std::io::BufReader::new(std::fs::File::open($e).unwrap());
                let mut $l = reader.lines();
            };
        }
        macro_rules! next_line {
            ($l:expr) => {$l.next()};
        }
        $($f)*
    };
    (@macros_async $($f:tt)*) => {
        macro_rules! switch {
            (($sync:expr)($async:expr)) => {$async.await};
        }
        macro_rules! read_dir{
            ($e:expr) => {tokio::fs::read_dir($e).await};
        }
        macro_rules! next_file{
            ($e:expr) => {match $e.next_entry().await {
                Ok(Some(a)) => Some(Ok(a)),
                Ok(None) => None,
                Err(e) => Some(Err(e))
            }};
        }
        macro_rules! wait {
            ($e:expr) => {$e.await};
        }
        macro_rules! read_file {
            ($e:expr) => {tokio::fs::read($e).await};
        }
        macro_rules! read_lines {
            ($l:ident <- $e:expr) => {
                let reader = tokio::io::BufReader::new(tokio::fs::File::open($e).await.unwrap());
                let mut $l = tokio::io::AsyncBufReadExt::lines(reader);
            };
        }
        macro_rules! next_line {
            ($l:expr) => {match $l.next_line().await {
                Ok(Some(a)) => Some(Ok(a)),
                Ok(None) => None,
                Err(e) => Some(Err(e))
            }};
        }
        $($f)*
    };
    ($(!$v:vis)? fn $name:ident $(<[$($params:tt)*]>)? ($($args:tt)*) $(-> $ret:ty)? {$($f:tt)*} ) => {
        $crate::asyncs!{$(!$v)? fn $name $(<@s[$($params)*]><@a[$($params)*]>)? (@s $($args)*) (@a $($args)*) $(-> $ret)? {$($f)*}}
    };
    (ALT $(!$v:vis)? fn $name:ident $(<[$($params:tt)*]>)? ($($args:tt)*) $(-> $ret:ty)? {$($f:tt)*} ) => {
        $crate::asyncs!{ALT $(!$v)? fn $name $(<@s[$($params)*]><@a[$($params)*]>)? (@s $($args)*) (@a $($args)*) $(-> $ret)? {$($f)*}}
    };
    ($(!$v:vis)? fn $name:ident $(<@s[$($params_s:tt)*]>)? $(<@a[$($params_a:tt)*]>)? (@s$($args_s:tt)*) (@a$($args_a:tt)*) $(-> $ret:ty)? {$($f:tt)*} ) => {
        $($v)? fn $name$(<$($params_s)*>)?($($args_s)*) $(-> $ret)? {
            $crate::asyncs!{@macros_sync $($f)*}
        }

        paste::paste!{
        #[cfg(feature="async")]
        $($v)? async fn [<$name _async>]$(<$($params_a)*>)?($($args_a)*) $(-> $ret)? {
            $crate::asyncs!{@macros_async $($f)*}
        }}
    };
    (ALT $(!$v:vis)? fn $name:ident $(<@s[$($params_s:tt)*]>)? $(<@a[$($params_a:tt)*]>)? (@s$($args_s:tt)*) (@a$($args_a:tt)*) $(-> $ret:ty)? {$($f:tt)*} ) => {
        #[cfg(not(feature="async"))]
        $($v)? fn $name$(<$($params_s)*>)?($($args_s)*) $(-> $ret)? {
            $crate::asyncs!{@macros_sync $($f)*}
        }

        #[cfg(feature="async")]
        $($v)? async fn $name$(<$($params_a)*>)?($($args_a)*) $(-> $ret)? {
            $crate::asyncs!{@macros_async $($f)*}
        }
    };
}