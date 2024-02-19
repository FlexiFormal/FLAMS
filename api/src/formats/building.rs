use crate::archives::ArchiveId;
use crate::formats::Id;
use crate::utils::HMap;
use crate::{CloneStr, FinalStr};
use async_trait::async_trait;
use std::any::Any;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Backend<'a> {
    pub mathhub: &'a Path,
    pub get_path: Box<dyn Fn(&ArchiveId) -> Option<Arc<Path>> + 'a>,
}

impl<'a> Backend<'a> {
    pub fn get_path(&self, id: &ArchiveId) -> Option<Arc<Path>> {
        (self.get_path)(id)
    }
}

pub struct BuildData {
    build_path: Option<Arc<Path>>,
    source: OnceCell<Option<FinalStr>>,
    pub state: HMap<&'static str, Box<dyn Any + Send>>,
}
impl BuildData {
    pub fn new(path: Option<Arc<Path>>) -> Self {
        Self {
            build_path: path,
            source: OnceCell::new(),
            state: HMap::default(),
        }
    }
    pub fn build_file(&self) -> Option<&Path> {
        self.build_path.as_deref()
    }
    #[cfg(feature = "fs")]
    pub fn source(&self) -> Option<&str> {
        self.source
            .get_or_init(|| match &self.build_path {
                None => None,
                Some(p) => std::fs::read_to_string(&**p).ok().map(|s| s.into()),
            })
            .as_deref()
    }
}

pub struct BuildInfo {
    pub archive_id: ArchiveId,
    pub format: Id,
    pub rel_path: CloneStr,
    pub archive_path: Option<Arc<Path>>,
    pub state_data: BuildData,
}
impl BuildInfo {
    #[cfg(feature = "fs")]
    pub fn source(&self) -> Option<&str> {
        self.state_data.source()
    }
    pub fn path(&self) -> Option<&Path> {
        self.state_data.build_file()
    }
}

pub enum BuildResult {
    Err(CloneStr),
    Success
}

pub trait TaskStep: Any + Send + Sync {
    fn run(&self, input: &mut BuildData, backend: &Backend<'_>) -> BuildResult;
}

#[derive(Clone)]
pub enum Dependency {
    Physical {
        id: &'static str,
        archive: ArchiveId,
        filepath: CloneStr,
        strong: bool,
    },
    Logical, // TODO
}

#[derive(Clone)]
pub enum BuildStepKind {
    Source(Arc<dyn TaskStep>),
    Check,
}
pub struct BuildStep {
    pub kind: BuildStepKind,
    pub dependencies: Vec<Dependency>,
    pub id: &'static str,
}
