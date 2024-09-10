pub mod vecmap;
pub mod time;
pub mod sourcerefs;
pub mod parsing;

pub mod prelude {
    pub use triomphe;
    pub use parking_lot;
    pub use super::vecmap::{VecMap,VecSet};
    pub type HMap<K,V> = rustc_hash::FxHashMap<K,V>;
    pub type HSet<V> = rustc_hash::FxHashSet<V>;
}