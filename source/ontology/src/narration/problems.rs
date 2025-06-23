use flams_utils::CSS;
use smallvec::SmallVec;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use crate::{
    uris::{DocumentElementURI, Name, SymbolURI},
    Checked, CheckingState, DocumentRange,
};

use super::{DocumentElement, LazyDocRef, NarrationTrait};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug)]
pub struct Problem<State: CheckingState> {
    pub sub_problem: bool,
    pub uri: DocumentElementURI,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: LazyDocRef<Solutions>, //State::Seq<SolutionData>,
    pub gnotes: State::Seq<LazyDocRef<GradingNote>>,
    pub hints: State::Seq<DocumentRange>,
    pub notes: State::Seq<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: State::Seq<DocumentElement<State>>,
    pub styles: Box<[Name]>,
    pub preconditions: State::Seq<(CognitiveDimension, SymbolURI)>,
    pub objectives: State::Seq<(CognitiveDimension, SymbolURI)>,
}

#[cfg(not(feature = "wasm"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Solutions(Box<[SolutionData]>);

#[cfg(feature = "wasm")]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[wasm_bindgen]
pub struct Solutions(Box<[SolutionData]>);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Solutions {
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn from_jstring(s: &str) -> Option<Self> {
        use flams_utils::Hexable;
        Self::from_hex(s).ok()
    }
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_jstring(&self) -> Option<String> {
        use flams_utils::Hexable;
        self.as_hex().ok()
    }

    #[inline]
    pub fn from_solutions(solutions: Box<[SolutionData]>) -> Self {
        Self(solutions)
    }

    #[inline]
    pub fn to_solutions(&self) -> Box<[SolutionData]> {
        self.0.clone()
    }

    #[must_use]
    #[inline]
    pub fn check_response(&self, response: &ProblemResponse) -> Option<ProblemFeedback> {
        self.check(response)
    }

    #[must_use]
    #[inline]
    pub fn default_feedback(&self) -> ProblemFeedback {
        self.default()
    }
}

impl crate::Resourcable for Solutions {}
impl Solutions {
    #[must_use]
    pub fn default(&self) -> ProblemFeedback {
        let mut solutions = SmallVec::new();
        let mut data = SmallVec::new();
        for sol in self.0.iter() {
            match sol {
                SolutionData::Solution { html, .. } => solutions.push(html.clone()),
                SolutionData::ChoiceBlock(ChoiceBlock {
                    multiple: false,
                    choices,
                    ..
                }) => data.push(CheckedResult::SingleChoice {
                    selected: None,
                    choices: choices
                        .iter()
                        .enumerate()
                        .map(|(_, c)| BlockFeedback {
                            is_correct: c.correct,
                            verdict_str: c.verdict.to_string(),
                            feedback: c.feedback.to_string(),
                        })
                        .collect(),
                }),
                SolutionData::ChoiceBlock(ChoiceBlock { choices, .. }) => {
                    data.push(CheckedResult::MultipleChoice {
                        selected: choices.iter().map(|_| false).collect(),
                        choices: choices
                            .iter()
                            .enumerate()
                            .map(|(_, c)| BlockFeedback {
                                is_correct: c.correct,
                                verdict_str: c.verdict.to_string(),
                                feedback: c.feedback.to_string(),
                            })
                            .collect(),
                    })
                }
                SolutionData::FillInSol(f) => {
                    let mut options = SmallVec::new();
                    for o in f.opts.iter() {
                        match o {
                            FillInSolOption::Exact {
                                value,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::Exact(value.to_string()),
                            }),
                            FillInSolOption::NumericalRange {
                                from,
                                to,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::NumRange {
                                    from: *from,
                                    to: *to,
                                },
                            }),
                            FillInSolOption::Regex {
                                regex,
                                verdict,
                                feedback,
                            } => options.push(FillinFeedback {
                                is_correct: *verdict,
                                feedback: feedback.to_string(),
                                kind: FillinFeedbackKind::Regex(regex.as_str().to_string()),
                            }),
                        }
                    }
                    data.push(CheckedResult::FillinSol {
                        matching: None,
                        options,
                        text: String::new(),
                    });
                }
            }
        }

        ProblemFeedback {
            correct: false,
            solutions,
            data,
            score_fraction: 0.0,
        }
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub fn check(&self, response: &ProblemResponse) -> Option<ProblemFeedback> {
        //println!("Here: {self:?}\n{response:?}");
        fn next_sol<'a>(
            solutions: &mut SmallVec<Box<str>, 1>,
            datas: &mut impl Iterator<Item = &'a SolutionData>,
        ) -> Option<&'a SolutionData> {
            loop {
                match datas.next() {
                    None => return None,
                    Some(SolutionData::Solution { html, .. }) => solutions.push(html.clone()),
                    Some(c) => return Some(c),
                }
            }
        }
        let mut correct = true;
        let mut pts: f32 = 0.0;
        let mut total: f32 = 0.0;
        let mut solutions = SmallVec::new();
        let mut data = SmallVec::new();
        let mut datas = self.0.iter();

        for response in &response.responses {
            total += 1.0;
            let sol = next_sol(&mut solutions, &mut datas)?;
            match (response, sol) {
                (
                    ProblemResponseType::SingleChoice { value: selected },
                    SolutionData::ChoiceBlock(ChoiceBlock {
                        multiple: false,
                        choices,
                        ..
                    }),
                ) => data.push(CheckedResult::SingleChoice {
                    selected: *selected,
                    choices: choices
                        .iter()
                        .enumerate()
                        .map(
                            |(
                                i,
                                Choice {
                                    correct: cr,
                                    verdict,
                                    feedback,
                                },
                            )| {
                                if selected.is_some_and(|j| j as usize == i) {
                                    correct = correct && *cr;
                                    if *cr {
                                        pts += 1.0;
                                    }
                                }
                                BlockFeedback {
                                    is_correct: *cr,
                                    verdict_str: verdict.to_string(),
                                    feedback: feedback.to_string(),
                                }
                            },
                        )
                        .collect(),
                }),
                (
                    ProblemResponseType::MultipleChoice { value: selected },
                    SolutionData::ChoiceBlock(ChoiceBlock {
                        multiple: true,
                        choices,
                        ..
                    }),
                ) => {
                    if selected.len() != choices.len() {
                        return None;
                    }
                    let mut corrects = 0;
                    let mut falses = 0;
                    data.push(CheckedResult::MultipleChoice {
                        selected: selected.clone(),
                        choices: choices
                            .iter()
                            .enumerate()
                            .map(
                                |(
                                    i,
                                    Choice {
                                        correct: cr,
                                        verdict,
                                        feedback,
                                    },
                                )| {
                                    if *cr == selected[i] {
                                        corrects += 1;
                                    } else {
                                        falses += 1;
                                    }
                                    correct = correct && (selected[i] == *cr);
                                    BlockFeedback {
                                        is_correct: *cr,
                                        verdict_str: verdict.to_string(),
                                        feedback: feedback.to_string(),
                                    }
                                },
                            )
                            .collect(),
                    });
                    if selected.iter().any(|b| *b) {
                        pts += ((corrects as f32 - falses as f32) / choices.len() as f32).max(0.0);
                    }
                }
                (ProblemResponseType::Fillinsol { value: s }, SolutionData::FillInSol(f)) => {
                    let mut fill_correct = None;
                    let mut matching = None;
                    let mut options = SmallVec::new();
                    for (i, o) in f.opts.iter().enumerate() {
                        match o {
                            FillInSolOption::Exact {
                                value: string,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() && &**string == s.as_str() {
                                    if *verdict {
                                        pts += 1.0;
                                    }
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::Exact(string.to_string()),
                                });
                            }
                            FillInSolOption::NumericalRange {
                                from,
                                to,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() {
                                    let num = if s.contains('.') {
                                        s.parse::<f32>().ok()
                                    } else {
                                        s.parse::<i32>().ok().map(|i| i as f32)
                                    };
                                    if let Some(f) = num {
                                        if !from.is_some_and(|v| f < v)
                                            && !to.is_some_and(|v| f > v)
                                        {
                                            if *verdict {
                                                pts += 1.0;
                                            }
                                            fill_correct = Some(*verdict);
                                            matching = Some(i);
                                        }
                                    }
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::NumRange {
                                        from: *from,
                                        to: *to,
                                    },
                                });
                            }
                            FillInSolOption::Regex {
                                regex,
                                verdict,
                                feedback,
                            } => {
                                if fill_correct.is_none() && regex.is_match(s) {
                                    if *verdict {
                                        pts += 1.0;
                                    }
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback {
                                    is_correct: *verdict,
                                    feedback: feedback.to_string(),
                                    kind: FillinFeedbackKind::Regex(regex.as_str().to_string()),
                                });
                            }
                        }
                    }
                    correct = correct && fill_correct.unwrap_or_default();
                    data.push(CheckedResult::FillinSol {
                        matching,
                        options,
                        text: s.to_string(),
                    });
                }
                _ => return None,
            }
        }

        if next_sol(&mut solutions, &mut datas).is_some() {
            return None;
        }

        Some(ProblemFeedback {
            correct,
            solutions,
            data,
            score_fraction: pts / total,
        })
    }
}

crate::serde_impl! {
    struct Problem[
        sub_problem,uri,range,autogradable,points,solutions,gnotes,
        hints,notes,title,children,styles,preconditions,
        objectives
    ]
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SolutionData {
    Solution {
        html: Box<str>,
        answer_class: Option<Box<str>>,
    },
    ChoiceBlock(ChoiceBlock),
    FillInSol(FillInSol),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ChoiceBlock {
    pub multiple: bool,
    pub inline: bool,
    pub range: DocumentRange,
    pub styles: Box<[Box<str>]>,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Choice {
    pub correct: bool,
    pub verdict: Box<str>,
    pub feedback: Box<str>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillInSol {
    pub width: Option<f32>,
    pub opts: Vec<FillInSolOption>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillInSolOption {
    Exact {
        value: Box<str>,
        verdict: bool,
        feedback: Box<str>,
    },
    NumericalRange {
        from: Option<f32>,
        to: Option<f32>,
        verdict: bool,
        feedback: Box<str>,
    },
    Regex {
        regex: flams_utils::regex::Regex,
        verdict: bool,
        feedback: Box<str>,
    },
}
impl FillInSolOption {
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_values(kind: &str, value: &str, verdict: bool) -> Option<Self> {
        match kind {
            "exact" => Some(Self::Exact {
                value: value.to_string().into(),
                verdict,
                feedback: String::new().into(),
            }),
            "numrange" => {
                let (s, neg) = value
                    .strip_prefix('-')
                    .map_or((value, false), |s| (s, true));
                let (from, to) = if let Some((from, to)) = s.split_once('-') {
                    (from, to)
                } else {
                    ("", s)
                };
                let from = if from.contains('.') {
                    Some(f32::from_str(from).ok()?)
                } else if from.is_empty() {
                    None
                } else {
                    Some(i128::from_str(from).ok()? as _)
                };
                let from = if neg { from.map(|f| -f) } else { from };
                let to = if to.contains('.') {
                    Some(f32::from_str(to).ok()?)
                } else if to.is_empty() {
                    None
                } else {
                    Some(i128::from_str(to).ok()? as _)
                };
                Some(Self::NumericalRange {
                    from,
                    to,
                    verdict,
                    feedback: String::new().into(),
                })
            }
            "regex" => Some(Self::Regex {
                regex: flams_utils::regex::Regex::new(
                    value, //&format!("^{value}?")
                )
                .ok()?,
                verdict,
                feedback: String::new().into(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GradingNote {
    pub html: Box<str>,
    pub answer_classes: Vec<AnswerClass>,
}
impl crate::Resourcable for GradingNote {}

#[cfg(not(feature = "wasm"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProblemFeedback {
    pub correct: bool,
    pub solutions: SmallVec<Box<str>, 1>,
    pub data: SmallVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(feature = "wasm")]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[wasm_bindgen]
pub struct ProblemFeedback {
    pub correct: bool,
    #[wasm_bindgen(skip)]
    pub solutions: SmallVec<Box<str>, 1>,
    //#[cfg_attr(feature="wasm", tsify(type = "CheckedResult[]"))]
    #[wasm_bindgen(skip)]
    pub data: SmallVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(feature = "wasm")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct ProblemFeedbackJson {
    pub correct: bool,
    #[cfg_attr(feature = "wasm", tsify(type = "string[]"))]
    pub solutions: SmallVec<Box<str>, 1>,
    #[cfg_attr(feature = "wasm", tsify(type = "CheckedResult[]"))]
    pub data: SmallVec<CheckedResult, 4>,
    pub score_fraction: f32,
}

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ProblemFeedback {
    #[must_use]
    pub fn from_jstring(s: &str) -> Option<Self> {
        use flams_utils::Hexable;
        Self::from_hex(s).ok()
    }

    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_jstring(&self) -> Option<String> {
        use flams_utils::Hexable;
        self.as_hex().ok()
    }

    #[inline]
    pub fn from_json(
        ProblemFeedbackJson {
            correct,
            solutions,
            data,
            score_fraction,
        }: ProblemFeedbackJson,
    ) -> Self {
        Self {
            correct,
            solutions,
            data,
            score_fraction,
        }
    }

    #[inline]
    pub fn to_json(&self) -> ProblemFeedbackJson {
        let Self {
            correct,
            solutions,
            data,
            score_fraction,
        } = self.clone();
        ProblemFeedbackJson {
            correct,
            solutions,
            data,
            score_fraction,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BlockFeedback {
    pub is_correct: bool,
    pub verdict_str: String,
    pub feedback: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillinFeedback {
    pub is_correct: bool,
    pub feedback: String,
    pub kind: FillinFeedbackKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillinFeedbackKind {
    Exact(String),
    NumRange { from: Option<f32>, to: Option<f32> },
    Regex(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum CheckedResult {
    SingleChoice {
        selected: Option<u16>,
        #[cfg_attr(feature = "wasm", tsify(type = "BlockFeedback[]"))]
        choices: SmallVec<BlockFeedback, 4>,
    },
    MultipleChoice {
        #[cfg_attr(feature = "wasm", tsify(type = "boolean[]"))]
        selected: SmallVec<bool, 8>,
        #[cfg_attr(feature = "wasm", tsify(type = "BlockFeedback[]"))]
        choices: SmallVec<BlockFeedback, 4>,
    },
    FillinSol {
        matching: Option<usize>,
        text: String,
        #[cfg_attr(feature = "wasm", tsify(type = "FillinFeedback[]"))]
        options: SmallVec<FillinFeedback, 4>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
//#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ProblemResponse {
    pub uri: DocumentElementURI,
    #[cfg_attr(feature = "wasm", tsify(type = "ProblemResponseType[]"))]
    pub responses: SmallVec<ProblemResponseType, 4>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
/// Either a list of booleans (multiple choice), a single integer (single choice),
/// or a string (fill-in-the-gaps)
pub enum ProblemResponseType {
    MultipleChoice {
        #[cfg_attr(feature = "wasm", tsify(type = "boolean[]"))]
        value: SmallVec<bool, 8>,
    },
    SingleChoice {
        value: Option<u16>,
    },
    Fillinsol {
        #[serde(rename = "value")]
        value: String,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AnswerClass {
    pub id: Box<str>,
    pub feedback: Box<str>,
    pub kind: AnswerKind,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum AnswerKind {
    Class(f32),
    Trait(f32),
}
impl FromStr for AnswerKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::cast_precision_loss)]
        fn num(s: &str) -> Result<f32, ()> {
            if s.contains('.') {
                s.parse().map_err(|_| ())
            } else {
                let i: Result<i32, ()> = s.parse().map_err(|_| ());
                i.map(|i| i as f32)
            }
        }
        let s = s.trim();
        s.strip_prefix('+').map_or_else(
            || {
                s.strip_prefix('-').map_or_else(
                    || num(s).map(AnswerKind::Class),
                    |s| num(s).map(|f| Self::Trait(-f)),
                )
            },
            |s| num(s).map(AnswerKind::Trait),
        )
    }
}

impl NarrationTrait for Problem<Checked> {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.children
    }
    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self>
    where
        Self: Sized,
    {
        if let DocumentElement::Problem(e) = e {
            Some(e)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Quiz {
    pub css: Vec<CSS>,
    pub title: Option<String>,
    pub elements: Vec<QuizElement>,
    pub solutions: HashMap<DocumentElementURI, String>,
    pub answer_classes: HashMap<DocumentElementURI, Vec<AnswerClass>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum QuizElement {
    Section {
        title: String,
        elements: Vec<QuizElement>,
    },
    Problem(QuizProblem),
    Paragraph {
        html: String,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct QuizProblem {
    pub html: String,
    pub title_html: Option<String>,
    pub uri: DocumentElementURI,
    //pub solution:String,//Solutions,
    pub total_points: Option<f32>,
    //pub is_sub_problem:bool,
    pub preconditions: Vec<(CognitiveDimension, SymbolURI)>,
    pub objectives: Vec<(CognitiveDimension, SymbolURI)>,
}
