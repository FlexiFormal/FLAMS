use crate::utils::sourcerefs::{ByteOffset, SourceRange};
use crate::uris::documents::DocumentURI;
use std::fmt::{Display, Formatter};
use std::io::{Read, SeekFrom, Write};
use std::path::Path;
use arrayvec::ArrayVec;
use crate::content::{ArgSpec, AssocType, Term};
use crate::uris::ContentURI;
use crate::uris::modules::ModuleURI;
use crate::uris::symbols::SymbolURI;
use crate::utils::{NestedDisplay, NestingFormatter, VecMap};

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    pub language: Language,
    pub uri: DocumentURI,
    pub title: String,
    pub elements: Vec<DocumentElement>,
}
impl NestedDisplay for Document {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"{}",self.uri)?;
        f.nest(|f| {
            for e in &self.elements {
                f.next()?;
                e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for Document {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
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
    Problem(Problem)
}

impl NestedDisplay for DocumentElement {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        use DocumentElement::*;
        match self {
            SetSectionLevel(l) => write!(f.inner(),"Set section level {l}"),
            Section(s) => s.fmt_nested(f),
            Module(m) => m.fmt_nested(f),
            MathStructure(m) => m.fmt_nested(f),
            InputRef(r) => write!(f.inner(),"Input reference {}: {}",r.id,r.target),
            VarNotation { name, id, .. } => {
                write!(f.inner(),"Variable notation {} for {}",id,name)
            },
            VarDef { name, .. } => {
                write!(f.inner(),"Variable {}",name)
            },
            Definiendum { uri, .. } => write!(f.inner(),"Definiendum {}",uri),
            Symref { uri, .. } => write!(f.inner(),"Symbol reference {}",uri),
            Varref { name, .. } => write!(f.inner(),"Variable reference {}",name),
            TopTerm(t) => write!(f.inner(),"Top term {t:?}"),
            UseModule(m) => write!(f.inner(),"Use module {}",m),
            Paragraph(p) => p.fmt_nested(f),
            Problem(p) => write!(f.inner(),"Problem {}",p.id)
        }
    }
}
impl Display for DocumentElement {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
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
impl NestedDisplay for Section {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Section {}",self.id)?;
        if let Some((title,_)) = &self.title {
            write!(f.inner(),": {title}")?;
        }
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for Section {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentModule {
    pub range: SourceRange<ByteOffset>,
    pub name: String,
    pub children: Vec<DocumentElement>,
}

impl NestedDisplay for DocumentModule {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Module {}",self.name)?;
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for DocumentModule {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentMathStructure {
    pub range: SourceRange<ByteOffset>,
    pub name: String,
    pub children: Vec<DocumentElement>,
}
impl NestedDisplay for DocumentMathStructure {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Structure {}",self.name)?;
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for DocumentMathStructure {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
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
impl Display for StatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StatementKind::*;
        write!(f,"{}",match self {
            Definition => "Definition",
            Assertion => "Assertion",
            Paragraph => "Paragraph",
            Proof => "Proof",
            SubProof => "Subproof",
            Example => "Example"
        })
    }

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
impl NestedDisplay for LogicalParagraph {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"{} {}",self.kind,self.id)?;
        if let Some((title,_)) = &self.title {
            write!(f.inner(),": {title}")?;
        }
        if !self.fors.is_empty() {
            write!(f.inner()," for ")?;
            for (i,uri) in self.fors.iter().enumerate() {
                if i > 0 {write!(f.inner(),", ")?}
                write!(f.inner(),"{}",uri)?;
            }
        }
        f.nest(|f| {
            for e in &self.children {
                f.next()?;e.fmt_nested(f)?;
            }
            Ok(())
        })
    }
}
impl Display for LogicalParagraph {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
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

impl NestedDisplay for Problem {
    fn fmt_nested<'a>(&self, f: &mut NestingFormatter) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f.inner(),"Problem {}",self.id)?;
        if let Some((title,_)) = &self.title {
            write!(f.inner(),": {title}")?;
        }
        Ok(())
    }
}
impl Display for Problem {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.in_display(f)
    }
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
impl Display for SectionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SectionLevel::*;
        write!(f,"{}",match self {
            Part => "Part",
            Chapter => "Chapter",
            Section => "Section",
            Subsection => "Subsection",
            Subsubsection => "Subsubsection",
            Paragraph => "Paragraph",
            Subparagraph => "Subparagraph"
        })
    }
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
    Romanian,
    Arabic,
    Bulgarian,
    Russian,
    Finnish,
    Turkish,
    Slovenian
}
impl Language {
    pub fn from_file(path:&Path) -> Language {
        if let Some(stem) = path.file_stem().map(|s| s.to_str()).flatten() {
            if stem.ends_with(".en") { Language::English }
            else if stem.ends_with(".de") {Language::German}
            else if stem.ends_with(".fr") {Language::French}
            else if stem.ends_with(".ro") {Language::Romanian}
            else if stem.ends_with(".ar") {Language::Arabic}
            else if stem.ends_with(".bg") {Language::Bulgarian}
            else if stem.ends_with(".ru") {Language::Russian}
            else if stem.ends_with(".fi") {Language::Finnish}
            else if stem.ends_with(".tr") {Language::Turkish}
            else if stem.ends_with(".sl") {Language::Slovenian}
            else {Language::English}
        } else { Language::English }
    }
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",Into::<&'static str>::into(*self))
    }
}
impl Into<&'static str> for Language {
    fn into(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Romanian => "ro",
            Language::Arabic => "ar",
            Language::Bulgarian => "bg",
            Language::Russian => "ru",
            Language::Finnish => "fi",
            Language::Turkish => "tr",
            Language::Slovenian => "sl"
        }
    }
}
impl TryFrom<&str> for Language {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "en" => Language::English,
            "de" => Language::German,
            "fr" => Language::French,
            "ro" => Language::Romanian,
            "ar" => Language::Arabic,
            "bg" => Language::Bulgarian,
            "ru" => Language::Russian,
            "fi" => Language::Finnish,
            "tr" => Language::Turkish,
            "sl" => Language::Slovenian,
            _ => return Err(()),
        })
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
    pub css: Vec<CSS>,
    pub body:SourceRange<ByteOffset>,
    pub refs:String
}
impl HTMLDocSpec {
    pub fn get_doc(file:&Path) -> Option<Document> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).ok()?;
        let refs = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(4i64*4)).ok()?;
        let mut buffer = vec![0; refs];
        file.read_exact(&mut buffer).ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(d,_)| d)
    }
    #[cfg(feature = "async")]
    pub async fn get_doc_async(file:&Path) -> Option<Document> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).await.ok()?;
        let refs = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(4i64*4)).await.ok()?;
        let mut buffer = vec![0; refs];
        file.read_exact(&mut buffer).await.ok()?;
        bincode::serde::decode_from_slice(&buffer,bincode::config::standard()).ok().map(|(d,_)| d)
    }
    pub fn get_resource(file:&Path,range:SourceRange<ByteOffset>) -> Option<String> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        let mut refs = [0u8,0,0,0];
        file.read_exact(&mut refs).ok()?;
        let refs = u32::from_le_bytes(refs) as i64;
        file.seek(SeekFrom::Current(4i64*4 + refs + range.start.offset as i64)).ok()?;
        let mut buffer = vec![0; range.end.offset - range.start.offset];
        file.read_exact(&mut buffer).ok()?;
        String::from_utf8(buffer).ok()
    }

    #[cfg(feature = "async")]
    pub async fn get_resource_async(file:&Path,range:SourceRange<ByteOffset>) -> Option<String> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        let mut refs = [0u8,0,0,0];
        file.read_exact(&mut refs).await.ok()?;
        let refs = u32::from_le_bytes(refs) as i64;
        file.seek(SeekFrom::Current(4i64*4 + refs + range.start.offset as i64)).await.ok()?;
        let mut buffer = vec![0; range.end.offset - range.start.offset];
        file.read_exact(&mut buffer).await.ok()?;
        String::from_utf8(buffer).ok()
    }
    pub fn get_css_and_body(file:&Path) -> Option<(Vec<CSS>,String)> {
        use std::io::Seek;
        let mut file = std::fs::File::open(file).ok()?;
        file.seek(SeekFrom::Start(4)).ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).ok()?;
        let css = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let html = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let body_start = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).ok()?;
        let body_end = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(css as i64)).ok()?;
        let mut css = vec![0; html - css];
        file.read_exact(&mut css).ok()?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        file.seek(SeekFrom::Current(body_start as i64 - html as i64)).ok()?;
        let mut html = vec![0; body_end - body_start];
        file.read_exact(&mut html).ok()?;
        String::from_utf8(html).ok().map(|html| (css,html))
    }
    #[cfg(feature = "async")]
    pub async fn get_css_and_body_async(file:&Path) -> Option<(Vec<CSS>,String)> {
        use tokio::io::{AsyncReadExt,AsyncSeekExt};
        let mut file = tokio::fs::File::open(file).await.ok()?;
        file.seek(SeekFrom::Start(4)).await.ok()?;
        let mut buf = [0u8,0,0,0];
        file.read_exact(&mut buf).await.ok()?;
        let css = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let html = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let body_start = u32::from_le_bytes(buf) as usize;
        file.read_exact(&mut buf).await.ok()?;
        let body_len = u32::from_le_bytes(buf) as usize;
        file.seek(SeekFrom::Current(css as i64)).await.ok()?;
        let mut css = vec![0; html - css];
        file.read_exact(&mut css).await.ok()?;
        let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
        file.seek(SeekFrom::Current(body_start as i64 - html as i64)).await.ok()?;
        let mut html = vec![0; body_len];
        file.read_exact(&mut html).await.ok()?;
        String::from_utf8(html).ok().map(|html| (css,html))
    }
    pub fn write(self,p:&Path) {
        let mut file = std::fs::File::create(p).unwrap();
        if let Ok(doc) = bincode::serde::encode_to_vec(&self.doc,bincode::config::standard()) {
            if let Ok(css) = bincode::serde::encode_to_vec(&self.css,bincode::config::standard()) {
                let mut len = doc.len() as u32;
                // refs
                file.write_all(&len.to_le_bytes()).unwrap();
                len += self.refs.len() as u32;
                // css
                file.write_all(&len.to_le_bytes()).unwrap();
                len += css.len() as u32;
                // html
                file.write_all(&len.to_le_bytes()).unwrap();
                // body start;
                file.write_all(&(len + self.body.start.offset as u32).to_le_bytes()).unwrap();
                // body len;
                file.write_all(&((self.body.end.offset - self.body.start.offset) as u32).to_le_bytes()).unwrap();


                file.write_all(doc.as_slice()).unwrap();
                file.write_all(self.refs.as_bytes()).unwrap();
                file.write_all(css.as_slice()).unwrap();
                file.write_all(self.html.as_bytes()).unwrap();
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }
}