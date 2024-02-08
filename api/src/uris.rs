use std::fmt::Display;
use oxrdf::NamedNode;
use crate::archives::ArchiveIdRef;
use crate::Str;

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub enum MMTUri {
    Dom(DomURI),
    Archive(ArchiveURI),
}
impl MMTUri {
    pub fn as_ref(&self) -> MMTUriRef<'_> {
        match self {
            MMTUri::Dom(d) => MMTUriRef::Dom(d.as_ref()),
            MMTUri::Archive(a) => MMTUriRef::Archive(a.as_ref())
        }
    }
}
#[derive(Clone,Copy,Hash,Debug,PartialEq,Eq)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode))]
pub enum MMTUriRef<'a> {
    Dom(DomURIRef<'a>),
    Archive(ArchiveURIRef<'a>),
}
impl<'a> MMTUriRef<'a> {
    pub fn to_owned(&self) -> MMTUri {
        match self {
            MMTUriRef::Dom(d) => MMTUri::Dom(d.to_owned()),
            MMTUriRef::Archive(a) => MMTUri::Archive(a.to_owned())
        }
    }
}

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct DomURI(Str);
impl DomURI {
    pub fn new<S:Into<Str>>(s:S) -> Self { Self(s.into()) } // TODO verify that this is a valid URI
    #[inline]
    pub fn as_ref(&self) -> DomURIRef<'_> {
        DomURIRef(&self.0)
    }
    #[inline]
    pub fn as_str(&self) -> &str { &self.0 }

    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#",self.0)).unwrap()
    }
}
impl Display for DomURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone,Copy,Hash,Debug,PartialEq,Eq)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode))]
pub struct DomURIRef<'a>(&'a str);
impl<'a> DomURIRef<'a> {
    #[inline]
    pub fn as_str(&self) -> &str { self.0 }
    #[inline]
    pub fn to_owned(&self) -> DomURI { DomURI(self.0.into()) }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#",self.0)).unwrap()
    }
}
impl<'a> Display for DomURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct ArchiveURI {
    inner:Str,
    archive_index:u8
}
impl ArchiveURI {
    #[inline]
    pub fn base(&self) -> DomURIRef<'_> {
        DomURIRef(
            &self.inner[0..self.archive_index as usize - 3]
        )
    }
    #[inline]
    pub fn id(&self) -> ArchiveIdRef {
        ArchiveIdRef(
            &self.inner[self.archive_index as usize..]
        )
    }

    fn as_ref(&self) -> ArchiveURIRef<'_> {
        ArchiveURIRef{
            base:self.base(),
            archive:self.id()
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#{}",self.base(),self.id())).unwrap()
    }
}
impl Display for ArchiveURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

#[derive(Clone,Copy,Hash,Debug,PartialEq,Eq)]
#[cfg_attr(feature="serde",derive(serde::Serialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode))]
pub struct ArchiveURIRef<'a>{
    pub(crate) base:DomURIRef<'a>,
    pub(crate) archive:ArchiveIdRef<'a>
}
impl<'a> ArchiveURIRef<'a> {
    pub fn base(&self) -> DomURIRef<'_> { self.base }
    pub fn id(&self) -> ArchiveIdRef { self.archive }
    pub fn to_owned(&self) -> ArchiveURI {
        let archive_index = (self.base.as_str().len() + 3) as u8;
        ArchiveURI{
            inner:self.to_string().into(),
            archive_index
        }
    }
    pub fn to_iri(&self) -> NamedNode {
        NamedNode::new(format!("{}#{}",self.base(),self.id())).unwrap()
    }
}
impl<'a> Display for ArchiveURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}?a={}",self.base,self.archive)
    }
}