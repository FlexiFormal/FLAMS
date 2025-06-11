use std::{path::Path, sync::Arc};

use either::Either;
use flams_ontology::{
    file_states::FileStateSummary,
    uris::{ArchiveURIRef, URIRefTrait},
};
use flams_utils::{
    change_listener::ChangeSender,
    prelude::{TreeChild, TreeLike},
    time::Timestamp,
    vecmap::VecMap,
    PathExt,
};

use super::ignore_regex::IgnoreSource;
use crate::{
    backend::{
        archives::{HasLocalOut, LocalArchive},
        BackendChange,
    },
    formats::{BuildTargetId, SourceFormatId},
};

#[derive(Debug)]
pub enum SourceEntry {
    Dir(SourceDir),
    File(SourceFile),
}
impl SourceEntry {
    #[inline]
    #[must_use]
    pub fn relative_path(&self) -> &str {
        match self {
            Self::Dir(dir) => &dir.relative_path,
            Self::File(file) => &file.relative_path,
        }
    }
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        let rel_path = self.relative_path();
        rel_path.rsplit_once('/').map_or(rel_path, |(_, b)| b)
    }
}

#[derive(Debug, Default)]
pub struct SourceDir {
    pub children: Vec<SourceEntry>,
    pub relative_path: Arc<str>,
    pub state: FileStates,
}
impl SourceDir {
    #[inline]
    #[must_use]
    pub const fn state(&self) -> &FileStates {
        &self.state
    }
}

#[derive(Debug)]
pub struct SourceFile {
    pub relative_path: Arc<str>,
    pub format: SourceFormatId,
    pub target_state: VecMap<BuildTargetId, FileState>,
    pub format_state: FileState,
}

impl TreeChild<SourceEntry> for &SourceEntry {
    fn children<'a>(&self) -> Option<<SourceEntry as TreeLike>::RefIter<'a>>
    where
        Self: 'a,
    {
        TreeLike::children(*self)
    }
}
impl TreeLike for SourceEntry {
    type Child<'a> = &'a Self;
    type RefIter<'a> = std::slice::Iter<'a, Self>;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        match self {
            Self::Dir(dir) => Some(dir.children.iter()),
            Self::File(_) => None,
        }
    }
}

impl TreeChild<SourceDir> for &SourceEntry {
    fn children<'a>(&self) -> Option<<SourceEntry as TreeLike>::RefIter<'a>>
    where
        Self: 'a,
    {
        TreeLike::children(*self)
    }
}

impl TreeLike for SourceDir {
    type Child<'a> = &'a SourceEntry;
    type RefIter<'a> = std::slice::Iter<'a, SourceEntry>;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        Some(self.children.iter())
    }
}

impl SourceDir {
    #[inline]
    fn index(&self, s: &str) -> Result<usize, usize> {
        self.children.binary_search_by_key(&s, SourceEntry::name)
    }

    #[must_use]
    pub fn find(&self, rel_path: &str) -> Option<Either<&Self, &SourceFile>> {
        let mut segments = rel_path.split('/');
        let mut current = self;
        while let Some(seg) = segments.next() {
            match current.index(seg) {
                Ok(i) => match &current.children[i] {
                    SourceEntry::Dir(dir) => {
                        current = dir;
                    }
                    SourceEntry::File(f) if segments.next().is_none() => {
                        return Some(Either::Right(f))
                    }
                    SourceEntry::File(_) => return None,
                },
                _ => return None,
            }
        }
        Some(Either::Left(current))
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    fn find_mut(&mut self, rel_path: &str) -> Option<Either<&mut Self, &mut SourceFile>> {
        let mut segments = rel_path.split('/');
        let mut current = self;
        while let Some(seg) = segments.next() {
            match current.index(seg) {
                Ok(i) => match &mut current.children[i] {
                    SourceEntry::Dir(dir) => {
                        current = dir;
                    }
                    SourceEntry::File(f) if segments.next().is_none() => {
                        return Some(Either::Right(f))
                    }
                    _ => return None,
                },
                _ => return None,
            }
        }
        Some(Either::Left(current))
    }

    fn remove(&mut self, s: &str) -> Option<SourceEntry> {
        let Some((p, r)) = s.rsplit_once('/') else {
            return if let Ok(i) = self.index(s) {
                Some(self.children.remove(i))
            } else {
                None
            };
        };
        match self.find_mut(p) {
            Some(Either::Left(d)) => {
                if let Ok(i) = d.index(r) {
                    let r = d.children.remove(i);
                    if d.children.is_empty() {
                        self.remove(p);
                    }
                    Some(r)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn insert(&mut self, f: SourceFile) {
        // TODO this logic overwrites existing entries, which would screw up the states.
        // In practice, that should never happen anyway.
        self.state.merge(f.format, &f.format_state);
        let Some((_, last)) = f.relative_path.rsplit_once('/') else {
            match self.index(&f.relative_path) {
                Ok(i) => self.children[i] = SourceEntry::File(f),
                Err(i) => self.children.insert(i, SourceEntry::File(f)),
            }
            return;
        };
        let mut curr_relpath = "";
        let steps = f.relative_path.split('/');
        let mut current = self;
        for step in steps {
            if step == last {
                break;
            }
            curr_relpath = if curr_relpath.is_empty() {
                step
            } else {
                &f.relative_path[..curr_relpath.len() + 1 + step.len()]
            };
            match current.index(step) {
                Ok(i) => {
                    if matches!(current.children[i], SourceEntry::Dir(_)) {
                        let SourceEntry::Dir(dir) = &mut current.children[i] else {
                            unreachable!()
                        };
                        dir.state.merge(f.format, &f.format_state);
                        current = dir;
                        continue;
                    }
                    let mut dir = Self {
                        children: Vec::new(),
                        relative_path: curr_relpath.into(),
                        state: FileStates::default(),
                    };
                    dir.state.merge(f.format, &f.format_state);
                    current.children[i] = SourceEntry::Dir(dir);
                    current = if let SourceEntry::Dir(d) = &mut current.children[i] {
                        d
                    } else {
                        unreachable!()
                    };
                }
                Err(i) => {
                    let mut dir = Self {
                        children: Vec::new(),
                        relative_path: curr_relpath.into(),
                        state: FileStates::default(),
                    };
                    dir.state.merge(f.format, &f.format_state);
                    current.children.insert(i, SourceEntry::Dir(dir));
                    current = if let SourceEntry::Dir(d) = &mut current.children[i] {
                        d
                    } else {
                        unreachable!()
                    };
                }
            };
        }
        match current.index(last) {
            Ok(i) => current.children[i] = SourceEntry::File(f),
            Err(i) => current.children.insert(i, SourceEntry::File(f)),
        }
    }

    pub(crate) fn update(
        &mut self,
        archive: ArchiveURIRef,
        top: &Path,
        sender: &ChangeSender<BackendChange>,
        ignore: &IgnoreSource,
        formats: &[SourceFormatId],
    ) {
        let filter = |e: &walkdir::DirEntry| {
            if ignore.ignores(e.path()) {
                tracing::trace!(target:"archives","Ignoring {} because of {}",e.path().display(),ignore);
                false
            } else {
                true
            }
        };
        let mut old = std::mem::take(self);
        let Some(topstr) = top.to_str() else {
            unreachable!()
        };

        for entry in walkdir::WalkDir::new(LocalArchive::source_dir_of(top))
            .min_depth(1)
            .into_iter()
            .filter_entry(filter)
            .filter_map(Result::ok)
        {
            let Ok(metadata) = entry.metadata() else {
                tracing::warn!(target:"archives","Invalid metadata: {}",entry.path().display());
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(format) = formats.iter().find(|t| t.file_exts().contains(&ext)) else {
                continue;
            };
            let Some(relative_path) = entry.path().to_str() else {
                tracing::warn!(target:"archives","Invalid path: {}",entry.path().display());
                continue;
            };
            let Some(relative_path) = relative_path.strip_prefix(topstr).and_then(|s| {
                s.strip_prefix(const_format::concatcp!(
                    std::path::PathBuf::PATH_SEPARATOR,
                    "source",
                    std::path::PathBuf::PATH_SEPARATOR
                ))
            }) else {
                unreachable!("{relative_path} does not start with {topstr}???")
            };
            #[cfg(target_os = "windows")]
            let relative_path: Arc<str> = relative_path
                .replace(std::path::PathBuf::PATH_SEPARATOR, "/")
                .to_string()
                .into();
            #[cfg(not(target_os = "windows"))]
            let relative_path: Arc<str> = relative_path.to_string().into();
            let states = FileState::from(top, &metadata, &relative_path, *format);
            let new = SourceFile {
                relative_path,
                format: *format,
                format_state: states
                    .iter()
                    .map(|(_, v)| v)
                    .min()
                    .cloned()
                    .unwrap_or(FileState::New),
                target_state: states,
            };
            if let Some(SourceEntry::File(previous)) = old.remove(&new.relative_path) {
                if previous.format_state != new.format_state {
                    sender.lazy_send(|| BackendChange::FileChange {
                        archive: URIRefTrait::owned(archive),
                        relative_path: new.relative_path.to_string(),
                        format: new.format,
                        old: Some(previous.format_state),
                        new: new.format_state.clone(),
                    });
                }
            } else {
                sender.lazy_send(|| BackendChange::FileChange {
                    archive: URIRefTrait::owned(archive),
                    relative_path: new.relative_path.to_string(),
                    format: new.format,
                    old: None,
                    new: new.format_state.clone(),
                });
            }
            self.insert(new);
        }
        // TODO deleted?
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ChangeState {
    pub last_built: Timestamp,
    pub last_changed: Timestamp,
    //last_watched:Timestamp,
    //md5:u128 TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FileState {
    Deleted,
    New,
    Stale(ChangeState),
    UpToDate(ChangeState),
}
impl PartialOrd for FileState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FileState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Deleted, Self::Deleted)
            | (Self::New, Self::New)
            | (Self::Stale(_), Self::Stale(_))
            | (Self::UpToDate(_), Self::UpToDate(_)) => std::cmp::Ordering::Equal,
            (Self::Deleted, _) => std::cmp::Ordering::Less,
            (_, Self::Deleted) => std::cmp::Ordering::Greater,
            (Self::New, _) => std::cmp::Ordering::Less,
            (_, Self::New) => std::cmp::Ordering::Greater,
            (Self::Stale(_), _) => std::cmp::Ordering::Less,
            (_, Self::Stale(_)) => std::cmp::Ordering::Greater,
        }
    }
}
impl FileState {
    fn from(
        top: &Path,
        source: &std::fs::Metadata,
        relative_path: &str,
        format: SourceFormatId,
    ) -> VecMap<BuildTargetId, Self> {
        let out = LocalArchive::out_dir_of(top).join(relative_path);
        let mut ret = VecMap::new();
        for t in *format.targets() {
            let log = out.join(t.name()).with_extension("log");
            if !log.exists() {
                ret.insert(*t, Self::New);
                continue;
            }
            let Ok(meta) = log.metadata() else {
                ret.insert(*t, Self::New);
                continue;
            };
            let Ok(last_built) = meta.modified() else {
                ret.insert(*t, Self::New);
                continue;
            };
            let Ok(last_changed) = source.modified() else {
                ret.insert(*t, Self::New);
                continue;
            };
            if last_built > last_changed {
                ret.insert(
                    *t,
                    Self::UpToDate(ChangeState {
                        last_built: last_built.into(),
                        last_changed: last_changed.into(),
                    }),
                );
            } else {
                ret.insert(
                    *t,
                    Self::Stale(ChangeState {
                        last_built: last_built.into(),
                        last_changed: last_changed.into(),
                    }),
                );
            }
        }
        ret
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct FileStates {
    pub formats: VecMap<SourceFormatId, FileStateSummary>,
}
impl FileStates {
    #[must_use]
    pub fn summarize(&self) -> FileStateSummary {
        let mut ret = FileStateSummary::default();
        for (_, v) in self.formats.iter() {
            ret.new = ret.new.max(v.new);
            ret.stale = ret.stale.max(v.stale);
            ret.up_to_date = ret.up_to_date.max(v.up_to_date);
            ret.last_built = std::cmp::max(ret.last_built, v.last_built);
            ret.last_changed = std::cmp::max(ret.last_changed, v.last_changed);
        }
        ret
    }

    pub(crate) fn merge(&mut self, format: SourceFormatId, state: &FileState) {
        let target = self
            .formats
            .get_or_insert_mut(format, FileStateSummary::default);
        match state {
            FileState::Deleted => target.deleted += 1,
            FileState::New => target.new += 1,
            FileState::Stale(s) => {
                target.stale += 1;
                target.last_built = std::cmp::max(target.last_built, s.last_built);
                target.last_changed = std::cmp::max(target.last_changed, s.last_changed);
            }
            FileState::UpToDate(s) => {
                target.up_to_date += 1;
                target.last_built = std::cmp::max(target.last_built, s.last_built);
            }
        }
    }
    /*#[inline]
    pub(crate) fn merge_one(&mut self, map: &VecMap<SourceFormatId, FileState>) {
        for (k, v) in map.iter() {
            self.merge(*k, v);
        }
    }*/
    pub(crate) fn merge_summary(&mut self, format: SourceFormatId, summary: &FileStateSummary) {
        let target = self
            .formats
            .get_or_insert_mut(format, FileStateSummary::default);
        target.new += summary.new;
        target.stale += summary.stale;
        target.deleted += summary.deleted;
        target.up_to_date += summary.up_to_date;
        target.last_built = std::cmp::max(target.last_built, summary.last_built);
        target.last_changed = std::cmp::max(target.last_changed, summary.last_changed);
    }
    pub(crate) fn merge_all(&mut self, other: &Self) {
        for (k, v) in other.formats.iter() {
            self.merge_summary(*k, v);
        }
    }
}
