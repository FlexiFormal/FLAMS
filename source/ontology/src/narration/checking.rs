use immt_utils::vecmap::VecMap;

use crate::{
    content::terms::Term,
    narration::{
        exercises::UncheckedExercise, paragraphs::UncheckedLogicalParagraph,
        sections::UncheckedSection,
    },
    uris::{DocumentElementURI, DocumentURI, ModuleURI, SymbolURI},
    DocumentRange, LocalBackend,
};

use super::{
    exercises::{CognitiveDimension, Exercise},
    paragraphs::{LogicalParagraph, ParagraphKind},
    sections::{Section, SectionLevel},
    DocumentElement, LazyDocRef, UncheckedDocumentElement,
};

pub trait DocumentChecker: LocalBackend {
    fn open(&mut self, elem: &mut UncheckedDocumentElement);
    fn close(&mut self, elem: &mut DocumentElement);
}

enum Elem {
    Section {
        range: DocumentRange,
        uri: DocumentElementURI,
        level: SectionLevel,
        title: Option<DocumentRange>,
    },
    Module {
        range: DocumentRange,
        module: ModuleURI,
    },
    Morphism {
        range: DocumentRange,
        morphism: SymbolURI,
    },
    MathStructure {
        range: DocumentRange,
        structure: SymbolURI,
    },
    Paragraph {
        kind: ParagraphKind,
        uri: DocumentElementURI,
        inline: bool,
        title: Option<DocumentRange>,
        range: DocumentRange,
        styles: Box<[Box<str>]>,
        fors: VecMap<SymbolURI, Option<Term>>,
    },
    Exercise {
        sub_problem: bool,
        uri: DocumentElementURI,
        autogradable: bool,
        range: DocumentRange,
        points: Option<f32>,
        solutions: Vec<LazyDocRef<Box<str>>>,
        hints: Vec<LazyDocRef<Box<str>>>,
        notes: Vec<LazyDocRef<Box<str>>>,
        gnotes: Vec<LazyDocRef<Box<str>>>,
        title: Option<DocumentRange>,
        styles: Box<[Box<str>]>,
        preconditions: Vec<(CognitiveDimension, SymbolURI)>,
        objectives: Vec<(CognitiveDimension, SymbolURI)>,
    },
}
impl Elem {
    fn close(self, v: Vec<DocumentElement>, checker: &mut impl DocumentChecker) -> DocumentElement {
        match self {
            Self::Section {
                range,
                uri,
                level,
                title,
            } => DocumentElement::Section(Section {
                range,
                uri,
                level,
                title,
                children: v.into_boxed_slice(),
            }),
            Self::Module { range, module } => DocumentElement::Module {
                range,
                module: checker.get_module(&module).map_or_else(|| Err(module), Ok),
                children: v.into_boxed_slice(),
            },
            Self::Morphism { range, morphism } => DocumentElement::Morphism {
                range,
                morphism: checker
                    .get_declaration(&morphism)
                    .map_or_else(|| Err(morphism), Ok),
                children: v.into_boxed_slice(),
            },
            Self::MathStructure { range, structure } => DocumentElement::MathStructure {
                range,
                structure: checker
                    .get_declaration(&structure)
                    .map_or_else(|| Err(structure), Ok),
                children: v.into_boxed_slice(),
            },
            Self::Paragraph {
                kind,
                uri,
                inline,
                title,
                fors,
                range,
                styles,
            } => DocumentElement::Paragraph(LogicalParagraph {
                kind,
                uri,
                inline,
                title,
                range,
                styles,
                fors,
                children: v.into_boxed_slice(),
            }),
            Self::Exercise {
                sub_problem,
                range,
                uri,
                autogradable,
                points,
                solutions,
                hints,
                notes,
                gnotes,
                title,
                preconditions,
                styles,
                objectives,
            } => DocumentElement::Exercise(Exercise {
                sub_problem,
                uri,
                autogradable,
                points,
                title,
                styles,
                range,
                solutions: solutions.into_boxed_slice(),
                hints: hints.into_boxed_slice(),
                notes: notes.into_boxed_slice(),
                gnotes: gnotes.into_boxed_slice(),
                preconditions: preconditions.into_boxed_slice(),
                objectives: objectives.into_boxed_slice(),
                children: v.into_boxed_slice(),
            }),
        }
    }
}

pub(super) struct DocumentCheckIter<'a, Check: DocumentChecker> {
    stack: Vec<(
        Elem,
        std::vec::IntoIter<UncheckedDocumentElement>,
        Vec<DocumentElement>,
    )>,
    curr_in: std::vec::IntoIter<UncheckedDocumentElement>,
    curr_out: Vec<DocumentElement>,
    checker: &'a mut Check,
    uri: &'a DocumentURI,
}

impl<Check: DocumentChecker> DocumentCheckIter<'_, Check> {
    pub(super) fn go(
        elems: Vec<UncheckedDocumentElement>,
        checker: &mut Check,
        uri: &DocumentURI,
    ) -> Vec<DocumentElement> {
        let mut slf = DocumentCheckIter {
            stack: Vec::new(),
            curr_in: elems.into_iter(),
            curr_out: Vec::new(),
            checker,
            uri,
        };
        loop {
            while let Some(next) = slf.curr_in.next() {
                slf.do_elem(next);
            }
            if let Some((e, curr_in, curr_out)) = slf.stack.pop() {
                slf.curr_in = curr_in;
                let dones = std::mem::replace(&mut slf.curr_out, curr_out);
                let mut closed = e.close(dones, slf.checker);
                slf.checker.close(&mut closed);
                slf.curr_out.push(closed);
            } else {
                return slf.curr_out;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn do_elem(&mut self, mut e: UncheckedDocumentElement) {
        use UncheckedDocumentElement::*;
        self.checker.open(&mut e);
        let mut ret = match e {
            SetSectionLevel(lvl) => DocumentElement::SetSectionLevel(lvl),
            DocumentReference { id, range, target } => {
                let target = self
                    .checker
                    .get_document(&target)
                    .map_or_else(|| Err(target), Ok);
                DocumentElement::DocumentReference { id, range, target }
            }
            SymbolDeclaration(uri) => {
                let symbol = self
                    .checker
                    .get_declaration(&uri)
                    .map_or_else(|| Err(uri), Ok);
                DocumentElement::SymbolDeclaration(symbol)
            }
            UseModule(module) => {
                let module = self
                    .checker
                    .get_module(&module)
                    .map_or_else(|| Err(module), Ok);
                DocumentElement::UseModule(module)
            }
            ImportModule(module) => {
                let module = self
                    .checker
                    .get_module(&module)
                    .map_or_else(|| Err(module), Ok);
                DocumentElement::ImportModule(module)
            }
            Notation {
                symbol,
                id,
                notation,
            } => DocumentElement::Notation {
                symbol,
                id,
                notation,
            },
            VariableNotation {
                variable,
                id,
                notation,
            } => DocumentElement::VariableNotation {
                variable,
                id,
                notation,
            },
            Variable(v) => DocumentElement::Variable(v),
            Definiendum { range, uri } => DocumentElement::Definiendum { range, uri },
            SymbolReference {
                range,
                uri,
                notation,
            } => DocumentElement::SymbolReference {
                range,
                uri,
                notation,
            },
            VariableReference {
                range,
                uri,
                notation,
            } => DocumentElement::VariableReference {
                range,
                uri,
                notation,
            },
            TopTerm { uri, term } => DocumentElement::TopTerm { uri, term },
            Section(UncheckedSection {
                range,
                uri,
                level,
                title,
                children,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack.push((
                    Elem::Section {
                        range,
                        uri,
                        level,
                        title,
                    },
                    old_in,
                    old_out,
                ));
                return;
            }
            Module {
                range,
                module,
                children,
            } => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Module { range, module }, old_in, old_out));
                return;
            }
            Morphism {
                range,
                morphism,
                children,
            } => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Morphism { range, morphism }, old_in, old_out));
                return;
            }
            MathStructure {
                range,
                structure,
                children,
            } => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::MathStructure { range, structure }, old_in, old_out));
                return;
            }
            Paragraph(UncheckedLogicalParagraph {
                kind,
                uri,
                inline,
                title,
                fors,
                range,
                styles,
                children,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack.push((
                    Elem::Paragraph {
                        kind,
                        uri,
                        inline,
                        title,
                        fors,
                        range,
                        styles,
                    },
                    old_in,
                    old_out,
                ));
                return;
            }
            Exercise(UncheckedExercise {
                sub_exercise: sub_problem,
                uri,
                autogradable,
                points,
                solutions,
                range,
                hints,
                styles,
                notes,
                gnotes,
                title,
                children,
                preconditions,
                objectives,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack.push((
                    Elem::Exercise {
                        sub_problem,
                        uri,
                        range,
                        autogradable,
                        points,
                        solutions,
                        styles,
                        hints,
                        gnotes,
                        notes,
                        title,
                        preconditions,
                        objectives,
                    },
                    old_in,
                    old_out,
                ));
                return;
            }
        };
        self.checker.close(&mut ret);
        self.curr_out.push(ret);
    }
}
