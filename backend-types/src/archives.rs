use ftml_ontology::utils::time::Timestamp;
use ftml_uris::ArchiveId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FileStateSummary {
    pub new: u32,
    pub stale: u32,
    pub deleted: u32,
    pub up_to_date: u32,
    pub last_built: Timestamp,
    pub last_changed: Timestamp,
}
impl Default for FileStateSummary {
    fn default() -> Self {
        Self {
            new: 0,
            stale: 0,
            up_to_date: 0,
            deleted: 0,
            last_built: Timestamp::zero(),
            last_changed: Timestamp::zero(),
        }
    }
}
impl FileStateSummary {
    pub fn merge(&mut self, other: Self) {
        self.new += other.new;
        self.stale += other.stale;
        self.deleted += other.deleted;
        self.up_to_date += other.up_to_date;
        self.last_built = self.last_built.max(other.last_built);
        self.last_changed = self.last_changed.max(other.last_changed);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ArchiveData {
    pub id: ArchiveId,
    #[cfg_attr(feature = "serde", serde(default))]
    pub git: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summary: Option<FileStateSummary>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ArchiveGroupData {
    pub id: ArchiveId,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summary: Option<FileStateSummary>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DirectoryData {
    pub rel_path: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summary: Option<FileStateSummary>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FileData {
    pub rel_path: String,
    pub format: String,
}
