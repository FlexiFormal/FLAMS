use flams_utils::gc::{TArcInterned, TArcInterner};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::num::NonZeroUsize;
use std::str::FromStr;
use triomphe::Arc;

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section))]
const TS_NAME: &str = "export type Name = string;";

lazy_static! {
    pub(super) static ref NAMES: Arc<Mutex<TArcInterner<str, 4, 100_000>>> =
        Arc::new(Mutex::new(TArcInterner::default()));
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NameStep(pub(super) TArcInterned<str>);
impl Ord for NameStep {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}
impl PartialOrd for NameStep {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl AsRef<str> for NameStep {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for NameStep {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}
impl Debug for NameStep {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name(pub(super) SmallVec<NameStep, 4>);
impl Name {
    #[inline]
    #[must_use]
    pub fn steps(&self) -> &[NameStep] {
        &self.0
    }
    #[inline]
    #[must_use]
    pub const fn is_simple(&self) -> bool {
        self.0.len() == 1
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.0.len()).unwrap_or_else(|| unreachable!())
    }

    #[inline]
    #[must_use]
    pub fn last_name(&self) -> &NameStep {
        self.0.last().unwrap_or_else(|| unreachable!())
    }

    #[must_use]
    pub fn with_last_name(mut self,s:NameStep) -> Self {
        if self.0.len() == 1 {
            Self(smallvec::smallvec![s])
        } else {
            self.0.pop();
            self.0.push(s);
            self
        }
    }
    #[inline]
    #[must_use]
    pub fn first_name(&self) -> &NameStep {
        self.0.first().unwrap_or_else(|| unreachable!())
    }
}
impl Display for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut steps = self.steps().iter();
        let Some(first) = steps.next() else {
            unreachable!()
        };
        Display::fmt(first, f)?;
        for step in steps {
            write!(f, "/{step}")?;
        }
        Ok(())
    }
}
impl Debug for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

pub const INVALID_CHARS: [char;3] = ['\\','{','}'];

#[derive(Debug)]
pub struct InvalidURICharacter;
impl Display for InvalidURICharacter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid URI character")
    }
}

impl FromStr for Name {
    type Err = InvalidURICharacter;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(INVALID_CHARS) || s.is_empty() {
            return Err(InvalidURICharacter);
        }
        let steps = s
            .split('/')
            .map(|s| NameStep(NAMES.lock().get_or_intern(s)));
        Ok(Self(steps.collect()))
    }
}

impl From<NameStep> for Name {
    #[inline]
    fn from(step: NameStep) -> Self {
        Self(SmallVec::from([step]))
    }
}

/*impl<A: AsRef<str>> TryFrom<A> for Name {
    type Error = InvalidURICharacter;
    #[inline]
    fn try_from(s: A) -> Result<Self,InvalidURICharacter> {
        s.as_ref().parse()
    }
}*/

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{InvalidURICharacter, Name, NameStep};
    use crate::uris::serialize;
    serialize!(as NameStep);
    impl<'de> serde::Deserialize<'de> for NameStep {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Ok(Self(super::NAMES.lock().get_or_intern(s.as_str())))
        }
    }
    serialize!(Name);
    impl<'de> serde::Deserialize<'de> for Name {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(|e:InvalidURICharacter| serde::de::Error::custom(e.to_string()))
        }
    }
}
