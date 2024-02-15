use std::any::Any;
use std::path::Path;
use async_trait::async_trait;
use crate::CloneStr;
use crate::formats::Id;

pub enum BuildResult {
    None,
    Err(CloneStr),
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

pub enum BuildTaskStepKind {
    Source(Box<dyn SourceTaskStep>),
    Complex(Box<dyn ComplexTaskStep>)
}
pub struct BuildTaskStep {
    pub kind:BuildTaskStepKind,
    pub id:Id
}

pub struct BuildTask {
    pub steps:Vec<BuildTaskStep>,
    pub state:Option<Box<dyn Any+Send>>
}
