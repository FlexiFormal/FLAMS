use std::borrow::Cow;
use flams_ontology::{uris::NarrativeURI, Unchecked};
use flams_utils::{prelude::HMap, vecmap::VecSet};
use smallvec::SmallVec;
use ftml_extraction::prelude::{Attributes, GnoteState, FTMLExtractor};
use leptos::{prelude::{expect_context, UpdateValue}, web_sys::Element};

#[derive(Default)]
pub struct DOMExtractor {
    in_notation:bool,
    in_term:bool,
    id_counter:HMap<Cow<'static,str>,u32>
}

impl FTMLExtractor for DOMExtractor {
    type Attr<'a> = NodeAttrs<'a>;

    #[inline(always)]
    fn add_error(&mut self, err: ftml_extraction::errors::FTMLError) {
        tracing::error!("{err}");
    }


    fn new_id(&mut self,prefix:Cow<'static,str>) -> Box<str> {
        match self.id_counter.entry(prefix) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
                format!("{}_{}",e.key(),e.get())
            },
            std::collections::hash_map::Entry::Vacant(e) => {
                let ret = e.key().to_string();
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
    fn add_triples<const N:usize>(&mut self, _triples:[flams_ontology::rdf::Triple;N]) {}

    #[cfg(feature="rdf")]
    fn get_content_iri(&self) -> Option<flams_ontology::rdf::NamedNode> {
        todo!()
    }
    #[cfg(feature="rdf")]
    fn get_document_iri(&self) -> flams_ontology::rdf::NamedNode {
        todo!()
    }

    fn add_objective(&mut self,_uri:flams_ontology::uris::SymbolURI,_dim:flams_ontology::narration::exercises::CognitiveDimension) {
        todo!()
    }
    fn add_precondition(&mut self,_uri:flams_ontology::uris::SymbolURI,_dim:flams_ontology::narration::exercises::CognitiveDimension) {
        todo!()
    }

    fn resolve_variable_name(&self,_name:flams_ontology::uris::Name) -> flams_ontology::content::terms::Var {
        todo!()
    }

    fn add_arg(&mut self,_pos:(u8,Option<u8>),_tm:flams_ontology::content::terms::Term,_mode:flams_ontology::content::terms::ArgMode) -> Result<(),()> {
        todo!()
    }
    fn add_content_element(&mut self,_elem:flams_ontology::content::declarations::OpenDeclaration<Unchecked>) -> Result<(),flams_ontology::content::declarations::OpenDeclaration<Unchecked>> {
        todo!()
    }
    fn add_document_element(&mut self,_elem:flams_ontology::narration::DocumentElement<Unchecked>) {
        todo!()
    }
    fn add_module(&mut self,_module:flams_ontology::content::modules::OpenModule<Unchecked>) {
        todo!()
    }
    fn add_notation(&mut self,_spec:ftml_extraction::prelude::NotationSpec) -> Result<(),ftml_extraction::prelude::NotationSpec> {
        todo!()
    }
    fn add_op_notation(&mut self,_op:flams_ontology::narration::notations::OpNotation) -> Result<(),flams_ontology::narration::notations::OpNotation> {
        todo!()
    }
    fn add_resource<T:flams_ontology::Resourcable>(&mut self,_t:&T) -> flams_ontology::narration::LazyDocRef<T> {
        todo!()
    }
    fn add_term(&mut self,_symbol:Option<flams_ontology::uris::SymbolURI>,_tm:flams_ontology::content::terms::Term) -> Result<(),flams_ontology::content::terms::Term> {
        todo!()
    }
    fn add_title(&mut self,_title:flams_ontology::DocumentRange) -> Result<(),flams_ontology::DocumentRange> {
        todo!()
    }
    fn add_type(&mut self,_tm:flams_ontology::content::terms::Term) -> Result<(),flams_ontology::content::terms::Term> {
        todo!()
    }
    fn close_args(&mut self) -> (Vec<flams_ontology::content::terms::Arg>,Option<flams_ontology::content::terms::Term>) {
        todo!()
    }
    fn close_complex_term(&mut self) -> Option<flams_ontology::content::terms::Term> {
        todo!()
    }
    fn close_content(&mut self) -> Option<(flams_ontology::uris::ModuleURI,Vec<flams_ontology::content::declarations::OpenDeclaration<Unchecked>>)> {
        todo!()
    }
    fn close_decl(&mut self) -> Option<(Option<flams_ontology::content::terms::Term>,Option<flams_ontology::content::terms::Term>)> {
        todo!()
    }
    fn close_exercise(&mut self) -> Option<ftml_extraction::prelude::ExerciseState> {
        todo!()
    }
    fn close_narrative(&mut self) -> Option<(flams_ontology::uris::NarrativeURI,Vec<flams_ontology::narration::DocumentElement<Unchecked>>)> {
        todo!()
    }
    fn close_notation(&mut self) -> Option<ftml_extraction::prelude::NotationState> {
        todo!()
    }
    fn close_paragraph(&mut self) -> Option<ftml_extraction::prelude::ParagraphState> {
       todo!() 
    }
    fn close_section(&mut self) -> Option<(flams_ontology::uris::DocumentElementURI,Option<flams_ontology::DocumentRange>,Vec<flams_ontology::narration::DocumentElement<Unchecked>>)> {
        todo!()
    }
    fn get_content_uri(&self) -> Option<&flams_ontology::uris::ModuleURI> {
        todo!()
    }
    fn get_narrative_uri(&self) -> flams_ontology::uris::NarrativeURI {
        expect_context::<NarrativeURI>()
    }
    fn with_exercise<R>(&mut self,then:impl FnOnce(&mut ftml_extraction::prelude::ExerciseState) -> R) -> Option<R> {
        todo!()
    }
    fn close_gnote(&mut self) -> Option<GnoteState> {
        todo!()   
    }
    fn close_choice_block(&mut self) -> Option<ftml_extraction::prelude::ChoiceBlockState> {
        todo!()
    }
    fn close_fillinsol(&mut self) -> Option<ftml_extraction::prelude::FillinsolState> {
        todo!()
    }

    fn add_definiendum(&mut self,_uri:flams_ontology::uris::SymbolURI) {}
    fn push_fillinsol_case(&mut self,case:flams_ontology::narration::exercises::FillInSolOption) {}
    fn open_fillinsol(&mut self,width:Option<f32>) {}
    fn push_answer_class(&mut self,id:Box<str>,kind:flams_ontology::narration::exercises::AnswerKind) {}
    fn push_problem_choice(&mut self,correct:bool) {}
    fn open_gnote(&mut self) {}
    fn open_choice_block(&mut self,multiple:bool,styles:Box<[Box<str>]>) {}
    fn open_args(&mut self) {}
    fn open_complex_term(&mut self) {}
    fn open_content(&mut self,_uri:flams_ontology::uris::ModuleURI) {}
    fn open_decl(&mut self) {}
    fn open_exercise(&mut self,_uri:flams_ontology::uris::DocumentElementURI) {}
    fn open_narrative(&mut self,_uri:Option<flams_ontology::uris::NarrativeURI>) {}
    fn open_notation(&mut self) {}
    fn open_paragraph(&mut self,_uri:flams_ontology::uris::DocumentElementURI,_fors:VecSet<flams_ontology::uris::SymbolURI>) {}
    fn open_section(&mut self,_uri:flams_ontology::uris::DocumentElementURI) {}
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