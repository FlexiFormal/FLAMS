use std::{fmt::Display, str::FromStr};

use smallvec::SmallVec;

use crate::{
    uris::{DocumentElementURI, SymbolURI}, Checked, CheckingState, DocumentRange, Resourcable, Unchecked
};

use super::{DocumentElement, LazyDocRef, NarrationTrait};

#[derive(Debug)]
pub struct Exercise<State:CheckingState> {
    pub sub_exercise: bool,
    pub uri: DocumentElementURI,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: State::Seq<SolutionData<State>>,
    pub hints: State::Seq<DocumentRange>,
    pub notes: State::Seq<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: State::Seq<DocumentElement<State>>,
    pub styles:Box<[Box<str>]>,
    pub preconditions: State::Seq<(CognitiveDimension, SymbolURI)>,
    pub objectives: State::Seq<(CognitiveDimension, SymbolURI)>,
}

crate::serde_impl!{
    struct Exercise[
        sub_exercise,uri,range,autogradable,points,solutions,
        hints,notes,title,children,styles,preconditions,
        objectives
    ]
}

#[derive(Debug)]
pub enum SolutionData<State:CheckingState> {
    Solution{
        html:LazyDocRef<Box<str>>,
        answer_class:Option<Box<str>>
    },
    Grading(GradingNote<State>),
    ChoiceBlock(ChoiceBlock<State>)
}

impl SolutionData<Unchecked> {
    #[must_use]
    pub fn close(self) -> SolutionData<Checked> {
        match self {
            Self::Solution{html,answer_class} => SolutionData::Solution{html,answer_class},
            Self::Grading(g) => SolutionData::Grading(
                GradingNote {
                    html:g.html,
                    answer_classes:g.answer_classes.into_boxed_slice()
                }
            ),
            Self::ChoiceBlock(ChoiceBlock{multiple,inline,range,styles,choices}) => SolutionData::ChoiceBlock(
                ChoiceBlock { multiple, inline, range,styles,choices:choices.into_boxed_slice() }
            )
        }
    }
}

impl Clone for SolutionData<Checked> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Solution{html,answer_class} => 
                Self::Solution{html:html.clone(), answer_class:answer_class.clone()},
            Self::Grading(g) => Self::Grading(g.clone()),
            Self::ChoiceBlock(b) =>
                Self::ChoiceBlock(b.clone())
        }
    }
}

crate::serde_impl!{mod solution_serde_impl =
    enum SolutionData{
        {0 = Solution{html,answer_class}}
        {1 = Grading(g)}
        {2 = ChoiceBlock(b)}
    }
}

#[derive(Debug)]
pub struct ChoiceBlock<State:CheckingState> { 
    pub multiple:bool,
    pub inline:bool,
    pub range:DocumentRange,
    pub styles:Box<[Box<str>]>,
    pub choices:State::Seq<Choice>
}

impl Clone for ChoiceBlock<Checked> {
    #[inline]
    fn clone(&self) -> Self {
        Self{
            multiple:self.multiple,
            inline:self.inline,
            range:self.range,
            styles:self.styles.clone(),
            choices:self.choices.clone()
        }
    }
}

crate::serde_impl!{mod choice_serde_impl =
    struct ChoiceBlock[multiple,inline,range,styles,choices]
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct Choice {
    pub correct:bool,
    pub verdict:Box<str>,
    pub feedback:Box<str>
}

#[derive(Debug)]
pub struct GradingNote<State:CheckingState> {
    pub html:LazyDocRef<Box<str>>,
    pub answer_classes:State::Seq<AnswerClass>
}

impl Clone for GradingNote<Checked> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            html:self.html.clone(),
            answer_classes:self.answer_classes.clone()
        }
    }
}

crate::serde_impl!{mod grading_note_serde_impl =
    struct GradingNote[html,answer_classes]
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExerciseResponse {
    pub uri:DocumentElementURI,
    pub responses:SmallVec<ExerciseResponseType,4>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature="serde", serde(untagged))]
pub enum ExerciseResponseType {
    MultipleChoice(SmallVec<bool,8>),
    SingleChoice(u16)
}



#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnswerClass {
    pub id:Box<str>,
    pub feedback:Box<str>,
    pub kind:AnswerKind
}

#[derive(Debug,Clone,Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnswerKind {
    Class(f32),
    Trait(f32),
}
impl FromStr for AnswerKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::cast_precision_loss)]
        fn num(s:&str) -> Result<f32,()> {
            if s.contains('.') {
                s.parse().map_err(|_| ())
            } else {
                let i: Result<i32,()> = s.parse().map_err(|_| ());
                i.map(|i| i as f32)
            }
        }
        let s = s.trim();
        s.strip_prefix('+').map_or_else(
            || s.strip_prefix('-').map_or_else(
                || num(s).map(AnswerKind::Class),
                |s| num(s).map(|f| Self::Trait(-f))
            ),
            |s| num(s).map(AnswerKind::Trait)
        )
    }
}

impl NarrationTrait for Exercise<Checked> {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.children
    }
    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        if let DocumentElement::Exercise(e) = e {Some(e)} else {None}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CognitiveDimension {
    Remember,
    Understand,
    Apply,
    Analyze,
    Evaluate,
    Create,
}
impl CognitiveDimension {
    #[cfg(feature = "rdf")]
    #[must_use]
    pub const fn to_iri(&self) -> crate::rdf::NamedNodeRef {
        use crate::rdf::ontologies::ulo2;
        use CognitiveDimension::*;
        match self {
            Remember => ulo2::REMEMBER,
            Understand => ulo2::UNDERSTAND,
            Apply => ulo2::APPLY,
            Analyze => ulo2::ANALYZE,
            Evaluate => ulo2::EVALUATE,
            Create => ulo2::CREATE,
        }
    }
}
impl Display for CognitiveDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CognitiveDimension::*;
        write!(
            f,
            "{}",
            match self {
                Remember => "remember",
                Understand => "understand",
                Apply => "apply",
                Analyze => "analyze",
                Evaluate => "evaluate",
                Create => "create",
            }
        )
    }
}
impl FromStr for CognitiveDimension {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CognitiveDimension::*;
        Ok(match s {
            "remember" => Remember,
            "understand" => Understand,
            "apply" => Apply,
            "analyze" | "analyse" => Analyze,
            "evaluate" => Evaluate,
            "create" => Create,
            _ => return Err(()),
        })
    }
}
