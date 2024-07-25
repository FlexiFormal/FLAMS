use std::path::Path;
use crate::building::formats::{FormatOrTarget, ShortId, SourceFormatId};
use crate::uris::archives::ArchiveId;
use crate::utils::filetree::FileChange;
use crate::utils::time::Delta;
use crate::utils::VecMap;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub enum BuildState {
    Deleted,
    New,
    Stale{last_built:u64,last_watched:u64,md5:u128},
    UpToDate{last_built:u64,last_watched:u64,md5:u128}
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Default)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct TargetState {
    pub new:u32,
    pub stale:(u32,u64,u64),
    pub up_to_date:(u32,u64,u64)
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,Default)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct AllStates {
    pub(crate) map:VecMap<SourceFormatId,TargetState>
}
impl AllStates {
    pub fn targets(&self) -> impl Iterator<Item = (SourceFormatId,&TargetState)> {
        self.map.iter().map(|(k,v)| (*k,v))
    }
    pub fn summary(&self) -> TargetState {
        let mut ret = TargetState {
            new:0,
            stale:(0,0,0),
            up_to_date:(0,0,0)
        };
        for (_,v) in self.map.iter() {
            ret.new += v.new;
            ret.stale.0 += v.stale.0;
            ret.stale.1 = std::cmp::max(ret.stale.1,v.stale.1);
            ret.stale.2 = std::cmp::max(ret.stale.2,v.stale.2);
            ret.up_to_date.0 += v.up_to_date.0;
            ret.up_to_date.1 = std::cmp::max(ret.up_to_date.1,v.up_to_date.1);
            ret.up_to_date.2 = std::cmp::max(ret.up_to_date.2,v.up_to_date.2);
        }
        ret
    }
    pub fn merge(&mut self, s: &BuildState,target:SourceFormatId) {
        let target = self.map.get_or_insert_mut(target, || TargetState::default());
        match s {
            BuildState::Deleted => {},
            BuildState::New => target.new += 1,
            BuildState::Stale { last_built,last_watched,md5:_ } => {
                let (n,lb,lw) = &mut target.stale;
                *n += 1;
                *lb = std::cmp::max(*lb,*last_built);
                *lw = std::cmp::max(*lw,*last_watched);
            },
            BuildState::UpToDate { last_built,last_watched,md5:_ } => {
                let (n,lb,lw) = &mut target.up_to_date;
                *n += 1;
                *lb = std::cmp::max(*lb,*last_built);
                *lw = std::cmp::max(*lw,*last_watched);
            }
        }
    }
    pub fn merge_cum(&mut self, o: &AllStates) {
        for (k,v) in o.map.iter() {
            let m = self.map.get_or_insert_mut(*k, || TargetState::default());
            m.new += v.new;
            m.stale.0 += v.stale.0;
            m.stale.1 = std::cmp::max(m.stale.1,v.stale.1);
            m.stale.2 = std::cmp::max(m.stale.2,v.stale.2);
            m.up_to_date.0 += v.up_to_date.0;
            m.up_to_date.1 = std::cmp::max(m.up_to_date.1,v.up_to_date.1);
            m.up_to_date.2 = std::cmp::max(m.up_to_date.2,v.up_to_date.2);
        }
    }
}
impl BuildState {
    crate::asyncs!{!pub fn update(&mut self,path:&Path,md:std::fs::Metadata,mut on_change:impl FnMut(FileChange)) -> bool {
        match self {
            Self::New => false,
            Self::Deleted => {
                on_change(FileChange{ previous:Some(Self::Deleted), new:Self::New });
                *self = Self::New;
                true
            }
            Self::UpToDate { last_watched,md5:md5s,last_built } => {
                let last_changed = md.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                if last_changed < *last_watched {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    if let Ok(md) = read_file!(path).map(|s| u128::from_be_bytes(md5::compute(s).0)) {
                        if md == *md5s {
                            *last_watched = now;
                        } else {
                            let state = Self::Stale { last_watched: now, md5: *md5s, last_built: *last_built };
                            on_change(FileChange{ previous: Some(BuildState::UpToDate { last_watched: *last_watched, md5: *md5s, last_built: *last_built }),
                                new: state.clone() });
                            *self = state;
                        }
                    }
                    true
                } else {false}
            },
            BuildState::Stale { last_watched,md5:md5s,last_built } => {
                let last_changed = md.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                if last_changed < *last_watched {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    if let Ok(md) = read_file!(path).map(|s| u128::from_be_bytes(md5::compute(s).0)) {
                        if md == *md5s {
                            let state = BuildState::UpToDate { last_watched: now, md5: *md5s, last_built: *last_built };
                            on_change(FileChange{ previous: Some(BuildState::Stale { last_watched: *last_watched, md5: *md5s, last_built: *last_built }),
                                new: state.clone() });
                            *self = state;
                        } else {
                            *last_watched = now;
                        }
                    }
                    true
                } else {false}
            }
        }
    }}
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct QueueEntry {
    pub archive:ArchiveId,
    pub rel_path:String,
    pub target:FormatOrTarget,
    pub step:(u8,u8)
}
impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.archive == other.archive && self.rel_path == other.rel_path && self.target == other.target
    }
}
impl QueueEntry {
    pub fn id(&self) -> String {
        md5::compute(format!("({},{},{})", self.archive,self.rel_path, self.target)).0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub enum QueueMessage {
    Idle {id:String,entries:Vec<QueueEntry>},
    Started {id:String,queue:Vec<QueueEntry>,blocked:Vec<QueueEntry>,failed:Vec<QueueEntry>,done:Vec<QueueEntry>,eta:Delta},
    TaskStarted{id:String,entry:QueueEntry,eta:Delta},
    TaskDoneRequeued{id:String,entry:QueueEntry,index:usize,eta:Delta},
    TaskDoneFinished{id:String,entry:QueueEntry,eta:Delta},
    TaskFailed{id:String,entry:QueueEntry,eta:Delta}
}