use flams_utils::time::Timestamp;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
