use std::num::NonZeroU32;

use flams_utils::prelude::HMap;

use flams_math_archives::formats::BuildTargetId;
use flams_math_archives::formats::TaskRef;
use ftml_uris::ArchiveId;
use ftml_uris::UriPath;

use crate::taskmap::buildtask::BuildTask;

pub mod buildtask;

#[derive(Debug)]
pub struct TaskMap {
    pub map: HMap<(ArchiveId, UriPath), BuildTask>,
    pub dependents: HMap<TaskRef, Vec<(BuildTask, BuildTargetId)>>,
    pub counter: NonZeroU32,
    pub total: usize,
}

impl Default for TaskMap {
    fn default() -> Self {
        Self {
            map: HMap::default(),
            dependents: HMap::default(),
            counter: NonZeroU32::new(1).unwrap_or_else(|| unreachable!()),
            total: 0,
        }
    }
}
