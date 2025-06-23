use std::{borrow::Cow, collections::hash_map::Entry, fmt::Display};

use crate::prelude::HMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CowStr {
    Borrowed(&'static str),
    Owned(Box<str>),
}
impl std::ops::Deref for CowStr {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s,
        }
    }
}
impl AsRef<str> for CowStr {
    #[inline]
    fn as_ref(&self) -> &str {
        &**self
    }
}

impl From<&'static str> for CowStr {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::Borrowed(s)
    }
}
impl From<String> for CowStr {
    #[inline]
    fn from(s: String) -> Self {
        Self::Owned(s.into_boxed_str())
    }
}
impl From<Box<str>> for CowStr {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::Owned(s)
    }
}
impl From<Cow<'static, str>> for CowStr {
    #[inline]
    fn from(s: Cow<'static, str>) -> Self {
        match s {
            Cow::Borrowed(s) => Self::Borrowed(s),
            Cow::Owned(s) => Self::Owned(s.into_boxed_str()),
        }
    }
}

impl Display for CowStr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct IdCounter {
    inner: HMap<CowStr, u32>,
}
impl Default for IdCounter {
    fn default() -> Self {
        let mut inner = HMap::default();
        inner.insert("EXTSTRUCT".into(), 0);
        Self { inner }
    }
}
impl IdCounter {
    pub fn new_id(&mut self, prefix: impl Into<CowStr>) -> Box<str> {
        let prefix = prefix.into();
        let r = match self.inner.entry(prefix) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
                format!("{}_{}", e.key(), e.get())
            }
            Entry::Vacant(e) => {
                let r = e.key().to_string();
                e.insert(0);
                r
            }
        };
        r.into_boxed_str()
    }
}
