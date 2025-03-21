#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

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
