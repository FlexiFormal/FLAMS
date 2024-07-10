use std::fmt::Display;
use crate::uris::archives::ArchiveId;

#[macro_export]
macro_rules! short_id {
    (+$name:ident) => {
        #[derive(serde::Serialize,serde::Deserialize,Debug,Clone,Copy,PartialEq,Eq,Hash)]
        pub struct $name($crate::building::formats::ShortId);
        short_id!(@from $name);
    };
    (?$name:ident) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize,serde::Deserialize))]
        pub struct $name($crate::building::formats::ShortId);
        short_id!(@from $name);
    };
    ($name:ident) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
        pub struct BuildFormatId($crate::building::formats::ShortId);
        short_id!(@from $name);
    };
    (@from $name:ident) => {
        impl From<$crate::building::formats::ShortId> for $name {
            fn from(id: $crate::building::formats::ShortId) -> Self { Self(id) }
        }
        impl $name {
            pub const fn new(id:$crate::building::formats::ShortId) -> Self { Self(id) }
        }
        impl std::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShortId([u8;8]);
impl std::fmt::Debug for ShortId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = std::str::from_utf8(&self.0[0..self.len() as usize])
            .map_err(|_| std::fmt::Error)?;
        f.write_str(str)
    }
}
impl ShortId {
    pub const CHECK: ShortId = ShortId::new_unchecked("check");
    pub const fn new_unchecked(id: &str) -> Self {
        assert!(id.len() > 0 && id.len() <= 8);
        let mut ret = [0,0,0,0,0,0,0,0];
        ret[0] = id.as_bytes()[0];
        if id.len() == 1 {return Self(ret)}
        ret[1] = id.as_bytes()[1];
        if id.len() == 2 {return Self(ret)}
        ret[2] = id.as_bytes()[2];
        if id.len() == 3 {return Self(ret)}
        ret[3] = id.as_bytes()[3];
        if id.len() == 4 {return Self(ret)}
        ret[4] = id.as_bytes()[4];
        if id.len() == 5 {return Self(ret)}
        ret[5] = id.as_bytes()[5];
        if id.len() == 6 {return Self(ret)}
        ret[6] = id.as_bytes()[6];
        if id.len() == 7 {return Self(ret)}
        ret[7] = id.as_bytes()[7];
        Self(ret)
    }
    pub fn new(id:&str) -> Option<Self> {
        if id.len() > 8 { return None; }
        Some(Self::new_unchecked(id))
    }
    fn len(&self) -> u8 {
        let mut i = 8u8;
        while i > 0 && self.0[i as usize - 1] == 0 { i -= 1 }
        i
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for ShortId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str = std::str::from_utf8(&self.0[0..self.len() as usize])
            .map_err(|_| serde::ser::Error::custom("invalid utf8"))?;
        serializer.serialize_str(str)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ShortId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Self::new(&str).ok_or(serde::de::Error::custom("invalid short id"))
    }
}


impl<'s> TryFrom<&'s str> for ShortId {
    type Error = ();
    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        if s.len() > 8 { return Err(()); }
        Ok(Self::new_unchecked(s))
    }
}
impl Display for ShortId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = std::str::from_utf8(&self.0[0..self.len() as usize])
            .map_err(|_| std::fmt::Error)?;
        f.write_str(str)
    }
}

short_id!(?SourceFormatId);
short_id!(?BuildTargetId);


#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize,serde::Deserialize))]
pub enum FormatOrTarget {
    Format(SourceFormatId),
    Target(BuildTargetId)
}
impl std::fmt::Display for FormatOrTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatOrTarget::Format(fmt) => fmt.fmt(f),
            FormatOrTarget::Target(t) => t.fmt(f)
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize,serde::Deserialize))]
pub enum BuildJobSpec {
    Group {
        id: ArchiveId,
        target:FormatOrTarget,
        stale_only:bool
    },
    Archive {
        id:ArchiveId,
        target:FormatOrTarget,
        stale_only:bool
    },
    Path {
        id:ArchiveId,
        rel_path:String,
        target:FormatOrTarget,
        stale_only:bool
    }
}