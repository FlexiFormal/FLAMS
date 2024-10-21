#![feature(ptr_as_ref_unchecked)]

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

#[derive(Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(Str),
    Inline(Str),
}

#[cfg(feature="tokio")]
pub fn background<F:FnOnce() + Send + 'static>(f:F) {
    tokio::task::spawn_blocking(f);
}

pub fn in_span<F:FnOnce() -> R,R>(f:F) -> impl FnOnce() -> R {
    let span = tracing::Span::current();
    move || {
        let _span = span.enter();
        f()
    }
}
