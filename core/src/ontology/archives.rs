use std::path::{Path, PathBuf};
use crate::building::formats::SourceFormatId;
use crate::uris::archives::{ArchiveId, ArchiveURI};
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
    pub formats: arrayvec::ArrayVec<SourceFormatId,8>
}
impl StorageSpec {
    #[inline]
    pub fn id(&self) -> ArchiveId { self.uri.id() }
    #[inline]
    pub fn parents(&self) -> std::str::Split<'static,char> {
        self.uri.id().steps()
    }
    #[inline]
    pub fn as_ref(&self) -> StorageSpecRef<'_> {
        StorageSpecRef {
            uri: self.uri,
            attributes: &self.attributes,
            formats: &self.formats
        }
    }
}

#[derive(Copy,Debug, Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serde",derive(serde::Serialize))]
pub struct StorageSpecRef<'a> {
    pub uri: ArchiveURI,
    pub attributes: &'a VecMap<Box<str>,Box<str>>,
    pub formats: &'a [SourceFormatId]
}

#[derive(Debug, Clone)]
pub struct MathArchiveSpec {
    pub storage: StorageSpec,
    pub ignore_source: IgnoreSource,
    pub path: std::sync::Arc<Path>
}

#[cfg(feature="serde")]
impl serde::Serialize for MathArchiveSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MathArchiveSpec",3)?;
        state.serialize_field("storage",&self.storage)?;
        state.serialize_field("ignore",&self.ignore_source)?;
        state.serialize_field("path",&*self.path)?;
        state.end()
    }
}

#[cfg(feature="serde")]
impl<'de> serde::Deserialize<'de> for MathArchiveSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MathArchiveSpec;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct MathArchiveSpec")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
                let mut storage = None;
                let mut ignore_source = None;
                let mut path = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "storage" => {
                            if storage.is_some() {
                                return Err(serde::de::Error::duplicate_field("storage"));
                            }
                            storage = Some(map.next_value()?);
                        }
                        "ignore" => {
                            if ignore_source.is_some() {
                                return Err(serde::de::Error::duplicate_field("ignore_source"));
                            }
                            ignore_source = Some(map.next_value()?);
                        }
                        "path" => {
                            if path.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            let pb : PathBuf = map.next_value()?;
                            path = Some(pb.into());
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(key, &["storage","ignore_source","path"]));
                        }
                    }
                }
                let storage = storage.ok_or_else(|| serde::de::Error::missing_field("storage"))?;
                let ignore_source = ignore_source.ok_or_else(|| serde::de::Error::missing_field("ignore_source"))?;
                let path = path.ok_or_else(|| serde::de::Error::missing_field("path"))?;
                Ok(MathArchiveSpec { storage, ignore_source, path })
            }
        }
        deserializer.deserialize_struct("MathArchiveSpec", &["storage","ignore_source","path"], Visitor)
    }
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