use immt_utils::time::Timestamp;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileStateSummary {
    pub new: u32,
    pub stale: u32,
    pub deleted: u32,
    pub up_to_date: u32,
    pub last_built: Timestamp,
    pub last_changed: Timestamp,
}
impl Default for FileStateSummary {
    #[must_use]
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