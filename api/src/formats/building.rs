use std::any::Any;
use std::future::Future;
use std::path::{Path, PathBuf};
use async_trait::async_trait;
use crate::Str;

pub enum BuildResult {
    None,
    Err(Str),
    Intermediate(Box<dyn Any>),
    Final
}

#[async_trait]
pub trait SourceTaskStep:Any {
    async fn run(&self,file:&Path) -> BuildResult;
}
#[async_trait]
pub trait ComplexTaskStep:Any {
    async fn run(&self,input:Box<dyn Any+Send>) -> BuildResult;
}

pub enum BuildTaskStep {
    Source(Box<dyn SourceTaskStep>),
    Complex(Box<dyn ComplexTaskStep>)
}

pub struct BuildTask {
    pub steps:Vec<BuildTaskStep>,
    pub state:Option<Box<dyn Any+Send>>
}
