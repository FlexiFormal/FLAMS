use std::{num::NonZeroU32, path::PathBuf};

use either::Either;
use ftml_uris::{DocumentUri, UriPath};

use crate::taskmap::buildtask::buildstep::BuildStep;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuildTaskId(pub(crate) NonZeroU32);
impl From<BuildTaskId> for u32 {
    #[inline]
    fn from(id: BuildTaskId) -> Self {
        id.0.get()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BuildTaskI {
    id: BuildTaskId,
    uri: DocumentUri,
    steps: Box<[BuildStep]>,
    source: Either<PathBuf, String>,
    rel_path: UriPath,
}
