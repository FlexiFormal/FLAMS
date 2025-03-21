#![recursion_limit = "256"]

use flams_ontology::file_states::FileStateSummary;
use flams_utils::vecmap::VecMap;

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod components;
pub mod index_components;
pub mod server_fns;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct FileStates(VecMap<String, FileStateSummary>);

#[cfg(feature = "ssr")]
impl From<flams_system::backend::archives::source_files::FileStates> for FileStates {
    fn from(value: flams_system::backend::archives::source_files::FileStates) -> Self {
        Self(
            value
                .formats
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

#[cfg(feature = "ssr")]
impl
    From<
        &VecMap<
            flams_system::formats::BuildTargetId,
            flams_system::backend::archives::source_files::FileState,
        >,
    > for FileStates
{
    fn from(
        value: &VecMap<
            flams_system::formats::BuildTargetId,
            flams_system::backend::archives::source_files::FileState,
        >,
    ) -> Self {
        use flams_system::backend::archives::source_files::FileState;
        Self(
            value
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        match v {
                            FileState::New => FileStateSummary {
                                new: 1,
                                ..Default::default()
                            },
                            FileState::Stale(s) => FileStateSummary {
                                stale: 1,
                                last_built: s.last_built,
                                last_changed: s.last_changed,
                                ..Default::default()
                            },
                            FileState::UpToDate(s) => FileStateSummary {
                                up_to_date: 1,
                                last_built: s.last_built,
                                ..Default::default()
                            },
                            FileState::Deleted => FileStateSummary {
                                deleted: 1,
                                ..Default::default()
                            },
                        },
                    )
                })
                .collect(),
        )
    }
}
