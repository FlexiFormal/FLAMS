#![feature(ptr_as_ref_unchecked)]

pub mod vecmap;
pub mod time;
pub mod sourcerefs;
pub mod parsing;
pub mod gc;
mod treelike;
pub mod escaping;
mod inner_arc;

pub use triomphe;
pub use parking_lot;

pub mod prelude {
    pub use super::vecmap::{VecMap,VecSet};
    pub type HMap<K,V> = rustc_hash::FxHashMap<K,V>;
    pub type HSet<V> = rustc_hash::FxHashSet<V>;
    pub use crate::treelike::*;
    pub use crate::inner_arc::InnerArc;
}

#[cfg(target_family = "wasm")]
type Str = String;
#[cfg(not(target_family = "wasm"))]
type Str = Box<str>;

pub fn hashstr<A:std::hash::Hash>(prefix:&str,a:&A) -> String {
    use std::hash::BuildHasher;
    let h = rustc_hash::FxBuildHasher.hash_one(a);
    format!("{prefix}{h:02x}")
}


#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(Str),
    Inline(Str),
}