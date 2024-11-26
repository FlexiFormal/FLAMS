use std::borrow::Cow;
use immt_ontology::{uris::NarrativeURI, Unchecked};
use immt_utils::{prelude::HMap, vecmap::VecSet};
use smallvec::SmallVec;
use shtml_extraction::prelude::{Attributes, SHTMLExtractor};
use leptos::{prelude::expect_context, web_sys::Element};

#[derive(Default)]
pub struct DOMExtractor {
    in_notation:bool,
    in_term:bool,
    id_counter:HMap<Cow<'static,str>,u32>
}

impl SHTMLExtractor for DOMExtractor {
    type Attr<'a> = NodeAttrs<'a>;

    #[inline(always)]
    fn add_error(&mut self, err: shtml_extraction::errors::SHTMLError) {
        tracing::error!("{err}");
    }


    fn new_id(&mut self,prefix:Cow<'static,str>) -> Box<str> {
        match self.id_counter.entry(prefix) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
                format!("{}_{}",e.key(),e.get())
            },
            std::collections::hash_map::Entry::Vacant(e) => {
                let ret = format!("{}_0",e.key());
                e.insert(0);
                ret
            }
        }.into_boxed_str()
    }

    #[inline]
    fn in_term(&self) -> bool { self.in_term }

    #[inline]
    fn set_in_term(&mut self, value: bool) { self.in_term = value }

    #[inline]
    fn in_notation(&self) -> bool { self.in_notation }

    #[cfg(feature="rdf")]
    const RDF: bool = false;

    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, _triples:[immt_ontology::rdf::Triple;N]) {}

    #[cfg(feature="rdf")]
    fn get_content_iri(&self) -> Option<immt_ontology::rdf::NamedNode> {
        todo!()
    }
    #[cfg(feature="rdf")]
    fn get_document_iri(&self) -> immt_ontology::rdf::NamedNode {
        todo!()
    }

    fn resolve_variable_name(&self,name:immt_ontology::uris::Name) -> immt_ontology::content::terms::Var {
        todo!()
    }

    fn add_arg(&mut self,pos:(u8,Option<u8>),tm:immt_ontology::content::terms::Term,mode:immt_ontology::content::terms::ArgMode) -> Result<(),()> {
        todo!()
    }
    fn add_content_element(&mut self,elem:immt_ontology::content::declarations::OpenDeclaration<Unchecked>) -> Result<(),immt_ontology::content::declarations::OpenDeclaration<Unchecked>> {
        todo!()
    }
    fn add_document_element(&mut self,elem:immt_ontology::narration::DocumentElement<Unchecked>) {
        todo!()
    }
    fn add_module(&mut self,module:immt_ontology::content::modules::OpenModule<Unchecked>) {
        todo!()
    }
    fn add_notation(&mut self,spec:shtml_extraction::prelude::NotationSpec) -> Result<(),shtml_extraction::prelude::NotationSpec> {
        todo!()
    }
    fn add_op_notation(&mut self,op:immt_ontology::narration::notations::OpNotation) -> Result<(),immt_ontology::narration::notations::OpNotation> {
        todo!()
    }
    fn add_definiendum(&mut self,uri:immt_ontology::uris::SymbolURI) {
        todo!()
    }
    fn add_resource<T:immt_ontology::Resourcable>(&mut self,t:&T) -> immt_ontology::narration::LazyDocRef<T> {
        todo!()
    }
    fn add_term(&mut self,symbol:Option<immt_ontology::uris::SymbolURI>,tm:immt_ontology::content::terms::Term) -> Result<(),immt_ontology::content::terms::Term> {
        todo!()
    }
    fn add_title(&mut self,title:immt_ontology::DocumentRange) -> Result<(),immt_ontology::DocumentRange> {
        todo!()
    }
    fn add_type(&mut self,tm:immt_ontology::content::terms::Term) -> Result<(),immt_ontology::content::terms::Term> {
        todo!()
    }
    fn close_args(&mut self) -> (Vec<immt_ontology::content::terms::Arg>,Option<immt_ontology::content::terms::Term>) {
        todo!()
    }
    fn close_complex_term(&mut self) -> Option<immt_ontology::content::terms::Term> {
        todo!()
    }
    fn close_content(&mut self) -> Option<(immt_ontology::uris::ModuleURI,Vec<immt_ontology::content::declarations::OpenDeclaration<Unchecked>>)> {
        todo!()
    }
    fn close_decl(&mut self) -> Option<(Option<immt_ontology::content::terms::Term>,Option<immt_ontology::content::terms::Term>)> {
        todo!()
    }
    fn close_exercise(&mut self) -> Option<shtml_extraction::prelude::ExerciseState> {
        todo!()
    }
    fn close_narrative(&mut self) -> Option<(immt_ontology::uris::NarrativeURI,Vec<immt_ontology::narration::DocumentElement<Unchecked>>)> {
        todo!()
    }
    fn close_notation(&mut self) -> Option<shtml_extraction::prelude::NotationState> {
        todo!()
    }
    fn close_paragraph(&mut self) -> Option<shtml_extraction::prelude::ParagraphState> {
       todo!() 
    }
    fn close_section(&mut self) -> Option<(immt_ontology::uris::DocumentElementURI,Option<immt_ontology::DocumentRange>,Vec<immt_ontology::narration::DocumentElement<Unchecked>>)> {
        todo!()
    }
    fn get_content_uri(&self) -> Option<&immt_ontology::uris::ModuleURI> {
        todo!()
    }
    fn get_narrative_uri(&self) -> immt_ontology::uris::NarrativeURI {
        expect_context::<NarrativeURI>()
    }
    fn open_args(&mut self) {}
    fn open_complex_term(&mut self) {}
    fn open_content(&mut self,_uri:immt_ontology::uris::ModuleURI) {}
    fn open_decl(&mut self) {}
    fn open_exercise(&mut self,_uri:immt_ontology::uris::DocumentElementURI) {}
    fn open_narrative(&mut self,_uri:Option<immt_ontology::uris::NarrativeURI>) {}
    fn open_notation(&mut self) {}
    fn open_paragraph(&mut self,_uri:immt_ontology::uris::DocumentElementURI,_fors:VecSet<immt_ontology::uris::SymbolURI>) {}
    fn open_section(&mut self,_uri:immt_ontology::uris::DocumentElementURI) {}
    fn set_document_title(&mut self,_title:Box<str>) {}
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
impl Attributes for NodeAttrs<'_> {
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
    fn take(&mut self,key:&str) -> Option<String> {
        let r = self.elem.get_attribute(key);
        let _ = self.elem.remove_attribute(key);
        r
    }
}