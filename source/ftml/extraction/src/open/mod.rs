use std::borrow::Cow;

use either::Either;
use flams_ontology::{
    content::{
        declarations::{
            morphisms::Morphism,
            structures::{Extension, MathStructure},
            symbols::{ArgSpec, AssocType, Symbol},
            OpenDeclaration,
        },
        modules::{NestedModule, OpenModule},
        terms::{Term, Var},
    },
    languages::Language,
    narration::{
        notations::Notation,
        paragraphs::{LogicalParagraph, ParagraphFormatting, ParagraphKind},
        problems::{
            ChoiceBlock, FillInSol, FillInSolOption, GradingNote, Problem, SolutionData, Solutions,
        },
        sections::{Section, SectionLevel},
        variables::Variable,
        DocumentElement,
    },
    uris::{
        ContentURI, DocumentElementURI, DocumentURI, ModuleURI, Name, SymbolURI, URIOrRefTrait,
    },
};
use smallvec::SmallVec;
use terms::{OpenArg, PreVar, VarOrSym};

#[cfg(feature = "rdf")]
use flams_ontology::triple;

use crate::{
    errors::FTMLError,
    prelude::{FTMLExtractor, FTMLNode, NotationState, ParagraphState, ProblemState},
    rules::FTMLElements,
};

pub mod terms;
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum OpenFTMLElement {
    Invisible,
    SetSectionLevel(SectionLevel),
    ImportModule(ModuleURI),
    UseModule(ModuleURI),
    Slide(DocumentElementURI),
    SlideNumber,
    ProofBody,
    Module {
        uri: ModuleURI,
        meta: Option<ModuleURI>,
        signature: Option<Language>,
    },
    MathStructure {
        uri: SymbolURI,
        macroname: Option<Box<str>>,
    },
    Morphism {
        uri: SymbolURI,
        domain: ModuleURI,
        total: bool,
    },
    Assign(SymbolURI),
    Section {
        lvl: SectionLevel,
        uri: DocumentElementURI,
    },
    SkipSection,
    Paragraph {
        uri: DocumentElementURI,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Name]>,
    },
    Problem {
        uri: DocumentElementURI,
        styles: Box<[Name]>,
        autogradable: bool,
        points: Option<f32>,
        sub_problem: bool,
    },
    Doctitle,
    Title,
    ProofTitle,
    SubproofTitle,
    Symdecl {
        uri: SymbolURI,
        arity: ArgSpec,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
    },
    Vardecl {
        uri: DocumentElementURI,
        arity: ArgSpec,
        bind: bool,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
        is_seq: bool,
    },
    Notation {
        id: Box<str>,
        symbol: VarOrSym,
        precedence: isize,
        argprecs: SmallVec<isize, 9>,
    },
    NotationComp,
    NotationOpComp,
    Definiendum(SymbolURI),
    Type,
    Conclusion {
        uri: SymbolURI,
        in_term: bool,
    },
    Definiens {
        uri: Option<SymbolURI>,
        in_term: bool,
    },
    OpenTerm {
        term: terms::OpenTerm,
        is_top: bool,
    },
    ClosedTerm(Term),
    MMTRule(Box<str>),
    ArgSep,
    ArgMap,
    ArgMapSep,
    HeadTerm,
    ProblemHint,
    ProblemSolution(Option<Box<str>>),
    ProblemGradingNote,
    AnswerClass,
    AnswerClassFeedback,
    ChoiceBlock {
        multiple: bool,
        inline: bool,
    },
    ProblemChoice,
    ProblemChoiceVerdict,
    ProblemChoiceFeedback,
    Fillinsol(Option<f32>),
    FillinsolCase,

    Inputref {
        uri: DocumentURI,
        id: Box<str>,
    },
    IfInputref(bool),

    Comp,
    MainComp,
    DefComp,
    Arg(OpenArg),
}

impl OpenFTMLElement {
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn close<E: FTMLExtractor, N: FTMLNode>(
        self,
        previous: &mut FTMLElements,
        next: &mut FTMLElements,
        extractor: &mut E,
        node: &N,
    ) -> Option<Self> {
        //println!("{self:?}}}");
        match self {
            Self::Invisible => {
                if !extractor.in_term() && !extractor.in_notation() {
                    node.delete();
                }
            }
            Self::SetSectionLevel(lvl) => {
                extractor.add_document_element(DocumentElement::SetSectionLevel(lvl))
            }
            Self::ImportModule(uri) => Self::close_importmodule(extractor, uri),
            Self::UseModule(uri) => Self::close_usemodule(extractor, uri),
            Self::Module {
                uri,
                meta,
                signature,
            } => Self::close_module(extractor, node, uri, meta, signature),
            Self::MathStructure { uri, macroname } => {
                Self::close_structure(extractor, node, uri, macroname)
            }
            Self::Morphism { uri, domain, total } => {
                Self::close_morphism(extractor, node, uri, domain, total)
            }

            Self::Assign(_sym) => {
                if extractor.close_complex_term().is_some() {}
                // TODO
            }
            Self::SkipSection => {
                if let Some((_, _, children)) = extractor.close_section() {
                    extractor.add_document_element(DocumentElement::SkipSection(children));
                } else {
                    extractor.add_error(FTMLError::NotInNarrative);
                };
            }

            Self::Section { lvl, uri } => Self::close_section(extractor, node, lvl, uri),
            Self::Slide(uri) => {
                if let Some(children) = extractor.close_slide() {
                    extractor.add_document_element(DocumentElement::Slide {
                        range: node.range(),
                        uri,
                        children,
                    });
                } else {
                    extractor.add_error(FTMLError::NotInNarrative);
                };
            }
            Self::Paragraph {
                kind,
                formatting,
                styles,
                uri,
            } => Self::close_paragraph(extractor, node, kind, formatting, styles, uri),
            Self::Problem {
                uri,
                styles,
                autogradable,
                points,
                sub_problem,
            } => Self::close_problem(
                extractor,
                node,
                uri,
                styles,
                autogradable,
                points,
                sub_problem,
            ),

            Self::Doctitle => {
                extractor.set_document_title(node.inner_string().into_boxed_str());
            }

            Self::Title => {
                if extractor.add_title(node.inner_range()).is_err() {
                    extractor.add_error(FTMLError::NotInNarrative);
                }
            }
            Self::Symdecl {
                uri,
                arity,
                macroname,
                role,
                assoctype,
                reordering,
            } => Self::close_symdecl(
                extractor, uri, arity, macroname, role, assoctype, reordering,
            ),
            Self::Vardecl {
                uri,
                arity,
                bind,
                macroname,
                role,
                assoctype,
                reordering,
                is_seq,
            } => Self::close_vardecl(
                extractor, uri, bind, arity, macroname, role, assoctype, reordering, is_seq,
            ),
            Self::Notation {
                id,
                symbol,
                precedence,
                argprecs,
            } => Self::close_notation(extractor, id, symbol, precedence, argprecs),
            Self::NotationComp => {
                if let Some(n) = node.as_notation() {
                    if extractor.add_notation(n).is_err() {
                        extractor.add_error(FTMLError::NotInNarrative);
                    }
                } else {
                    extractor.add_error(FTMLError::NotInNarrative);
                }
            }
            Self::NotationOpComp => {
                if let Some(n) = node.as_op_notation() {
                    if extractor.add_op_notation(n).is_err() {
                        extractor.add_error(FTMLError::NotInNarrative);
                    }
                } else {
                    extractor.add_error(FTMLError::NotInNarrative);
                }
            }
            Self::Type => {
                extractor.set_in_term(false);
                let tm = Self::as_term(next, node);
                if extractor.add_type(tm).is_err() {
                    extractor.add_error(FTMLError::NotInContent);
                }
            }
            Self::Conclusion { uri, in_term } => {
                extractor.set_in_term(in_term);
                let tm = Self::as_term(next, node);
                if extractor.add_term(Some(uri), tm).is_err() {
                    extractor.add_error(FTMLError::NotInContent);
                }
            }
            Self::Definiens { uri, in_term } => {
                extractor.set_in_term(in_term);
                let tm = Self::as_term(next, node);
                if extractor.add_term(uri, tm).is_err() {
                    extractor.add_error(FTMLError::NotInContent);
                }
            }
            Self::OpenTerm { term, is_top: true } => {
                let term = term.close(extractor);
                let uri = match extractor.get_narrative_uri()
                    & &*extractor.new_id(Cow::Borrowed("term"))
                {
                    Ok(uri) => uri,
                    Err(_) => {
                        extractor
                            .add_error(FTMLError::InvalidURI("(should be impossible)".to_string()));
                        return None;
                    }
                };
                extractor.set_in_term(false);
                if !matches!(term, Term::OMID { .. } | Term::OMV { .. }) {
                    extractor.add_document_element(DocumentElement::TopTerm { uri, term });
                }
            }
            Self::OpenTerm {
                term,
                is_top: false,
            } => {
                let term = term.close(extractor);
                return Some(Self::ClosedTerm(term));
            }
            Self::MMTRule(_id) => {
                let _ = extractor.close_args();
                // TODO
            }
            Self::ArgSep => {
                return Some(Self::ArgSep);
            }
            Self::ArgMap => {
                return Some(Self::ArgMap);
            }
            Self::ArgMapSep => {
                return Some(Self::ArgMapSep);
            }
            Self::Arg(a) => {
                if extractor.in_notation() {
                    return Some(self);
                }
                let t = node.as_term();
                let pos = match a.index {
                    Either::Left(u) => (u, None),
                    Either::Right((a, b)) => (a, Some(b)),
                };
                if extractor.add_arg(pos, t, a.mode).is_err() {
                    //println!("HERE 1");
                    extractor.add_error(FTMLError::IncompleteArgs(3));
                }
            }
            Self::HeadTerm => {
                let tm = node.as_term();
                if extractor.add_term(None, tm).is_err() {
                    //println!("HERE 2");
                    extractor.add_error(FTMLError::IncompleteArgs(4));
                }
            }

            Self::Comp | Self::MainComp if extractor.in_notation() => {
                return Some(self);
            }
            Self::DefComp if extractor.in_notation() => {
                return Some(Self::Comp);
            }
            Self::ClosedTerm(_) => return Some(self),

            Self::Inputref { uri, id } => {
                let top = extractor.get_narrative_uri();
                #[cfg(feature = "rdf")]
                if E::RDF {
                    extractor.add_triples([triple!(<(top.to_iri())> dc:HAS_PART <(uri.to_iri())>)]);
                }
                extractor.add_document_element(DocumentElement::DocumentReference {
                    id: match top & &*id {
                        Ok(id) => id,
                        Err(_) => {
                            extractor.add_error(FTMLError::InvalidURI(format!("5: {id}")));
                            return None;
                        }
                    },
                    range: node.range(),
                    target: uri,
                });
                previous.elems.retain(|e| !matches!(e, Self::Invisible));
            }
            Self::ProblemHint => {
                if extractor
                    .with_problem(|ex| ex.hints.push(node.inner_range()))
                    .is_none()
                {
                    extractor.add_error(FTMLError::NotInProblem("a"));
                }
            }
            Self::ProblemSolution(id) => {
                let s = node.inner_string().into_boxed_str();
                node.delete_children();
                if extractor
                    .with_problem(|ex| {
                        ex.solutions.push(SolutionData::Solution {
                            html: s,
                            answer_class: id,
                        });
                    })
                    .is_none()
                {
                    extractor.add_error(FTMLError::NotInProblem("b"));
                }
            }
            Self::ProblemGradingNote => {
                let s = node.inner_string().into_boxed_str();
                node.delete_children();
                if let Some(gnote) = extractor.close_gnote() {
                    let gnote = GradingNote {
                        answer_classes: gnote.answer_classes,
                        html: s,
                    };
                    let r = extractor.add_resource(&gnote);
                    if extractor.with_problem(|ex| ex.gnotes.push(r)).is_none() {
                        extractor.add_error(FTMLError::NotInProblem("c"));
                    }
                } else {
                    extractor.add_error(FTMLError::NotInProblem("d"));
                }
            }
            Self::ChoiceBlock { .. } => {
                let range = node.range();
                if let Some(cb) = extractor.close_choice_block() {
                    if extractor
                        .with_problem(|ex| {
                            ex.solutions.push(SolutionData::ChoiceBlock(ChoiceBlock {
                                multiple: cb.multiple,
                                inline: cb.inline,
                                range,
                                styles: cb.styles,
                                choices: cb.choices,
                            }))
                        })
                        .is_none()
                    {
                        extractor.add_error(FTMLError::NotInProblem("e"));
                    }
                } else {
                    extractor.add_error(FTMLError::NotInProblem("f"));
                }
            }
            Self::AnswerClassFeedback => {
                let s = node.string().into_boxed_str();
                node.delete();
                if !extractor
                    .with_problem(|ex| {
                        if let Some(n) = &mut ex.gnote {
                            if let Some(ac) = n.answer_classes.last_mut() {
                                ac.feedback = s;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .unwrap_or_default()
                {
                    extractor.add_error(FTMLError::NotInProblem("g"));
                }
            }
            Self::ProblemChoiceVerdict => {
                let s = node.string().into_boxed_str();
                node.delete();
                if !extractor
                    .with_problem(|ex| {
                        if let Some(n) = &mut ex.choice_block {
                            if let Some(ac) = n.choices.last_mut() {
                                ac.verdict = s;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .unwrap_or_default()
                {
                    extractor.add_error(FTMLError::NotInProblem("h"));
                }
            }
            Self::ProblemChoiceFeedback => {
                let s = node.string().into_boxed_str();
                node.delete();
                if !extractor
                    .with_problem(|ex| {
                        if let Some(n) = &mut ex.choice_block {
                            if let Some(ac) = n.choices.last_mut() {
                                ac.feedback = s;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .unwrap_or_default()
                {
                    extractor.add_error(FTMLError::NotInProblem("i"));
                }
            }
            Self::Fillinsol(width) => {
                if !extractor
                    .with_problem(|ex| {
                        if let Some(n) = std::mem::take(&mut ex.fillinsol) {
                            ex.solutions.push(SolutionData::FillInSol(FillInSol {
                                width,
                                opts: n.cases,
                            }));
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap_or_default()
                {
                    extractor.add_error(FTMLError::NotInProblem("j"));
                }
                node.delete_children();
            }
            Self::FillinsolCase => {
                let s = node.inner_string().into_boxed_str();
                node.delete();
                if !extractor
                    .with_problem(|ex| {
                        if let Some(n) = &mut ex.fillinsol {
                            n.cases.last_mut().is_some_and(|n| match n {
                                FillInSolOption::Exact { feedback, .. }
                                | FillInSolOption::NumericalRange { feedback, .. }
                                | FillInSolOption::Regex { feedback, .. } => {
                                    *feedback = s;
                                    true
                                }
                            })
                        } else {
                            false
                        }
                    })
                    .unwrap_or_default()
                {
                    extractor.add_error(FTMLError::NotInProblem("k"));
                }
            }
            Self::IfInputref(_)
            | Self::Definiendum(_)
            | Self::Comp
            | Self::MainComp
            | Self::DefComp
            | Self::AnswerClass
            | Self::ProblemChoice
            | Self::SlideNumber
            | Self::ProofBody
            | Self::ProofTitle
            | Self::SubproofTitle => (),
        }
        None
    }

    fn as_term<N: FTMLNode>(next: &mut FTMLElements, node: &N) -> Term {
        if let Some(i) = next.iter().position(|e| matches!(e, Self::ClosedTerm(_))) {
            let Self::ClosedTerm(t) = next.elems.remove(i) else {
                unreachable!()
            };
            return t;
        }
        node.as_term()
    }

    fn close_importmodule<E: FTMLExtractor>(extractor: &mut E, uri: ModuleURI) {
        #[cfg(feature = "rdf")]
        if E::RDF {
            if let Some(m) = extractor.get_content_iri() {
                extractor.add_triples([triple!(<(m)> ulo:IMPORTS <(uri.to_iri())>)]);
            }
        }
        extractor.add_document_element(DocumentElement::ImportModule(uri.clone()));
        if extractor
            .add_content_element(OpenDeclaration::Import(uri))
            .is_err()
        {
            extractor.add_error(FTMLError::NotInContent);
        }
    }

    fn close_usemodule<E: FTMLExtractor>(extractor: &mut E, uri: ModuleURI) {
        #[cfg(feature = "rdf")]
        if E::RDF {
            extractor.add_triples([
                triple!(<(extractor.get_document_iri())> dc:REQUIRES <(uri.to_iri())>),
            ]);
        }
        extractor.add_document_element(DocumentElement::UseModule(uri));
    }

    fn close_module<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        uri: ModuleURI,
        meta: Option<ModuleURI>,
        signature: Option<Language>,
    ) {
        let Some((_, narrative)) = extractor.close_narrative() else {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        };
        let Some((_, mut content)) = extractor.close_content() else {
            extractor.add_error(FTMLError::NotInContent);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            let iri = uri.to_iri();
            extractor.add_triples([
                triple!(<(iri.clone())> : ulo:THEORY),
                triple!(<(extractor.get_document_iri())> ulo:CONTAINS <(iri)>),
            ]);
        }

        extractor.add_document_element(DocumentElement::Module {
            range: node.range(),
            module: uri.clone(),
            children: narrative,
        });

        if uri.name().is_simple() {
            extractor.add_module(OpenModule {
                uri,
                meta,
                signature,
                elements: content,
            });
        } else {
            // NestedModule
            let Some(sym) = uri.into_symbol() else {
                unreachable!()
            };
            #[cfg(feature = "rdf")]
            if E::RDF {
                if let Some(m) = extractor.get_content_iri() {
                    extractor.add_triples([triple!(<(m)> ulo:CONTAINS <(sym.to_iri())>)]);
                }
            }
            if extractor
                .add_content_element(OpenDeclaration::NestedModule(NestedModule {
                    uri: sym,
                    elements: std::mem::take(&mut content),
                }))
                .is_err()
            {
                extractor.add_error(FTMLError::NotInContent);
            }
        }
    }

    fn close_structure<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        uri: SymbolURI,
        macroname: Option<Box<str>>,
    ) {
        let Some((_, narrative)) = extractor.close_narrative() else {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        };
        let Some((_, content)) = extractor.close_content() else {
            extractor.add_error(FTMLError::NotInContent);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            if let Some(cont) = extractor.get_content_iri() {
                let iri = uri.to_iri();
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:STRUCTURE),
                    triple!(<(cont)> ulo:CONTAINS <(iri)>),
                ]);
            }
        }

        if uri.name().last_name().as_ref().starts_with("EXTSTRUCT") {
            let Some(target) = content.iter().find_map(|d| match d {
                OpenDeclaration::Import(uri)
                    if !uri.name().last_name().as_ref().starts_with("EXTSTRUCT") =>
                {
                    Some(uri)
                }
                _ => None,
            }) else {
                extractor.add_error(FTMLError::NotInContent);
                return;
            };
            let Some(target) = target.clone().into_symbol() else {
                extractor.add_error(FTMLError::NotInContent);
                return;
            };

            #[cfg(feature = "rdf")]
            if E::RDF {
                extractor.add_triples([triple!(<(uri.to_iri())> ulo:EXTENDS <(target.to_iri())>)]);
            }
            extractor.add_document_element(DocumentElement::Extension {
                range: node.range(),
                extension: uri.clone(),
                target: target.clone(),
                children: narrative,
            });
            if extractor
                .add_content_element(OpenDeclaration::Extension(Extension {
                    uri,
                    elements: content,
                    target,
                }))
                .is_err()
            {
                extractor.add_error(FTMLError::NotInContent);
            }
        } else {
            extractor.add_document_element(DocumentElement::MathStructure {
                range: node.range(),
                structure: uri.clone(),
                children: narrative,
            });
            if extractor
                .add_content_element(OpenDeclaration::MathStructure(MathStructure {
                    uri,
                    elements: content,
                    macroname,
                }))
                .is_err()
            {
                extractor.add_error(FTMLError::NotInContent);
            }
        }
    }

    fn close_morphism<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        uri: SymbolURI,
        domain: ModuleURI,
        total: bool,
    ) {
        let Some((_, narrative)) = extractor.close_narrative() else {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        };
        let Some((_, content)) = extractor.close_content() else {
            extractor.add_error(FTMLError::NotInContent);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            if let Some(cont) = extractor.get_content_iri() {
                let iri = uri.to_iri(); // TODO
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:MORPHISM),
                    triple!(<(iri.clone())> rdfs:DOMAIN <(domain.to_iri())>),
                    triple!(<(cont)> ulo:CONTAINS <(iri)>),
                ]);
            }
        }

        extractor.add_document_element(DocumentElement::Morphism {
            range: node.range(),
            morphism: uri.clone(),
            children: narrative,
        });
        if extractor
            .add_content_element(OpenDeclaration::Morphism(Morphism {
                uri,
                domain,
                total,
                elements: content,
            }))
            .is_err()
        {
            extractor.add_error(FTMLError::NotInContent);
        }
    }

    fn close_section<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        lvl: SectionLevel,
        uri: DocumentElementURI,
    ) {
        let Some((_, title, children)) = extractor.close_section() else {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            let doc = extractor.get_document_iri();
            let iri = uri.to_iri();
            extractor.add_triples([
                triple!(<(iri.clone())> : ulo:SECTION),
                triple!(<(doc)> ulo:CONTAINS <(iri)>),
            ]);
        }

        extractor.add_document_element(DocumentElement::Section(Section {
            range: node.range(),
            level: lvl,
            title,
            uri,
            children,
        }));
    }

    fn close_paragraph<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        kind: ParagraphKind,
        formatting: ParagraphFormatting,
        styles: Box<[Name]>,
        uri: DocumentElementURI,
    ) {
        let Some(ParagraphState {
            children,
            fors,
            title,
            ..
        }) = extractor.close_paragraph()
        else {
            extractor.add_error(FTMLError::NotInParagraph);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            let doc = extractor.get_document_iri();
            let iri = uri.to_iri();
            if kind.is_definition_like(&styles) {
                for (f, _) in fors.iter() {
                    extractor.add_triples([triple!(<(iri.clone())> ulo:DEFINES <(f.to_iri())>)]);
                }
            } else if kind == ParagraphKind::Example {
                for (f, _) in fors.iter() {
                    extractor
                        .add_triples([triple!(<(iri.clone())> ulo:EXAMPLE_FOR <(f.to_iri())>)]);
                }
            }
            extractor.add_triples([
                triple!(<(iri.clone())> : <(kind.rdf_type().into_owned())>),
                triple!(<(doc)> ulo:CONTAINS <(iri)>),
            ]);
        }

        extractor.add_document_element(DocumentElement::Paragraph(LogicalParagraph {
            range: node.range(),
            kind,
            formatting,
            styles,
            fors,
            uri,
            children,
            title,
        }));
    }

    fn close_problem<E: FTMLExtractor, N: FTMLNode>(
        extractor: &mut E,
        node: &N,
        uri: DocumentElementURI,
        styles: Box<[Name]>,
        autogradable: bool,
        points: Option<f32>,
        sub_problem: bool,
    ) {
        let Some(ProblemState {
            solutions,
            hints,
            notes,
            gnotes,
            title,
            children,
            preconditions,
            objectives,
            ..
        }) = extractor.close_problem()
        else {
            extractor.add_error(FTMLError::NotInProblem("l"));
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            let doc = extractor.get_document_iri();
            let iri = uri.to_iri();
            for (d, s) in &preconditions {
                let b = flams_ontology::rdf::BlankNode::default();
                extractor.add_triples([
                    triple!(<(iri.clone())> ulo:PRECONDITION (b.clone())!),
                    triple!((b.clone())! ulo:COGDIM <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:POSYMBOL <(s.to_iri())>),
                ]);
            }
            for (d, s) in &objectives {
                let b = flams_ontology::rdf::BlankNode::default();
                extractor.add_triples([
                    triple!(<(iri.clone())> ulo:OBJECTIVE (b.clone())!),
                    triple!((b.clone())! ulo:COGDIM <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:POSYMBOL <(s.to_iri())>),
                ]);
            }

            extractor.add_triples([
                if sub_problem {
                    triple!(<(iri.clone())> : ulo:SUBPROBLEM)
                } else {
                    triple!(<(iri.clone())> : ulo:PROBLEM)
                },
                triple!(<(doc)> ulo:CONTAINS <(iri)>),
            ]);
        }
        let solutions =
            extractor.add_resource(&Solutions::from_solutions(solutions.into_boxed_slice()));

        extractor.add_document_element(DocumentElement::Problem(Problem {
            range: node.range(),
            uri,
            styles,
            autogradable,
            points,
            sub_problem,
            gnotes,
            solutions,
            hints,
            notes,
            title,
            children,
            preconditions,
            objectives,
        }));
    }

    fn close_symdecl<E: FTMLExtractor>(
        extractor: &mut E,
        uri: SymbolURI,
        arity: ArgSpec,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
    ) {
        let Some((tp, df)) = extractor.close_decl() else {
            extractor.add_error(FTMLError::NotInContent);
            return;
        };
        #[cfg(feature = "rdf")]
        if E::RDF {
            if let Some(m) = extractor.get_content_iri() {
                let iri = uri.to_iri();
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:DECLARATION),
                    triple!(<(m)> ulo:DECLARES <(iri)>),
                ]);
            }
        }
        extractor.add_document_element(DocumentElement::SymbolDeclaration(uri.clone()));
        if extractor
            .add_content_element(OpenDeclaration::Symbol(Symbol {
                uri,
                arity,
                macroname,
                role,
                tp,
                df,
                assoctype,
                reordering,
            }))
            .is_err()
        {
            extractor.add_error(FTMLError::NotInContent);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn close_vardecl<E: FTMLExtractor>(
        extractor: &mut E,
        uri: DocumentElementURI,
        bind: bool,
        arity: ArgSpec,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
        is_seq: bool,
    ) {
        let Some((tp, df)) = extractor.close_decl() else {
            extractor.add_error(FTMLError::NotInContent);
            return;
        };

        #[cfg(feature = "rdf")]
        if E::RDF {
            let iri = uri.to_iri();
            extractor.add_triples([
                triple!(<(iri.clone())> : ulo:VARIABLE),
                triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
            ]);
        }

        extractor.add_document_element(DocumentElement::Variable(Variable {
            uri,
            arity,
            macroname,
            bind,
            role,
            tp,
            df,
            assoctype,
            reordering,
            is_seq,
        }));
    }

    fn close_notation<E: FTMLExtractor>(
        extractor: &mut E,
        id: Box<str>,
        symbol: VarOrSym,
        precedence: isize,
        argprecs: SmallVec<isize, 9>,
    ) {
        let Some(NotationState {
            attribute_index,
            inner_index,
            is_text,
            components,
            op,
        }) = extractor.close_notation()
        else {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        };
        if attribute_index == 0 {
            extractor.add_error(FTMLError::NotInNarrative);
            return;
        }
        let uri = match extractor.get_narrative_uri() & &*id {
            Ok(uri) => uri,
            Err(_) => {
                extractor.add_error(FTMLError::InvalidURI(format!("6: {id}")));
                return;
            }
        };
        let notation = extractor.add_resource(&Notation {
            attribute_index,
            is_text,
            inner_index,
            components,
            op,
            precedence,
            id,
            argprecs,
        });
        match symbol {
            VarOrSym::S(ContentURI::Symbol(symbol)) => {
                #[cfg(feature = "rdf")]
                if E::RDF {
                    let iri = uri.to_iri();
                    extractor.add_triples([
                        triple!(<(iri.clone())> : ulo:NOTATION),
                        triple!(<(iri.clone())> ulo:NOTATION_FOR <(symbol.to_iri())>),
                        triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
                    ]);
                }
                extractor.add_document_element(DocumentElement::Notation {
                    symbol,
                    id: uri,
                    notation,
                });
            }
            VarOrSym::S(_) => unreachable!(),
            VarOrSym::V(PreVar::Resolved(variable)) => {
                extractor.add_document_element(DocumentElement::VariableNotation {
                    variable,
                    id: uri,
                    notation,
                })
            }
            VarOrSym::V(PreVar::Unresolved(name)) => match extractor.resolve_variable_name(name) {
                Var::Name(name) => extractor.add_error(FTMLError::UnresolvedVariable(name)),
                Var::Ref { declaration, .. } => {
                    extractor.add_document_element(DocumentElement::VariableNotation {
                        variable: declaration,
                        id: uri,
                        notation,
                    })
                }
            },
        }
    }
}
