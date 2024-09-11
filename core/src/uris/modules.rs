use std::fmt::Display;
use std::ops::{BitAnd, Div, Neg, Not};
use std::str::FromStr;
use immt_ontology::rdf::NamedNode;
use crate::narration::Language;
use crate::uris::archives::{ArchiveURI};
use crate::uris::Name;
use crate::uris::symbols::SymbolURI;

#[derive(Clone, Copy,Debug, PartialEq, Eq, Hash)]
pub struct ModuleURI {
    archive: ArchiveURI,
    path: Option<Name>,
    name: Name,
    language:Language
}

impl Div<Name> for ModuleURI {
    type Output = Self;
    fn div(self, rhs: Name) -> Self::Output {
        let newname = Name::new(self.name.as_ref().to_string() + "/" + rhs.as_ref());
        Self::new(self.archive, self.path, newname,self.language)
    }
}

impl BitAnd<Name> for ModuleURI {
    type Output = SymbolURI;
    fn bitand(self, rhs: Name) -> Self::Output {
        SymbolURI::new(self,rhs)
    }
}

impl FromStr for ModuleURI {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut f = s.split('&');
        let archive = f.next().ok_or_else(|| "No archive")?;
        let archive : ArchiveURI = archive.parse().map_err(|_| "Invalid archive")?;
        let maybe_path = f.next().ok_or_else(|| "No name")?;
        if maybe_path.starts_with("p=") {
            let path = &maybe_path[2..];
            let path = if path.is_empty() {None} else {Some(Name::new(path))};
            let name = f.next().ok_or_else(|| "No name")?;
            if !name.starts_with("m=") {
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
            if !maybe_path.starts_with("m=") {
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
impl serde::Serialize for ModuleURI {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for ModuleURI {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(|s| serde::de::Error::custom(s))
    }
}

impl ModuleURI {
    #[inline]
    pub fn new(archive:ArchiveURI,path:Option<impl Into<Name>>,name:impl Into<Name>,lang:Language) -> Self {
        Self { archive, path:path.map(|p| p.into()), name:name.into(), language:lang }
    }
    #[inline]
    pub fn name(&self) -> Name { self.name }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(self.to_string()
            .replace(' ',"%20")
            .replace('\\',"%5C")
            .replace('^',"%5E")
            .replace('[',"%5B")
            .replace(']',"%5D")
        ).unwrap()
    }
    #[inline]
    pub fn language(&self) -> Language { self.language }
    #[inline]
    pub fn archive(&self) -> ArchiveURI { self.archive }
    #[inline]
    pub fn path(&self) -> Option<Name> { self.path }
}

impl Not for ModuleURI {
    type Output = Self;
    fn not(self) -> Self::Output {
        let name = self.name.as_ref().split('/').next().unwrap();
        if name == self.name.as_ref() { self} else {
            Self::new(self.archive, self.path, Name::new(name),self.language)
        }
    }
}

impl Display for ModuleURI {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.path {
            None => write!(f, "{}&m={}&l={}", self.archive, self.name,self.language),
            Some(p) => write!(f, "{}&p={}&m={}&l={}", self.archive, p, self.name,self.language)
        }
    }
}
