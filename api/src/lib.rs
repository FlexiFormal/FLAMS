#![recursion_limit="256"]

#[cfg(debug_assertions)]
pub type Str = String;
#[cfg(not(debug_assertions))]
pub type Str = Box<str>;

#[cfg(debug_assertions)]
pub type Seq<A> = Vec<A>;
#[cfg(not(debug_assertions))]
pub type Seq<A> = Box<[A]>;

pub mod ontology {
    pub mod rdf;
}

pub mod formats;
pub mod source_files;
pub mod uris;
pub mod archives;

pub mod utils {
    use crate::Str;

    pub mod parsing;
    pub mod problems;
    pub mod iter;

    pub type HMap<A,B> = ahash::HashMap<A,B>;

    pub type MMTURI = Str;
}
