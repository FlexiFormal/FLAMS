pub mod __reexport {
    pub use inventory::*;
}

#[derive(Copy, Clone, Debug)]
pub struct SourceFormat {
    pub name: &'static str,
    pub description: &'static str,
    pub targets: &'static [BuildTargetId],
    pub file_extensions: &'static [&'static str],
    // dependencies
}
impl SourceFormat {
    #[inline]
    #[must_use]
    pub const fn id(&'static self) -> SourceFormatId {
        SourceFormatId(self)
    }
}
inventory::collect!(SourceFormat);
#[macro_export]
macro_rules! source_format {
    ($i:ident { $($t:tt)* }) => {
        pub static $i : $crate::formats::SourceFormat = $crate::formats::SourceFormat { $($t)* };
        $crate::formats::__reexport::submit!{ $i }
    };
}

impl SourceFormat {
    #[inline]
    pub fn all() -> impl Iterator<Item = SourceFormatId> {
        inventory::iter.into_iter().map(SourceFormatId)
    }
    #[must_use]
    pub fn get(name: &str) -> Option<SourceFormatId> {
        Self::all().find(|e| e.name == name)
    }
}

source_format! { FTML {
    name:"ftml",
    description:"Flexiformal HTML",
    targets:&[FTML_CONTENT.id()],
    file_extensions: &["html"]
}}

#[derive(Copy, Clone, Debug)]
pub struct BuildTarget {
    pub name: &'static str,
    pub description: &'static str,
    // dependencies
    // yields
    // run
}
impl BuildTarget {
    #[inline]
    #[must_use]
    pub const fn id(&'static self) -> BuildTargetId {
        BuildTargetId(self)
    }
}
inventory::collect!(BuildTarget);
#[macro_export]
macro_rules! build_target {
    ($i:ident { $($t:tt)* }) => {
        pub static $i : $crate::formats::BuildTarget = $crate::formats::BuildTarget { $($t)* };
        $crate::formats::__reexport::submit!{ $i }
    };
}

impl BuildTarget {
    #[inline]
    pub fn all() -> impl Iterator<Item = BuildTargetId> {
        inventory::iter.into_iter().map(BuildTargetId)
    }
    #[must_use]
    pub fn get(name: &str) -> Option<BuildTargetId> {
        Self::all().find(|e| e.name == name)
    }
}

build_target! { FTML_CONTENT {
    name:"ftml->content",
    description:"imports existent FTML"
}}

#[derive(Copy, Clone)]
pub struct SourceFormatId(&'static SourceFormat);
impl std::ops::Deref for SourceFormatId {
    type Target = &'static SourceFormat;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for SourceFormatId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.0, other.0)
    }
}
impl Eq for SourceFormatId {}
impl std::hash::Hash for SourceFormatId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}
impl std::fmt::Debug for SourceFormatId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.name.fmt(f)
    }
}

impl std::fmt::Display for SourceFormatId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.name.fmt(f)
    }
}

impl ::serde::Serialize for SourceFormatId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.0.name)
    }
}

impl<'de> ::serde::Deserialize<'de> for SourceFormatId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SourceFormat::get(&s).map_or_else(
            || Err(::serde::de::Error::custom("Unknown source format")),
            Ok,
        )
    }
}

#[derive(Copy, Clone)]
pub struct BuildTargetId(&'static BuildTarget);
impl std::ops::Deref for BuildTargetId {
    type Target = &'static BuildTarget;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for BuildTargetId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.0, other.0)
    }
}
impl Eq for BuildTargetId {}
impl std::hash::Hash for BuildTargetId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}
impl std::fmt::Debug for BuildTargetId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.name.fmt(f)
    }
}

impl std::fmt::Display for BuildTargetId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.name.fmt(f)
    }
}

impl ::serde::Serialize for BuildTargetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.0.name)
    }
}

impl<'de> ::serde::Deserialize<'de> for BuildTargetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BuildTarget::get(&s).map_or_else(
            || Err(::serde::de::Error::custom("Unknown build target")),
            Ok,
        )
    }
}
