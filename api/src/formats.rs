pub mod building;

use std::fmt::{Display, Debug, Write};
use std::path::Path;
use async_trait::async_trait;
use crate::formats::building::{Backend, BuildInfo, BuildTask};

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct Id([u8;4]);
impl Id {
    pub const fn new_unchecked(id:[u8;4]) -> Self { Self(id) }
}
impl<'a> TryFrom<&'a str> for Id {
    type Error = ();
    fn try_from(s:&'a str) -> Result<Self,Self::Error> {
        if s.len() != 4 { return Err(()); }
        let mut id = [0u8;4];
        for (i,b) in s.bytes().enumerate() {
            id[i] = b;
        }
        Ok(Self(id))
    }
}
impl Display for Id {
    #[inline]
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        for b in self.0 {
            if b != 0 {f.write_char(b as char)?} else {break}
        }
        Ok(())
    }
}
impl Debug for Id {
    #[inline]
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result { <Self as Display>::fmt(self,f) }
}

pub trait FormatExtension:Send+Sync {
    //fn estimate_dependencies(&self,source:&Path);
    fn get_task(&self, info:&BuildInfo,backend:&Backend<'_>) -> Option<BuildTask>;
}

pub struct Format {
    id: Id,
    file_extensions:&'static [&'static str],
    pub extension: Box<dyn FormatExtension>
}
impl Format {
    #[inline]
    pub const fn new(id: Id, file_extensions:&'static [&'static str], extension:Box<dyn FormatExtension>) -> Self {
        Self { id, file_extensions,extension }
    }
    #[inline]
    pub fn id(&self) -> Id { self.id }
    #[inline]
    pub fn get_extensions(&self) -> &'static [&'static str] { self.file_extensions }
}


#[derive(Default)]
pub struct FormatStore {
    formats:Vec<Format>
}
impl FormatStore {
    pub fn from_ext<S:AsRef<str>>(&self,s:S) -> Option<Id> {
        let s = s.as_ref();
        for f in &self.formats {
            if f.get_extensions().iter().any(|e| e.eq_ignore_ascii_case(s)) {
                return Some(f.id())
            }
        }
        None
    }
    pub fn register(&mut self,format:Format) {
        match self.formats.iter_mut().find(|f| f.id() == format.id()) {
            Some(f) => *f = format,
            _ => self.formats.push(format)
        }
    }
    pub fn get(&self,id:Id) -> Option<&Format> {
        self.formats.iter().find(|f| f.id() == id)
    }
}