use std::num::NonZeroU32;

pub mod server_fns;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GitState {
    None,
    Queued {
        commit: String,
        queue: NonZeroU32,
    },
    Live {
        commit: String,
        updates: Vec<(String, flams_git::Commit)>,
    },
}
