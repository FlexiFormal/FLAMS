use crate::utils::sourcerefs::{ByteOffset, SourceRange};
use crate::uris::documents::DocumentURI;
use std::fmt::Display;
use std::path::Path;
use arrayvec::ArrayVec;
use crate::content::{ArgSpec, AssocType, Term};
use crate::uris::ContentURI;
use crate::uris::modules::ModuleURI;
use crate::uris::symbols::SymbolURI;
use crate::utils::VecMap;

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    pub language: Language,
    pub uri: DocumentURI,
    pub title: String,
    pub elements: Vec<DocumentElement>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Section {
    pub range: SourceRange<ByteOffset>,
    pub id: String,
    pub level: SectionLevel,
    pub title: Option<(String, SourceRange<ByteOffset>)>,
    pub children: Vec<DocumentElement>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentModule {
    pub range: SourceRange<ByteOffset>,
    pub name: String,
    pub children: Vec<DocumentElement>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentMathStructure {
    pub range: SourceRange<ByteOffset>,
    pub name: String,
    pub children: Vec<DocumentElement>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentReference {
    pub range: SourceRange<ByteOffset>,
    pub id: String,
    pub target: DocumentURI,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum StatementKind {
    Definition,Assertion,Paragraph,Proof,SubProof,Example
}

impl StatementKind {
    pub fn from_shtml(s:&str) -> Option<StatementKind> {
        Some(match s {
            "shtml:definition" => StatementKind::Definition,
            "shtml:assertion" => StatementKind::Assertion,
            "shtml:paragraph" => StatementKind::Paragraph,
            "shtml:proof" => StatementKind::Proof,
            "shtml:subproof" => StatementKind::SubProof,
            _ => return None
        })
    }
    pub fn is_definition_like(&self,styles:&Vec<String>) -> bool {
        match &self {
            StatementKind::Definition | StatementKind::Assertion => true,
            _ => styles.iter().any(|s| s == "symdoc" || s == "decl")
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicalParagraph {
    pub kind:StatementKind,
    pub id: String,
    pub inline:bool,
    pub title: Option<(String, SourceRange<ByteOffset>)>,
    pub fors: Vec<ContentURI>,
    pub range: SourceRange<ByteOffset>,
    pub styles:Vec<String>,
    pub children: Vec<DocumentElement>,
    pub terms:VecMap<SymbolURI,Term>
}
impl LogicalParagraph {
    pub fn definition_like(&self) -> bool {
        self.kind.is_definition_like(&self.styles)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Problem {
    pub id:String,
    pub autogradable:bool,
    pub language:Language,
    pub points:Option<f32>,
    pub solution:Option<SourceRange<ByteOffset>>,
    pub hint:Option<SourceRange<ByteOffset>>,
    pub note:Option<SourceRange<ByteOffset>>,
    pub gnote:Option<SourceRange<ByteOffset>>,
    pub title:Option<(String,SourceRange<ByteOffset>)>
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proof {

}


#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentElement {
    SetSectionLevel(SectionLevel),
    Section(Section),
    Module(DocumentModule),
    MathStructure(DocumentMathStructure),
    InputRef(DocumentReference),
    VarNotation {
        name:String,
        id:String,
        precedence:isize,
        argprecs:ArrayVec<isize,9>,
        inner:Option<String>
    },
    VarDef {
        name:String,
        arity:ArgSpec,
        macroname:Option<String>,
        range:SourceRange<ByteOffset>,
        role:Option<Vec<String>>,
        tp:Option<Term>,
        df:Option<Term>,
        is_sequence:bool,
        assoctype : Option<AssocType>,
        reordering:Option<String>,
        bind:bool
    },
    Definiendum {
        uri:SymbolURI,
        range: SourceRange<ByteOffset>,
    },
    Symref {
        uri:ContentURI,
        range: SourceRange<ByteOffset>,
        notation:Option<String>,
    },
    Varref {
        name:String,
        range: SourceRange<ByteOffset>,
        notation:Option<String>,
    },
    TopTerm(Term),
    UseModule(ModuleURI),
    Paragraph(LogicalParagraph),
    Problem(Problem),
    Proof(Proof)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Language {
    English,
    German,
    French,
}
impl Language {
    pub fn from_file(path:&Path) -> Language {
        if let Some(stem) = path.file_stem().map(|s| s.to_str()).flatten() {
            if stem.ends_with(".en") { Language::English }
            else if stem.ends_with(".de") {Language::German}
            else if stem.ends_with(".fr") {Language::French}
            else {Language::English}
        } else { Language::English }
    }
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

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CSS {
    Link(String),
    Inline(String),
}

#[derive(Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HTMLDocSpec {
    pub doc:Document,
    pub html:String,
    pub head:SourceRange<ByteOffset>,
    pub body:SourceRange<ByteOffset>,
    pub notations:Vec<String>
}