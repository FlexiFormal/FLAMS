use std::{fmt::Display, str::FromStr};

use smallvec::SmallVec;

use crate::{
    uris::{DocumentElementURI, SymbolURI}, Checked, CheckingState, DocumentRange
};

use super::{DocumentElement, LazyDocRef, NarrationTrait};

#[derive(Debug)]
pub struct Exercise<State:CheckingState> {
    pub sub_exercise: bool,
    pub uri: DocumentElementURI,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: LazyDocRef<Solutions>,//State::Seq<SolutionData>,
    pub gnotes: State::Seq<LazyDocRef<GradingNote>>,
    pub hints: State::Seq<DocumentRange>,
    pub notes: State::Seq<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: State::Seq<DocumentElement<State>>,
    pub styles:Box<[Box<str>]>,
    pub preconditions: State::Seq<(CognitiveDimension, SymbolURI)>,
    pub objectives: State::Seq<(CognitiveDimension, SymbolURI)>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Solutions(
    #[cfg_attr(feature = "wasm", wasm_bindgen(skip))]
    pub Box<[SolutionData]>
);

#[cfg(feature="wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Solutions{
    #[inline]
    #[must_use]
    pub fn from_json(json:&str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
    #[inline]
    #[must_use]
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    #[inline]
    #[must_use]
    pub fn check_response(&self,response:&ExerciseResponse) -> Option<ExerciseFeedback> {
        self.check(response)
    }
}

impl crate::Resourcable for Solutions {}
impl Solutions {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub fn check(&self,response:&ExerciseResponse) -> Option<ExerciseFeedback> {
        fn next_sol<'a>(solutions:&mut SmallVec<Box<str>,1>,datas: &mut impl Iterator<Item=&'a SolutionData>) -> Option<&'a SolutionData> {
            loop {
                match datas.next() {
                    None => return None,
                    Some(SolutionData::Solution{html,..}) => solutions.push(html.clone()),
                    Some(c) => return Some(c)
                }
            }
        }
        let mut correct = true;
        let mut solutions = SmallVec::new();
        let mut data = SmallVec::new();
        let mut datas = self.0.iter();

        for response in &response.responses {
            let sol = next_sol(&mut solutions,&mut datas)?;
            match (response,sol) {
                (ExerciseResponseType::SingleChoice(selected),SolutionData::ChoiceBlock(ChoiceBlock {multiple:false,choices,..})) => 
                    data.push(CheckedResult::SingleChoice {
                        selected:*selected,
                        choices:choices.iter().enumerate().map(|(i,Choice{correct:cr,verdict,feedback})| {
                            if *selected as usize == i {
                                correct = correct && *cr;
                            }
                            BlockFeedback { is_correct: *cr, verdict_str: verdict.to_string(), feedback: feedback.to_string() }
                        }).collect()
                    }),
                (ExerciseResponseType::MultipleChoice(selected),SolutionData::ChoiceBlock(ChoiceBlock {multiple:true,choices,..})) => {
                    if selected.len() != choices.len() {
                        return None
                    }
                    data.push(CheckedResult::MultipleChoice { 
                        selected: selected.clone(), 
                        choices: choices.iter().enumerate().map(|(i,Choice{correct:cr,verdict,feedback})| {
                            correct = correct && (selected[i] == *cr);
                            BlockFeedback {
                                is_correct: *cr, verdict_str: verdict.to_string(), feedback: feedback.to_string() 
                            }
                        }).collect()
                    });
                }
                (ExerciseResponseType::Fillinsol(s),SolutionData::FillInSol(f)) => {
                    let mut fill_correct = None;
                    let mut matching = None;
                    let mut options = SmallVec::new();
                    for (i,o) in f.opts.iter().enumerate() {
                        match o {
                            FillInSolOption::Exact { value: string, verdict, feedback } => {
                                if fill_correct.is_none() && &**string == s.as_str() {
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback{ 
                                    is_correct:*verdict,
                                    feedback:feedback.to_string(),
                                    kind:FillinFeedbackKind::Exact(string.to_string())
                                });
                            }
                            FillInSolOption::NumericalRange { from, to, verdict, feedback } => {
                                if fill_correct.is_none() {
                                    let num = if s.contains('.') { s.parse::<f32>().ok() } else {
                                        s.parse::<i32>().ok().map(|i| i as f32)
                                    };
                                    if let Some(f) = num {
                                        if f >= *from && f <= *to {
                                            fill_correct = Some(*verdict);
                                            matching = Some(i);
                                        }
                                    }
                                }
                                options.push(FillinFeedback{ 
                                    is_correct:*verdict,
                                    feedback:feedback.to_string(),
                                    kind:FillinFeedbackKind::NumRange{from:*from,to:*to}
                                });
                            }
                            FillInSolOption::Regex { regex, verdict, feedback } => {
                                if fill_correct.is_none() && regex.0.is_match(s) {
                                    fill_correct = Some(*verdict);
                                    matching = Some(i);
                                }
                                options.push(FillinFeedback{ 
                                    is_correct:*verdict,
                                    feedback:feedback.to_string(),
                                    kind:FillinFeedbackKind::Regex(regex.0.as_str().to_string())
                                });
                            }
                        }
                    }
                    correct = correct && fill_correct.unwrap_or_default();
                    data.push(CheckedResult::FillinSol { 
                        matching, options, text: s.to_string()
                    });
                }
                _ => return None
            }
        }

        if next_sol(&mut solutions,&mut datas).is_some() {
            return None
        }

        Some(ExerciseFeedback {
            correct,solutions,data
        })
    }
}


crate::serde_impl!{
    struct Exercise[
        sub_exercise,uri,range,autogradable,points,solutions,gnotes,
        hints,notes,title,children,styles,preconditions,
        objectives
    ]
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SolutionData {
    Solution{
        html:Box<str>,
        answer_class:Option<Box<str>>
    },
    ChoiceBlock(ChoiceBlock),
    FillInSol(FillInSol)
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ChoiceBlock { 
    pub multiple:bool,
    pub inline:bool,
    pub range:DocumentRange,
    pub styles:Box<[Box<str>]>,
    pub choices:Vec<Choice>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Choice {
    pub correct:bool,
    pub verdict:Box<str>,
    pub feedback:Box<str>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillInSol {
    pub width:Option<f32>,
    pub opts: Vec<FillInSolOption>
}

#[derive(Debug,Clone)]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillinRegex(
    //#[cfg_attr(feature="wasm", tsify(type = "string"))]
    regex::Regex
);

#[cfg(feature="serde")]
mod regex_serde {
    use super::FillinRegex;
    impl serde::Serialize for FillinRegex {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
            serializer.serialize_str(self.0.as_str())
        }
    }

    impl<'de> serde::Deserialize<'de> for FillinRegex {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
            let s = String::deserialize(deserializer)?;
            regex::Regex::new(&s).map(FillinRegex).map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillInSolOption{
    Exact{
        value:Box<str>,
        verdict:bool,
        feedback:Box<str>
    },
    NumericalRange{
        from:f32,to:f32,
        verdict:bool,
        feedback:Box<str>
    },
    Regex{
        regex:FillinRegex,
        verdict:bool,
        feedback:Box<str>
    }
}
impl FillInSolOption {
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_values(kind:&str,value:&str,verdict:bool) -> Option<Self> {
        match kind {
            "exact" => Some(Self::Exact { 
                value: value.to_string().into(), 
                verdict, 
                feedback:String::new().into()
            }),
            "numrange" => {
                let (from,to) = value.split_once('-')?;
                let from = if from.contains('.') {
                    f32::from_str(from).ok()?
                } else {
                    i32::from_str(from).ok()? as _   
                };
                let to = if to.contains('.') {
                    f32::from_str(to).ok()?
                } else {
                    i32::from_str(to).ok()? as _   
                };
                Some(Self::NumericalRange { 
                    from, 
                    to, 
                    verdict, 
                    feedback:String::new().into()
                })
            },
            "regex" => Some(Self::Regex { 
                regex: FillinRegex(regex::Regex::new(
                    value
                    //&format!("^{value}?")
                ).ok()?), 
                verdict, 
                feedback:String::new().into()
            }),
            _ => None
        }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct GradingNote {
    pub html:Box<str>,
    pub answer_classes:Vec<AnswerClass>
}
impl crate::Resourcable for GradingNote {}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ExerciseFeedback {
    pub correct:bool,
    //#[cfg_attr(feature="wasm", tsify(type = "string[]"))]
    #[cfg_attr(feature = "wasm", wasm_bindgen(skip))]
    pub solutions:SmallVec<Box<str>,1>,
    //#[cfg_attr(feature="wasm", tsify(type = "CheckedResult[]"))]
    #[cfg_attr(feature = "wasm", wasm_bindgen(skip))]
    pub data:SmallVec<CheckedResult,4>
}

#[cfg(feature="wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ExerciseFeedback {
    #[inline]
    #[must_use]
    pub fn from_json(json:&str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
    #[inline]
    #[must_use]
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct BlockFeedback {
    pub is_correct:bool,
    pub verdict_str:String,
    pub feedback:String
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FillinFeedback {
    pub is_correct:bool,
    pub feedback:String,
    pub kind:FillinFeedbackKind,
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum FillinFeedbackKind {
    Exact(String),
    NumRange{from:f32,to:f32},
    Regex(String)
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
//#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
//#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum CheckedResult {
    SingleChoice{
        selected:u16,
        //#[cfg_attr(feature="wasm", tsify(type = "BlockFeedback[]"))]
        choices:SmallVec<BlockFeedback,4>
    },
    MultipleChoice{
        //#[cfg_attr(feature="wasm", tsify(type = "boolean[]"))]
        selected:SmallVec<bool,8>,
        //#[cfg_attr(feature="wasm", tsify(type = "BlockFeedback[]"))]
        choices:SmallVec<BlockFeedback,4>
    },
    FillinSol {
        matching:Option<usize>,
        text:String,
        //#[cfg_attr(feature="wasm", tsify(type = "FillinFeedback[]"))]
        options:SmallVec<FillinFeedback,4>
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
//#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ExerciseResponse {
    #[cfg_attr(feature="wasm", tsify(type = "string"))]
    pub uri:DocumentElementURI,
    #[cfg_attr(feature="wasm", tsify(type = "ExerciseResponseType[]"))]
    pub responses:SmallVec<ExerciseResponseType,4>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature="serde", serde(untagged))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
/// Either a list of booleans (multiple choice), a single integer (single choice),
/// or a string (fill-in-the-gaps)
pub enum ExerciseResponseType {
    MultipleChoice(
        #[cfg_attr(feature="wasm", tsify(type = "boolean[]"))]
        SmallVec<bool,8>
    ),
    SingleChoice(u16),
    Fillinsol(String)
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
