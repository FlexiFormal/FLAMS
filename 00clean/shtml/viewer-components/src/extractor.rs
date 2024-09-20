use std::borrow::Cow;
use smallvec::SmallVec;
use shtml_extraction::prelude::{Attributes, SHTMLExtractor};
use leptos::web_sys::Element;
#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct DOMExtractor {
    in_notation:bool,
    in_term:bool
}

impl SHTMLExtractor for DOMExtractor {
    type Attr<'a> = NodeAttrs<'a>;

    #[inline(always)]
    fn add_error(&mut self, err: shtml_extraction::errors::SHTMLError) {
        tracing::error!("{err}");
    }
    fn resolve_variable_name(&self, _name: &immt_ontology::uris::Name) -> immt_ontology::content::terms::Var {
        todo!()
    }

    #[inline]
    fn in_term(&self) -> bool { self.in_term }

    #[inline]
    fn set_in_term(&mut self, value: bool) { self.in_term = value }

    #[inline]
    fn in_notation(&self) -> bool { self.in_notation }

    #[inline]
    fn set_in_notation(&mut self, value: bool) { self.in_notation = value }

    #[cfg(feature="rdf")]
    const RDF: bool = false;

    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, _triples:[immt_ontology::rdf::Triple;N]) {}

    #[cfg(feature="rdf")]
    #[must_use]
    fn narrative_iri(&self) -> immt_ontology::rdf::NamedNode {
        todo!()
    }
}


pub struct NodeAttrs<'n> {
    elem:Cow<'n, Element>,
    keys:SmallVec<String,4>
}
impl<'n> NodeAttrs<'n> {
    pub(crate) fn new(elem:&'n Element) -> Self {
        Self{elem:Cow::Borrowed(elem),keys:Self::attr_names(elem)}
    }

    fn attr_names(e:&Element) -> SmallVec<String,4> {
        let mut ret = SmallVec::new();
        for k in e.get_attribute_names() {
            if let Some(s) = k.as_string() {ret.push(s)}
        }
        ret
    }
}
impl<'n> Attributes for NodeAttrs<'n> {
    type KeyIter<'a> = std::iter::Map<std::slice::Iter<'a, String>,for<'b> fn(&'b String) -> &'b str> where Self:'a;
    type Value<'a> = String where Self:'a;
    fn keys(&self) -> Self::KeyIter<'_> {
        self.keys.iter().map(AsRef::as_ref)
    }
    fn value(&self,key:&str) -> Option<Self::Value<'_>> {
        self.elem.get_attribute(key)
    }
    fn set(&mut self, key: &str, value: &str) {
        let _ = self.elem.set_attribute(key, value);
    }
}