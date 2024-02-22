use crate::archives::{ArchiveId, ArchiveIdRef};
use crate::formats::Id;
use crate::narration::document::Document;
use crate::uris::{ArchiveURI, ArchiveURIRef, DocumentURI};
use crate::utils::HMap;
use crate::{CloneStr, FinalStr};
use async_trait::async_trait;
use std::any::Any;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Backend<'a> {
    pub mathhub: &'a Path,
    pub get_path:
        Box<dyn Fn(ArchiveIdRef<'_>) -> (Option<Arc<Path>>, Option<ArchiveURIRef<'a>>) + 'a>,
    pub find_archive: Box<dyn Fn(&Path) -> Option<ArchiveURI> + 'a>,
}

impl<'a> Backend<'a> {
    pub fn get_path(&self, id: ArchiveIdRef<'_>) -> (Option<Arc<Path>>, Option<ArchiveURIRef<'a>>) {
        (self.get_path)(id)
    }
    pub fn find_archive(&self, path: &Path) -> Option<ArchiveURI> {
        (self.find_archive)(path)
    }
}

pub struct BuildData {
    pub build_path: Option<Arc<Path>>,
    pub rel_path: CloneStr,
    pub archive_path: Option<Arc<Path>>,
    pub archive_uri: ArchiveURI,
    source: OnceCell<Option<FinalStr>>,
    pub state: HMap<&'static str, Box<dyn Any + Send>>,
    pub document: Option<(Document, FinalStr)>,
}
impl BuildData {
    pub fn new(
        build_path: Option<Arc<Path>>,
        archive_path: Option<Arc<Path>>,
        archive_uri: ArchiveURI,
        rel_path: CloneStr,
    ) -> Self {
        Self {
            build_path,
            archive_path,
            archive_uri,
            rel_path,
            document: None,
            source: OnceCell::new(),
            state: HMap::default(),
        }
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
    pub format: Id,
    pub state_data: BuildData,
}
impl BuildInfo {
    #[cfg(feature = "fs")]
    pub fn source(&self) -> Option<&str> {
        self.state_data.source()
    }
    pub fn build_path(&self) -> Option<&Path> {
        self.state_data.build_path.as_deref()
    }
}

pub enum BuildResult {
    Err(CloneStr),
    Success,
}

pub trait TaskStep: Any + Send + Sync {
    fn run(&self, input: &mut BuildData, backend: &Backend<'_>) -> BuildResult;
}

#[derive(Clone)]
pub enum Dependency {
    Physical {
        id: &'static str,
        archive: ArchiveURI,
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
