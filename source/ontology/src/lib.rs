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


use std::borrow::Cow;

use content::{declarations::DeclarationTrait, modules::{Module, Signature}, ContentReference, ModuleLike};
use immt_utils::sourcerefs::{ByteOffset, SourceRange};
use languages::Language;
use narration::documents::Document;
use uris::{DocumentURI, ModuleURI, SymbolURI};

pub mod content;
pub mod languages;
pub mod narration;
pub mod file_states;
#[cfg(feature = "rdf")]
pub mod rdf;
pub mod uris;
pub mod shtml;

mod sealed {
    pub trait Sealed {}
}

#[cfg(not(feature="serde"))]
pub trait CheckingState: sealed::Sealed+std::fmt::Debug {
    type ModuleLike:std::fmt::Debug;
    type Module:std::fmt::Debug;
    type Seq<A:std::fmt::Debug>:std::fmt::Debug;
    type Decl<D:DeclarationTrait + Resolvable<From=SymbolURI>>:std::fmt::Debug;
    type Doc:std::fmt::Debug;
    type Sig:std::fmt::Debug;
}
#[cfg(feature="serde")]
pub trait CheckingState: sealed::Sealed+std::fmt::Debug {//+serde::Serialize {
    type ModuleLike:std::fmt::Debug + serde::Serialize;
    type Module:std::fmt::Debug + serde::Serialize;
    type Seq<A:std::fmt::Debug + serde::Serialize>:std::fmt::Debug + serde::Serialize;
    type Decl<D:DeclarationTrait + Resolvable<From=SymbolURI>>:std::fmt::Debug + serde::Serialize;
    type Doc:std::fmt::Debug + serde::Serialize;
    type Sig:std::fmt::Debug + serde::Serialize;
}

#[derive(Debug,Copy,Clone)]
//#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct Unchecked;
impl sealed::Sealed for Unchecked {}
impl CheckingState for Unchecked {
    type ModuleLike = ModuleURI;
    type Module = ModuleURI;
    type Decl<D:DeclarationTrait + Resolvable<From=SymbolURI>> = SymbolURI;
    #[cfg(feature="serde")]
    type Seq<A:std::fmt::Debug + serde::Serialize> = Vec<A>;
    #[cfg(not(feature="serde"))]
    type Seq<A:std::fmt::Debug> = Vec<A>;
    type Doc = DocumentURI;
    type Sig = Language;
}
#[derive(Debug,Copy,Clone)]
//#[cfg_attr(feature="serde",derive(serde::Serialize))]
pub struct Checked;
impl sealed::Sealed for Checked {}
impl CheckingState for Checked {
    type ModuleLike = MaybeResolved<ModuleLike>;
    type Module = MaybeResolved<Module>;
    type Decl<D:DeclarationTrait + Resolvable<From=SymbolURI>> = MaybeResolved<ContentReference<D>>;
    #[cfg(feature="serde")]
    type Seq<A:std::fmt::Debug + serde::Serialize> = Box<[A]>;
    #[cfg(not(feature="serde"))]
    type Seq<A:std::fmt::Debug> = Box<[A]>;
    type Doc = MaybeResolved<Document>;
    type Sig = MaybeResolved<Signature>;
}

pub trait Resolvable:std::fmt::Debug {
    type From:std::fmt::Debug+Clone;
    fn id(&self) -> Cow<'_,Self::From>;
}

#[derive(Debug)]
enum MaybeResolvedI<T:Resolvable> {
    Resolved(T),
    Unresolved(T::From)
}

#[derive(Debug)]
pub struct MaybeResolved<T:Resolvable> {
    inner:MaybeResolvedI<T>
}
impl<T:Resolvable> MaybeResolved<T> {
    #[inline]
    pub fn id(&self) -> Cow<'_,T::From> {
        match &self.inner {
            MaybeResolvedI::Resolved(r) => r.id(),
            MaybeResolvedI::Unresolved(i) => Cow::Borrowed(i)
        }
    }
    #[inline]
    pub const fn is_resolved(&self) -> bool {
        matches!(self.inner,MaybeResolvedI::Resolved(_))
    }
    #[inline]
    pub const fn get(&self) -> Option<&T> {
        if let MaybeResolvedI::Resolved(i) = &self.inner {
            Some(i)
        } else { None }
    }
    #[inline]
    pub const fn unresolved(id:T::From) -> Self {
        Self {
            inner: MaybeResolvedI::Unresolved(id)
        }
    }
    #[inline]
    pub const fn resolved(value:T) -> Self {
        Self {
            inner: MaybeResolvedI::Resolved(value)
        }
    }
    #[inline]
    pub fn resolve(id:T::From,resolve:impl FnOnce(&T::From) -> Option<T>) -> Self {
        resolve(&id).map_or_else(|| Self::unresolved(id), Self::resolved)
    }
}

#[cfg(feature="serde")]
mod serde_resolved {
    use crate::{MaybeResolved, MaybeResolvedI, Resolvable};

    impl<T:Resolvable> serde::Serialize for MaybeResolved<T> where T::From: serde::Serialize {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
            match &self.inner {
                MaybeResolvedI::Unresolved(t) => t.serialize(serializer),
                MaybeResolvedI::Resolved(s) => {
                    let id = s.id();
                    let id = &*id;
                    id.serialize(serializer)
                }
            }
        }
    } 
}

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
/*
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
*/

pub mod metatheory {
    use crate::uris::{BaseURI, ModuleURI, SymbolURI};
    use lazy_static::lazy_static;
    lazy_static! {
        pub static ref URI: ModuleURI =
            BaseURI::new_unchecked("http://mathhub.info") & "sTeX/meta-inf" | "Metatheory";
        pub static ref FIELD_PROJECTION: SymbolURI = URI.clone() | "record field";
        pub static ref OF_TYPE: SymbolURI = URI.clone() | "of type";
        pub static ref SEQUENCE_EXPRESSION: SymbolURI = URI.clone() | "sequence expression";
        pub(crate) static ref NOTATION_DUMMY: SymbolURI = URI.clone() | "notation dummy";
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


macro_rules! serde_impl {
    (@i_count ) => { 0 };
    (@i_count $r:ident $($rs:tt)* ) => { 1 + crate::serde_impl!(@i_count $($rs)*) };
    (@count $($r:ident)*) => { crate::serde_impl!(@i_count $($r)*)};
    
    (@caseI $f:ident) => {
        Self::$f
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident) => {
        $ser.serialize_unit_variant(stringify!($s),$idx,stringify!($f))
    };
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident) => {
        {$ser.unit_variant()?;Ok(Self::$f)}
    };

    (@caseI $f:ident($nt:ident)) => {
        Self::$f($nt)
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident($nt:ident)) => {
        $ser.serialize_newtype_variant(stringify!($s),$idx,stringify!($f),$nt)
    };
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident($nt:ident)) => {
        $ser.newtype_variant().map($s::$f)
    };

    (@caseI $f:ident{ $($n:ident),* }) => {
        Self::$f{$($n),*}
    };
    (@caseII $ser:ident $s:ident $idx:literal $f:ident{ $($n:ident),* }) => {{
        let mut s = $ser.serialize_struct_variant(stringify!($s),$idx,stringify!($f),
            crate::serde_impl!(@count $($n)*)
        )?;
        $(
            s.serialize_field(stringify!($n),$n)?;
        )*
        s.end()
    }};
    (@caseIII $ser:ident $s:ident $idx:literal $f:ident{ $($n:ident),* }) => {{
        struct SVisitor;

        #[derive(serde::Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field { $($n),* }
        impl<'de> serde::de::Visitor<'de> for SVisitor {
            type Value = $s<Unchecked>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(stringify!($f))
            }
            #[allow(unused_assignments)]
            fn visit_seq<V>(self, mut seq: V) -> Result<$s<Unchecked>, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let mut count = 0;
                $(
                    let $n = seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(count, &self))?;
                    count += 1;
                )*
                Ok($s::$f{ $($n),* })
            }
            fn visit_map<V>(self, mut map: V) -> Result<$s<Unchecked>, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                $(
                    let mut $n = None;
                )*
                while let Some(key) = map.next_key()? {
                    match key {
                        $(
                            Field::$n => {
                                if $n.is_some() {
                                    return Err(serde::de::Error::duplicate_field(stringify!($n)));
                                }
                                $n = Some(map.next_value()?);
                            }
                        )*
                    }
                }
                $(
                    let $n = $n.ok_or_else(|| serde::de::Error::missing_field(stringify!($n)))?;
                )*
                Ok($s::$f { $($n),* })
            }
        }

        $ser.struct_variant(&[ $(stringify!($n)),* ],SVisitor)
    }};

    ($(mod $m:ident = )? struct $s:ident[$($f:ident),+] ) => {crate::serde_impl!{$(mod $m = )? $s : slf
        s => {
            let mut s = s.serialize_struct(
                stringify!($s),
                crate::serde_impl!(@count $($f)*)
            )?;
            $(
                s.serialize_field(stringify!($f),&slf.$f)?;
            )*
            s.end()
        }
        d => {
            #[derive(serde::Deserialize)]
            #[allow(non_camel_case_types)]
            enum Field { $($f),* }
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = $s<Unchecked>;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(stringify!($s))
                }
                #[allow(unused_assignments)]
                fn visit_seq<V>(self, mut seq: V) -> Result<$s<Unchecked>, V::Error>
                where
                    V: serde::de::SeqAccess<'de>,
                {
                    let mut count = 0;
                    $(
                        let $f = seq.next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(count, &self))?;
                        count += 1;
                    )*
                    Ok($s{ $($f),* })
                }
                fn visit_map<V>(self, mut map: V) -> Result<$s<Unchecked>, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
                {
                    $(
                        let mut $f = None;
                    )*
                    while let Some(key) = map.next_key()? {
                        match key {
                            $(
                                Field::$f => {
                                    if $f.is_some() {
                                        return Err(serde::de::Error::duplicate_field(stringify!($f)));
                                    }
                                    $f = Some(map.next_value()?);
                                }
                            )*
                        }
                    }
                    $(
                        let $f = $f.ok_or_else(|| serde::de::Error::missing_field(stringify!($f)))?;
                    )*
                    Ok($s { $($f),* })
                }
            }
            d.deserialize_struct(stringify!($s),&[$(stringify!($f)),*],Visitor)

        }
    }};

    ($(mod $m:ident = )? enum $s:ident{ $( {$idx:literal = $f:ident $($spec:tt)*} )+ } ) => {
        crate::serde_impl!{$(mod $m = )? $s : slf 
            ser => {
                match slf {
                    $(
                        crate::serde_impl!(@caseI $f $($spec)*) =>
                        crate::serde_impl!{@caseII ser $s $idx $f $($spec)* }
                    ),*
                }
            } 
            de => {
                #[derive(serde::Deserialize)]
                enum Fields {
                    $(
                        $f = $idx
                    ),*
                }
                struct Visitor;
                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $s<Unchecked>;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(stringify!($s))
                    }
                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: EnumAccess<'de>,
                    {
                        let (v,var) = data.variant()?;
                        match v {
                            $(
                                Fields::$f => crate::serde_impl!{@caseIII var $s $idx $f $($spec)* },
                            )*
                            //s => Err(A::Error::unknown_variant(s, &[ $(stringify!($f)),* ]))
                        }
                        
                    }

                }
                
                de.deserialize_enum(
                    stringify!($s), 
                    &[ $( stringify!($f) ),* ], 
                    Visitor
                )
            }
        }
    };

    ($s:ident : $slf:ident $ser:ident => {$($ser_impl:tt)*} $de:ident => {$($de_impl:tt)*}) => {
        crate::serde_impl!{mod serde_impl = $s : $slf $ser => {$($ser_impl)*} $de => {$($de_impl)*}}
    };

    (mod $m:ident = $s:ident : $slf:ident $ser:ident => {$($ser_impl:tt)*} $de:ident => {$($de_impl:tt)*}) => {
        #[cfg(feature="serde")]#[allow(unused_imports)]
        mod $m {
            use super::$s;
            use crate::Unchecked;
            use ::serde::ser::{SerializeStruct,SerializeStructVariant};
            use ::serde::de::{EnumAccess,VariantAccess,Error};
            impl<State:$crate::CheckingState> ::serde::Serialize for $s<State> {
                fn serialize<S: ::serde::Serializer>(&self,$ser:S) -> Result<S::Ok,S::Error> {
                    let $slf = self;
                    $($ser_impl)*
                }
            }
            impl<'de> ::serde::Deserialize<'de> for $s<Unchecked> {
                fn deserialize<D: ::serde::de::Deserializer<'de>>($de: D) -> Result<Self, D::Error> {
                    $($de_impl)*
                }
            }
        }
    };
}
pub(crate) use serde_impl;