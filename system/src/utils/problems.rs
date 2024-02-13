use std::sync::Arc;
use tracing::{event, warn};
use immt_api::utils::HMap;
/*
pub struct PHandler2 {
    id:&'static str,
    sender:std::sync::mpsc::Sender<String>,
    parent:Option<Box<PHandler2>>
}
impl PHandler2 {
    pub fn new() -> Self {
        let (send,recv) = std::sync::mpsc::channel();
    }
}

 */

#[derive(Clone)]
pub struct ProblemHandler(Arc<parking_lot::RwLock<ProblemHandlerI>>);
impl ProblemHandler {
    fn add_i(&self,kind:&'static str,message:String) {
        warn!("Problem {}: {}",kind,message);
        self.0.write().add(kind,message);
    }
}
impl Default for ProblemHandler {
    fn default() -> Self {
        ProblemHandler(Arc::new(ProblemHandlerI{problems:HMap::default(),counter:0}.into()))
    }
}
impl immt_api::utils::problems::ProblemHandler for ProblemHandler {
    fn add(&self,kind:&'static str,message:impl std::fmt::Display) {
        self.add_i(kind,message.to_string());
    }
}

struct ProblemHandlerI {
    problems:HMap<usize,Problem>,
    counter:usize
}
impl ProblemHandlerI {
    fn add(&mut self,kind:&'static str,message:String) {
        self.problems.insert(self.counter,Problem{kind,message});
        self.counter += 1;
    }
}

pub(crate) struct Problem {
    pub(crate) kind:&'static str,
    pub(crate) message:String
}