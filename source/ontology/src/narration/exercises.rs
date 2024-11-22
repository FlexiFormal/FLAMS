use std::{fmt::Display, str::FromStr};

use crate::{
    uris::{DocumentElementURI, SymbolURI}, CheckingState, DocumentRange
};

use super::{DocumentElement, LazyDocRef};

#[derive(Debug)]
pub struct Exercise<State:CheckingState> {
    pub sub_exercise: bool,
    pub uri: DocumentElementURI,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: State::Seq<LazyDocRef<Box<str>>>,
    pub hints: State::Seq<LazyDocRef<Box<str>>>,
    pub notes: State::Seq<LazyDocRef<Box<str>>>,
    pub gnotes: State::Seq<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: State::Seq<DocumentElement<State>>,
    pub styles:Box<[Box<str>]>,
    pub preconditions: State::Seq<(CognitiveDimension, SymbolURI)>,
    pub objectives: State::Seq<(CognitiveDimension, SymbolURI)>,
}

crate::serde_impl!{
    struct Exercise[
        sub_exercise,uri,range,autogradable,points,solutions,
        hints,notes,gnotes,title,children,styles,preconditions,
        objectives
    ]
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
        use crate::rdf::NamedNodeRef;
        use CognitiveDimension::*;
        match self {
            Remember => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#remember"),
            Understand => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#understand"),
            Apply => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#apply"),
            Analyze => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#analyze"),
            Evaluate => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#evaluate"),
            Create => NamedNodeRef::new_unchecked("http://mathhub.info/ulo#create"),
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
