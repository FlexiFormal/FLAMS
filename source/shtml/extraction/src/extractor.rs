#![allow(clippy::result_large_err)]

use std::borrow::Cow;
use immt_ontology::content::declarations::UncheckedDeclaration;
use immt_ontology::content::modules::UncheckedModule;
use immt_ontology::content::terms::{Arg, ArgMode, Term, Var};
use immt_ontology::languages::Language;
use immt_ontology::narration::exercises::CognitiveDimension;
use immt_ontology::narration::notations::{NotationComponent, OpNotation};
use immt_ontology::narration::sections::SectionLevel;
use immt_ontology::narration::variables::Variable;
use immt_ontology::narration::{LazyDocRef, UncheckedDocumentElement};
use immt_ontology::uris::{DocumentElementURI, DocumentURI, ModuleURI, Name, NarrativeURI, NarrativeURIRef, NarrativeURITrait, SymbolURI, URIRefTrait};
use immt_ontology::{DocumentRange, Resourcable};
use immt_utils::prelude::HMap;
use immt_utils::vecmap::VecMap;
use crate::errors::SHTMLError;
use crate::open::terms::TermOrList;
use crate::rules::SHTMLElements;
use crate::tags::SHTMLTag;

pub struct NotationSpec {
    pub attribute_index: u8,
    pub is_text:bool,
    pub components:Box<[NotationComponent]>
}

pub trait SHTMLNode {
    type Ancestors<'a>:Iterator<Item=Self> where Self:'a;
    fn ancestors(&self) -> Self::Ancestors<'_>;
    fn with_elements<R>(&mut self,f:impl FnMut(Option<&mut SHTMLElements>) -> R) -> R;
    fn delete(&self);
    fn range(&self) -> DocumentRange;
    fn string(&self) -> String;
    fn as_notation(&self) -> Option<NotationSpec>;
    fn as_op_notation(&self) -> Option<OpNotation>;
    fn as_term(&self) -> Term;
}

#[derive(Clone,Debug)]
pub struct ParagraphState {
    pub uri:DocumentElementURI,
    pub children:Vec<UncheckedDocumentElement>,
    pub fors:VecMap<SymbolURI,Option<Term>>,
    pub title:Option<DocumentRange>
}

#[derive(Clone,Debug)]
pub struct NotationState {
    pub attribute_index: u8,
    pub is_text: bool,
    pub components: Box<[NotationComponent]>,
    pub op: Option<OpNotation>,
}


#[derive(Clone,Debug)]
pub struct ExerciseState {
    pub uri: DocumentElementURI,
    pub solutions: Vec<LazyDocRef<Box<str>>>,
    pub hints: Vec<LazyDocRef<Box<str>>>,
    pub notes: Vec<LazyDocRef<Box<str>>>,
    pub gnotes: Vec<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: Vec<UncheckedDocumentElement>,
    pub preconditions: Vec<(CognitiveDimension, SymbolURI)>,
    pub objectives: Vec<(CognitiveDimension, SymbolURI)>,
}

#[cfg(feature="full")]
#[derive(Clone,Debug)]
pub enum Narrative {
    Container(NarrativeURI,Vec<UncheckedDocumentElement>),
    Paragraph(ParagraphState),
    Section{
        uri:DocumentElementURI,
        title:Option<DocumentRange>,
        children:Vec<UncheckedDocumentElement>
    },
    Exercise(ExerciseState),
    Notation(NotationState)
}

#[cfg(feature="full")]
#[derive(Clone,Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Content {
    Container(ModuleURI,Vec<UncheckedDeclaration>),
    SingleTerm(Option<Term>),
    Symdecl{
        tp:Option<Term>,
        df:Option<Term>
    },
    Args(Vec<Option<(TermOrList,ArgMode)>>,Option<Term>)
}

#[cfg(feature="full")]
#[derive(Clone,Debug)]
pub struct ExtractorState {
    pub(crate) in_term:bool,
    pub(crate) ids: HMap<Cow<'static,str>,usize>,
    pub(crate) narrative:Vec<Narrative>,
    pub(crate) content:Vec<Content>,
    pub(crate) modules:Vec<UncheckedModule>,
}
#[cfg(feature="full")]
impl ExtractorState {
    #[must_use]
    pub fn document_uri(&self) -> &DocumentURI {
        let Some(Narrative::Container(NarrativeURI::Document(ref ret),_)) = self.narrative.first().as_ref() else {
            unreachable!()
        };
        ret
    }
    #[must_use]
    pub fn new(document:DocumentURI) -> Self {
        Self {
            in_term:false,
            ids:HMap::default(),
            narrative:vec![Narrative::Container(document.into(),Vec::new())],
            content:Vec::new(),
            modules:Vec::new()
        }
    }
    /// #### Errors
    #[allow(clippy::result_unit_err)]
    pub fn take(mut self) -> Result<(DocumentURI,Vec<UncheckedDocumentElement>,Vec<UncheckedModule>),()> {
        if self.narrative.len() == 1 {
            let Some(Narrative::Container(document,elements)) = self.narrative.pop() else { unreachable!() };
            match document {
                NarrativeURI::Document(d) => Ok((d,elements,self.modules)),
                NarrativeURI::Element(_) => Err(())
            }
        } else {Err(())}
    }
    pub(crate) fn push_narr(&mut self,uri:Option<NarrativeURI>) {
        let uri = uri.unwrap_or_else(|| 
            self.narrative.iter().rev().find_map(|t| match t {
                Narrative::Container(uri,_) => Some(uri.clone()),
                _ => None
            }).unwrap_or_else(|| unreachable!())
        );
        self.narrative.push(Narrative::Container(uri,Vec::new()));
    }
}


#[cfg(feature="full")]
pub trait StatefulExtractor {
    type Attr<'a>:Attributes;
    #[cfg(feature="rdf")]
    const RDF: bool;
    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, triples:[immt_ontology::rdf::Triple;N]);

    fn state_mut(&mut self) -> &mut ExtractorState;
    fn state(&self) -> &ExtractorState;
    fn add_error(&mut self,err:SHTMLError);
    fn set_document_title(&mut self,title:Box<str>);

    #[inline]
    fn get_sym_uri_as_mod(&self, s:&str) -> Option<ModuleURI> { s.parse().ok() }
    #[inline]
    fn get_sym_uri(&self, s:&str) -> Option<SymbolURI> { s.parse().ok() }
    #[inline]
    fn get_mod_uri(&self, s:&str) -> Option<ModuleURI> { s.parse().ok() }
    #[inline]
    fn get_doc_uri(&self, s:&str) -> Option<DocumentURI> { s.parse().ok() }

    fn add_resource<T:Resourcable>(&mut self,t:&T) -> LazyDocRef<T>;

}
#[cfg(feature="full")]
impl<E:StatefulExtractor> SHTMLExtractor for E {
    type Attr<'a> = <Self as StatefulExtractor>::Attr<'a>;
    #[cfg(feature="rdf")]
    const RDF: bool = <Self as StatefulExtractor>::RDF;
    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, triples:[immt_ontology::rdf::Triple;N]) {
        <Self as StatefulExtractor>::add_triples(self,triples);
    }
    fn add_error(&mut self,err:SHTMLError) {
        <Self as StatefulExtractor>::add_error(self,err);
    }

    #[inline]
    fn get_sym_uri_as_mod(&self, s:&str) -> Option<ModuleURI> {
        <Self as StatefulExtractor>::get_sym_uri_as_mod(self, s)
    }
    #[inline]
    fn get_sym_uri(&self, s:&str) -> Option<SymbolURI> {
        <Self as StatefulExtractor>::get_sym_uri(self, s)
    }
    #[inline]
    fn get_mod_uri(&self, s:&str) -> Option<ModuleURI> {
        <Self as StatefulExtractor>::get_mod_uri(self, s)
    }
    #[inline]
    fn get_doc_uri(&self, s:&str) -> Option<DocumentURI> {
        <Self as StatefulExtractor>::get_doc_uri(self, s)
    }
    #[inline]
    fn set_document_title(&mut self,title:Box<str>) {
        <Self as StatefulExtractor>::set_document_title(self, title);
    }

    #[inline]
    fn add_resource<T:Resourcable>(&mut self,t:&T) -> LazyDocRef<T> {
        <Self as StatefulExtractor>::add_resource(self, t)
    }

    fn resolve_variable_name(&self,name:Name) -> Var {
        let names = name.steps();
        for n in self.state().narrative.iter().rev() {
            let ch = match n {
                Narrative::Container(_,c) => c,
                Narrative::Exercise(ExerciseState { children, .. }) |
                Narrative::Section { children, .. } |
                Narrative::Paragraph(ParagraphState { children, .. }) => children,
                Narrative::Notation(_) => continue
            };
            for c in ch.iter().rev() {
                match c {
                    UncheckedDocumentElement::Variable(Variable{uri,is_seq,..}) if uri.name().steps().ends_with(names) =>
                        return Var::Ref { declaration: uri.clone(), is_sequence: Some(*is_seq) },
                    _ => ()
                }
            }
        }
        Var::Name(name)
    }

    fn open_content(&mut self,uri:ModuleURI) {
        self.state_mut().content.push(Content::Container(uri,Vec::new()));
    }
    fn open_narrative(&mut self,uri:Option<NarrativeURI>) {
        self.state_mut().push_narr(uri);
    }
    fn open_complex_term(&mut self) {
        self.state_mut().content.push(Content::SingleTerm(None));
    }
    fn close_content(&mut self) -> Option<(ModuleURI,Vec<UncheckedDeclaration>)> {
        match self.state_mut().content.pop() {
            Some(Content::Container(uri,elements)) => return Some((uri,elements)),
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        None
    }
    fn close_narrative(&mut self) -> Option<(NarrativeURI,Vec<UncheckedDocumentElement>)> {
        let state = self.state_mut();
        let r =state.narrative.pop().unwrap_or_else(|| unreachable!());
        if state.narrative.is_empty() {
            state.narrative.push(r);
            return None
        }
        if let Narrative::Container(uri,elements ) = r {
            Some((uri,elements))
        } else {
            state.narrative.push(r);
            None
        }
    }
    fn close_complex_term(&mut self) -> Option<Term> {
        match self.state_mut().content.pop() {
            Some(Content::SingleTerm(t)) => return t,
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        None
    }

    fn open_section(&mut self,uri:DocumentElementURI) {
        self.state_mut().narrative.push(Narrative::Section { title: None, children: Vec::new(),uri });
    }
    fn close_section(&mut self) -> Option<(DocumentElementURI,Option<DocumentRange>,Vec<UncheckedDocumentElement>)> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Section { title, children,uri }) => return Some((uri,title,children)),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }
    
    fn open_paragraph(&mut self,uri:DocumentElementURI,fors:Vec<SymbolURI>) {
        let fors = fors.into_iter().map(|s| (s,None)).collect();
        self.state_mut().narrative.push(Narrative::Paragraph(ParagraphState {
            uri, children:Vec::new(), fors, title: None 
        }));
    }
    fn close_paragraph(&mut self) -> Option<ParagraphState> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Paragraph(state)) => return Some(state),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }
    fn open_exercise(&mut self,uri:DocumentElementURI) {
        self.state_mut().narrative.push(Narrative::Exercise(ExerciseState {
            uri, children:Vec::new(), title: None,solutions:Vec::new(),
            hints:Vec::new(),notes:Vec::new(),gnotes:Vec::new(),
            preconditions:Vec::new(),objectives:Vec::new(), 
        }));
    }
    fn close_exercise(&mut self) -> Option<ExerciseState> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Exercise(state)) => return Some(state),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }
    fn open_decl(&mut self) {
        self.state_mut().content.push(Content::Symdecl{df:None,tp:None});
    }
    fn close_decl(&mut self) -> Option<(Option<Term>,Option<Term>)> {
        match self.state_mut().content.pop() {
            Some(Content::Symdecl{df,tp}) => return Some((tp,df)),
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        None
    }
    fn open_notation(&mut self) {
        self.state_mut().narrative.push(Narrative::Notation(NotationState { 
            attribute_index:  0, is_text: false, components: Box::default(), op: None 
        }));
    }
    fn close_notation(&mut self) -> Option<NotationState> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Notation(state)) => return Some(state),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }
    fn open_args(&mut self) {
        self.state_mut().content.push(Content::Args(Vec::new(),None));
    }
    fn close_args(&mut self) -> (Vec<Arg>,Option<Term>) {
        match self.state_mut().content.pop() {
            Some(Content::Args(args,head)) => {
                let mut ret = Vec::new();
                let mut iter = args.into_iter();
                while let Some(Some((a,m))) = iter.next() {
                    ret.push(match a.close(m) {
                        Ok(a) => a,
                        Err(a) => {
                            self.add_error(SHTMLError::IncompleteArgs);
                            a
                        }
                    });
                }
                for e in iter {
                    if e.is_some() {
                        self.add_error(SHTMLError::IncompleteArgs);
                    }
                }
                return (ret,head);
            },
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        (Vec::new(),None)
    }

    fn get_narrative_uri(&self) -> NarrativeURI {
        self.state().narrative.iter().rev().find_map(|t| match t {
            Narrative::Container(uri,_) => Some(uri.as_narrative().owned()),
            Narrative::Paragraph(ParagraphState { uri, .. }) | 
            Narrative::Exercise(ExerciseState { uri,.. }) |
            Narrative::Section{uri,..} => Some(uri.as_narrative().owned()),
            Narrative::Notation(_) => None
        }).unwrap_or_else(|| unreachable!())
    }

    fn get_content_uri(&self) -> Option<&ModuleURI> {
        self.state().content.iter().rev().find_map(|t| match t {
            Content::Container(uri,_) => Some(uri),
            _ => None
        })
    }

    fn add_module(&mut self,module:UncheckedModule) {
        self.state_mut().modules.push(module);
    }

    fn new_id(&mut self,prefix:Cow<'static,str>) -> Box<str> {
        match self.state_mut().ids.entry(prefix) {
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
    
    fn in_notation(&self) -> bool { self.state().narrative.iter().rev().any(|s| matches!(s,Narrative::Notation(_))) }
    fn in_term(&self) -> bool { self.state().in_term }
    fn set_in_term(&mut self,b:bool) { self.state_mut().in_term = b }

    fn add_document_element(&mut self,elem:UncheckedDocumentElement) {
        for narr in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Container(_,c) = narr {
                c.push(elem); return
            }
            if let Narrative::Paragraph(ParagraphState {  children,.. }) | 
                Narrative::Exercise(ExerciseState {  children,.. }) |
                Narrative::Section { children ,..} = narr {
                children.push(elem); return
            }
        }
        unreachable!()
    }
    fn add_title(&mut self,ttl:DocumentRange) -> Result<(),DocumentRange> {
        for narr in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Paragraph(ParagraphState {  title,.. }) | 
                Narrative::Exercise(ExerciseState {  title,.. }) |
                Narrative::Section { title ,..} = narr {
                    *title = Some(ttl); return Ok(())
            }
        }
        Err(ttl)
    }

    /// ### Errors
    fn add_content_element(&mut self,elem:UncheckedDeclaration) -> Result<(),UncheckedDeclaration> {
        for cont in self.state_mut().content.iter_mut().rev() {
            if let Content::Container(_,c) = cont {
                c.push(elem); return Ok(())
            }
        }
        Err(elem)
    }
    fn add_notation(&mut self,NotationSpec{components,attribute_index,is_text}:NotationSpec) -> Result<(),NotationSpec> {
        if let Some(Narrative::Notation(NotationState{components:comps,attribute_index:idx,is_text:text,..})) = self.state_mut().narrative.last_mut() {
            *comps = components;
            *idx = attribute_index;
            *text = is_text;
            Ok(())
        } else {
            Err(NotationSpec{attribute_index,is_text,components})
        }
    }
    fn add_op_notation(&mut self,op:OpNotation) -> Result<(),OpNotation> {
        if let Some(Narrative::Notation(NotationState{op:ops,..})) = self.state_mut().narrative.last_mut() {
            *ops = Some(op);
            Ok(())
        } else {
            Err(op)
        }
    }
    fn add_type(&mut self,tm:Term) -> Result<(),Term> {
        match self.state_mut().content.last_mut() {
            Some(Content::Symdecl { tp, .. }) =>
                *tp = Some(tm),
            _ => return Err(tm)
        }
        Ok(())
    }
    /// #### Errors
    fn add_term(&mut self,symbol:Option<SymbolURI>,tm:Term) -> Result<(),Term> {
        if symbol.is_none() { 
            match self.state_mut().content.last_mut() {
                Some(Content::Symdecl { df, .. }) => {
                    *df = Some(tm);
                    return Ok(())
                }
                Some(Content::Args(_,o) | Content::SingleTerm(o)) => {
                    *o = Some(tm);
                    return Ok(())
                }
                _ => ()
            }
         }
         for e in self.state_mut().narrative.iter_mut().rev() {
             if let Narrative::Paragraph(ParagraphState {fors,..}) = e {
                 if let Some(symbol) = symbol {
                    fors.insert(symbol,Some(tm));
                    return Ok(())
                 }
                 if fors.0.len() == 1 {
                    fors.0.last_mut().unwrap_or_else(|| unreachable!()).1 = Some(tm);
                    return Ok(())
                 }
             }
         }
         Err(tm)
    }

    fn add_arg(&mut self,(idx,maybe_ls):(u8,Option<u8>),tm:Term,mode:ArgMode) -> Result<(),()> {
        if let Some(Content::Args(v,_)) = self.state_mut().content.last_mut() {
            if v.len() <= idx as usize {
                v.resize(idx as usize + 1,None);
            }
            let tl = v.get_mut((idx - 1) as usize).unwrap_or_else(|| unreachable!());
            if let Some(idx) = maybe_ls { 
                if tl.is_none() { *tl = Some((TermOrList::List(vec![]),mode)); }
                if let Some((TermOrList::List(ls),_)) = tl {
                    if ls.len() <= idx as usize {
                        ls.resize(idx as usize + 1,None);
                    }
                    let tl = ls.get_mut((idx - 1) as usize).unwrap_or_else(|| unreachable!());
                    *tl = Some(tm);
                } else {
                    return Err(())
                }
             } else {
                *tl = Some((TermOrList::Term(tm),mode));
            }
            Ok(())
        } else {Err(())}
    }

}

pub trait SHTMLExtractor {
    type Attr<'a>:Attributes;

    #[cfg(feature="rdf")]
    const RDF: bool;

    #[cfg(feature="rdf")]
    fn add_triples<const N:usize>(&mut self, triples:[immt_ontology::rdf::Triple;N]);

    fn get_narrative_uri(&self) -> NarrativeURI;
    fn get_content_uri(&self) -> Option<&ModuleURI>;

    #[cfg(feature="rdf")]
    fn get_document_iri(&self) -> immt_ontology::rdf::NamedNode {
        use immt_ontology::uris::URIOrRefTrait;
        self.get_narrative_uri().to_iri()
    }

    #[cfg(feature="rdf")]
    fn get_content_iri(&self) -> Option<immt_ontology::rdf::NamedNode> {
        use immt_ontology::uris::URIOrRefTrait;
        self.get_content_uri().map(URIOrRefTrait::to_iri)
    }

    fn resolve_variable_name(&self,name:Name) -> Var;
    fn add_error(&mut self,err:SHTMLError);
    fn add_module(&mut self,module:UncheckedModule);
    fn new_id(&mut self,prefix:Cow<'static,str>) -> Box<str>;
    fn in_notation(&self) -> bool;
    fn in_term(&self) -> bool;
    fn set_in_term(&mut self,b:bool);
    fn add_document_element(&mut self,elem:UncheckedDocumentElement);
    /// ### Errors
    fn add_content_element(&mut self,elem:UncheckedDeclaration) -> Result<(),UncheckedDeclaration>;

    fn open_content(&mut self,uri:ModuleURI);
    fn open_narrative(&mut self,uri:Option<NarrativeURI>);
    fn open_complex_term(&mut self);
    fn close_content(&mut self) -> Option<(ModuleURI,Vec<UncheckedDeclaration>)>;
    fn close_narrative(&mut self) -> Option<(NarrativeURI,Vec<UncheckedDocumentElement>)>;
    fn close_complex_term(&mut self) -> Option<Term>;
    fn open_section(&mut self,uri:DocumentElementURI);
    fn close_section(&mut self) -> Option<(DocumentElementURI,Option<DocumentRange>,Vec<UncheckedDocumentElement>)>;
    fn open_paragraph(&mut self,uri:DocumentElementURI,fors:Vec<SymbolURI>);
    fn close_paragraph(&mut self) -> Option<ParagraphState>;
    fn open_exercise(&mut self,uri:DocumentElementURI);
    fn close_exercise(&mut self) -> Option<ExerciseState>;
    fn set_document_title(&mut self,title:Box<str>);
    /// #### Errors
    fn add_title(&mut self,title:DocumentRange) -> Result<(),DocumentRange>;
    fn open_decl(&mut self);
    fn close_decl(&mut self) -> Option<(Option<Term>,Option<Term>)>;
    fn open_notation(&mut self);
    fn close_notation(&mut self) -> Option<NotationState>;
    fn open_args(&mut self);
    fn close_args(&mut self) -> (Vec<Arg>,Option<Term>);
    /// #### Errors
    #[allow(clippy::result_unit_err)]
    fn add_arg(&mut self,pos:(u8,Option<u8>),tm:Term,mode:ArgMode) -> Result<(),()>;


    fn add_resource<T:Resourcable>(&mut self,t:&T) -> LazyDocRef<T>;
    /// #### Errors
    fn add_notation(&mut self,spec:NotationSpec) -> Result<(),NotationSpec>;
    /// #### Errors
    fn add_op_notation(&mut self,op:OpNotation) -> Result<(),OpNotation>;
    /// #### Errors
    fn add_type(&mut self,tm:Term) -> Result<(),Term>;
    /// #### Errors
    fn add_term(&mut self,symbol:Option<SymbolURI>,tm:Term) -> Result<(),Term>;

    #[inline]
    fn get_sym_uri_as_mod(&self, s:&str) -> Option<ModuleURI> { s.parse().ok() }
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
    fn set(&mut self,key:&str,value:&str);
    fn take(&mut self,key:&str) -> Option<String>;

    #[inline]
    fn get(&self,tag:SHTMLTag) -> Option<Self::Value<'_>> {
        self.value(tag.attr_name())
    }
    #[inline]
    fn remove(&mut self,tag:SHTMLTag) -> Option<String> {
        self.take(tag.attr_name())
    }

    /// #### Errors
    fn get_typed<E,T>(&self,key:SHTMLTag,f:impl FnOnce(&str) -> Result<T,E>) -> Result<T,SHTMLError> {
        let Some(v) = self.get(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        f(v.as_ref()).map_err(|_| SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into())))
    }

    /// #### Errors
    fn take_typed<E,T>(&mut self,key:SHTMLTag,f:impl FnOnce(&str) -> Result<T,E>) -> Result<T,SHTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        f(v.as_ref()).map_err(|_| SHTMLError::InvalidKeyFor(key.as_str(), Some(v)))
    }

    /// #### Errors
    fn get_section_level(&self,key:SHTMLTag) -> Result<SectionLevel,SHTMLError> {
        use std::str::FromStr;
        let Some(v) = self.get(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        let Ok(u) = u8::from_str(v.as_ref()) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into())))
        };
        SectionLevel::try_from(u).map_err(|()| SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into())))
    }

    /// #### Errors
    fn take_section_level(&mut self,key:SHTMLTag) -> Result<SectionLevel,SHTMLError> {
        use std::str::FromStr;
        let Some(v) = self.remove(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        let Ok(u) = u8::from_str(v.as_ref()) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v)))
        };
        SectionLevel::try_from(u).map_err(|()| SHTMLError::InvalidKeyFor(key.as_str(), Some(v)))
    }

    /// #### Errors
    fn get_language(&self,key:SHTMLTag) -> Result<Language,SHTMLError> {
        use std::str::FromStr;
        self.get_typed(key,Language::from_str)
    }

    /// #### Errors
    fn take_language(&mut self,key:SHTMLTag) -> Result<Language,SHTMLError> {
        use std::str::FromStr;
        self.take_typed(key,Language::from_str)
    }

    /// #### Errors
    fn get_module_uri<E:SHTMLExtractor>(&self,key:SHTMLTag,extractor:&mut E) -> Result<ModuleURI,SHTMLError> {
        let Some(v) = self.get(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_mod_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into()))),
            Ok
        )
    }

    /// #### Errors
    fn take_module_uri<E:SHTMLExtractor>(&mut self,key:SHTMLTag,extractor:&mut E) -> Result<ModuleURI,SHTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_mod_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v))),
            Ok
        )
    }

    /// #### Errors
    fn get_symbol_uri<E:SHTMLExtractor>(&self,key:SHTMLTag,extractor:&mut E) -> Result<SymbolURI,SHTMLError> {
        let Some(v) = self.get(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_sym_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into()))),
            Ok
        )
    }

    /// #### Errors
    fn take_symbol_uri<E:SHTMLExtractor>(&mut self,key:SHTMLTag,extractor:&mut E) -> Result<SymbolURI,SHTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_sym_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v))),
            Ok
        )
    }

    /// #### Errors
    fn get_document_uri<E:SHTMLExtractor>(&self,key:SHTMLTag,extractor:&mut E) -> Result<DocumentURI,SHTMLError> {
        let Some(v) = self.get(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_doc_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v.into()))),
            Ok
        )
    }

    /// #### Errors
    fn take_document_uri<E:SHTMLExtractor>(&mut self,key:SHTMLTag,extractor:&mut E) -> Result<DocumentURI,SHTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(SHTMLError::InvalidKeyFor(key.as_str(), None))
        };
        extractor.get_doc_uri(v.as_ref()).map_or_else(
            || Err(SHTMLError::InvalidKeyFor(key.as_str(), Some(v))),
            Ok
        )
    }

    fn get_id<E:SHTMLExtractor>(&self,extractor:&mut E,prefix:Cow<'static,str>) -> Box<str> {
        self.get(SHTMLTag::Id).map_or_else(
            || extractor.new_id(prefix),
            |v| Into::<String>::into(v).into_boxed_str()
        )
    }

    fn get_bool(&self,key:SHTMLTag) -> bool {
        self.get(key)
            .and_then(|s| s.as_ref().parse().ok())
            .unwrap_or_default()
    }

    fn take_bool(&mut self,key:SHTMLTag) -> bool {
        self.remove(key)
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}
