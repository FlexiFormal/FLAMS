use crate::uris::documents::{DocumentURI, NarrativeURI};
use std::fmt::{Display, Formatter};
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use arrayvec::ArrayVec;
use oxrdf::{BlankNode, NamedNodeRef, Quad};
use crate::content::{ArgSpec, AssocType, ContentElement, Module, Notation, Term, VarNameOrURI};
use crate::{SemanticElement, ulo};
use crate::uris::{ContentURI, Name, NarrDeclURI};
use crate::uris::modules::ModuleURI;
use crate::uris::symbols::SymbolURI;
use immt_utils::{prelude::*,sourcerefs::{ByteOffset, SourceRange}};
use crate::utils::{NestedDisplay, NestingFormatter};

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Title {
    pub range: SourceRange<ByteOffset>
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    pub uri: DocumentURI,
    pub title: Option<Box<str>>,
    pub elements: Vec<DocumentElement>,
}

pub struct ElemIter<'a> {
    iter: std::slice::Iter<'a,DocumentElement>,
    stack:Vec<std::slice::Iter<'a,DocumentElement>>
}
impl<'a> Iterator for ElemIter<'a> {
    type Item = &'a DocumentElement;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(e) = self.iter.next() {
                match e {
                    DocumentElement::Section(s) => {
                        self.stack.push(std::mem::replace(&mut self.iter, s.children.iter()));
                    },
                    DocumentElement::Module(m) => {
                        self.stack.push(std::mem::replace(&mut self.iter, m.children.iter()));
                    },
                    DocumentElement::MathStructure(m) => {
                        self.stack.push(std::mem::replace(&mut self.iter, m.children.iter()));
                    },
                    DocumentElement::Paragraph(p) => {
                        self.stack.push(std::mem::replace(&mut self.iter, p.children.iter()));
                    },
                    DocumentElement::Problem(p) => {
                        self.stack.push(std::mem::replace(&mut self.iter, p.children.iter()));
                    },
                    _ => ()
                }
                return Some(e)
            } else {
                if let Some(iter) = self.stack.pop() {
                    self.iter = iter;
                } else {
                    return None
                }
            }
        }
    }
}

impl Document {
    pub fn iter(&self) -> ElemIter {
        ElemIter {
            iter: self.elements.iter(),
            stack: Vec::new()
        }
    }
    pub fn get(&self,name:Name) -> Option<&DocumentElement> {
        let name = name.as_ref();
        let mut names = name.split('/');
        let mut curr = &self.elements;
        let mut ret = None;
        'top: while let Some(name) = names.next() {
            for e in curr {match e {
                DocumentElement::Section(s) if s.uri.name().as_ref().split('/').last() == Some(name)
                    => {
                    ret = Some(e);
                    curr = &s.children;
                    continue 'top
                },
                DocumentElement::Module(m)if m.uri.name().as_ref().split('/').last() == Some(name)
                    => {
                    ret = Some(e);
                    curr = &m.children;
                    continue 'top
                },
                DocumentElement::MathStructure(m)if m.uri.name().as_ref().split('/').last() == Some(name)
                    => {
                    ret = Some(e);
                    curr = &m.children;
                    continue 'top
                },
                DocumentElement::Paragraph(p) if p.uri.name().as_ref().split('/').last() == Some(name)
                    => {
                    ret = Some(e);
                    curr = &p.children;
                    continue 'top
                },
                DocumentElement::Problem(p) if p.uri.name().as_ref().split('/').last() == Some(name)
                    => {
                    ret = Some(e);
                    curr = &p.children;
                    continue 'top
                },
                _ => ret = None
            }}
        }
        ret
    }
    /*
    pub fn triples(&self) -> impl Iterator<Item=Quad> + Clone + '_ {
        TripleIterator {
            current: None,
            stack: Vec::new(),
            buf: Vec::new(),
            uri: self.uri.into(),
            content_uri: None,
            doc: self,
            doc_iri: self.uri.to_iri(),
            curr_iri: self.uri.to_iri()
        }
    }

     */
}
/*
#[derive(Clone)]
struct Stack<'a> {
    iter: std::slice::Iter<'a,DocumentElement>,
    iri: crate::ontology::rdf::terms::NamedNode,
    narr: NarrativeURI,
    content: Option<ModuleURI>
}

#[derive(Clone)]
struct TripleIterator<'a> {
    current: Option<std::slice::Iter<'a,DocumentElement>>,
    stack:Vec<Stack<'a>>,
    buf: Vec<Quad>,
    uri:NarrativeURI,
    content_uri: Option<ModuleURI>,
    doc: &'a Document,
    doc_iri: crate::ontology::rdf::terms::NamedNode,
    curr_iri: crate::ontology::rdf::terms::NamedNode
}
impl<'a> Iterator for TripleIterator<'a> {
    type Item = Quad;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::ontology::rdf::ontologies::*;
        if let Some(q) = self.buf.pop() { return Some(q) }
        match &mut self.current {
            None => {
                self.current = Some(self.doc.elements.iter());
                self.buf.push(
                    ulo!((self.doc_iri.clone()) (dc::LANGUAGE) = (self.doc.uri.language().to_string()) IN self.doc_iri.clone())
                );
                Some(ulo!( (self.doc_iri.clone()) : DOCUMENT IN self.doc_iri.clone()))
            }
            Some(it) => loop {
                macro_rules! next{
                    ($iter:expr,$iri:expr,$nuri:expr,$curi:expr) => {
                        let next = $iter;
                        let next = std::mem::replace(&mut self.current,Some(next)).unwrap();
                        let iri = std::mem::replace(&mut self.curr_iri,$iri);
                        let uri = std::mem::replace(&mut self.uri,$nuri);
                        let curi = std::mem::replace(&mut self.content_uri,Some($curi));
                        self.stack.push(Stack{iter:next,iri,narr:uri,content:curi});
                    };
                    ($iter:expr,$iri:expr,$nuri:expr) => {
                        let next = $iter;
                        let next = std::mem::replace(&mut self.current,Some(next)).unwrap();
                        let iri = std::mem::replace(&mut self.curr_iri,$iri);
                        let uri = std::mem::replace(&mut self.uri,$nuri);
                        self.stack.push(Stack{iter:next,iri,narr:uri,content:self.content_uri});
                    }
                }
                // TODO derecursify, maybe use arrayvec
                if let Some(next) = it.next() {
                    match next {
                        // TODO: terms maybe
                        DocumentElement::SetSectionLevel(..) | DocumentElement::VarNotation{..} | DocumentElement::VarDef{..} | DocumentElement::Definiendum {..} | DocumentElement::Symref {..} | DocumentElement::Varref{..}| DocumentElement::TopTerm{..} => (),
                        DocumentElement::Section(section) => {
                            let next_uri = section.uri;
                            let next_iri = next_uri.to_iri();
                            self.buf.push(
                                ulo!((next_iri.clone()) : SECTION IN self.doc_iri.clone())
                            );
                            self.buf.push(
                                ulo!((next_iri.clone()) (dc::LANGUAGE) = (self.doc.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            let ret = ulo!((self.curr_iri.clone()) CONTAINS (next_iri.clone()) IN self.doc_iri.clone());
                            if !section.children.is_empty() {
                                next!(section.children.iter(),next_iri,next_uri.into());
                            }
                            return Some(ret)
                        }
                        DocumentElement::Paragraph(p) => {
                            let next_uri = p.uri;
                            let next_iri = next_uri.to_iri();
                            let tp = p.kind.rdf_type().into_owned();
                            let ret = ulo!((next_iri.clone()) : >(tp) IN self.doc_iri.clone());
                            self.buf.push(
                                ulo!((self.curr_iri.clone()) CONTAINS (next_iri.clone()) IN self.doc_iri.clone())
                            );
                            self.buf.push(
                                ulo!((next_iri.clone()) (dc::LANGUAGE) = (self.doc.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            match p.kind {
                                StatementKind::Example => for u in &p.fors {
                                    self.buf.push(
                                        ulo!((next_iri.clone()) EXAMPLE_FOR (u.to_iri()) IN self.doc_iri.clone())
                                    );
                                },
                                StatementKind::Proof | StatementKind::SubProof => for u in &p.fors {
                                    self.buf.push(
                                        ulo!((next_iri.clone()) JUSTIFIES (u.to_iri()) IN self.doc_iri.clone())
                                    );
                                },
                                _ => for u in &p.fors {
                                    self.buf.push(
                                        ulo!((next_iri.clone()) DEFINES (u.to_iri()) IN self.doc_iri.clone())
                                    );
                                },
                            }
                            if !p.children.is_empty() {
                                next!(p.children.iter(),next_iri,next_uri.into());
                            }
                            return Some(ret)
                        }
                        DocumentElement::Module(m) => {
                            let next_uri = m.uri;
                            let curi = m.module_uri;
                            /*
                            let next_uri = self.uri;
                            let curi = match self.content_uri {
                                Some(c) => c / m.name,
                                _ => self.doc.uri * m.name
                            };
                             */
                            let next_iri = next_uri.to_iri();
                            self.buf.push(
                                ulo!((self.curr_iri.clone()) CONTAINS (curi.to_iri()) IN self.doc_iri.clone())
                            );
                            self.buf.push(
                                ulo!((curi.to_iri()) (dc::LANGUAGE) = (self.doc.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            let ret =
                                ulo!((curi.to_iri()) : THEORY IN self.doc_iri.clone());
                            if !m.children.is_empty() {
                                next!(m.children.iter(),next_iri,next_uri.into(),curi);
                            }
                            return Some(ret)
                        }
                        DocumentElement::MathStructure(m) => {
                            let next_uri = m.uri;
                            let curi = m.module_uri;
                            /*
                            let next_uri = self.uri;
                            let curi = match self.content_uri {
                                Some(c) => c / m.name,
                                _ => self.doc.uri * m.name
                            };

                             */
                            let next_iri = self.curr_iri.clone();
                            self.buf.push(
                                ulo!((self.curr_iri.clone()) CONTAINS (curi.to_iri()) IN self.doc_iri.clone())
                            );
                            self.buf.push(
                                ulo!((curi.to_iri()) (dc::LANGUAGE) = (self.doc.uri.language().to_string()) IN self.doc_iri.clone())
                            );
                            let ret =
                                ulo!((curi.to_iri()) : STRUCTURE IN self.doc_iri.clone());
                            if !m.children.is_empty() {
                                next!(m.children.iter(),next_iri,next_uri.into(),curi);
                            }
                            return Some(ret)
                        }
                        DocumentElement::InputRef(drf) => {
                            return Some(ulo!((self.curr_iri.clone()) !(dc::HAS_PART) (drf.target.to_iri()) IN self.doc_iri.clone() ))
                        }
                        DocumentElement::ConstantDecl(_uri) => (),
                        DocumentElement::UseModule(m) => {
                            return Some(ulo!((self.curr_iri.clone()) !(dc::REQUIRES) (m.to_iri()) IN self.doc_iri.clone()))
                        }

                        DocumentElement::Problem(p) => {
                            let next_uri = p.uri;
                            let next_iri = next_uri.to_iri();
                            let ret = ulo!((next_iri.clone()) : PROBLEM IN self.doc_iri.clone());
                            self.buf.push(
                                ulo!((self.curr_iri.clone()) CONTAINS (next_iri.clone()) IN self.doc_iri.clone())
                            );
                            /*self.buf.push(
                                ulo!((next_iri.clone()) (dc::LANGUAGE) = (p.language.to_string()) IN self.doc_iri.clone())
                            );*/
                            for (d,s) in &p.preconditions {
                                let n = BlankNode::default();
                                self.buf.push(
                                    ulo!((next_iri.clone()) PRECONDITION >>(crate::ontology::rdf::terms::Term::BlankNode(n.clone())) IN self.doc_iri.clone())
                                );
                                self.buf.push(
                                    ulo!(>>(crate::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()) IN self.doc_iri.clone())
                                );
                                self.buf.push(
                                    ulo!(>>(crate::ontology::rdf::terms::Subject::BlankNode(n)) POSYMBOL (s.to_iri()) IN self.doc_iri.clone())
                                );
                            }
                            for (d,s) in &p.objectives {
                                let n = BlankNode::default();
                                self.buf.push(
                                    ulo!((next_iri.clone()) OBJECTIVE >>(crate::ontology::rdf::terms::Term::BlankNode(n.clone())) IN self.doc_iri.clone())
                                );
                                self.buf.push(
                                    ulo!(>>(crate::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()) IN self.doc_iri.clone())
                                );
                                self.buf.push(
                                    ulo!(>>(crate::ontology::rdf::terms::Subject::BlankNode(n)) POSYMBOL (s.to_iri()) IN self.doc_iri.clone())
                                );
                            }
                            if !p.children.is_empty() {
                                next!(p.children.iter(),next_iri,next_uri.into());
                            }
                            return Some(ret)
                        }
                    }
                } else {
                    if let Some(Stack{iter,iri,narr,content}) = self.stack.pop() {
                        self.current = Some(iter);
                        self.curr_iri = iri;
                        self.uri = narr;
                        self.content_uri = content;
                        return self.next()
                    } else {
                        return None
                    }
                }
            }
        }
    }
}

 */

impl NestedDisplay for Document {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"{}",self.uri)?;
        f.nest(|f| {
            for e in &self.elements {
                f.next()?;
                e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for Document {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentElement {
    SetSectionLevel(SectionLevel),
    Section(Section),
    Module(DocumentModule),
    Morphism(DocumentMorphism),
    MathStructure(DocumentMathStructure),
    InputRef(DocumentReference),
    ConstantDecl(SymbolURI),
    VarNotation {
        name:VarNameOrURI,
        id:Name,
        notation:NarrativeRef<Notation>
    },
    VarDef {
        uri:NarrDeclURI,
        arity:ArgSpec,
        macroname:Option<String>,
        range:SourceRange<ByteOffset>,
        role:Option<Vec<String>>,
        tp:Option<Term>,
        df:Option<Term>,
        is_sequence:bool,
        assoctype : Option<AssocType>,
        reordering:Option<String>,
        bind:bool
    },
    Definiendum {
        uri:SymbolURI,
        range: SourceRange<ByteOffset>,
    },
    Symref {
        uri:ContentURI,
        range: SourceRange<ByteOffset>,
        notation:Option<Name>,
    },
    Varref {
        name:VarNameOrURI,
        range: SourceRange<ByteOffset>,
        notation:Option<Name>,
    },
    TopTerm(Term),
    UseModule(ModuleURI),
    ImportModule(ModuleURI),
    Paragraph(LogicalParagraph),
    Problem(Problem)
}

impl NestedDisplay for DocumentElement {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        use DocumentElement::*;
        match self {
            SetSectionLevel(l) => write!(f.inner(),"Set section level {l}"),
            Section(s) => s.fmt_nested(f),
            Module(m) => m.fmt_nested(f),
            MathStructure(m) => m.fmt_nested(f),
            Morphism(d) => d.fmt_nested(f),
            InputRef(r) => write!(f.inner(),"Input reference {}: {}",r.id,r.target),
            VarNotation { name, id, .. } => {
                write!(f.inner(),"Variable notation {} for {}",id,name)
            },
            ConstantDecl(uri) => write!(f.inner(),"Constant declaration {}",uri),
            VarDef { uri: name, .. } => {
                write!(f.inner(),"Variable {}",name)
            },
            Definiendum { uri, .. } => write!(f.inner(),"Definiendum {}",uri),
            Symref { uri, .. } => write!(f.inner(),"Symbol reference {}",uri),
            Varref { name, .. } => write!(f.inner(),"Variable reference {}",name),
            TopTerm(t) => write!(f.inner(),"Top term {t:?}"),
            UseModule(m) => write!(f.inner(),"Use module {}",m),
            ImportModule(m) => write!(f.inner(),"Import module {}",m),
            Paragraph(p) => p.fmt_nested(f),
            Problem(p) => write!(f.inner(),"Problem {}",p.uri)
        }
    }
}
impl Display for DocumentElement {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Section {
    pub range: SourceRange<ByteOffset>,
    pub uri: NarrDeclURI,
    pub level: SectionLevel,
    pub title: Option<Title>,
    pub children: Vec<DocumentElement>,
}
impl NestedDisplay for Section {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Section {}",self.uri)?;
        if let Some(Title{range,..}) = &self.title {
            write!(f.inner(),": {range:?}")?;
        }
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for Section {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentModule {
    pub range: SourceRange<ByteOffset>,
    pub module_uri: ModuleURI,
    pub uri: NarrDeclURI,
    pub children: Vec<DocumentElement>,
}

impl NestedDisplay for DocumentModule {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Module {}",self.uri.name())?;
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for DocumentModule {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentMorphism {
    pub range: SourceRange<ByteOffset>,
    pub domain: ModuleURI,
    pub total:bool,
    pub uri: NarrDeclURI,
    pub content_uri:ModuleURI,
    pub children: Vec<DocumentElement>,
}

impl NestedDisplay for DocumentMorphism {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Morphism {}: {}",self.uri.name(),self.domain)?;
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for DocumentMorphism {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}


#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentMathStructure {
    pub range: SourceRange<ByteOffset>,
    pub module_uri: ModuleURI,
    pub uri:NarrDeclURI,
    pub children: Vec<DocumentElement>,
}
impl NestedDisplay for DocumentMathStructure {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Structure {}",self.uri.name())?;
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for DocumentMathStructure {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentReference {
    pub range: SourceRange<ByteOffset>,
    pub id: Name,
    pub target: DocumentURI,
}

#[derive(Debug, Copy,Clone,PartialEq,Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum StatementKind {
    Definition,Assertion,Paragraph,Proof,SubProof,Example
}
impl Display for StatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StatementKind::*;
        write!(f,"{}",match self {
            Definition => "Definition",
            Assertion => "Assertion",
            Paragraph => "Paragraph",
            Proof => "Proof",
            SubProof => "Subproof",
            Example => "Example"
        })
    }

}

impl StatementKind {
    pub fn from_shtml(s:&str) -> Option<StatementKind> {
        Some(match s {
            "shtml:definition" => StatementKind::Definition,
            "shtml:assertion" => StatementKind::Assertion,
            "shtml:paragraph" => StatementKind::Paragraph,
            "shtml:proof" => StatementKind::Proof,
            "shtml:subproof" => StatementKind::SubProof,
            _ => return None
        })
    }
    pub fn is_definition_like(&self,styles:&Vec<String>) -> bool {
        match &self {
            StatementKind::Definition | StatementKind::Assertion => true,
            _ => styles.iter().any(|s| s == "symdoc" || s == "decl")
        }
    }
    pub fn rdf_type(&self) -> NamedNodeRef {
        use crate::ontology::rdf::ontologies::*;
        match self {
            StatementKind::Definition => ulo2::DEFINITION,
            StatementKind::Assertion => ulo2::PROPOSITION,
            StatementKind::Paragraph => ulo2::PARA,
            StatementKind::Proof => ulo2::PROOF,
            StatementKind::SubProof => ulo2::SUBPROOF,
            StatementKind::Example => ulo2::EXAMPLE
        }

    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicalParagraph {
    pub kind:StatementKind,
    pub uri: NarrDeclURI,
    pub inline:bool,
    pub title: Option<Title>,
    pub fors: Vec<ContentURI>,
    pub range: SourceRange<ByteOffset>,
    pub styles:Vec<String>,
    pub children: Vec<DocumentElement>,
    pub terms:VecMap<SymbolURI,Term>
}
impl LogicalParagraph {
    pub fn definition_like(&self) -> bool {
        self.kind.is_definition_like(&self.styles)
    }
}
impl NestedDisplay for LogicalParagraph {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"{} {}",self.kind,self.uri)?;
        if let Some(Title{range,..}) = &self.title {
            write!(f.inner(),": {range:?}")?;
        }
        if !self.fors.is_empty() {
            write!(f.inner()," for ")?;
            for (i,uri) in self.fors.iter().enumerate() {
                if i > 0 {write!(f.inner(),", ")?}
                write!(f.inner(),"{}",uri)?;
            }
        }
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for LogicalParagraph {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone,Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CognitiveDimension {
    Remember,
    Understand,
    Apply,
    Analyze,
    Evaluate,
    Create
}
impl CognitiveDimension {
    pub fn to_iri(&self) -> NamedNodeRef {
        match self {
            CognitiveDimension::Remember => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#remember"),
            CognitiveDimension::Understand => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#understand"),
            CognitiveDimension::Apply => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#apply"),
            CognitiveDimension::Analyze => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#analyze"),
            CognitiveDimension::Evaluate => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#evaluate"),
            CognitiveDimension::Create => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#create")
        }
    }
}
impl Display for CognitiveDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CognitiveDimension::*;
        write!(f,"{}",match self {
            Remember => "remember",
            Understand => "understand",
            Apply => "apply",
            Analyze => "analyze",
            Evaluate => "evaluate",
            Create => "create"
        })
    }
}
impl FromStr for CognitiveDimension {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "remember" => CognitiveDimension::Remember,
            "understand" => CognitiveDimension::Understand,
            "apply" => CognitiveDimension::Apply,
            "analyze"|"analyse" => CognitiveDimension::Analyze,
            "evaluate" => CognitiveDimension::Evaluate,
            "create" => CognitiveDimension::Create,
            _ => return Err(())
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Problem {
    pub sub:bool,
    pub uri:NarrDeclURI,
    pub autogradable:bool,
    pub points:Option<f32>,
    pub solutions:Vec<NarrativeRef<String>>,
    pub hints:Vec<NarrativeRef<String>>,
    pub notes:Vec<NarrativeRef<String>>,
    pub gnotes:Vec<NarrativeRef<String>>,
    pub title:Option<Title>,
    pub children:Vec<DocumentElement>,
    pub preconditions:Vec<(CognitiveDimension,SymbolURI)>,
    pub objectives:Vec<(CognitiveDimension,SymbolURI)>
}

impl NestedDisplay for Problem {
    fn fmt_nested<'a>(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Problem {}",self.uri)?;
        if let Some(Title{range,..}) = &self.title {
            write!(f.inner(),": {range:?}")?;
        }
        Ok(())
    }
}
impl Display for Problem {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
}
impl Display for SectionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SectionLevel::*;
        write!(f,"{}",match self {
            Part => "Part",
            Chapter => "Chapter",
            Section => "Section",
            Subsection => "Subsection",
            Subsubsection => "Subsubsection",
            Paragraph => "Paragraph",
            Subparagraph => "Subparagraph"
        })
    }
}
impl TryFrom<u8> for SectionLevel {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(SectionLevel::Part),
            1 => Ok(SectionLevel::Chapter),
            2 => Ok(SectionLevel::Section),
            3 => Ok(SectionLevel::Subsection),
            4 => Ok(SectionLevel::Subsubsection),
            5 => Ok(SectionLevel::Paragraph),
            6 => Ok(SectionLevel::Subparagraph),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Language {
    English,
    German,
    French,
    Romanian,
    Arabic,
    Bulgarian,
    Russian,
    Finnish,
    Turkish,
    Slovenian
}
impl Language {
    #[inline]
    fn check(s:impl AsRef<str>) -> Language {
        let s = s.as_ref();
        if s.ends_with(".en") { Language::English }
        else if s.ends_with(".de") {Language::German}
        else if s.ends_with(".fr") {Language::French}
        else if s.ends_with(".ro") {Language::Romanian}
        else if s.ends_with(".ar") {Language::Arabic}
        else if s.ends_with(".bg") {Language::Bulgarian}
        else if s.ends_with(".ru") {Language::Russian}
        else if s.ends_with(".fi") {Language::Finnish}
        else if s.ends_with(".tr") {Language::Turkish}
        else if s.ends_with(".sl") {Language::Slovenian}
        else {Language::English}
    }
    pub fn from_rel_path(s:impl AsRef<str>) -> Language {
        let mut s = s.as_ref();
        s = s.strip_suffix(".tex").unwrap_or(s);
        Self::check(s)
    }
    pub fn from_file(path:&Path) -> Language {
        if let Some(stem) = path.file_stem().map(|s| s.to_str()).flatten() {
            Self::check(stem)
        } else { Language::English }
    }
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",Into::<&'static str>::into(*self))
    }
}
impl Into<&'static str> for Language {
    fn into(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Romanian => "ro",
            Language::Arabic => "ar",
            Language::Bulgarian => "bg",
            Language::Russian => "ru",
            Language::Finnish => "fi",
            Language::Turkish => "tr",
            Language::Slovenian => "sl"
        }
    }
}
impl TryFrom<&str> for Language {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "en" => Language::English,
            "de" => Language::German,
            "fr" => Language::French,
            "ro" => Language::Romanian,
            "ar" => Language::Arabic,
            "bg" => Language::Bulgarian,
            "ru" => Language::Russian,
            "fi" => Language::Finnish,
            "tr" => Language::Turkish,
            "sl" => Language::Slovenian,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(Box<str>),
    Inline(Box<str>),
}

#[derive(Debug)]
//#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FullDocument {
    pub doc:Document,
    pub html:String,
    pub css: Vec<CSS>,
    pub body:SourceRange<ByteOffset>,
    pub refs:Vec<u8>,
    pub modules : Vec<Module>,
    pub triples: Vec<Quad>
}

#[derive(Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NarrativeRef<T> {
    pub start:usize,
    pub end:usize,
    pub in_doc:DocumentURI,
    phantom_data: PhantomData<T>
}
impl<T> Clone for NarrativeRef<T> {
    fn clone(&self) -> Self {
        Self {start:self.start,end:self.end,in_doc:self.in_doc,phantom_data:PhantomData}
    }
}
impl<T> Copy for NarrativeRef<T> {}
impl<T> NarrativeRef<T> {
    pub fn new(start:usize,end:usize,in_doc:DocumentURI) -> Self {
        Self {start,end,in_doc,phantom_data:PhantomData}
    }
}
/*
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
enum DocumentReaderState {
    Start,DocRead,RefsRead,CssRead,Finished
}
pub struct DocumentReader {
    file:std::fs::File,
    state:DocumentReaderState,
    refs_start:u32,css_start:u32,html_start:u32,body_start:u32,body_len:u32,
    refs:Option<Box<[u8]>>,
    css:Option<Box<str>>,
    html:Option<Box<str>>,
    document:Option<Document>
}

#[cfg(feature = "async")]
pub struct DocumentData {
    body_start:u32,body_len:u32,
    refs:Box<[u8]>,
    css:Box<[CSS]>,
    html:Box<str>,
    document:Document
}
*/

#[derive(Debug)]
struct DocDataI {
    path:Box<Path>,
    doc:Document,
    refs_offset:u32,
    css_offset:u32,
    html_offset:u32,
    body_offset:u32,
    body_len:u32
}
#[derive(Debug,Clone)]
pub struct DocData(triomphe::Arc<DocDataI>);

impl DocData {
    #[inline]
    pub fn iter(&self) -> ElemIter {
        self.0.doc.iter()
    }
    #[inline]
    pub fn strong_count(&self) -> usize {
        triomphe::Arc::strong_count(&self.0)
    }
    pub fn get(path:PathBuf) -> Option<DocData> {
        let mut file = std::fs::File::open(&path).ok()?;
        let mut buf = [0u8;20];
        file.read_exact(&mut buf).ok()?;
        let refs_start = u32::from_le_bytes([buf[0],buf[1],buf[2],buf[3]]);
        let css_start = u32::from_le_bytes([buf[4],buf[5],buf[6],buf[7]]);
        let html_start = u32::from_le_bytes([buf[8],buf[9],buf[10],buf[11]]);
        let body_start = u32::from_le_bytes([buf[12],buf[13],buf[14],buf[15]]);
        let body_len = u32::from_le_bytes([buf[16],buf[17],buf[18],buf[19]]);

        let mut buffer = vec![0; refs_start as usize];
        file.read_exact(&mut buffer).ok()?;
        let document = bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok()?.0;
        Some(Self(triomphe::Arc::new(DocDataI {
            doc:document,path:path.into(),refs_offset:refs_start + 20,css_offset:css_start + 20,
            html_offset: html_start + 20,body_offset:body_start+20,body_len
        })))
    }
    #[cfg(feature = "async")]
    pub async fn get_async(path:PathBuf) -> Option<DocData> {
        use tokio::io::AsyncReadExt;
        let mut file = tokio::fs::File::open(&path).await.ok()?;
        let mut buf = [0u8;20];
        file.read_exact(&mut buf).await.ok()?;
        let refs_start = u32::from_le_bytes([buf[0],buf[1],buf[2],buf[3]]);
        let css_start = u32::from_le_bytes([buf[4],buf[5],buf[6],buf[7]]);
        let html_start = u32::from_le_bytes([buf[8],buf[9],buf[10],buf[11]]);
        let body_start = u32::from_le_bytes([buf[12],buf[13],buf[14],buf[15]]);
        let body_len = u32::from_le_bytes([buf[16],buf[17],buf[18],buf[19]]);

        let refs_offset = 20 + refs_start;
        let css_offset = css_start + 20;

        let mut buffer = vec![0; refs_start as usize];
        file.read_exact(&mut buffer).await.ok()?;
        let document = bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok()?.0;
        Some(Self(triomphe::Arc::new(DocDataI {
            doc:document,path:path.into(),refs_offset,css_offset,
            html_offset: html_start + 20,body_offset:body_start+20,body_len
        })))
    }

    #[inline]
    fn read(&self,start:usize,end:Option<usize>) -> Option<(Vec<u8>,std::fs::File)> {
        use std::io::Seek;
        let mut file = std::fs::File::open(&self.0.path).ok()?;
        file.seek(SeekFrom::Start(start as u64)).ok()?;
        if let Some(end) = end {
            let mut buf = vec![0;end-start];
            file.read_exact(&mut buf).ok()?;
            Some((buf,file))
        } else {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok()?;
            Some((buf,file))
        }
    }
    #[cfg(feature = "async")]
    #[inline]
    async fn read_async(&self,start:usize,end:Option<usize>) -> Option<(Vec<u8>,tokio::fs::File)> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(&self.0.path).await.ok()?;
        file.seek(SeekFrom::Start(start as u64)).await.ok()?;
        if let Some(end) = end {
            let mut buf = vec![0;end-start];
            file.read_exact(&mut buf).await.ok()?;
            Some((buf,file))
        } else {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.ok()?;
            Some((buf,file))
        }
    }

    #[inline]
    fn read_bincode<T:for<'a> serde::Deserialize<'a>>(f:&mut std::fs::File,len:usize) -> Option<T> {
        let mut buf = vec![0;len];
        f.read_exact(&mut buf).ok()?;
        bincode::serde::decode_from_slice(&buf,bincode::config::standard()).ok().map(|(r,_)| r)
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn read_bincode_async<T:for<'a> serde::Deserialize<'a>>(f:&mut tokio::fs::File,len:usize) -> Option<T> {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0;len];
        f.read_exact(&mut buf).await.ok()?;
        bincode::serde::decode_from_slice(&buf,bincode::config::standard()).ok().map(|(r,_)| r)
    }

    #[inline]
    fn read_str(f:&mut std::fs::File,len:usize) -> Option<String> {
        let mut buf = vec![0;len];
        f.read_exact(&mut buf).ok()?;
        std::str::from_utf8(&buf).ok().map(|s| s.into())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn read_str_async(f:&mut tokio::fs::File,len:usize) -> Option<String> {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0;len];
        f.read_exact(&mut buf).await.ok()?;
        std::str::from_utf8(&buf).ok().map(|s| s.into())
    }

    #[inline]
    pub fn read_css_and(&self) -> Option<(Box<[CSS]>,std::fs::File)> {
        let (css,mut file) = self.read(self.0.css_offset as usize,Some(self.0.html_offset as usize))?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        Some((css,file))
    }

    #[cfg(feature = "async")]
    #[inline]
    pub async fn read_css_and_async(&self) -> Option<(Box<[CSS]>,tokio::fs::File)> {
        let (css,mut file) = self.read_async(self.0.css_offset as usize,Some(self.0.html_offset as usize)).await?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        Some((css,file))
    }

    pub fn read_css_and_body(&self) -> Option<(Box<[CSS]>,Box<str>)> {
        use std::io::Seek;
        let (css,mut file) = self.read_css_and()?;
        file.seek(SeekFrom::Start(self.0.body_offset as u64)).ok()?;
        let body = Self::read_str(&mut file,self.0.body_len as usize)?;
        Some((css,body.into()))
    }
    #[cfg(feature = "async")]
    pub async fn read_css_and_body_async(&self) -> Option<(Box<[CSS]>,Box<str>)> {
        use tokio::io::AsyncSeekExt;
        let (css,mut file) = self.read_css_and_async().await?;
        file.seek(SeekFrom::Start(self.0.body_offset as u64)).await.ok()?;
        let body = Self::read_str_async(&mut file,self.0.body_len as usize).await?;
        Some((css,body.into()))
    }
    pub fn read_snippet(&self, range:SourceRange<ByteOffset>) -> Option<(Box<[CSS]>,Box<str>)> {
        use std::io::Seek;
        let (css,mut file) = self.read_css_and()?;
        file.seek(SeekFrom::Start(self.0.html_offset as u64 + range.start.offset as u64)).ok()?;
        let snippet = Self::read_str(&mut file,range.end.offset - range.start.offset)?;
        Some((css,snippet.into()))
    }
    #[cfg(feature = "async")]
    pub async fn read_snippet_async(&self, range:SourceRange<ByteOffset>) -> Option<(Box<[CSS]>,Box<str>)> {
        use tokio::io::AsyncSeekExt;
        let (css,mut file) = self.read_css_and_async().await?;
        file.seek(SeekFrom::Start(self.0.html_offset as u64 + range.start.offset as u64)).await.ok()?;
        let snippet = Self::read_str_async(&mut file,range.end.offset - range.start.offset).await?;
        Some((css,snippet.into()))
    }


    pub fn read_resource<T:for<'a> serde::Deserialize<'a>>(&self, rf:NarrativeRef<T>) -> Option<T> {
        let (buffer,_) = self.read(self.0.refs_offset as usize + rf.start,Some(self.0.refs_offset as usize + rf.end))?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(r,_)| r)
    }
/*
    pub fn as_doc(s:triomphe::Arc<Self>) -> DocRef {
        let rf = &s.doc as *const Document;
        DocRef {data:s,elem:rf}
    }

 */
    pub fn into_elem<E,F:Fn(&DocumentElement) -> Option<&E>>(self,name:Name,get:F) -> Result<DocElemRef<E>,Self> {
        if let Some(e) = self.as_ref().get(name).and_then(get) {
            let elem = e as *const E;
            Ok(DocElemRef {data:self,elem})
        } else { Err(self) }
    }
}

impl AsRef<Document> for DocData {
    #[inline]
    fn as_ref(&self) -> &Document {
        &self.0.doc
    }
}


#[derive(Clone)]
pub struct DocElemRef<E> {
    data:DocData,
    elem: *const E
}
impl<E> PartialEq for DocElemRef<E> {
    fn eq(&self, other: &Self) -> bool {
        self.elem == other.elem
    }
}
impl<E> AsRef<E> for DocElemRef<E> {
    fn as_ref(&self) -> &E {
        // safe, because data holds an Arc to the DocData this comes from,
        // and no inner mutability is employed that might move the
        // element
        unsafe{self.elem.as_ref().unwrap()}
    }
}

impl<E> DocElemRef<E> {
    #[inline]
    pub fn doc(&self) -> &DocData {
        &self.data
    }
    #[inline]
    pub fn take(self) -> DocData {
        self.data
    }
    #[inline]
    pub fn get(&self) -> &E {
        // safe, because data holds an Arc to the Document this comes from,
        // and no inner mutability is employed that might move the
        // element
        unsafe{self.elem.as_ref().unwrap()}
    }
}


impl FullDocument {
    pub fn get_doc(file:&Path) -> Option<Document> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).ok()?;
        let refs = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(4i64*4)).ok()?;
        let mut buffer = vec![0; refs];
        file.read_exact(&mut buffer).ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(d,_)| d)
    }
    #[cfg(feature = "async")]
    pub async fn get_doc_async(file:&Path) -> Option<Document> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).await.ok()?;
        let refs = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(4i64*4)).await.ok()?;
        let mut buffer = vec![0; refs];
        file.read_exact(&mut buffer).await.ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(d,_)| d)
    }
    #[cfg(feature="serde")]
    pub fn get_resource<T:for<'a> serde::Deserialize<'a>>(file:&Path,rf:NarrativeRef<T>) -> Option<T> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        let mut refs = [0u8,0,0,0];
        file.read_exact(&mut refs).ok()?;
        let refs = u32::from_le_bytes(refs) as i64;
        file.seek(SeekFrom::Current(4i64*4 + refs + rf.start as i64)).ok()?;
        let mut buffer = vec![0; rf.end - rf.start];
        file.read_exact(&mut buffer).ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(p,_)| p)
    }

    #[cfg(all(feature = "async",feature="serde"))]
    pub async fn get_resource_async<T:for<'a> serde::Deserialize<'a>>(file:&Path,rf:NarrativeRef<T>) -> Option<T> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        let mut refs = [0u8,0,0,0];
        file.read_exact(&mut refs).await.ok()?;
        let refs = u32::from_le_bytes(refs) as i64;
        file.seek(SeekFrom::Current(4i64*4 + refs + rf.start as i64)).await.ok()?;
        let mut buffer = vec![0; rf.end - rf.start];
        file.read_exact(&mut buffer).await.ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(p,_)| p)
    }
    #[cfg(feature="serde")]
    pub fn get_css_and_body(file:&Path) -> Option<(Vec<CSS>,String)> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        file.seek(SeekFrom::Start(4)).ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).ok()?;
        let css = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let html = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let body_start = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let body_end = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(css as i64)).ok()?;
        let mut css = vec![0; html - css];
        file.read_exact(&mut css).ok()?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        file.seek(SeekFrom::Current(body_start as i64 - html as i64)).ok()?;
        let mut html = vec![0; body_end - body_start];
        file.read_exact(&mut html).ok()?;
        String::from_utf8(html).ok().map(|html| (css,html))
    }
    #[cfg(all(feature = "async",feature="serde"))]
    pub async fn get_css_and_body_async(file:&Path) -> Option<(Box<[CSS]>,Box<str>)> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        file.seek(SeekFrom::Start(4)).await.ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).await.ok()?;
        let css = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let html = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let body_start = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let body_len = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(css as i64)).await.ok()?;
        let mut css = vec![0; html - css];
        file.read_exact(&mut css).await.ok()?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        file.seek(SeekFrom::Current(body_start as i64 - html as i64)).await.ok()?;
        let mut html = vec![0; body_len];
        file.read_exact(&mut html).await.ok()?;
        String::from_utf8(html).ok().map(|html| (css,html.into()))
    }
    #[cfg(feature="serde")]
    pub fn write(self,p:&Path) {
        let mut file = std::fs::File::create(p).unwrap();
        if let Ok(doc) = bincode::serde::encode_to_vec(&self.doc,bincode::config::standard()) {
            if let Ok(css) = bincode::serde::encode_to_vec(&self.css,bincode::config::standard()) {
                let mut len = doc.len() as u32;
                // refs
                file.write_all(&len.to_le_bytes()).unwrap();
                len += self.refs.len() as u32;
                // css
                file.write_all(&len.to_le_bytes()).unwrap();
                len += css.len() as u32;
                // html
                file.write_all(&len.to_le_bytes()).unwrap();
                // body start;
                file.write_all(&(len + self.body.start.offset as u32).to_le_bytes()).unwrap();
                // body len;
                file.write_all(&((self.body.end.offset - self.body.start.offset) as u32).to_le_bytes()).unwrap();


                file.write_all(doc.as_slice()).unwrap();
                file.write_all(&self.refs).unwrap();
                file.write_all(css.as_slice()).unwrap();
                file.write_all(self.html.as_bytes()).unwrap();
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    pub fn check(&mut self,mut initial:impl FnMut(&mut SemanticElement),mut close:impl FnMut(&mut SemanticElement)) {
        /*let mut curr = &mut self.doc;
        let mut content = None;
        let mut _empty = Vec::new();
        let mut ls = &mut _empty;
        let mut stack = Vec::new();*/
    }
}