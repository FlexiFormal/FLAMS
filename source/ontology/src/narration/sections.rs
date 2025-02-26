use std::fmt::Display;

use crate::{uris::DocumentElementURI, Checked, CheckingState, DocumentRange};

use super::{DocumentElement, NarrationTrait};

/*
#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedSection {
    pub range: DocumentRange,
    pub uri: DocumentElementURI,
    pub level: SectionLevel,
    pub title: Option<DocumentRange>,
    pub children: Vec<UncheckedDocumentElement>,
}
    */

#[derive(Debug)]
pub struct Section<State:CheckingState> {
    pub range: DocumentRange,
    pub uri: DocumentElementURI,
    pub level: SectionLevel,
    pub title: Option<DocumentRange>,
    pub children: State::Seq<DocumentElement<State>>,
}

crate::serde_impl!{
    struct Section[range,uri,level,title,children]
}

impl NarrationTrait for Section<Checked> {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.children
    }

    #[inline]
    fn from_element(e: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        if let DocumentElement::Section(e) = e {Some(e)} else {None}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
}
impl Ord for SectionLevel {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let su : u8 = (*self).into();
        let ou : u8 = (*other).into();
        su.cmp(&ou)
    }
}
impl PartialOrd for SectionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl SectionLevel {
    #[must_use]
    pub const fn inc(self) -> Self {
        match self {
            Self::Part => Self::Chapter,
            Self::Chapter => Self::Section,
            Self::Section => Self::Subsection,
            Self::Subsection => Self::Subsubsection,
            Self::Subsubsection => Self::Paragraph,
            _ => Self::Subparagraph,
        }
    }
}
impl Display for SectionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SectionLevel::*;
        write!(
            f,
            "{}",
            match self {
                Part => "Part",
                Chapter => "Chapter",
                Section => "Section",
                Subsection => "Subsection",
                Subsubsection => "Subsubsection",
                Paragraph => "Paragraph",
                Subparagraph => "Subparagraph",
            }
        )
    }
}
impl TryFrom<u8> for SectionLevel {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        use SectionLevel::*;
        match value {
            0 => Ok(Part),
            1 => Ok(Chapter),
            2 => Ok(Section),
            3 => Ok(Subsection),
            4 => Ok(Subsubsection),
            5 => Ok(Paragraph),
            6 => Ok(Subparagraph),
            _ => Err(()),
        }
    }
}
impl From<SectionLevel> for u8 {
    fn from(s: SectionLevel) -> Self {
        use SectionLevel::*;
        match s {
            Part => 0,
            Chapter => 1,
            Section => 2,
            Subsection => 3,
            Subsubsection => 4,
            Paragraph => 5,
            Subparagraph => 6,
        }
    }
}
