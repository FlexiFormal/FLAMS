use std::fmt::{Display, Debug,Write};

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct FormatId([u8;4]);
impl FormatId {
    pub const fn new_unchecked(id:[u8;4]) -> Self { Self(id) }
}
impl<'a> TryFrom<&'a str> for FormatId {
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
impl Display for FormatId {
    #[inline]
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        for b in self.0 {
            if b != 0 {f.write_char(b as char)?} else {break}
        }
        Ok(())
    }
}
impl Debug for FormatId {
    #[inline]
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result { <Self as Display>::fmt(self,f) }
}

pub struct Format {
    id:FormatId,
    file_extensions:&'static [&'static str],
}
impl Format {
    #[inline]
    pub const fn new(id:FormatId,file_extensions:&'static [&'static str]) -> Self {
        Self { id, file_extensions }
    }
    #[inline]
    pub fn id(&self) -> FormatId { self.id }
    #[inline]
    pub fn get_extensions(&self) -> &'static [&'static str] { self.file_extensions }
    /*
    pub fn register_new(id:&'static str, file_extensions:&'static [&'static str]) -> Result<FormatRef,FormatRef> {
        let r = Self { id, file_extensions };
        register_format(r)
    }
    pub fn parse<S:AsRef<str>>(s:S) -> Option<FormatRef> {
        FORMAT_STORE.read().iter().enumerate().find_map(|(i,f)| {
            if f.id.eq_ignore_ascii_case(s.as_ref()) { Some(FormatRef(i as u16)) }
            else { None }
        })
    }
    pub fn from_extension<S:AsRef<str>>(ext:S) -> Option<FormatRef> {
        FORMAT_STORE.read().iter().enumerate().find_map(|(i,f)| {
            if f.file_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext.as_ref())) { Some(FormatRef(i as u16)) }
            else { None }
        })
    }

     */
}