//#![feature(ptr_as_ref_unchecked)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod binary;
#[cfg(feature = "async")]
pub mod change_listener;
pub mod escaping;
pub mod gc;
pub mod globals;
pub mod id_counters;
mod inner_arc;
pub mod logs;
pub mod parsing;
pub mod regex;
pub mod settings;
pub mod sourcerefs;
pub mod time;
mod treelike;
pub mod vecmap;
//pub mod file_id;

pub use parking_lot;
pub use triomphe;

pub mod prelude {
    pub use super::vecmap::{VecMap, VecSet};
    pub type HMap<K, V> = rustc_hash::FxHashMap<K, V>;
    pub type HSet<V> = rustc_hash::FxHashSet<V>;
    pub use crate::inner_arc::InnerArc;
    pub use crate::treelike::*;
}

#[cfg(target_family = "wasm")]
type Str = String;
#[cfg(not(target_family = "wasm"))]
type Str = Box<str>;

pub fn hashstr<A: std::hash::Hash>(prefix: &str, a: &A) -> String {
    use std::hash::BuildHasher;
    let h = rustc_hash::FxBuildHasher.hash_one(a);
    format!("{prefix}{h:02x}")
}

#[cfg(feature = "tokio")]
pub fn background<F: FnOnce() + Send + 'static>(f: F) {
    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || span.in_scope(f));
}

pub fn in_span<F: FnOnce() -> R, R>(f: F) -> impl FnOnce() -> R {
    let span = tracing::Span::current();
    move || {
        let _span = span.enter();
        f()
    }
}

#[cfg(feature = "serde")]
pub trait Hexable: Sized {
    /// #### Errors
    fn as_hex(&self) -> eyre::Result<String>;
    /// #### Errors
    fn from_hex(s: &str) -> eyre::Result<Self>;
}
#[cfg(feature = "serde")]
impl<T: Sized + serde::Serialize + for<'de> serde::Deserialize<'de>> Hexable for T {
    fn as_hex(&self) -> eyre::Result<String> {
        use std::fmt::Write;
        let bc = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        let mut ret = String::with_capacity(bc.len() * 2);
        for b in bc {
            write!(ret, "{b:02X}")?;
        }
        Ok(ret)
    }
    fn from_hex(s: &str) -> eyre::Result<Self> {
        let bytes: Result<Vec<_>, _> = if s.len() % 2 == 0 {
            (0..s.len())
                .step_by(2)
                .filter_map(|i| s.get(i..i + 2))
                .map(|sub| u8::from_str_radix(sub, 16))
                .collect()
        } else {
            return Err(eyre::eyre!("Incompatible string length"));
        };
        bincode::serde::decode_from_slice(&bytes?, bincode::config::standard())
            .map(|(r, _)| r)
            .map_err(Into::into)
    }
}

pub mod fs {
    use std::path::Path;

    /// #### Errors
    pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dst).map_err(|e| format!("{e} ({})", dst.display()))?;
        for entry in std::fs::read_dir(src).map_err(|e| format!("{e} ({})", src.display()))? {
            let entry = entry.map_err(|e| format!("{e} ({})", src.display()))?;
            let ty = entry
                .file_type()
                .map_err(|e| format!("{e} ({})", entry.path().display()))?;
            let target = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &target)?;
            } else {
                let md = entry
                    .metadata()
                    .map_err(|e| format!("{e} ({})", entry.path().display()))?;
                std::fs::copy(entry.path(), &target).map_err(|e| {
                    format!("{e} ({} => {})", entry.path().display(), target.display())
                })?;
                let mtime = filetime::FileTime::from_last_modification_time(&md);
                filetime::set_file_mtime(&target, mtime)
                    .map_err(|e| format!("{e} ({})", target.display()))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
    Inline(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
    Class {
        #[cfg_attr(feature = "wasm", tsify(type = "string"))]
        name: Str,
        #[cfg_attr(feature = "wasm", tsify(type = "string"))]
        css: Str,
    },
}
impl PartialOrd for CSS {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CSS {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn classnum(s: &str) -> u8 {
            match s {
                s if s.starts_with("ftml-subproblem") => 1,
                s if s.starts_with("ftml-problem") => 2,
                s if s.starts_with("ftml-example") => 3,
                s if s.starts_with("ftml-definition") => 4,
                s if s.starts_with("ftml-paragraph") => 5,
                "ftml-subsubsection" => 6,
                "ftml-subsection" => 7,
                "ftml-section" => 8,
                "ftml-chapter" => 9,
                "ftml-part" => 10,
                _ => 0,
            }
        }
        use std::cmp::Ordering;
        match (self, other) {
            (Self::Link(l1), Self::Link(l2)) | (Self::Inline(l1), Self::Inline(l2)) => l1.cmp(l2),
            (Self::Link(_), Self::Inline(_))
            | (Self::Link(_) | Self::Inline(_), Self::Class { .. }) => Ordering::Less,
            (Self::Inline(_), Self::Link(_))
            | (Self::Class { .. }, Self::Inline(_) | Self::Link(_)) => Ordering::Greater,
            (Self::Class { name: n1, css: c1 }, Self::Class { name: n2, css: c2 }) => {
                (classnum(n1), n1, c1).cmp(&(classnum(n2), n2, c2))
            }
        }
    }
}
impl CSS {
    #[must_use]
    pub fn split(css: &str) -> Vec<Self> {
        use lightningcss::traits::ToCss;
        use lightningcss::{
            printer::PrinterOptions,
            rules::CssRule,
            selector::Component,
            stylesheet::{ParserOptions, StyleSheet},
        };
        let Ok(ruleset) = StyleSheet::parse(css, ParserOptions::default()) else {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())];
        };
        if ruleset.sources.iter().any(|s| !s.is_empty()) {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())];
        }
        ruleset
            .rules
            .0
            .into_iter()
            .filter_map(|rule| match rule {
                CssRule::Style(style) => {
                    if style.vendor_prefix.is_empty()
                        && style.selectors.0.len() == 1
                        && style.selectors.0[0].len() == 1
                        && matches!(
                            style.selectors.0[0].iter().next(),
                            Some(Component::Class(_))
                        )
                    {
                        let Some(Component::Class(class_name)) = style.selectors.0[0].iter().next()
                        else {
                            impossible!()
                        };
                        style
                            .to_css_string(PrinterOptions::default())
                            .ok()
                            .map(|s| Self::Class {
                                name: class_name.to_string().into(),
                                css: s.into(),
                            })
                    } else {
                        style
                            .to_css_string(PrinterOptions::default())
                            .ok()
                            .map(|s| {
                                tracing::warn!("Not class-able: {s}");
                                Self::Inline(s.into())
                            })
                    }
                }
                o => o.to_css_string(PrinterOptions::default()).ok().map(|s| {
                    tracing::warn!("Not class-able: {s}");
                    Self::Inline(s.into())
                }),
            })
            .collect()
    }
}

#[macro_export]
macro_rules! impossible {
    () => {{
        #[cfg(debug_assertions)]
        {
            unreachable!()
        }
        #[cfg(not(debug_assertions))]
        {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }};
    ($s:literal) => {
        #[cfg(debug_assertions)]
        {
            panic!($s)
        }
        #[cfg(not(debug_assertions))]
        {
            unsafe { std::hint::unreachable_unchecked() }
        }
    };
    (?) => {
        unreachable!()
    };
    (? $s:literal) => {{
        panic!($s)
    }};
}

#[macro_export]
macro_rules! unwrap {
    ($e: expr) => { $e.unwrap_or_else(|| {$crate::impossible!();}) };
    (? $e: expr) => { $e.unwrap_or_else(|| {$crate::impossible!(?);}) };
    ($e: expr;$l:literal) => { $e.unwrap_or_else(|| {$crate::impossible!($l);}) };
    (? $e: expr;$l:literal) => { $e.unwrap_or_else(|| {$crate::impossible!(? $l);}) };
}

#[cfg(feature = "serde")]
pub trait CondSerialize: serde::Serialize {}
#[cfg(feature = "serde")]
impl<T: serde::Serialize> CondSerialize for T {}

#[cfg(not(feature = "serde"))]
pub trait CondSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> CondSerialize for T {}

#[allow(clippy::unwrap_used)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::similar_names)]
#[test]
fn css_things() {
    use lightningcss::traits::ToCss;
    use lightningcss::{
        printer::PrinterOptions,
        rules::CssRule,
        selector::Component,
        stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    };
    tracing_subscriber::fmt().init();
    let css = include_str!("../../../resources/assets/rustex.css");
    let rules = StyleSheet::parse(css, ParserOptions::default()).unwrap();
    let roundtrip = rules.to_css(PrinterOptions::default()).unwrap();
    tracing::info!("{}", roundtrip.code);
    let test = "
        .ftml-paragraph {
            > .ftml-title {
                font-weight: bold;
            }
            margin: 0;
        }
    ";
    let mut ruleset = StyleSheet::parse(test, ParserOptions::default()).unwrap();
    ruleset.minify(MinifyOptions::default()).unwrap();
    assert!(ruleset.sources.iter().all(String::is_empty));
    tracing::info!("Result: {ruleset:#?}");
    for rule in ruleset.rules.0 {
        match rule {
            CssRule::Style(style) => {
                assert!(style.vendor_prefix.is_empty());
                assert!(style.selectors.0.len() == 1);
                assert!(style.selectors.0[0].len() == 1);
                tracing::info!(
                    "Here: {}",
                    style.to_css_string(PrinterOptions::default()).unwrap()
                );
                let sel = style.selectors.0[0].iter().next().unwrap();
                assert!(matches!(sel, Component::Class(_)));
                let Component::Class(cls) = sel else {
                    impossible!()
                };
                let cls_str = &**cls;
                tracing::info!("Class: {cls_str}");
            }
            o => panic!("Unexpected rule: {o:#?}"),
        }
    }
}
