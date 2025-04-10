#![allow(clippy::result_large_err)]

use crate::errors::FTMLError;
use crate::open::terms::TermOrList;
use crate::rules::FTMLElements;
use flams_ontology::content::declarations::OpenDeclaration;
use flams_ontology::content::modules::OpenModule;
use flams_ontology::content::terms::{Arg, ArgMode, Term, Var};
use flams_ontology::ftml::FTMLKey;
use flams_ontology::languages::Language;
use flams_ontology::narration::documents::DocumentStyles;
use flams_ontology::narration::notations::{NotationComponent, OpNotation};
use flams_ontology::narration::problems::{
    AnswerClass, AnswerKind, Choice, CognitiveDimension, FillInSolOption, GradingNote, SolutionData,
};
use flams_ontology::narration::sections::SectionLevel;
use flams_ontology::narration::variables::Variable;
use flams_ontology::narration::{DocumentElement, LazyDocRef};
use flams_ontology::uris::{
    DocumentElementURI, DocumentURI, ModuleURI, Name, NarrativeURI, NarrativeURITrait, SymbolURI,
    URIRefTrait,
};
use flams_ontology::{DocumentRange, Resourcable, Unchecked};
use flams_utils::id_counters::IdCounter;
use flams_utils::vecmap::{VecMap, VecSet};
use std::borrow::Cow;
use std::str::FromStr;

pub trait FTMLExtractor {
    type Attr<'a>: Attributes;

    #[cfg(feature = "rdf")]
    const RDF: bool;

    fn styles(&mut self) -> &mut DocumentStyles;

    #[cfg(feature = "rdf")]
    fn add_triples<const N: usize>(&mut self, triples: [flams_ontology::rdf::Triple; N]);

    fn get_narrative_uri(&self) -> NarrativeURI;
    fn get_content_uri(&self) -> Option<&ModuleURI>;

    #[cfg(feature = "rdf")]
    fn get_document_iri(&self) -> flams_ontology::rdf::NamedNode {
        use flams_ontology::uris::URIOrRefTrait;
        self.get_narrative_uri().to_iri()
    }

    #[cfg(feature = "rdf")]
    fn get_content_iri(&self) -> Option<flams_ontology::rdf::NamedNode> {
        use flams_ontology::uris::URIOrRefTrait;
        self.get_content_uri().map(URIOrRefTrait::to_iri)
    }

    fn with_problem<R>(&mut self, then: impl FnOnce(&mut ProblemState) -> R) -> Option<R>;

    fn resolve_variable_name(&self, name: Name) -> Var;
    fn add_error(&mut self, err: FTMLError);
    fn add_module(&mut self, module: OpenModule<Unchecked>);
    fn new_id(&mut self, prefix: Cow<'static, str>) -> Box<str>;
    fn in_notation(&self) -> bool;
    fn in_term(&self) -> bool;
    fn set_in_term(&mut self, b: bool);
    fn add_document_element(&mut self, elem: DocumentElement<Unchecked>);
    /// ### Errors
    fn add_content_element(
        &mut self,
        elem: OpenDeclaration<Unchecked>,
    ) -> Result<(), OpenDeclaration<Unchecked>>;

    fn open_content(&mut self, uri: ModuleURI);
    fn open_narrative(&mut self, uri: Option<NarrativeURI>);
    fn open_complex_term(&mut self);
    fn close_content(&mut self) -> Option<(ModuleURI, Vec<OpenDeclaration<Unchecked>>)>;
    fn close_narrative(&mut self) -> Option<(NarrativeURI, Vec<DocumentElement<Unchecked>>)>;
    fn close_complex_term(&mut self) -> Option<Term>;
    fn open_section(&mut self, uri: DocumentElementURI);
    fn close_section(
        &mut self,
    ) -> Option<(
        DocumentElementURI,
        Option<DocumentRange>,
        Vec<DocumentElement<Unchecked>>,
    )>;
    fn open_slide(&mut self);
    fn close_slide(&mut self) -> Option<Vec<DocumentElement<Unchecked>>>;
    fn open_paragraph(&mut self, uri: DocumentElementURI, fors: VecSet<SymbolURI>);
    fn close_paragraph(&mut self) -> Option<ParagraphState>;
    fn open_problem(&mut self, uri: DocumentElementURI);
    fn close_problem(&mut self) -> Option<ProblemState>;
    fn open_gnote(&mut self);
    fn close_gnote(&mut self) -> Option<GnoteState>;
    fn open_choice_block(&mut self, multiple: bool, styles: Box<[Box<str>]>);
    fn close_choice_block(&mut self) -> Option<ChoiceBlockState>;
    fn open_fillinsol(&mut self, width: Option<f32>);
    fn close_fillinsol(&mut self) -> Option<FillinsolState>;
    fn push_fillinsol_case(&mut self, case: FillInSolOption);
    fn push_answer_class(&mut self, id: Box<str>, kind: AnswerKind);
    fn push_problem_choice(&mut self, correct: bool);

    fn set_document_title(&mut self, title: Box<str>);
    /// #### Errors
    fn add_title(&mut self, title: DocumentRange) -> Result<(), DocumentRange>;
    fn open_decl(&mut self);
    fn close_decl(&mut self) -> Option<(Option<Term>, Option<Term>)>;
    fn open_notation(&mut self);
    fn close_notation(&mut self) -> Option<NotationState>;
    fn open_args(&mut self);
    fn close_args(&mut self) -> (Vec<Arg>, Option<Term>);

    fn add_precondition(&mut self, uri: SymbolURI, dim: CognitiveDimension);
    fn add_objective(&mut self, uri: SymbolURI, dim: CognitiveDimension);
    /// #### Errors
    #[allow(clippy::result_unit_err)]
    fn add_arg(&mut self, pos: (u8, Option<u8>), tm: Term, mode: ArgMode) -> Result<(), ()>;

    fn add_definiendum(&mut self, uri: SymbolURI);

    fn add_resource<T: Resourcable>(&mut self, t: &T) -> LazyDocRef<T>;
    /// #### Errors
    fn add_notation(&mut self, spec: NotationSpec) -> Result<(), NotationSpec>;
    /// #### Errors
    fn add_op_notation(&mut self, op: OpNotation) -> Result<(), OpNotation>;
    /// #### Errors
    fn add_type(&mut self, tm: Term) -> Result<(), Term>;
    /// #### Errors
    fn add_term(&mut self, symbol: Option<SymbolURI>, tm: Term) -> Result<(), Term>;
}

pub trait Attributes {
    type KeyIter<'a>: Iterator<Item = &'a str>
    where
        Self: 'a;
    type Value<'a>: AsRef<str> + Into<Cow<'a, str>> + Into<String>
    where
        Self: 'a;
    fn keys(&self) -> Self::KeyIter<'_>;
    fn value(&self, key: &str) -> Option<Self::Value<'_>>;
    fn set(&mut self, key: &str, value: &str);
    fn take(&mut self, key: &str) -> Option<String>;

    #[inline]
    fn get(&self, tag: FTMLKey) -> Option<Self::Value<'_>> {
        self.value(tag.attr_name())
    }
    #[inline]
    fn remove(&mut self, tag: FTMLKey) -> Option<String> {
        self.take(tag.attr_name())
    }

    /// #### Errors
    fn get_typed<E, T>(
        &self,
        key: FTMLKey,
        f: impl FnOnce(&str) -> Result<T, E>,
    ) -> Result<T, FTMLError> {
        let Some(v) = self.get(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        f(v.as_ref()).map_err(|_| FTMLError::InvalidKeyFor(key.as_str(), Some(v.into())))
    }

    /// #### Errors
    fn get_typed_vec<E, T>(
        &self,
        key: FTMLKey,
        mut f: impl FnMut(&str) -> Result<T, E>,
    ) -> Result<Vec<T>, FTMLError> {
        let Some(v) = self.get(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        let mut vec = Vec::new();
        for e in v.as_ref().split(',') {
            match f(e) {
                Ok(v) => vec.push(v),
                Err(_) => return Err(FTMLError::InvalidKeyFor(key.as_str(), Some(v.into()))),
            }
        }
        Ok(vec)
    }

    /// #### Errors
    fn take_typed<E, T>(
        &mut self,
        key: FTMLKey,
        f: impl FnOnce(&str) -> Result<T, E>,
    ) -> Result<T, FTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        f(v.as_ref()).map_err(|_| FTMLError::InvalidKeyFor(key.as_str(), Some(v)))
    }

    /// #### Errors
    fn get_section_level(&self, key: FTMLKey) -> Result<SectionLevel, FTMLError> {
        use std::str::FromStr;
        let Some(v) = self.get(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        let Ok(u) = u8::from_str(v.as_ref()) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), Some(v.into())));
        };
        SectionLevel::try_from(u)
            .map_err(|()| FTMLError::InvalidKeyFor(key.as_str(), Some(v.into())))
    }

    /// #### Errors
    fn take_section_level(&mut self, key: FTMLKey) -> Result<SectionLevel, FTMLError> {
        use std::str::FromStr;
        let Some(v) = self.remove(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        let Ok(u) = u8::from_str(v.as_ref()) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), Some(v)));
        };
        SectionLevel::try_from(u).map_err(|()| FTMLError::InvalidKeyFor(key.as_str(), Some(v)))
    }

    /// #### Errors
    #[inline]
    fn get_language(&self, key: FTMLKey) -> Result<Language, FTMLError> {
        self.get_typed(key, Language::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_language(&mut self, key: FTMLKey) -> Result<Language, FTMLError> {
        self.take_typed(key, Language::from_str)
    }

    /// #### Errors
    #[inline]
    fn get_module_uri<E: FTMLExtractor>(
        &self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<ModuleURI, FTMLError> {
        self.get_typed(key, ModuleURI::from_str)
    }

    /// #### Errors
    fn get_new_module_uri<E: FTMLExtractor>(
        &self,
        key: FTMLKey,
        extractor: &mut E,
    ) -> Result<ModuleURI, FTMLError> {
        let Some(v) = self.get(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        extractor
            .get_content_uri()
            .map_or_else(
                || {
                    extractor
                        .get_narrative_uri()
                        .document()
                        .module_uri_from(v.as_ref())
                },
                |m| m.clone() / v.as_ref(),
            )
            .map_err(|_| FTMLError::InvalidURI(format!("1: {}", v.as_ref())))
    }

    /// #### Errors
    #[inline]
    fn take_module_uri<E: FTMLExtractor>(
        &mut self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<ModuleURI, FTMLError> {
        self.take_typed(key, ModuleURI::from_str)
    }

    /// #### Errors
    fn take_new_module_uri<E: FTMLExtractor>(
        &mut self,
        key: FTMLKey,
        extractor: &mut E,
    ) -> Result<ModuleURI, FTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        extractor
            .get_content_uri()
            .map_or_else(
                || extractor.get_narrative_uri().document().module_uri_from(&v),
                |m| m.clone() / v.as_str(),
            )
            .map_err(|_| FTMLError::InvalidURI(format!("2: {v}")))
    }

    /// #### Errors
    #[inline]
    fn get_symbol_uri<E: FTMLExtractor>(
        &self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<SymbolURI, FTMLError> {
        self.get_typed(key, SymbolURI::from_str)
    }

    /// #### Errors
    fn get_new_symbol_uri<E: FTMLExtractor>(
        &self,
        key: FTMLKey,
        extractor: &mut E,
    ) -> Result<SymbolURI, FTMLError> {
        let Some(v) = self.get(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        let Some(module) = extractor.get_content_uri() else {
            return Err(FTMLError::NotInContent);
        };
        (module.owned() | v.as_ref())
            .map_err(|_| FTMLError::InvalidURI(format!("3: {}", v.as_ref())))
    }

    /// #### Errors
    #[inline]
    fn take_symbol_uri<E: FTMLExtractor>(
        &mut self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<SymbolURI, FTMLError> {
        self.take_typed(key, SymbolURI::from_str)
    }

    /// #### Errors
    fn take_new_symbol_uri<E: FTMLExtractor>(
        &mut self,
        key: FTMLKey,
        extractor: &mut E,
    ) -> Result<SymbolURI, FTMLError> {
        let Some(v) = self.remove(key) else {
            return Err(FTMLError::InvalidKeyFor(key.as_str(), None));
        };
        let Some(module) = extractor.get_content_uri() else {
            return Err(FTMLError::NotInContent);
        };
        (module.owned() | v.as_str()).map_err(|_| FTMLError::InvalidURI(format!("4: {v}")))
    }

    /// #### Errors
    #[inline]
    fn get_document_uri<E: FTMLExtractor>(
        &self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<DocumentURI, FTMLError> {
        self.get_typed(key, DocumentURI::from_str)
    }

    /// #### Errors
    #[inline]
    fn take_document_uri<E: FTMLExtractor>(
        &mut self,
        key: FTMLKey,
        _extractor: &mut E,
    ) -> Result<DocumentURI, FTMLError> {
        self.take_typed(key, DocumentURI::from_str)
    }

    fn get_id<E: FTMLExtractor>(&self, extractor: &mut E, prefix: Cow<'static, str>) -> Box<str> {
        self.get(FTMLKey::Id).map_or_else(
            || extractor.new_id(prefix),
            |v| {
                let v = v.as_ref();
                if v.starts_with("http") && v.contains('?') {
                    v.rsplit_once('?')
                        .unwrap_or_else(|| unreachable!())
                        .1
                        .into()
                } else {
                    Into::<String>::into(v).into_boxed_str()
                }
            },
        )
    }

    fn get_bool(&self, key: FTMLKey) -> bool {
        self.get(key)
            .and_then(|s| s.as_ref().parse().ok())
            .unwrap_or_default()
    }

    fn take_bool(&mut self, key: FTMLKey) -> bool {
        self.remove(key)
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

pub trait FTMLNode {
    type Ancestors<'a>: Iterator<Item = Self>
    where
        Self: 'a;
    fn ancestors(&self) -> Self::Ancestors<'_>;
    fn with_elements<R>(&mut self, f: impl FnMut(Option<&mut FTMLElements>) -> R) -> R;
    fn delete(&self);
    fn delete_children(&self);
    fn range(&self) -> DocumentRange;
    fn inner_range(&self) -> DocumentRange;
    fn string(&self) -> String;
    fn inner_string(&self) -> String;
    fn as_notation(&self) -> Option<NotationSpec>;
    fn as_op_notation(&self) -> Option<OpNotation>;
    fn as_term(&self) -> Term;
}

#[derive(Debug)]
pub struct ParagraphState {
    pub uri: DocumentElementURI,
    pub children: Vec<DocumentElement<Unchecked>>,
    pub fors: VecMap<SymbolURI, Option<Term>>,
    pub title: Option<DocumentRange>,
}

#[derive(Clone, Debug)]
pub struct NotationState {
    pub attribute_index: u8,
    pub inner_index: u16,
    pub is_text: bool,
    pub components: Box<[NotationComponent]>,
    pub op: Option<OpNotation>,
}

#[derive(Debug)]
pub struct ProblemState {
    pub uri: DocumentElementURI,
    pub solutions: Vec<SolutionData>,
    pub gnote: Option<GnoteState>,
    pub choice_block: Option<ChoiceBlockState>,
    pub fillinsol: Option<FillinsolState>,
    pub hints: Vec<DocumentRange>,
    pub notes: Vec<LazyDocRef<Box<str>>>,
    pub gnotes: Vec<LazyDocRef<GradingNote>>,
    pub title: Option<DocumentRange>,
    pub children: Vec<DocumentElement<Unchecked>>,
    pub preconditions: Vec<(CognitiveDimension, SymbolURI)>,
    pub objectives: Vec<(CognitiveDimension, SymbolURI)>,
}
impl ProblemState {
    #[must_use]
    pub const fn new(uri: DocumentElementURI) -> Self {
        Self {
            uri,
            solutions: Vec::new(),
            gnote: None,
            choice_block: None,
            hints: Vec::new(),
            fillinsol: None,
            gnotes: Vec::new(),
            notes: Vec::new(),
            title: None,
            children: Vec::new(),
            preconditions: Vec::new(),
            objectives: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct GnoteState {
    pub answer_classes: Vec<AnswerClass>,
}

#[derive(Debug)]
pub struct ChoiceBlockState {
    pub multiple: bool,
    pub inline: bool,
    pub styles: Box<[Box<str>]>,
    pub choices: Vec<Choice>,
}

#[derive(Debug)]
pub struct FillinsolState {
    pub cases: Vec<FillInSolOption>,
}

pub struct NotationSpec {
    pub attribute_index: u8,
    pub inner_index: u16,
    pub is_text: bool,
    pub components: Box<[NotationComponent]>,
}

#[cfg(feature = "full")]
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Narrative {
    Container(NarrativeURI, Vec<DocumentElement<Unchecked>>),
    Paragraph(ParagraphState),
    Section {
        uri: DocumentElementURI,
        title: Option<DocumentRange>,
        children: Vec<DocumentElement<Unchecked>>,
    },
    Slide {
        children: Vec<DocumentElement<Unchecked>>,
    },
    Problem(ProblemState),
    Notation(NotationState),
}

#[cfg(feature = "full")]
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Content {
    Container(ModuleURI, Vec<OpenDeclaration<Unchecked>>),
    SingleTerm(Option<Term>),
    Symdecl { tp: Option<Term>, df: Option<Term> },
    Args(Vec<Option<(TermOrList, ArgMode)>>, Option<Term>),
}

#[cfg(feature = "full")]
#[derive(Debug)]
pub struct ExtractorState {
    pub(crate) in_term: bool,
    pub(crate) ids: IdCounter,
    pub(crate) narrative: Vec<Narrative>,
    pub(crate) content: Vec<Content>,
    pub(crate) modules: Vec<OpenModule<Unchecked>>,
    pub(crate) styles: DocumentStyles,
}
#[cfg(feature = "full")]
impl ExtractorState {
    #[must_use]
    pub fn document_uri(&self) -> &DocumentURI {
        let Some(Narrative::Container(NarrativeURI::Document(ref ret), _)) =
            self.narrative.first().as_ref()
        else {
            unreachable!()
        };
        ret
    }
    #[must_use]
    pub fn new(document: DocumentURI) -> Self {
        Self {
            in_term: false,
            ids: IdCounter::default(),
            narrative: vec![Narrative::Container(document.into(), Vec::new())],
            content: Vec::new(),
            styles: DocumentStyles::default(),
            modules: Vec::new(),
        }
    }
    /// #### Errors
    #[allow(clippy::result_unit_err)]
    #[allow(clippy::type_complexity)]
    pub fn take(
        mut self,
    ) -> Result<
        (
            DocumentURI,
            Vec<DocumentElement<Unchecked>>,
            Vec<OpenModule<Unchecked>>,
            DocumentStyles,
        ),
        (),
    > {
        if self.narrative.len() == 1 {
            let Some(Narrative::Container(document, elements)) = self.narrative.pop() else {
                unreachable!()
            };
            match document {
                NarrativeURI::Document(d) => Ok((d, elements, self.modules, self.styles)),
                NarrativeURI::Element(_) => Err(()),
            }
        } else {
            Err(())
        }
    }
    pub(crate) fn push_narr(&mut self, uri: Option<NarrativeURI>) {
        let uri = uri.unwrap_or_else(|| {
            self.narrative
                .iter()
                .rev()
                .find_map(|t| match t {
                    Narrative::Container(uri, _) => Some(uri.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| unreachable!())
        });
        self.narrative.push(Narrative::Container(uri, Vec::new()));
    }
}

#[cfg(feature = "full")]
pub trait StatefulExtractor {
    type Attr<'a>: Attributes;
    #[cfg(feature = "rdf")]
    const RDF: bool;
    #[cfg(feature = "rdf")]
    fn add_triples<const N: usize>(&mut self, triples: [flams_ontology::rdf::Triple; N]);

    fn state_mut(&mut self) -> &mut ExtractorState;
    fn state(&self) -> &ExtractorState;
    fn add_error(&mut self, err: FTMLError);
    fn set_document_title(&mut self, title: Box<str>);
    fn add_resource<T: Resourcable>(&mut self, t: &T) -> LazyDocRef<T>;
}
#[cfg(feature = "full")]
impl<E: StatefulExtractor> FTMLExtractor for E {
    type Attr<'a> = <Self as StatefulExtractor>::Attr<'a>;
    #[cfg(feature = "rdf")]
    const RDF: bool = <Self as StatefulExtractor>::RDF;
    #[cfg(feature = "rdf")]
    fn add_triples<const N: usize>(&mut self, triples: [flams_ontology::rdf::Triple; N]) {
        <Self as StatefulExtractor>::add_triples(self, triples);
    }
    fn add_error(&mut self, err: FTMLError) {
        <Self as StatefulExtractor>::add_error(self, err);
    }

    fn styles(&mut self) -> &mut DocumentStyles {
        &mut self.state_mut().styles
    }

    #[inline]
    fn set_document_title(&mut self, title: Box<str>) {
        <Self as StatefulExtractor>::set_document_title(self, title);
    }

    #[inline]
    fn add_resource<T: Resourcable>(&mut self, t: &T) -> LazyDocRef<T> {
        <Self as StatefulExtractor>::add_resource(self, t)
    }

    fn resolve_variable_name(&self, name: Name) -> Var {
        let names = name.steps();
        for n in self.state().narrative.iter().rev() {
            let ch = match n {
                Narrative::Container(_, c) => c,
                Narrative::Problem(ProblemState { children, .. })
                | Narrative::Section { children, .. }
                | Narrative::Paragraph(ParagraphState { children, .. })
                | Narrative::Slide { children, .. } => children,
                Narrative::Notation(_) => continue,
            };
            for c in ch.iter().rev() {
                match c {
                    DocumentElement::Variable(Variable { uri, is_seq, .. })
                        if uri.name().steps().ends_with(names) =>
                    {
                        return Var::Ref {
                            declaration: uri.clone(),
                            is_sequence: Some(*is_seq),
                        }
                    }
                    _ => (),
                }
            }
        }
        Var::Name(name)
    }

    fn open_content(&mut self, uri: ModuleURI) {
        self.state_mut()
            .content
            .push(Content::Container(uri, Vec::new()));
    }
    fn open_narrative(&mut self, uri: Option<NarrativeURI>) {
        self.state_mut().push_narr(uri);
    }
    fn open_complex_term(&mut self) {
        self.state_mut().content.push(Content::SingleTerm(None));
    }
    fn close_content(&mut self) -> Option<(ModuleURI, Vec<OpenDeclaration<Unchecked>>)> {
        match self.state_mut().content.pop() {
            Some(Content::Container(uri, elements)) => return Some((uri, elements)),
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        None
    }
    fn close_narrative(&mut self) -> Option<(NarrativeURI, Vec<DocumentElement<Unchecked>>)> {
        let state = self.state_mut();
        let r = state.narrative.pop().unwrap_or_else(|| unreachable!());
        if state.narrative.is_empty() {
            state.narrative.push(r);
            return None;
        }
        if let Narrative::Container(uri, elements) = r {
            Some((uri, elements))
        } else {
            state.narrative.push(r);
            None
        }
    }

    fn with_problem<R>(&mut self, then: impl FnOnce(&mut ProblemState) -> R) -> Option<R> {
        let state = self.state_mut();
        for e in state.narrative.iter_mut().rev() {
            if let Narrative::Problem(e) = e {
                return Some(then(e));
            }
        }
        None
    }

    fn push_answer_class(&mut self, id: Box<str>, kind: AnswerKind) {
        if !self
            .with_problem(|e| {
                if let Some(gnote) = e.gnote.as_mut() {
                    gnote.answer_classes.push(AnswerClass {
                        id,
                        kind,
                        feedback: "".into(),
                    });
                    true
                } else {
                    false
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("1"));
        }
    }

    fn push_problem_choice(&mut self, correct: bool) {
        if !self
            .with_problem(|ex| {
                if let Some(block) = &mut ex.choice_block {
                    block.choices.push(Choice {
                        correct,
                        verdict: Box::default(),
                        feedback: Box::default(),
                    });
                    true
                } else {
                    false
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("2"));
        }
    }

    fn push_fillinsol_case(&mut self, case: FillInSolOption) {
        if !self
            .with_problem(|ex| {
                if let Some(fillin) = &mut ex.fillinsol {
                    fillin.cases.push(case);
                    true
                } else {
                    false
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("3"));
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

    fn open_section(&mut self, uri: DocumentElementURI) {
        self.state_mut().narrative.push(Narrative::Section {
            title: None,
            children: Vec::new(),
            uri,
        });
    }
    fn close_section(
        &mut self,
    ) -> Option<(
        DocumentElementURI,
        Option<DocumentRange>,
        Vec<DocumentElement<Unchecked>>,
    )> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Section {
                title,
                children,
                uri,
            }) => return Some((uri, title, children)),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }

    fn open_slide(&mut self) {
        self.state_mut().narrative.push(Narrative::Slide {
            children: Vec::new(),
        });
    }
    fn close_slide(&mut self) -> Option<Vec<DocumentElement<Unchecked>>> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Slide { children }) => return Some(children),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }

    fn open_paragraph(&mut self, uri: DocumentElementURI, fors: VecSet<SymbolURI>) {
        let fors = fors.into_iter().map(|s| (s, None)).collect();
        self.state_mut()
            .narrative
            .push(Narrative::Paragraph(ParagraphState {
                uri,
                children: Vec::new(),
                fors,
                title: None,
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
    fn open_gnote(&mut self) {
        if !self
            .with_problem(|e| {
                if e.gnote.is_some() {
                    false
                } else {
                    e.gnote = Some(GnoteState {
                        answer_classes: Vec::new(),
                    });
                    true
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("4"));
        }
    }

    fn close_gnote(&mut self) -> Option<GnoteState> {
        self.with_problem(|e| e.gnote.take()).flatten()
    }

    fn open_fillinsol(&mut self, _width: Option<f32>) {
        if !self
            .with_problem(|ex| {
                if ex.fillinsol.is_some() {
                    false
                } else {
                    ex.fillinsol = Some(FillinsolState { cases: Vec::new() });
                    true
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("5"));
        }
    }

    fn close_fillinsol(&mut self) -> Option<FillinsolState> {
        self.with_problem(|e| e.fillinsol.take()).flatten()
    }

    fn open_choice_block(&mut self, multiple: bool, styles: Box<[Box<str>]>) {
        if !self
            .with_problem(|e| {
                if e.choice_block.is_some() {
                    false
                } else {
                    e.choice_block = Some(ChoiceBlockState {
                        multiple,
                        inline: styles.iter().any(|s| &**s == "inline"),
                        styles,
                        choices: Vec::new(),
                    });
                    true
                }
            })
            .unwrap_or_default()
        {
            self.add_error(FTMLError::NotInProblem("6"));
        }
    }
    fn close_choice_block(&mut self) -> Option<ChoiceBlockState> {
        self.with_problem(|e| e.choice_block.take()).flatten()
    }

    fn open_problem(&mut self, uri: DocumentElementURI) {
        self.state_mut()
            .narrative
            .push(Narrative::Problem(ProblemState::new(uri)));
    }
    fn close_problem(&mut self) -> Option<ProblemState> {
        match self.state_mut().narrative.pop() {
            Some(Narrative::Problem(state)) => return Some(state),
            Some(o) => self.state_mut().narrative.push(o),
            None => {}
        }
        None
    }
    fn add_precondition(&mut self, uri: SymbolURI, dim: CognitiveDimension) {
        let e = self.state_mut().narrative.iter_mut().rev().find_map(|e| {
            if let Narrative::Problem(e) = e {
                Some(e)
            } else {
                None
            }
        });
        if let Some(e) = e {
            e.preconditions.push((dim, uri))
        } else {
            self.add_error(FTMLError::NotInNarrative);
        }
    }
    fn add_objective(&mut self, uri: SymbolURI, dim: CognitiveDimension) {
        let e = self.state_mut().narrative.iter_mut().rev().find_map(|e| {
            if let Narrative::Problem(e) = e {
                Some(e)
            } else {
                None
            }
        });
        if let Some(e) = e {
            e.objectives.push((dim, uri))
        } else {
            self.add_error(FTMLError::NotInNarrative);
        }
    }
    fn open_decl(&mut self) {
        self.state_mut()
            .content
            .push(Content::Symdecl { df: None, tp: None });
    }
    fn close_decl(&mut self) -> Option<(Option<Term>, Option<Term>)> {
        match self.state_mut().content.pop() {
            Some(Content::Symdecl { df, tp }) => return Some((tp, df)),
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        None
    }
    fn open_notation(&mut self) {
        self.state_mut()
            .narrative
            .push(Narrative::Notation(NotationState {
                attribute_index: 0,
                inner_index: 0,
                is_text: false,
                components: Box::default(),
                op: None,
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
        self.state_mut()
            .content
            .push(Content::Args(Vec::new(), None));
    }
    fn close_args(&mut self) -> (Vec<Arg>, Option<Term>) {
        match self.state_mut().content.pop() {
            Some(Content::Args(args, head)) => {
                let mut ret = Vec::new();
                let mut iter = args.into_iter();
                while let Some(Some((a, m))) = iter.next() {
                    ret.push(match a.close(m) {
                        Ok(a) => a,
                        Err(a) => {
                            self.add_error(FTMLError::IncompleteArgs(1));
                            a
                        }
                    });
                }
                for e in iter {
                    if e.is_some() {
                        self.add_error(FTMLError::IncompleteArgs(2));
                    }
                }
                return (ret, head);
            }
            Some(o) => self.state_mut().content.push(o),
            None => {}
        }
        (Vec::new(), None)
    }

    fn get_narrative_uri(&self) -> NarrativeURI {
        self.state()
            .narrative
            .iter()
            .rev()
            .find_map(|t| match t {
            Narrative::Container(uri,_) => Some(uri.as_narrative().owned()),
            Narrative::Paragraph(ParagraphState { uri, .. }) |
            Narrative::Problem(ProblemState { uri,.. }) |
            //Narrative::Slide{uri,..} |
            Narrative::Section{uri,..} => Some(uri.as_narrative().owned()),
            Narrative::Notation(_) | Narrative::Slide{..} => None
        })
            .unwrap_or_else(|| unreachable!())
    }

    fn add_definiendum(&mut self, uri: SymbolURI) {
        for n in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Paragraph(ParagraphState { fors, .. }) = n {
                fors.get_or_insert_mut(uri, || None);
                return;
            }
        }
        self.add_error(FTMLError::NotInNarrative);
    }

    fn get_content_uri(&self) -> Option<&ModuleURI> {
        self.state().content.iter().rev().find_map(|t| match t {
            Content::Container(uri, _) => Some(uri),
            _ => None,
        })
    }

    fn add_module(&mut self, module: OpenModule<Unchecked>) {
        self.state_mut().modules.push(module);
    }

    #[inline]
    fn new_id(&mut self, prefix: Cow<'static, str>) -> Box<str> {
        self.state_mut().ids.new_id(prefix)
    }

    fn in_notation(&self) -> bool {
        self.state()
            .narrative
            .iter()
            .rev()
            .any(|s| matches!(s, Narrative::Notation(_)))
    }
    fn in_term(&self) -> bool {
        self.state().in_term
    }
    fn set_in_term(&mut self, b: bool) {
        self.state_mut().in_term = b
    }

    fn add_document_element(&mut self, elem: DocumentElement<Unchecked>) {
        for narr in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Container(_, c) = narr {
                c.push(elem);
                return;
            }
            if let Narrative::Paragraph(ParagraphState { children, .. })
            | Narrative::Problem(ProblemState { children, .. })
            | Narrative::Section { children, .. }
            | Narrative::Slide { children, .. } = narr
            {
                children.push(elem);
                return;
            }
        }
        unreachable!()
    }
    fn add_title(&mut self, ttl: DocumentRange) -> Result<(), DocumentRange> {
        for narr in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Paragraph(ParagraphState { title, .. })
            | Narrative::Problem(ProblemState { title, .. })
            | Narrative::Section { title, .. } = narr
            {
                *title = Some(ttl);
                return Ok(());
            }
        }
        Err(ttl)
    }

    /// ### Errors
    fn add_content_element(
        &mut self,
        elem: OpenDeclaration<Unchecked>,
    ) -> Result<(), OpenDeclaration<Unchecked>> {
        for cont in self.state_mut().content.iter_mut().rev() {
            if let Content::Container(_, c) = cont {
                c.push(elem);
                return Ok(());
            }
        }
        Err(elem)
    }
    fn add_notation(
        &mut self,
        NotationSpec {
            components,
            attribute_index,
            inner_index,
            is_text,
        }: NotationSpec,
    ) -> Result<(), NotationSpec> {
        if let Some(Narrative::Notation(NotationState {
            components: comps,
            attribute_index: idx,
            inner_index: iidx,
            is_text: text,
            ..
        })) = self.state_mut().narrative.last_mut()
        {
            *comps = components;
            *iidx = inner_index;
            *idx = attribute_index;
            *text = is_text;
            Ok(())
        } else {
            Err(NotationSpec {
                attribute_index,
                inner_index,
                is_text,
                components,
            })
        }
    }
    fn add_op_notation(&mut self, op: OpNotation) -> Result<(), OpNotation> {
        if let Some(Narrative::Notation(NotationState { op: ops, .. })) =
            self.state_mut().narrative.last_mut()
        {
            *ops = Some(op);
            Ok(())
        } else {
            Err(op)
        }
    }
    fn add_type(&mut self, tm: Term) -> Result<(), Term> {
        match self.state_mut().content.last_mut() {
            Some(Content::Symdecl { tp, .. }) => *tp = Some(tm),
            _ => return Err(tm),
        }
        Ok(())
    }
    /// #### Errors
    fn add_term(&mut self, symbol: Option<SymbolURI>, tm: Term) -> Result<(), Term> {
        if symbol.is_none() {
            match self.state_mut().content.last_mut() {
                Some(Content::Symdecl { df, .. }) => {
                    *df = Some(tm);
                    return Ok(());
                }
                Some(Content::Args(_, o) | Content::SingleTerm(o)) => {
                    *o = Some(tm);
                    return Ok(());
                }
                _ => (),
            }
        }
        for e in self.state_mut().narrative.iter_mut().rev() {
            if let Narrative::Paragraph(ParagraphState { fors, .. }) = e {
                if let Some(symbol) = symbol {
                    fors.insert(symbol, Some(tm));
                    return Ok(());
                }
                if fors.0.len() == 1 {
                    fors.0.last_mut().unwrap_or_else(|| unreachable!()).1 = Some(tm);
                    return Ok(());
                }
            }
        }
        Err(tm)
    }

    fn add_arg(
        &mut self,
        (idx, maybe_ls): (u8, Option<u8>),
        tm: Term,
        mode: ArgMode,
    ) -> Result<(), ()> {
        if let Some(Content::Args(v, _)) = self.state_mut().content.last_mut() {
            if v.len() <= idx as usize {
                v.resize(idx as usize + 1, None);
            }
            let tl = v
                .get_mut((idx - 1) as usize)
                .unwrap_or_else(|| unreachable!());
            if let Some(idx) = maybe_ls {
                if tl.is_none() {
                    *tl = Some((TermOrList::List(vec![]), mode));
                }
                if let Some((TermOrList::List(ls), _)) = tl {
                    if ls.len() <= idx as usize {
                        ls.resize(idx as usize + 1, None);
                    }
                    let tl = ls
                        .get_mut((idx - 1) as usize)
                        .unwrap_or_else(|| unreachable!());
                    *tl = Some(tm);
                } else {
                    return Err(());
                }
            } else {
                *tl = Some((TermOrList::Term(tm), mode));
            }
            Ok(())
        } else {
            Err(())
        }
    }
}
