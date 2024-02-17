use std::any::Any;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use crate::archives::ArchiveId;
use crate::{CloneStr, FinalStr};
use crate::formats::Id;

pub struct Backend<'a> {
    pub get_path: &'a dyn Fn(&ArchiveId) -> Option<Arc<Path>>,
}
impl<'a,F> From<&'a F> for Backend<'a> where F:Fn(&ArchiveId) -> Option<Arc<Path>> {
    fn from(f:&'a F) -> Self {
        Self { get_path:f }
    }
}

impl<'a> Backend<'a> {
    pub fn get_path(&self,id:&ArchiveId) -> Option<Arc<Path>> {
        (self.get_path)(id)
    }
}

pub struct BuildInfo {
    pub archive_id:ArchiveId,
    pub format:Id,
    pub rel_path:CloneStr,
    pub archive_path:Option<Arc<Path>>,
    pub build_path:Option<Arc<Path>>,
    pub source:OnceCell<Option<FinalStr>>
}
impl BuildInfo {
    #[cfg(feature = "fs")]
    pub fn source(&self) -> Option<&str> {
        self.source.get_or_init(|| {
            match &self.build_path {
                None => None,
                Some(p) => {
                    std::fs::read_to_string(&**p).ok().map(|s| s.into())
                }
            }
        }).as_deref()
    }
}

pub enum BuildResult {
    None,
    Err(CloneStr),
    Intermediate(Box<dyn Any>),
    Final
}

#[async_trait]
pub trait SourceTaskStep:Any+Send {
    async fn run(&self,file:&Path) -> BuildResult;
}
#[async_trait]
pub trait ComplexTaskStep:Any+Send {
    async fn run(&self,input:Box<dyn Any+Send>) -> BuildResult;
}

pub enum Dependency {
    Physical {
        id:&'static str,
        archive:ArchiveId,
        filepath:CloneStr,
        strong:bool
    },
    Logical // TODO
}

pub enum BuildStepKind {
    Source(Box<dyn SourceTaskStep>),
    Complex(Box<dyn ComplexTaskStep>),
    Check
}
pub struct BuildStep {
    pub kind: BuildStepKind,
    pub dependencies:Vec<Dependency>,
    pub id:&'static str
}

pub struct BuildTask {
    pub steps:Vec<BuildStep>,
    pub state:Option<Box<dyn Any+Send>>
}
