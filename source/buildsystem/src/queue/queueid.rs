use std::{fmt::Display, num::NonZeroU32};

#[derive(Debug)]
pub struct QueueId(NonZeroU32);
impl QueueId {
    #[must_use]
    pub fn global() -> Self {
        Self(NonZeroU32::new(1).unwrap_or_else(|| unreachable!()))
    }
}
impl Display for QueueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "queue {}", self.0)
    }
}
impl From<QueueId> for NonZeroU32 {
    #[inline]
    fn from(id: QueueId) -> Self {
        id.0
    }
}
impl From<NonZeroU32> for QueueId {
    #[inline]
    fn from(id: NonZeroU32) -> Self {
        Self(id)
    }
}
