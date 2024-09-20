#![recursion_limit = "256"]
#![feature(box_patterns)]


#[cfg(feature = "rdf")]
pub mod rdf;
pub mod uris;
pub mod languages;
pub mod content;
pub mod narration;

pub mod metatheory {
    use lazy_static::lazy_static;
    use crate::uris::{BaseURI, ModuleURI, SymbolURI};
    lazy_static! {
        pub static ref URI: ModuleURI =
            BaseURI::new_unchecked("http://mathhub.info")
            & "sTeX/meta-inf" | "Metatheory";
        pub static ref FIELD_PROJECTION : SymbolURI =
            URI.clone() | "record field";
        pub static ref OF_TYPE : SymbolURI =
            URI.clone() | "of type";
        pub static ref SEQUENCE_EXPRESSION : SymbolURI =
            URI.clone() | "sequence expression";
    }
}