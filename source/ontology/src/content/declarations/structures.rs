use crate::{
    content::ModuleTrait, uris::{ContentURIRef, SymbolURI}, Checked, CheckingState, Resolvable
};

use super::{Declaration, DeclarationTrait, OpenDeclaration};


#[derive(Debug)]
//#[cfg_attr(feature="serde", derive(serde::Serialize))]
pub struct MathStructure<State:CheckingState> {
    pub uri: SymbolURI,
    pub elements: State::Seq<OpenDeclaration<State>>,
    pub macroname: Option<Box<str>>,
}
impl Resolvable for MathStructure<Checked> {
    type From = SymbolURI;
    fn id(&self) -> std::borrow::Cow<'_,Self::From> {
        std::borrow::Cow::Borrowed(&self.uri)
    }
}
impl super::private::Sealed for MathStructure<Checked> {}
impl DeclarationTrait for MathStructure<Checked> {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::MathStructure(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for MathStructure<Checked> {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Symbol(&self.uri)
    }
}


#[derive(Debug)]
pub struct Extension<State:CheckingState> {
    pub uri: SymbolURI,
    pub target: State::Decl<MathStructure<Checked>>,
    pub elements: State::Seq<OpenDeclaration<State>>,
}
impl Resolvable for Extension<Checked> {
    type From = SymbolURI;
    fn id(&self) -> std::borrow::Cow<'_,Self::From> {
        std::borrow::Cow::Borrowed(&self.uri)
    }
}
impl super::private::Sealed for Extension<Checked> {}
impl DeclarationTrait for Extension<Checked> {
    #[inline]
    fn from_declaration(decl: &Declaration) -> Option<&Self> {
        match decl {
            Declaration::Extension(m) => Some(m),
            _ => None,
        }
    }
}
impl ModuleTrait for Extension<Checked> {
    #[inline]
    fn declarations(&self) -> &[Declaration] {
        &self.elements
    }
    #[inline]
    fn content_uri(&self) -> ContentURIRef {
        ContentURIRef::Symbol(&self.uri)
    }
}

crate::serde_impl!{mod serde_impl_struct =
    struct MathStructure[uri,elements,macroname]
}
crate::serde_impl!{mod serde_impl_ext =
    struct Extension[uri,target,elements]
}

/*
crate::serde_impl!{
    MathStructure : slf
    s => {
        let mut s = s.serialize_struct("MathStructure", 3)?;
        s.serialize_field("uri", &slf.uri);
        s.serialize_field("elements", &slf.elements);
        s.serialize_field("macroname", &slf.macroname);
        s.end()
    } 
    de => {
        #[derive(serde::Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field {uri,elements,macroname}

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MathStructure<Unchecked>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct MathStructure")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<MathStructure<Unchecked>, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let uri = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let elements = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let macroname = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(MathStructure{ uri, elements, macroname })
            }

            fn visit_map<V>(self, mut map: V) -> Result<MathStructure<Unchecked>, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut uri = None;
                let mut elements = None;
                let mut macroname = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::uri => {
                            if uri.is_some() {
                                return Err(serde::de::Error::duplicate_field("uri"));
                            }
                            uri = Some(map.next_value()?);
                        }
                        Field::elements => {
                            if elements.is_some() {
                                return Err(serde::de::Error::duplicate_field("elements"));
                            }
                            elements = Some(map.next_value()?);
                        }
                        Field::macroname => {
                            if macroname.is_some() {
                                return Err(serde::de::Error::duplicate_field("macroname"));
                            }
                            macroname = Some(map.next_value()?);
                        }
                    }
                }
                let uri = uri.ok_or_else(|| serde::de::Error::missing_field("uri"))?;
                let elements = elements.ok_or_else(|| serde::de::Error::missing_field("elements"))?;
                let macroname = macroname.ok_or_else(|| serde::de::Error::missing_field("macroname"))?;
                Ok(MathStructure { uri, elements, macroname })
            }
        }
        de.deserialize_struct(stringify!(MathStructure), &["uri","elements","macroname"], Visitor)
    }
}
     */