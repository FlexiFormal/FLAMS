use std::fmt::Display;
use std::ops::{BitAnd, Div, DivAssign, Mul};
use std::str::FromStr;
use immt_ontology::rdf::NamedNode;
use crate::narration::Language;
use crate::uris::archives::{ArchiveURI};
use crate::uris::modules::ModuleURI;
use crate::uris::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DocumentURI {
    archive: ArchiveURI,
    path: Option<Name>,
    language: Language,
    name: Name
}
impl DocumentURI {
    #[inline]
    pub fn archive(&self) -> ArchiveURI { self.archive}
    #[inline]
    pub fn path(&self) -> Option<Name> { self.path }
    #[inline]
    pub fn name(&self) -> Name { self.name }
    #[inline]
    pub fn language(&self) -> Language { self.language }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string().replace(' ',"%20")).unwrap()
    }
    pub fn new(archive: ArchiveURI, path: Option<impl Into<Name>>, name: impl Into<Name>,lang:Language) -> Self {
        Self { archive, path:path.map(|n| n.into()), name:name.into(), language: lang }
    }
}
impl Display for DocumentURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.path {
            Some(p) => write!(f, "{}&p={}&d={}&l={}", self.archive, p, self.name,self.language),
            None => write!(f, "{}&d={}&l={}", self.archive, self.name,self.language)
        }
    }
}
impl FromStr for DocumentURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut f = s.split('&');
        let archive = f.next().ok_or_else(|| "No archive")?;
        let archive : ArchiveURI = archive.parse().map_err(|_| "Invalid archive")?;
        let mut maybe_path = f.next().ok_or_else(|| "No name")?;
        if maybe_path.starts_with("p=") {
            let path = &maybe_path[2..];

            let path = if path.is_empty() {None} else {Some(Name::new(&maybe_path[2..]))};
            let name = f.next().ok_or_else(|| "No name")?;
            if !name.starts_with("d=") {
                return Err("Invalid name");
            }
            let lang = match f.next() {
                Some(s) if s.starts_with("l=") => Language::try_from(&s[2..]).map_err(|_| "Invalid language")?,
                Some(_) => return Err("Too many '?'-parts"),
                _ => Language::English
            };
            let name = Name::new(&name[2..]);
            Ok(Self::new(archive, path, name,lang))
        } else {
            if !maybe_path.starts_with("d=") {
                return Err("Invalid name");
            }
            let lang = match f.next() {
                Some(s) if s.starts_with("l=") => Language::try_from(&s[2..]).map_err(|_| "Invalid language")?,
                Some(_) => return Err("Too many '?'-parts"),
                _ => Language::English
            };
            let name = Name::new(&maybe_path[2..]);
            Ok(Self::new(archive, None::<&str>, name,lang))
        }
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for DocumentURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for DocumentURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NarrDeclURI {
    doc:DocumentURI,
    name: Name
}
impl NarrDeclURI {
    #[inline]
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string()
            .replace(' ',"%20")
            .replace('\\',"%5C")
            .replace('^',"%5E")
            .replace('[',"%5B")
            .replace(']',"%5D")
        ).unwrap()
    }
    pub fn new(doc: DocumentURI, name: impl Into<Name>) -> Self {
        Self { doc, name:name.into() }
    }
    #[inline]
    pub fn name(&self) -> Name { self.name }
    #[inline]
    pub fn doc(&self) -> DocumentURI { self.doc }
}
impl Display for NarrDeclURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&e={}", self.doc, self.name)
    }
}
impl FromStr for NarrDeclURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut f = s.split("&e=");
        let doc = f.next().ok_or_else(|| "No document")?;
        let doc : DocumentURI = doc.parse().map_err(|_| "Invalid document")?;
        let name = f.next().ok_or_else(|| "No name")?;
        if f.next().is_some() {
            return Err("Too many '?'-parts");
        }
        Ok(Self::new(doc, name))
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for NarrDeclURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for NarrDeclURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}

impl BitAnd<Name> for DocumentURI {
    type Output = NarrDeclURI;
    fn bitand(self, rhs: Name) -> Self::Output {
        NarrDeclURI::new(self, rhs)
    }
}

impl Mul<Name> for DocumentURI {
    type Output = ModuleURI;
    fn mul(self, rhs: Name) -> Self::Output {
        if rhs == self.name {
            ModuleURI::new(self.archive,self.path,rhs,self.language)
        } else {
            let path = match self.path {
                Some(p) => Name::new(format!("{}/{}",p,self.name)),
                _ => self.name
            };
            ModuleURI::new(self.archive,Some(path),rhs,self.language)
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum NarrativeURI {
    Doc(DocumentURI),
    Decl(NarrDeclURI)
}
impl NarrativeURI {
    pub fn name(&self) -> Name {
        match self {
            NarrativeURI::Doc(d) => d.name,
            NarrativeURI::Decl(d) => d.name
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        match self {
            NarrativeURI::Doc(d) => d.to_iri(),
            NarrativeURI::Decl(d) => d.to_iri()
        }
    }
    pub fn doc(&self) -> DocumentURI {
        match self {
            NarrativeURI::Doc(d) => *d,
            NarrativeURI::Decl(d) => d.doc
        }
    }
    pub fn language(&self) -> Language {
        match self {
            NarrativeURI::Doc(d) => d.language,
            NarrativeURI::Decl(d) => d.doc.language
        }
    }
}
impl Display for NarrativeURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NarrativeURI::Doc(d) => Display::fmt(d,f),
            NarrativeURI::Decl(d) => Display::fmt(d,f),
        }
    }
}

impl FromStr for NarrativeURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("&e=") {
            NarrDeclURI::from_str(s).map(NarrativeURI::Decl)
        } else {
            DocumentURI::from_str(s).map(NarrativeURI::Doc)
        }
    }
}

impl From<DocumentURI> for NarrativeURI {
    fn from(value: DocumentURI) -> Self {
        Self::Doc(value)
    }
}
impl From<NarrDeclURI> for NarrativeURI {
    fn from(value: NarrDeclURI) -> Self {
        Self::Decl(value)
    }
}

#[cfg(feature="serde")]
impl serde::Serialize for NarrativeURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for NarrativeURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}

impl Div<Name> for NarrativeURI {
    type Output = NarrDeclURI;
    fn div(self, rhs: Name) -> Self::Output {
        match self {
            NarrativeURI::Doc(d) => NarrDeclURI::new(d, rhs),
            NarrativeURI::Decl(d) =>
                NarrDeclURI::new(d.doc, Name::new(&format!("{}/{rhs}",d.name))),
        }
    }
}