use std::fmt::Display;
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
    #[inline]
    pub fn as_ref(&self) -> DomURIRef<'_> {
        DomURIRef(self.0.as_str())
    }
    #[inline]
    pub fn as_str(&self) -> &str { self.0.as_str() }
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
            &self.inner.as_str()[0..self.archive_index as usize - 3]
        )
    }
    #[inline]
    pub fn id(&self) -> ArchiveIdRef {
        ArchiveIdRef(
            &self.inner.as_str()[self.archive_index as usize..]
        )
    }

    fn as_ref(&self) -> ArchiveURIRef<'_> {
        ArchiveURIRef{
            base:self.base(),
            archive:self.id()
        }
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
            inner:self.to_string(),
            archive_index
        }
    }
}
impl<'a> Display for ArchiveURIRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}?a={}",self.base,self.archive)
    }
}
