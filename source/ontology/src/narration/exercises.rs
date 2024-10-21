use std::{fmt::Display, str::FromStr};

use crate::{
    uris::{DocumentElementURI, SymbolURI},
    DocumentRange,
};

use super::{DocumentElement, LazyDocRef, UncheckedDocumentElement};

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedExercise {
    pub sub_exercise: bool,
    pub range: DocumentRange,
    pub uri: DocumentElementURI,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: Vec<LazyDocRef<Box<str>>>,
    pub hints: Vec<LazyDocRef<Box<str>>>,
    pub notes: Vec<LazyDocRef<Box<str>>>,
    pub gnotes: Vec<LazyDocRef<Box<str>>>,
    pub title: Option<DocumentRange>,
    pub children: Vec<UncheckedDocumentElement>,
    pub styles:Box<[Box<str>]>,
    pub preconditions: Vec<(CognitiveDimension, SymbolURI)>,
    pub objectives: Vec<(CognitiveDimension, SymbolURI)>,
}

#[derive(Debug)]
pub struct Exercise {
    pub sub_problem: bool,
    pub uri: DocumentElementURI,
    pub range: DocumentRange,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub solutions: Box<[LazyDocRef<Box<str>>]>,
    pub hints: Box<[LazyDocRef<Box<str>>]>,
    pub notes: Box<[LazyDocRef<Box<str>>]>,
    pub gnotes: Box<[LazyDocRef<Box<str>>]>,
    pub title: Option<DocumentRange>,
    pub children: Box<[DocumentElement]>,
    pub styles:Box<[Box<str>]>,
    pub preconditions: Box<[(CognitiveDimension, SymbolURI)]>,
    pub objectives: Box<[(CognitiveDimension, SymbolURI)]>,
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
