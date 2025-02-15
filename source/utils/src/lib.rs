#![feature(ptr_as_ref_unchecked)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod binary;
#[cfg(feature="async")]
pub mod change_listener;
pub mod escaping;
pub mod gc;
pub mod globals;
mod inner_arc;
pub mod parsing;
pub mod sourcerefs;
pub mod time;
mod treelike;
pub mod vecmap;
pub mod settings;
pub mod logs;
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

#[cfg(feature="tokio")]
pub fn background<F:FnOnce() + Send + 'static>(f:F) {
    use tracing::Instrument;
    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || span.in_scope(f));
}

pub fn in_span<F:FnOnce() -> R,R>(f:F) -> impl FnOnce() -> R {
    let span = tracing::Span::current();
    move || {
        let _span = span.enter();
        f()
    }
}

pub mod fs {
    use std::path::Path;

    /// #### Errors
    pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let target = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &target)?;
            } else {
                let md = entry.metadata()?;
                std::fs::copy(entry.path(), &target)?;
                let mtime = filetime::FileTime::from_last_modification_time(&md);
                filetime::set_file_mtime(&target, mtime)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
    Inline(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
    Class{
        #[cfg_attr(feature = "wasm", tsify(type = "string"))] name:Str,
        #[cfg_attr(feature = "wasm", tsify(type = "string"))] css:Str
    }
}
impl CSS {
    #[must_use]
    pub fn split(css:&str) -> Vec<Self> {
        use lightningcss::{stylesheet::{ParserOptions,StyleSheet},printer::PrinterOptions,rules::CssRule,selector::Component};
        use lightningcss::traits::ToCss;
        let Ok(ruleset) = StyleSheet::parse(css, ParserOptions::default()) else {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())]
        };
        if ruleset.sources.iter().any(|s| !s.is_empty()) {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())];
        }
        ruleset.rules.0.into_iter().filter_map(|rule| {
            match rule {
                CssRule::Style(style) => {
                    if style.vendor_prefix.is_empty() && style.selectors.0.len() == 1 &&
                        style.selectors.0[0].len() == 1 && matches!(style.selectors.0[0].iter().next(),Some(Component::Class(_))) {
                        let Some(Component::Class(class_name)) = style.selectors.0[0].iter().next() else { unreachable!() };
                        style.to_css_string(PrinterOptions::default()).ok().map(|s| Self::Class{ name: class_name.to_string().into(), css: s.into() })
                    } else {
                        style.to_css_string(PrinterOptions::default()).ok().map(|s| {tracing::warn!("Not class-able: {s}");Self::Inline(s.into())})
                    }
                }
                o => {
                    o.to_css_string(PrinterOptions::default()).ok().map(|s| {tracing::warn!("Not class-able: {s}");Self::Inline(s.into())})
                }
            }
        }).collect()
    }
}

#[allow(clippy::unwrap_used)]
#[test]
fn css_things() {
    use lightningcss::{stylesheet::{ParserOptions,StyleSheet,MinifyOptions},printer::PrinterOptions,rules::CssRule,selector::Component};
    use lightningcss::traits::ToCss;
    tracing_subscriber::fmt().init();
    let css = include_str!("../../../resources/assets/rustex.css");
    let rules = StyleSheet::parse(css, ParserOptions::default()).unwrap();
    let roundtrip = rules.to_css(PrinterOptions::default()).unwrap();
    tracing::info!("{}",roundtrip.code);
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
    assert!(ruleset.sources.iter().all(|s| s.is_empty()));
    tracing::info!("Result: {ruleset:#?}");
    for rule in ruleset.rules.0.into_iter() {
        match rule {
            CssRule::Style(style) => {
                assert!(style.vendor_prefix.is_empty());
                assert!(style.selectors.0.len() == 1);
                assert!(style.selectors.0[0].len() == 1);
                tracing::info!("Here: {}",style.to_css_string(PrinterOptions::default()).unwrap());
                let sel = style.selectors.0[0].iter().next().unwrap();
                assert!(matches!(sel,Component::Class(_)));
                let Component::Class(cls) = sel else {unreachable!()};
                let cls_str = &**cls;
                tracing::info!("Class: {cls_str}");
            }
            o => panic!("Unexpected rule: {o:#?}"),
        }
    }
}