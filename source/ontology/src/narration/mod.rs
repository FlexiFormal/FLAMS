#![allow(clippy::large_enum_variant)]

pub mod checking;
pub mod documents;
pub mod exercises;
pub mod notations;
pub mod paragraphs;
pub mod sections;
pub mod variables;

use std::marker::PhantomData;

use exercises::Exercise;
use notations::Notation;
use paragraphs::LogicalParagraph;
use sections::{Section, SectionLevel};
use variables::Variable;

use crate::{
    content::{
        declarations::{morphisms::Morphism, structures::MathStructure, symbols::Symbol},
        terms::Term,
    }, uris::{DocumentElementURI, DocumentURI, NameStep, SymbolURI}, Checked, CheckingState, DocumentRange, Unchecked
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


pub struct ElementHasNoChildren;

//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentElement<State:CheckingState> {
    SetSectionLevel(SectionLevel),
    Section(Section<State>),
    Module {
        range: DocumentRange,
        module: State::ModuleLike,
        children: State::Seq<DocumentElement<State>>,
    },
    Morphism {
        range: DocumentRange,
        morphism: State::Decl<Morphism<Checked>>,
        children: State::Seq<DocumentElement<State>>,
    },
    MathStructure {
        range: DocumentRange,
        structure: State::Decl<MathStructure<Checked>>,
        children: State::Seq<DocumentElement<State>>,
    },
    DocumentReference {
        id: DocumentElementURI,
        range: DocumentRange,
        target: State::Doc,
    },
    SymbolDeclaration(State::Decl<Symbol>),
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
    UseModule(State::ModuleLike),
    ImportModule(State::ModuleLike),
    Paragraph(LogicalParagraph<State>),
    Exercise(Exercise<State>),
}


crate::serde_impl! {
    enum DocumentElement {
        {0 = SetSectionLevel(l)}
        {1 = Section(s)}
        {2 = Module{range,module,children}}
        {3 = Morphism{range,morphism,children}}
        {4 = MathStructure{range,structure,children}}
        {5 = DocumentReference { id, range, target }}
        {6 = SymbolDeclaration(s)}
        {7 = Notation{symbol,id,notation}}
        {8 = VariableNotation { variable, id, notation }}
        {9 = Variable(v)}
        {10 = Definiendum { range, uri }}
        {11 = SymbolReference { range, uri, notation }}
        {12 = VariableReference { range, uri, notation }}
        {13 = TopTerm { uri, term }}
        {14 = UseModule(m)}
        {15 = ImportModule(m)}
        {16 = Paragraph(p)}
        {17 = Exercise(e)}
    }
}

impl<State:CheckingState> std::fmt::Debug for DocumentElement<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetSectionLevel(level) => f.debug_tuple("SetSectionLevel").field(level).finish(),
            Self::Section(section) => f.debug_tuple("Section").field(section).finish(),
            Self::Module { range, module, children } => f.debug_struct("Module").field("range", range).field("module", module).field("children", children).finish(),
            Self::Morphism { range, morphism, children } => f.debug_struct("Morphism").field("range", range).field("morphism", morphism).field("children", children).finish(),
            Self::MathStructure { range, structure, children } => f.debug_struct("MathStructure").field("range", range).field("structure", structure).field("children", children).finish(),
            Self::DocumentReference { id, range, target } => f.debug_struct("DocumentReference").field("id", id).field("range", range).field("target", target).finish(),
            Self::SymbolDeclaration(symbol) => f.debug_tuple("SymbolDeclaration").field(symbol).finish(),
            Self::Notation { symbol, id, notation } => f.debug_struct("Notation").field("symbol", symbol).field("id", id).field("notation", notation).finish(), 
            Self::VariableNotation { variable, id, notation } => f.debug_struct("VariableNotation").field("variable", variable).field("id", id).field("notation", notation).finish(),
            Self::Variable(variable) => f.debug_tuple("Variable").field(variable).finish(), 
            Self::Definiendum { range, uri } => f.debug_struct("Definiendum").field("range", range).field("uri", uri).finish(),
            Self::SymbolReference { range, uri, notation } => f.debug_struct("SymbolReference").field("range", range).field("uri", uri).field("notation", notation).finish(),
            Self::VariableReference { range, uri, notation } => f.debug_struct("VariableReference").field("range", range).field("uri", uri).field("notation", notation).finish(),
            Self::TopTerm { uri, term } => f.debug_struct("TopTerm").field("uri", uri).field("term", term).finish(),
            Self::UseModule(module) => f.debug_tuple("UseModule").field(module).finish(),
            Self::ImportModule(module) => f.debug_tuple("ImportModule").field(module).finish(),
            Self::Paragraph(paragraph) => f.debug_tuple("Paragraph").field(paragraph).finish(),
            Self::Exercise(exercise) => f.debug_tuple("Exercise").field(exercise).finish(),
        }
    }
}

impl DocumentElement<Unchecked> {
    #[allow(clippy::missing_errors_doc)]
    pub fn set_children(&mut self, new_children: Vec<Self>) -> Result<(), ElementHasNoChildren> {
        match self {
            Self::Section(s) => s.children = new_children,
            Self::Paragraph(p) => p.children = new_children,
            Self::Exercise(e) => e.children = new_children,
            Self::Module { children, .. }
            | Self::Morphism { children, .. }
            | Self::MathStructure { children, .. } => *children = new_children,
            _ => return Err(ElementHasNoChildren),
        }
        Ok(())
    }
}