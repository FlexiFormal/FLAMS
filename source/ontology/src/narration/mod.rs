#![allow(clippy::large_enum_variant)]

pub mod checking;
pub mod documents;
pub mod exercises;
pub mod notations;
pub mod paragraphs;
pub mod sections;
pub mod variables;

use std::marker::PhantomData;

use documents::Document;
use exercises::{Exercise, UncheckedExercise};
use notations::Notation;
use paragraphs::{LogicalParagraph, UncheckedLogicalParagraph};
use sections::{Section, SectionLevel, UncheckedSection};
use variables::Variable;

use crate::{
    content::{
        declarations::{morphisms::Morphism, structures::MathStructure, symbols::Symbol},
        terms::Term,
        ContentReference, ModuleLike,
    },
    uris::{DocumentElementURI, DocumentURI, ModuleURI, NameStep, SymbolURI},
    DocumentRange,
};

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LazyDocRef<T> {
    start: usize,
    end: usize,
    in_doc: DocumentURI,
    phantom_data: PhantomData<T>,
}
impl<T> LazyDocRef<T> {
    #[inline]#[must_use]
    pub const fn new(start:usize,end:usize,in_doc:DocumentURI) -> Self {
        Self { start, end, in_doc, phantom_data: PhantomData }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UncheckedDocumentElement {
    SetSectionLevel(SectionLevel),
    Section(UncheckedSection),
    Module {
        range: DocumentRange,
        module: ModuleURI,
        children: Vec<UncheckedDocumentElement>,
    },
    Morphism {
        range: DocumentRange,
        morphism: SymbolURI,
        children: Vec<UncheckedDocumentElement>,
    },
    MathStructure {
        range: DocumentRange,
        structure: SymbolURI,
        children: Vec<UncheckedDocumentElement>,
    },
    DocumentReference {
        id: DocumentElementURI,
        range: DocumentRange,
        target: DocumentURI,
    },
    SymbolDeclaration(SymbolURI),
    Notation {
        symbol: SymbolURI,
        id: DocumentElementURI,
        notation: LazyDocRef<Notation>,
    },
    VariableNotation {
        variable: DocumentElementURI,
        id: DocumentElementURI,
        notation: LazyDocRef<Notation>,
    },
    Variable(Variable),
    Definiendum {
        range: DocumentRange,
        uri: SymbolURI,
    },
    SymbolReference {
        range: DocumentRange,
        uri: SymbolURI,
        notation: Option<NameStep>,
    },
    VariableReference {
        range: DocumentRange,
        uri: DocumentElementURI,
        notation: Option<NameStep>,
    },
    TopTerm {
        uri: DocumentElementURI,
        term: Term,
    },
    UseModule(ModuleURI),
    ImportModule(ModuleURI),
    Paragraph(UncheckedLogicalParagraph),
    Exercise(UncheckedExercise),
}

impl UncheckedDocumentElement {
    #[allow(clippy::missing_errors_doc)]
    pub fn set_children(&mut self, new_children: Vec<Self>) -> Result<(), ElementHasNoChildren> {
        use UncheckedDocumentElement::*;
        match self {
            Section(s) => s.children = new_children,
            Paragraph(p) => p.children = new_children,
            Exercise(e) => e.children = new_children,
            Module { children, .. }
            | Morphism { children, .. }
            | MathStructure { children, .. } => *children = new_children,
            _ => return Err(ElementHasNoChildren),
        }
        Ok(())
    }
}

pub struct ElementHasNoChildren;

#[derive(Debug)]
pub enum DocumentElement {
    SetSectionLevel(SectionLevel),
    Section(Section),
    Module {
        range: DocumentRange,
        module: Result<ModuleLike, ModuleURI>,
        children: Box<[DocumentElement]>,
    },
    Morphism {
        range: DocumentRange,
        morphism: Result<ContentReference<Morphism>, SymbolURI>,
        children: Box<[DocumentElement]>,
    },
    MathStructure {
        range: DocumentRange,
        structure: Result<ContentReference<MathStructure>, SymbolURI>,
        children: Box<[DocumentElement]>,
    },
    DocumentReference {
        id: DocumentElementURI,
        range: DocumentRange,
        target: Result<Document, DocumentURI>,
    },
    SymbolDeclaration(Result<ContentReference<Symbol>, SymbolURI>),
    Notation {
        symbol: SymbolURI,
        id: DocumentElementURI,
        notation: LazyDocRef<Notation>,
    },
    VariableNotation {
        variable: DocumentElementURI,
        id: DocumentElementURI,
        notation: LazyDocRef<Notation>,
    },
    Variable(Variable),
    Definiendum {
        range: DocumentRange,
        uri: SymbolURI,
    },
    SymbolReference {
        range: DocumentRange,
        uri: SymbolURI,
        notation: Option<NameStep>,
    },
    VariableReference {
        range: DocumentRange,
        uri: DocumentElementURI,
        notation: Option<NameStep>,
    },
    TopTerm {
        uri: DocumentElementURI,
        term: Term,
    },
    UseModule(Result<ModuleLike, ModuleURI>),
    ImportModule(Result<ModuleLike, ModuleURI>),
    Paragraph(LogicalParagraph),
    Exercise(Exercise),
}
