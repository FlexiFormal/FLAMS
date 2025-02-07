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

#[derive(Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
    Inline(#[cfg_attr(feature = "wasm", tsify(type = "string"))] Str),
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