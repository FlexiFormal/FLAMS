use flams_utils::vecmap::VecMap;

use crate::{
    content::terms::Term, uris::{DocumentElementURI, DocumentURI, ModuleURI, SymbolURI}, Checked, DocumentRange, LocalBackend, MaybeResolved, Unchecked
};

use super::{
    exercises::{CognitiveDimension, Exercise, GradingNote, Solutions},
    paragraphs::{LogicalParagraph, ParagraphKind},
    sections::{Section, SectionLevel},
    DocumentElement, LazyDocRef
};

pub trait DocumentChecker: LocalBackend {
    fn open(&mut self, elem: &mut DocumentElement<Unchecked>);
    fn close(&mut self, elem: &mut DocumentElement<Checked>);
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
    Extension {
        range: DocumentRange,
        extension: SymbolURI,
        target:SymbolURI
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
        sub_exercise: bool,
        uri: DocumentElementURI,
        autogradable: bool,
        range: DocumentRange,
        points: Option<f32>,
        solutions: LazyDocRef<Solutions>,
        gnotes: Vec<LazyDocRef<GradingNote>>,
        hints: Vec<DocumentRange>,
        notes: Vec<LazyDocRef<Box<str>>>,
        title: Option<DocumentRange>,
        styles: Box<[Box<str>]>,
        preconditions: Vec<(CognitiveDimension, SymbolURI)>,
        objectives: Vec<(CognitiveDimension, SymbolURI)>,
    },
}
impl Elem {
    fn close(self, v: Vec<DocumentElement<Checked>>, checker: &mut impl DocumentChecker) -> DocumentElement<Checked> {
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
                module: MaybeResolved::resolve(module,|m| checker.get_module(m)),
                children: v.into_boxed_slice(),
            },
            Self::Morphism { range, morphism } => DocumentElement::Morphism {
                range,
                morphism: MaybeResolved::resolve(morphism,|m| checker.get_declaration(m)),
                children: v.into_boxed_slice(),
            },
            Self::MathStructure { range, structure } => DocumentElement::MathStructure {
                range,
                structure:MaybeResolved::resolve(structure,|m| checker.get_declaration(m)),
                children: v.into_boxed_slice(),
            },
            Self::Extension { range, extension,target } => DocumentElement::Extension {
                range,
                extension:MaybeResolved::resolve(extension,|m| checker.get_declaration(m)),
                target:MaybeResolved::resolve(target,|m| checker.get_declaration(m)),
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
                sub_exercise: sub_problem,
                range,
                uri,
                autogradable,
                points,
                solutions,
                gnotes,
                hints,
                notes,
                title,
                preconditions,
                styles,
                objectives,
            } => DocumentElement::Exercise(Exercise {
                sub_exercise: sub_problem,
                uri,
                autogradable,
                points,
                title,
                styles,
                range,
                solutions,
                gnotes: gnotes.into_boxed_slice(),
                hints: hints.into_boxed_slice(),
                notes: notes.into_boxed_slice(),
                preconditions: preconditions.into_boxed_slice(),
                objectives: objectives.into_boxed_slice(),
                children: v.into_boxed_slice(),
            }),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) struct DocumentCheckIter<'a, Check: DocumentChecker> {
    stack: Vec<(
        Elem,
        std::vec::IntoIter<DocumentElement<Unchecked>>,
        Vec<DocumentElement<Checked>>,
    )>,
    curr_in: std::vec::IntoIter<DocumentElement<Unchecked>>,
    curr_out: Vec<DocumentElement<Checked>>,
    checker: &'a mut Check,
    uri: &'a DocumentURI,
}

impl<Check: DocumentChecker> DocumentCheckIter<'_, Check> {
    pub(super) fn go(
        elems: Vec<DocumentElement<Unchecked>>,
        checker: &mut Check,
        uri: &DocumentURI,
    ) -> Vec<DocumentElement<Checked>> {
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
    fn do_elem(&mut self, mut e: DocumentElement<Unchecked>) {
        self.checker.open(&mut e);
        let mut ret = match e {
            DocumentElement::SetSectionLevel(lvl) => DocumentElement::SetSectionLevel(lvl),
            DocumentElement::DocumentReference { id, range, target } => {
                let target = MaybeResolved::resolve(target,|m| self.checker.get_document(m));
                DocumentElement::DocumentReference { id, range, target }
            }
            DocumentElement::SymbolDeclaration(uri) => {
                let symbol = MaybeResolved::resolve(uri,|m| self.checker.get_declaration(m));
                DocumentElement::SymbolDeclaration(symbol)
            }
            DocumentElement::UseModule(module) => {
                let module = MaybeResolved::resolve(module,|m| self.checker.get_module(m));
                DocumentElement::UseModule(module)
            }
            DocumentElement::ImportModule(module) => {
                let module = MaybeResolved::resolve(module, |m| self.checker.get_module(m));
                DocumentElement::ImportModule(module)
            }
            DocumentElement::Notation {
                symbol,
                id,
                notation,
            } => DocumentElement::Notation {
                symbol,
                id,
                notation,
            },
            DocumentElement::VariableNotation {
                variable,
                id,
                notation,
            } => DocumentElement::VariableNotation {
                variable,
                id,
                notation,
            },
            DocumentElement::Variable(v) => DocumentElement::Variable(v),
            DocumentElement::Definiendum { range, uri } => DocumentElement::Definiendum { range, uri },
            DocumentElement::SymbolReference {
                range,
                uri,
                notation,
            } => DocumentElement::SymbolReference {
                range,
                uri,
                notation,
            },
            DocumentElement::VariableReference {
                range,
                uri,
                notation,
            } => DocumentElement::VariableReference {
                range,
                uri,
                notation,
            },
            DocumentElement::TopTerm { uri, term } => DocumentElement::TopTerm { uri, term },
            DocumentElement::Section(Section {
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
            DocumentElement::Module {
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
            DocumentElement::Morphism {
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
            DocumentElement::MathStructure {
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
            DocumentElement::Extension {
                range,
                extension,
                target,
                children,
            } => {
                let old_in = std::mem::replace(&mut self.curr_in, children.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Extension { range, extension,target }, old_in, old_out));
                return;
            }
            DocumentElement::Paragraph(LogicalParagraph {
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
            DocumentElement::Exercise(Exercise {
                sub_exercise,
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
                        sub_exercise,
                        uri,
                        range,
                        autogradable,
                        points,
                        solutions,
                        gnotes,
                        styles,
                        hints,
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
