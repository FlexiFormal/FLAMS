#![recursion_limit = "256"]
#![feature(box_patterns)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
/*#![feature(adt_const_params)]

#[derive(std::marker::ConstParamTy,PartialEq,Eq)]
pub enum Test {
    A,
    B
}

pub struct Foo<const T:Test>(String);
*/

pub const SHTML_PREFIX: &str = "data-shtml-";

use content::{declarations::DeclarationTrait, ContentReference, ModuleLike};
use immt_utils::sourcerefs::{ByteOffset, SourceRange};
use narration::documents::Document;
use uris::{DocumentURI, ModuleURI, SymbolURI, URIParseError};

pub mod content;
pub mod languages;
pub mod narration;
pub mod file_states;
#[cfg(feature = "rdf")]
pub mod rdf;
pub mod uris;

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentRange {
    pub start: usize,
    pub end: usize,
}
impl From<SourceRange<ByteOffset>> for DocumentRange {
    #[inline]
    fn from(value: SourceRange<ByteOffset>) -> Self {
        Self {
            start: value.start.offset,
            end: value.end.offset,
        }
    }
}
impl From<DocumentRange> for SourceRange<ByteOffset> {
    #[inline]
    fn from(value: DocumentRange) -> Self {
        Self {
            start: ByteOffset {
                offset: value.start,
            },
            end: ByteOffset { offset: value.end },
        }
    }
}

pub enum DecodeError {
    URIParse(URIParseError),
    Io(immt_utils::binary::DecodeError),
    UnknownDiscriminant,
}
impl From<URIParseError> for DecodeError {
    #[inline]
    fn from(value: URIParseError) -> Self {
        Self::URIParse(value)
    }
}
impl From<immt_utils::binary::DecodeError> for DecodeError {
    #[inline]
    fn from(value: immt_utils::binary::DecodeError) -> Self {
        Self::Io(value)
    }
}
impl From<std::io::Error> for DecodeError {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.into())
    }
}

pub mod metatheory {
    use crate::uris::{BaseURI, ModuleURI, SymbolURI};
    use lazy_static::lazy_static;
    lazy_static! {
        pub static ref URI: ModuleURI =
            BaseURI::new_unchecked("http://mathhub.info") & "sTeX/meta-inf" | "Metatheory";
        pub static ref FIELD_PROJECTION: SymbolURI = URI.clone() | "record field";
        pub static ref OF_TYPE: SymbolURI = URI.clone() | "of type";
        pub static ref SEQUENCE_EXPRESSION: SymbolURI = URI.clone() | "sequence expression";
    }
}

pub trait LocalBackend {
    fn get_document(&mut self, uri: &DocumentURI) -> Option<Document>;

    fn get_module(&mut self, uri: &ModuleURI) -> Option<ModuleLike>;

    fn get_declaration<T: DeclarationTrait>(
        &mut self,
        uri: &SymbolURI,
    ) -> Option<ContentReference<T>>;
}

#[cfg(feature="serde")]
pub trait Resourcable:serde::Serialize + for <'a> serde::Deserialize<'a> {}

#[cfg(not(feature="serde"))]
pub trait Resourcable {}
