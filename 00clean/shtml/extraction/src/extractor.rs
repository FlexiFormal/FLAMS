use std::borrow::Cow;
use immt_ontology::content::terms::Var;
use immt_ontology::uris::{DocumentURI, ModuleURI, Name, SymbolURI};
use crate::errors::SHTMLError;
use crate::tags::SHTMLTag;

#[allow(clippy::module_name_repetitions)]
pub trait SHTMLExtractor {
    type Attr<'a>:Attributes;

    #[cfg(feature="rdf")]
    const RDF: bool;

    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, triples:[immt_ontology::rdf::Triple;N]);
    #[cfg(feature="rdf")]
    #[must_use]
    fn narrative_iri(&self) -> immt_ontology::rdf::NamedNode;
    fn resolve_variable_name(&self,name:&Name) -> Var;
    fn add_error(&mut self,err:SHTMLError);
    fn in_notation(&self) -> bool;
    fn set_in_notation(&mut self,value:bool);
    fn in_term(&self) -> bool;
    fn set_in_term(&mut self,value:bool);


    #[inline]
    fn get_sym_uri(&self, s:&str) -> Option<SymbolURI> { s.parse().ok() }
    #[inline]
    fn get_mod_uri(&self, s:&str) -> Option<ModuleURI> { s.parse().ok() }
    #[inline]
    fn get_doc_uri(&self, s:&str) -> Option<DocumentURI> { s.parse().ok() }
}

pub trait Attributes {
    type KeyIter<'a>:Iterator<Item=&'a str> where Self:'a;
    type Value<'a>:AsRef<str> + Into<Cow<'a,str>>+Into<String> where Self:'a;
    fn keys(&self) -> Self::KeyIter<'_>;
    fn value(&self,key:&str) -> Option<Self::Value<'_>>;

    fn get(&self,tag:SHTMLTag) -> Option<Self::Value<'_>> {
        self.value(tag.attr_name())
    }
    fn set(&mut self,key:&str,value:&str);
}
