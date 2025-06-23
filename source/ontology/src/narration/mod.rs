#![allow(clippy::large_enum_variant)]

pub mod checking;
pub mod documents;
pub mod notations;
pub mod paragraphs;
pub mod problems;
pub mod sections;
pub mod variables;

use std::marker::PhantomData;

use documents::Document;
use flams_utils::prelude::InnerArc;
use notations::Notation;
use paragraphs::LogicalParagraph;
use problems::{CognitiveDimension, Problem};
use sections::{Section, SectionLevel};
use variables::Variable;

use crate::{
    content::{
        declarations::{
            morphisms::Morphism,
            structures::{Extension, MathStructure},
            symbols::Symbol,
        },
        terms::Term,
    },
    uris::{DocumentElementURI, DocumentURI, Name, NameStep, SymbolURI},
    Checked, CheckingState, DocumentRange, Unchecked,
};

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum LOKind {
    Definition,
    Example,
    Problem(CognitiveDimension),
    SubProblem(CognitiveDimension),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LazyDocRef<T> {
    pub start: usize,
    pub end: usize,
    pub in_doc: DocumentURI,
    phantom_data: PhantomData<T>,
}
impl<T> LazyDocRef<T> {
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize, in_doc: DocumentURI) -> Self {
        Self {
            start,
            end,
            in_doc,
            phantom_data: PhantomData,
        }
    }
}

pub trait NarrationTrait {
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self>
    where
        Self: Sized;
    fn children(&self) -> &[DocumentElement<Checked>];

    fn find<T: NarrationTrait>(&self, steps: &[NameStep]) -> Option<&T> {
        enum I<'a> {
            One(std::slice::Iter<'a, DocumentElement<Checked>>),
            Mul(
                std::slice::Iter<'a, DocumentElement<Checked>>,
                Vec<std::slice::Iter<'a, DocumentElement<Checked>>>,
            ),
        }
        impl<'a> I<'a> {
            fn push(&mut self, es: &'a [DocumentElement<Checked>]) {
                match self {
                    Self::One(_) => {
                        let new = Self::Mul(es.iter(), Vec::with_capacity(1));
                        let Self::One(s) = std::mem::replace(self, new) else {
                            unreachable!()
                        };
                        let Self::Mul(_, v) = self else {
                            unreachable!()
                        };
                        v.push(s);
                    }
                    Self::Mul(f, r) => {
                        let of = std::mem::replace(f, es.iter());
                        r.push(of);
                    }
                }
            }
        }
        impl<'a> Iterator for I<'a> {
            type Item = &'a DocumentElement<Checked>;
            #[allow(clippy::option_if_let_else)]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::One(s) => s.next(),
                    Self::Mul(f, r) => loop {
                        if let Some(n) = f.next() {
                            return Some(n);
                        }
                        let Some(mut n) = r.pop() else { unreachable!() };
                        if r.is_empty() {
                            let r = n.next();
                            *self = Self::One(n);
                            return r;
                        }
                        *f = n;
                    },
                }
            }
        }
        let mut steps = steps;
        let mut curr = I::One(self.children().iter());
        'outer: while !steps.is_empty() {
            let step = &steps[0];
            steps = &steps[1..];
            while let Some(c) = curr.next() {
                match c {
                    DocumentElement::Section(Section { uri, children, .. })
                    | DocumentElement::Paragraph(LogicalParagraph { uri, children, .. })
                    | DocumentElement::Problem(Problem { uri, children, .. })
                        if uri.name().last_name() == step =>
                    {
                        if steps.is_empty() {
                            return T::from_element(c);
                        }
                        curr = I::One(children.iter());
                        continue 'outer;
                    }
                    DocumentElement::Slide { uri, .. }
                        if uri.name().last_name() == step && steps.is_empty() =>
                    {
                        return T::from_element(c);
                    }
                    DocumentElement::Module { children, .. }
                    | DocumentElement::Morphism { children, .. }
                    | DocumentElement::MathStructure { children, .. }
                    | DocumentElement::Slide { children, .. }
                    | DocumentElement::Extension { children, .. } => curr.push(children),
                    DocumentElement::Notation { id: uri, .. }
                    | DocumentElement::VariableNotation { id: uri, .. }
                    | DocumentElement::Variable(Variable { uri, .. })
                    | DocumentElement::TopTerm { uri, .. }
                        if uri.name().last_name() == step =>
                    {
                        if steps.is_empty() {
                            return T::from_element(c);
                        }
                        return None;
                    }
                    DocumentElement::Section(_)
                    | DocumentElement::Paragraph(_)
                    | DocumentElement::Problem(_)
                    | DocumentElement::SetSectionLevel(_)
                    | DocumentElement::SymbolDeclaration(_)
                    | DocumentElement::UseModule(_)
                    | DocumentElement::ImportModule(_)
                    | DocumentElement::SkipSection(_)
                    | DocumentElement::Variable(_)
                    | DocumentElement::Definiendum { .. }
                    | DocumentElement::SymbolReference { .. }
                    | DocumentElement::VariableReference { .. }
                    | DocumentElement::DocumentReference { .. }
                    | DocumentElement::Notation { .. }
                    | DocumentElement::VariableNotation { .. }
                    | DocumentElement::TopTerm { .. } => (),
                }
            }
        }
        None
    }
}

pub struct NarrativeReference<T: NarrationTrait>(InnerArc<Document, T>);

impl<T: NarrationTrait> NarrativeReference<T> {
    #[must_use]
    pub fn new(d: &Document, name: &Name) -> Option<Self> {
        unsafe {
            InnerArc::new(d, |d| &d.0, |d| d.find(name.steps()).ok_or(()))
                .ok()
                .map(Self)
        }
    }
    #[must_use]
    #[inline]
    pub const fn top(&self) -> &Document {
        self.0.outer()
    }
}

impl<T: NarrationTrait> AsRef<T> for NarrativeReference<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: NarrationTrait + std::fmt::Debug> std::fmt::Debug for NarrativeReference<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_ref(), f)
    }
}

//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentElement<State: CheckingState> {
    SetSectionLevel(SectionLevel),
    Section(Section<State>),
    Slide {
        range: DocumentRange,
        uri: DocumentElementURI,
        children: State::Seq<DocumentElement<State>>,
    },
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
    Extension {
        range: DocumentRange,
        extension: State::Decl<Extension<Checked>>,
        target: State::Decl<MathStructure<Checked>>,
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
    Problem(Problem<State>),
    SkipSection(State::Seq<DocumentElement<State>>),
}

crate::serde_impl! {
    enum DocumentElement {
        {0 = SetSectionLevel(l)}
        {1 = Section(s)}
        {2 = Module{range,module,children}}
        {3 = Morphism{range,morphism,children}}
        {4 = MathStructure{range,structure,children}}
        {5 = Extension{range,extension,target,children}}
        {6 = DocumentReference { id, range, target }}
        {7 = SymbolDeclaration(s)}
        {8 = Notation{symbol,id,notation}}
        {9 = VariableNotation { variable, id, notation }}
        {10 = Variable(v)}
        {11 = Definiendum { range, uri }}
        {12 = SymbolReference { range, uri, notation }}
        {13 = VariableReference { range, uri, notation }}
        {14 = TopTerm { uri, term }}
        {15 = UseModule(m)}
        {16 = ImportModule(m)}
        {17 = Paragraph(p)}
        {18 = Problem(e)}
        {19 = SkipSection(children)}
        {20 = Slide{ uri, range, children}}
    }
}

impl NarrationTrait for DocumentElement<Checked> {
    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self>
    where
        Self: Sized,
    {
        Some(e)
    }
    fn children(&self) -> &[DocumentElement<Checked>] {
        match self {
            Self::Section(s) => s.children(),
            Self::Paragraph(p) => p.children(),
            Self::Problem(e) => e.children(),
            Self::Module { children, .. }
            | Self::Morphism { children, .. }
            | Self::MathStructure { children, .. }
            | Self::Extension { children, .. }
            | Self::SkipSection(children)
            | Self::Slide { children, .. } => children,
            _ => &[],
        }
    }
}

impl<State: CheckingState> std::fmt::Debug for DocumentElement<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetSectionLevel(level) => f.debug_tuple("SetSectionLevel").field(level).finish(),
            Self::Section(section) => f.debug_tuple("Section").field(section).finish(),
            Self::Module {
                range,
                module,
                children,
            } => f
                .debug_struct("Module")
                .field("range", range)
                .field("module", module)
                .field("children", children)
                .finish(),
            Self::Morphism {
                range,
                morphism,
                children,
            } => f
                .debug_struct("Morphism")
                .field("range", range)
                .field("morphism", morphism)
                .field("children", children)
                .finish(),
            Self::MathStructure {
                range,
                structure,
                children,
            } => f
                .debug_struct("MathStructure")
                .field("range", range)
                .field("structure", structure)
                .field("children", children)
                .finish(),
            Self::Extension {
                range,
                extension,
                target,
                children,
            } => f
                .debug_struct("Extension")
                .field("range", range)
                .field("extension", extension)
                .field("target", target)
                .field("children", children)
                .finish(),
            Self::DocumentReference { id, range, target } => f
                .debug_struct("DocumentReference")
                .field("id", id)
                .field("range", range)
                .field("target", target)
                .finish(),
            Self::SymbolDeclaration(symbol) => {
                f.debug_tuple("SymbolDeclaration").field(symbol).finish()
            }
            Self::Notation {
                symbol,
                id,
                notation,
            } => f
                .debug_struct("Notation")
                .field("symbol", symbol)
                .field("id", id)
                .field("notation", notation)
                .finish(),
            Self::VariableNotation {
                variable,
                id,
                notation,
            } => f
                .debug_struct("VariableNotation")
                .field("variable", variable)
                .field("id", id)
                .field("notation", notation)
                .finish(),
            Self::Variable(variable) => f.debug_tuple("Variable").field(variable).finish(),
            Self::Definiendum { range, uri } => f
                .debug_struct("Definiendum")
                .field("range", range)
                .field("uri", uri)
                .finish(),
            Self::SymbolReference {
                range,
                uri,
                notation,
            } => f
                .debug_struct("SymbolReference")
                .field("range", range)
                .field("uri", uri)
                .field("notation", notation)
                .finish(),
            Self::VariableReference {
                range,
                uri,
                notation,
            } => f
                .debug_struct("VariableReference")
                .field("range", range)
                .field("uri", uri)
                .field("notation", notation)
                .finish(),
            Self::TopTerm { uri, term } => f
                .debug_struct("TopTerm")
                .field("uri", uri)
                .field("term", term)
                .finish(),
            Self::UseModule(module) => f.debug_tuple("UseModule").field(module).finish(),
            Self::ImportModule(module) => f.debug_tuple("ImportModule").field(module).finish(),
            Self::Paragraph(paragraph) => f.debug_tuple("Paragraph").field(paragraph).finish(),
            Self::Problem(problem) => f.debug_tuple("Problem").field(problem).finish(),
            Self::SkipSection(children) => f.debug_tuple("SkipSection").field(children).finish(),
            Self::Slide {
                uri,
                range,
                children,
            } => f
                .debug_struct("Slide")
                .field("uri", uri)
                .field("range", range)
                .field("children", children)
                .finish(),
        }
    }
}

impl DocumentElement<Unchecked> {
    #[allow(clippy::missing_errors_doc)]
    pub fn set_children(&mut self, new_children: Vec<Self>) -> Result<(), ElementHasNoChildren> {
        match self {
            Self::Section(s) => s.children = new_children,
            Self::Paragraph(p) => p.children = new_children,
            Self::Problem(e) => e.children = new_children,
            Self::Module { children, .. }
            | Self::Morphism { children, .. }
            | Self::MathStructure { children, .. }
            | Self::SkipSection(children)
            | Self::Slide { children, .. } => *children = new_children,
            _ => return Err(ElementHasNoChildren),
        }
        Ok(())
    }
}

pub struct ElementHasNoChildren;
