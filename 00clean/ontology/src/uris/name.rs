use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::num::NonZeroUsize;
use std::str::FromStr;
use lazy_static::lazy_static;
use immt_utils::gc::{TArcInterner, TArcInterned};
use triomphe::Arc;
use parking_lot::Mutex;
use smallvec::SmallVec;

lazy_static!{
    pub(super) static ref NAMES: Arc<Mutex<TArcInterner<str,4,1000>>> =
        Arc::new(Mutex::new(TArcInterner::default()));
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
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
    fn as_ref(&self) -> &str { self.0.as_ref() }

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
pub struct Name(pub(super) SmallVec<NameStep,4>);
impl Name {
    #[inline]
    #[must_use]
    pub fn steps(&self) -> &[NameStep] { &self.0 }
    #[inline]
    #[must_use]
    pub const fn is_simple(&self) -> bool { self.0.len() == 1 }
    #[inline]
    #[must_use]
    pub fn len(&self) -> NonZeroUsize { NonZeroUsize::new(self.0.len()).unwrap_or_else(|| unreachable!()) }

    #[inline]
    #[must_use]
    pub fn last_name(&self) -> &NameStep {
        self.0.last().unwrap_or_else(|| unreachable!())
    }
}
impl Display for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut steps = self.steps().iter();
        let Some(first) = steps.next() else { unreachable!() };
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
impl FromStr for Name {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let steps = s.split('/').map(|s|
            NameStep(NAMES.lock().get_or_intern(s))
        );
        Ok(Self(steps.collect()))
    }
}

#[allow(clippy::fallible_impl_from)]
impl<A:AsRef<str>> From<A> for Name {
    #[inline]
    fn from(s: A) -> Self {
        s.as_ref().parse().unwrap()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::uris::serialize;
    use super::{Name, NameStep};
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
            Ok(s.parse().unwrap_or_else(|_| unreachable!()))
        }
    }
}