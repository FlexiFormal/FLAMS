use oxrdf::Quad;
use crate::narration::{DocumentElement, Language};
use crate::ulo;
use crate::uris::documents::{DocumentURI, NarrativeURI};
use crate::uris::modules::ModuleURI;

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module {
    pub uri:ModuleURI,
    pub meta:Option<ModuleURI>,
    pub signature:Option<Language>,
    pub elements: Vec<ContentElement>,
}
impl Module {
    pub fn triples(&self,in_doc:DocumentURI) -> impl Iterator<Item=Quad> + '_ + Clone {
        TripleIterator{
            current: None,
            stack: Vec::new(),
            buf: Vec::new(),
            curr_iri: self.uri.to_iri(),
            uri: self.uri,
            module: self,
            doc_iri: in_doc.to_iri()
        }
    }
}

#[derive(Clone)]
struct Stack<'a> {
    iter: std::slice::Iter<'a,ContentElement>,
    iri: crate::ontology::rdf::terms::NamedNode,
    uri:ModuleURI
}

#[derive(Clone)]
struct TripleIterator<'a> {
    current: Option<std::slice::Iter<'a,ContentElement>>,
    stack:Vec<Stack<'a>>,
    buf: Vec<Quad>,
    curr_iri: crate::ontology::rdf::terms::NamedNode,
    uri:ModuleURI,
    module:&'a Module,
    doc_iri:crate::ontology::rdf::terms::NamedNode
}
impl<'a> Iterator for TripleIterator<'a> {
    type Item = Quad;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::ontology::rdf::ontologies::*;
        if let Some(q) = self.buf.pop() { return Some(q) }
        match &mut self.current {
            None => {
                self.current = Some(self.module.elements.iter());
                self.buf.push(
                    ulo!((self.curr_iri.clone()) (dc::LANGUAGE) = (self.uri.language().to_string()) IN self.doc_iri.clone())
                );
                Some(ulo!( (self.curr_iri.clone()) : THEORY IN self.doc_iri.clone()))
            }
            Some(it) => loop {
                macro_rules! next{
                    ($iter:expr,$iri:expr,$nuri:expr) => {
                        let next = $iter;
                        let next = std::mem::replace(&mut self.current,Some(next)).unwrap();
                        let iri = std::mem::replace(&mut self.curr_iri,$iri);
                        let uri = std::mem::replace(&mut self.uri,$nuri);
                        self.stack.push(Stack{iter:next,iri,uri:uri});
                    }
                }
                // TODO derecursify, maybe use arrayvec
                if let Some(next) = it.next() {
                    match next {
                        ContentElement::NestedModule(m) => {
                            let next_uri = m.uri;
                            let next_iri = next_uri.to_iri();
                            self.buf.push(
                                ulo!((next_iri.clone()) (dc::LANGUAGE) = (m.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            let ret = ulo!( (next_iri.clone()) : THEORY IN self.doc_iri.clone());
                            if !m.elements.is_empty() {
                                next!(m.elements.iter(),next_iri,next_uri);
                            }
                            return Some(ret)
                        }
                        ContentElement::MathStructure(s) => {
                            // TODO Symbols for structures
                            let next_uri = s.uri;
                            let next_iri = next_uri.to_iri();
                            self.buf.push(
                                ulo!((next_iri.clone()) (dc::LANGUAGE) = (self.module.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            let ret = ulo!( (next_iri.clone()) : STRUCTURE IN self.doc_iri.clone());
                            if !s.elements.is_empty() {
                                next!(s.elements.iter(),next_iri,next_uri);
                            }
                            return Some(ret)
                        }
                        ContentElement::Import(uri) => {
                            return Some(ulo!(
                                (self.curr_iri.clone()) IMPORTS (uri.to_iri()) IN self.doc_iri.clone()
                            ))
                        }
                        ContentElement::Constant(c) => {
                            self.buf.push(
                                ulo!((c.uri.to_iri()) : DECLARATION IN self.doc_iri.clone())
                            );
                            return Some(ulo!(
                                (self.curr_iri.clone()) DECLARES (c.uri.to_iri()) IN self.doc_iri.clone()
                            ))
                        }
                        ContentElement::Notation(c) => {
                            let uri = self.uri & c.id;
                            self.buf.push(
                                ulo!((uri.to_iri()) : NOTATION IN self.doc_iri.clone())
                            );
                            self.buf.push(
                                ulo!((uri.to_iri()) NOTATION_FOR (c.uri.to_iri()) IN self.doc_iri.clone())
                            );
                            return Some(ulo!(
                                (self.curr_iri.clone()) DECLARES (uri.to_iri()) IN self.doc_iri.clone()
                            ))
                        }
                    }
                } else {
                    if let Some(Stack{iter,iri,uri}) = self.stack.pop() {
                        self.current = Some(iter);
                        self.curr_iri = iri;
                        self.uri = uri;
                        return self.next()
                    } else {
                        return None
                    }

                }
            }
        }

    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MathStructure {
    pub uri:ModuleURI,
    pub elements: Vec<ContentElement>,
    pub macroname:Option<String>
}


#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContentElement {
    NestedModule(Module),
    Import(ModuleURI),
    Constant(super::constants::Constant),
    Notation(super::constants::NotationRef),
    MathStructure(MathStructure)
}