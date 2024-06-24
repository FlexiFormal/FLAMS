use std::path::Path;
use crate::building::formats::ShortId;
use crate::uris::archives::{ArchiveId, ArchiveURI, ArchiveURIRef};
use crate::utils::VecMap;
use crate::utils::arrayvec;
use crate::utils::ignore_regex::IgnoreSource;
use crate::building::buildstate::AllStates;

#[derive(Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct StorageSpec {
    pub uri: ArchiveURI,
    pub is_meta:bool,
    pub attributes: VecMap<Box<str>,Box<str>>,
    pub formats: arrayvec::ArrayVec<ShortId,8>
}
impl StorageSpec {
    #[inline]
    pub fn id(&self) -> &ArchiveId { self.uri.id() }
    #[inline]
    pub fn parents(&self) -> std::str::Split<'_,char> {
        self.uri.id().steps()
    }
    #[inline]
    pub fn as_ref(&self) -> StorageSpecRef<'_> {
        StorageSpecRef {
            uri: self.uri.as_ref(),
            attributes: &self.attributes,
            formats: &self.formats
        }
    }
}

#[derive(Copy,Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serde",derive(serde::Serialize))]
pub struct StorageSpecRef<'a> {
    pub uri: ArchiveURIRef<'a>,
    pub attributes: &'a VecMap<Box<str>,Box<str>>,
    pub formats: &'a [ShortId]
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct MathArchiveSpec {
    pub storage: StorageSpec,
    pub ignore_source: IgnoreSource,
    pub path: Box<Path>
}
impl MathArchiveSpec {
    pub fn as_ref(&self) -> MathArchiveSpecRef<'_> {
        MathArchiveSpecRef {
            storage: self.storage.as_ref(),
            ignore_source: &self.ignore_source,
            path: &self.path
        }
    }
}


#[derive(Copy,Debug, Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize))]
pub struct MathArchiveSpecRef<'a> {
    pub storage: StorageSpecRef<'a>,
    pub ignore_source: &'a IgnoreSource,
    pub path: &'a Path
}
impl PartialEq for MathArchiveSpecRef<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { 
        self.storage == other.storage && self.path == other.path && self.ignore_source == other.ignore_source
    }
}


#[derive(Debug)]
pub enum ArchiveGroup {
    Archive(ArchiveId),
    Group {
        id: ArchiveId,
        has_meta: bool,
        children: Vec<ArchiveGroup>,
        state: AllStates
    }
}
static DEFAULT: AllStates = AllStates {map:VecMap::new() };
impl ArchiveGroup {
    pub const fn id(&self) -> &ArchiveId {
        match self {
            Self::Archive(id) | Self::Group { id, .. } => id
        }
    }
    pub fn state<F: Fn(&ArchiveId) -> Option<&AllStates>>(&self, get:&F) -> &AllStates {
        match self {
            Self::Group { state, .. } => state,
            Self::Archive(id) => get(id).unwrap_or(&DEFAULT)
        }
    }
    pub fn update<'a, F: Fn(&ArchiveId) -> Option<&'a AllStates>>(&mut self, get:&'a F) {
        match self {
            Self::Group { children, state, .. } => {
                *state = AllStates::default();
                for c in children.iter_mut() {
                    c.update(get);
                    match c {
                        Self::Group { state:s, .. } => state.merge_cum(s),
                        Self::Archive(id) => match get(id){
                            Some(s) => state.merge_cum(s),
                            None => {}
                        }
                    }
                }
            },
            Self::Archive(_) => {}
        }

    }
}