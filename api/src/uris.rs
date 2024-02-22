use crate::archives::ArchiveIdRef;
use crate::CloneStr;
use oxrdf::NamedNode;
use std::fmt::Display;

// SERDE HAS URL ENCODING!

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum MMTUri {
    Dom(DomURI),
    Archive(ArchiveURI),
}
impl MMTUri {
    pub fn as_ref(&self) -> MMTUriRef<'_> {
        match self {
            MMTUri::Dom(d) => MMTUriRef::Dom(d.as_ref()),
            MMTUri::Archive(a) => MMTUriRef::Archive(a.as_ref()),
        }
    }
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub enum MMTUriRef<'a> {
    Dom(DomURIRef<'a>),
    Archive(ArchiveURIRef<'a>),
}
impl<'a> MMTUriRef<'a> {
    pub fn to_owned(&self) -> MMTUri {
        match self {
            MMTUriRef::Dom(d) => MMTUri::Dom(d.to_owned()),
            MMTUriRef::Archive(a) => MMTUri::Archive(a.to_owned()),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct DomURI(CloneStr);
impl DomURI {
    pub fn new<S: Into<CloneStr>>(s: S) -> Self {
        Self(s.into())
    } // TODO verify that this is a valid URI
    #[inline]
    pub fn as_ref(&self) -> DomURIRef<'_> {
        DomURIRef(&self.0)
    }
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#", self.0)).unwrap()
    }
}
impl Display for DomURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub struct DomURIRef<'a>(&'a str);
impl<'a> DomURIRef<'a> {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0
    }
    #[inline]
    pub fn to_owned(&self) -> DomURI {
        DomURI(self.0.into())
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#", self.0)).unwrap()
    }
}
impl<'a> Display for DomURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ArchiveURI {
    inner: CloneStr,
    archive_index: u8,
}
impl ArchiveURI {
    #[inline]
    pub fn base(&self) -> DomURIRef<'_> {
        DomURIRef(&self.inner[0..self.archive_index as usize - 3])
    }
    #[inline]
    pub fn id(&self) -> ArchiveIdRef {
        ArchiveIdRef(&self.inner[self.archive_index as usize..])
    }

    pub fn as_ref(&self) -> ArchiveURIRef<'_> {
        ArchiveURIRef {
            base: self.base(),
            archive: self.id(),
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#a={}", self.base(), self.id())).unwrap()
    }
}
impl Display for ArchiveURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub struct ArchiveURIRef<'a> {
    pub(crate) base: DomURIRef<'a>,
    pub(crate) archive: ArchiveIdRef<'a>,
}
impl<'a> ArchiveURIRef<'a> {
    pub fn base(&self) -> DomURIRef<'_> {
        self.base
    }
    pub fn id(&self) -> ArchiveIdRef {
        self.archive
    }
    pub fn to_owned(&self) -> ArchiveURI {
        let archive_index = (self.base.as_str().len() + 3) as u8;
        ArchiveURI {
            inner: self.to_string().into(),
            archive_index,
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#a={}", self.base(), self.id())).unwrap()
    }
}
impl<'a> Display for ArchiveURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?a={}", self.base, self.archive)
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub struct DocumentURI {
    inner: CloneStr,
    archive_index: u8,
    path_index: u16,
    document_index: u16,
}

impl DocumentURI {
    pub fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref();
        let archive_index = (s.find("?a=").unwrap() + 3) as u8;
        let path_index = s.find("&D&f=").unwrap() as u16 + 5;
        let document_index = s.find("&n=").unwrap() as u16 + 3;
        DocumentURI {
            inner: s.into(),
            archive_index,
            path_index,
            document_index,
        }
    }
    pub fn from_relpath(archive_uri: ArchiveURIRef, relpath: &str) -> Self {
        let archive_index = (archive_uri.base().as_str().len() + 3) as u8;
        let mut path = if relpath.starts_with('/') {
            &relpath[1..]
        } else {
            relpath
        };
        let (p, mut d) = if let Some((p, d)) = path.rsplit_once('/') {
            (p, d)
        } else {
            ("", path)
        };
        if let Some((a, b)) = d.rsplit_once('.') {
            d = a;
        }
        path = p;
        let path_index = (archive_index as u16) + archive_uri.id().as_str().len() as u16 + 5;
        let document_index = path_index + path.len() as u16 + 3;
        DocumentURI {
            inner: format!(
                "{}?a={}&D&f={}&n={}",
                archive_uri.base(),
                archive_uri.id(),
                path,
                d
            )
            .into(),
            archive_index,
            path_index,
            document_index,
        }
    }
    #[inline]
    pub fn base(&self) -> DomURIRef<'_> {
        DomURIRef(&self.inner[0..self.archive_index as usize - 3])
    }
    #[inline]
    pub fn archive_id(&self) -> ArchiveIdRef {
        ArchiveIdRef(&self.inner[self.archive_index as usize..self.path_index as usize - 5])
    }
    #[inline]
    pub fn path(&self) -> &str {
        &self.inner[self.path_index as usize..self.document_index as usize - 3]
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.inner[self.document_index as usize..]
    }

    fn as_ref(&self) -> DocumentURIRef<'_> {
        DocumentURIRef {
            base: self.base(),
            archive: self.archive_id(),
            path: self.path(),
            name: self.name(),
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!(
            "{}?a={}&D&f={}#n={}",
            self.base(),
            self.archive_id(),
            self.path(),
            self.name()
        ))
        .unwrap()
    }
}
impl Display for DocumentURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub struct DocumentURIRef<'a> {
    pub(crate) base: DomURIRef<'a>,
    pub(crate) archive: ArchiveIdRef<'a>,
    pub(crate) path: &'a str,
    pub(crate) name: &'a str,
}

impl<'a> DocumentURIRef<'a> {
    pub fn base(&self) -> DomURIRef<'_> {
        self.base
    }
    pub fn archive_id(&self) -> ArchiveIdRef {
        self.archive
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn path(&self) -> &str {
        self.path
    }
    pub fn to_owned(&self) -> DocumentURI {
        let archive_index = (self.base.as_str().len() + 3) as u8;
        let path_index = (archive_index as u16) + self.archive.as_str().len() as u16 + 5;
        let document_index = path_index + self.path.len() as u16 + 3;
        DocumentURI {
            inner: self.to_string().into(),
            archive_index,
            path_index,
            document_index,
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!(
            "{}?a={}&D&f={}#n={}",
            self.base(),
            self.archive_id(),
            self.path(),
            self.name()
        ))
        .unwrap()
    }
}
impl<'a> Display for DocumentURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}?a={}&D&f={}&n={}",
            self.base, self.archive, self.path, self.name
        )
    }
}
