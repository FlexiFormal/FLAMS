use std::collections::VecDeque;
use immt_core::utils::triomphe::Arc;
use crate::building::targets::{BuildDataFormat, BuildTarget, SourceFormat};

struct QueuedTask();

struct Queue {
    queue: VecDeque<QueuedTask>,
}
struct BuildQueueI {
    source_formats:Box<[SourceFormat]>,
    build_data_formats: Box<[BuildDataFormat]>,
    build_targets: Box<[BuildTarget]>,
    inner: Queue
}
pub struct BuildQueue(Arc<BuildQueueI>);
impl BuildQueue {
    pub fn new(source_formats:Box<[SourceFormat]>,build_data_formats: Box<[BuildDataFormat]>,build_targets: Box<[BuildTarget]>) -> Self {
        Self(Arc::new(BuildQueueI {
            source_formats,
            build_data_formats,
            build_targets,
            inner: Queue {
                queue: VecDeque::new()
            }
        }))
    }
    pub fn formats(&self) -> &[SourceFormat] {
        &self.0.source_formats
    }
}
