use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::event;

#[derive(Clone)]
pub struct ProblemHandler(Arc<parking_lot::RwLock<ProblemHandlerI>>);
impl ProblemHandler {
    pub fn add(&self,kind:&'static str,message:impl std::fmt::Display) {
        self.add_i(kind,message.to_string());
    }
    fn add_i(&self,kind:&'static str,message:String) {
        event!(tracing::Level::WARN,"Problem {}: {}",kind,message);
        self.0.write().add(kind,message);
    }
    pub(crate) fn new() -> Self {
        ProblemHandler(Arc::new(ProblemHandlerI{problems:BTreeMap::new(),counter:0}.into()))
    }
}

struct ProblemHandlerI {
    problems:BTreeMap<usize,Problem>,
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