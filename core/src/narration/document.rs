use crate::utils::sourcerefs::{ByteOffset, SourceRange};
use crate::uris::documents::DocumentURI;
use std::fmt::Display;
use crate::uris::ContentURI;
use crate::uris::symbols::SymbolURI;

#[derive(Debug, Clone)]
pub enum CSS {
    Link(String),
    Inline(String),
}

#[derive(Debug, Clone)]
pub struct Document {
    pub language: Language,
    pub uri: DocumentURI,
    pub title: String,
    pub css: Vec<CSS>,
    pub elements: Vec<DocumentElement>,
}

#[derive(Debug, Clone)]
pub enum DocumentElement {
    SetSectionLevel(SectionLevel),
    Section {
        range: SourceRange<ByteOffset>,
        id: String,
        level: SectionLevel,
        title: Option<(String, SourceRange<ByteOffset>)>,
        children: Vec<DocumentElement>,
    },
    Module {
        range: SourceRange<ByteOffset>,
        name: String,
        children: Vec<DocumentElement>,
    },
    InputRef {
        id: String,
        target: DocumentURI,
        range: SourceRange<ByteOffset>,
    },
    Definiendum {
        uri:SymbolURI,
        range: SourceRange<ByteOffset>,
    },
    Symref {
        uri:SymbolURI,
        range: SourceRange<ByteOffset>,
        notation:Option<String>,
    },
    LogicalParagraph {
        kind:&'static str,
        definition_like:bool,
        id: String,
        inline:bool,
        title: Option<(String, SourceRange<ByteOffset>)>,
        fors: Vec<ContentURI>,
        range: SourceRange<ByteOffset>,
        styles:Vec<String>,
        children: Vec<DocumentElement>,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
}
impl TryFrom<u8> for SectionLevel {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(SectionLevel::Part),
            1 => Ok(SectionLevel::Chapter),
            2 => Ok(SectionLevel::Section),
            3 => Ok(SectionLevel::Subsection),
            4 => Ok(SectionLevel::Subsubsection),
            5 => Ok(SectionLevel::Paragraph),
            6 => Ok(SectionLevel::Subparagraph),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Language {
    English,
    German,
    French,
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "en"),
            Language::German => write!(f, "de"),
            Language::French => write!(f, "fr"),
        }
    }
}
impl Into<&'static str> for Language {
    fn into(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
        }
    }
}
impl TryFrom<&str> for Language {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "en" => Ok(Language::English),
            "de" => Ok(Language::German),
            "fr" => Ok(Language::French),
            _ => Err(()),
        }
    }
}
