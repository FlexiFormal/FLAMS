use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tracing::{Instrument, instrument};
use immt_core::building::formats::{BuildTargetId, FormatOrTarget, SourceFormatId};
use immt_core::uris::archives::{ArchiveId, ArchiveURI, ArchiveURIRef};
use immt_core::uris::modules::ModuleURI;
use immt_core::utils::triomphe;
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::Storage;
use crate::building::targets::BuildFormatId;
use crate::controller::{Controller, ControllerAsync};
use crate::extensions::FormatExtension;
use crate::HMap;

#[cfg(feature = "async")]
type Lock<A> = tokio::sync::RwLock<A>;
#[cfg(not(feature = "async"))]
type Lock<A> = parking_lot::RwLock<A>;

#[derive(Debug)]
struct StepData {
    done:Option<Result<String,PathBuf>>,
    needs_repeating:bool,
    out:Option<Box<dyn Any+Send+Sync>>,
    yields:Box<[ModuleURI]>,
    requires:Vec<Dependency>,
    dependents:Vec<(BuildTask,u8)>
}

#[derive(Debug)]
pub struct BuildStep {
    target:BuildTargetId,
    data:Lock<StepData>
}
impl BuildStep {
    immt_core::asyncs!{ALT !pub fn push_dependency(&self,dependency: Dependency) {
        let mut data = wait!(self.data.write());
        data.requires.push(dependency);
    }}
}

#[derive(Debug)]
struct BuildTaskI {
    archive: ArchiveURI,
    base_path:PathBuf,
    steps: Box<[BuildStep]>,
    next: Option<u8>,
    path:PathBuf,
    rel_path:Box<str>,
    format:FormatOrTarget,
}
impl Hash for BuildTaskI {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.archive.id().hash(state);
        self.rel_path.hash(state);
        self.format.hash(state);
    }
}
impl PartialEq for BuildTaskI {
    fn eq(&self, other: &Self) -> bool {
        self.archive.id() == other.archive.id() && self.rel_path == other.rel_path && self.format == other.format
    }
}
impl Eq for BuildTaskI {}

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct BuildTask(triomphe::Arc<BuildTaskI>);
impl BuildTask {
    pub fn archive(&self) -> ArchiveURIRef {
        self.0.archive.as_ref()
    }
    pub fn rel_path(&self) -> &str {
        &self.0.rel_path
    }

    pub fn find_step(&self,step:BuildTargetId) -> Option<&BuildStep> {
        self.0.steps.iter().find(|s| s.target == step)
    }
    pub fn path(&self) -> &Path {
        &self.0.path
    }
    /*
    async fn new_async<Ctrl:ControllerAsync>(tr:&TaskRef,ctrl:&Ctrl) -> Self {
        let uri = ctrl.archives().with_archives(|archs| archs.iter().find(|a| a.id() == &tr.archive).unwrap().uri()).await.to_owned();
        triomphe::Arc::new(BuildTaskI {

        })
    }*/

    #[cfg(feature="async")]
    fn new<'a,Ctrl:ControllerAsync>(tr:TaskSpec<'a>,ctrl:&Ctrl) -> Option<Self> {
        let format = match tr.target {
            FormatOrTarget::Target(_) => todo!(),
            FormatOrTarget::Format(f) => ctrl.get_format(f)?
        };
        Some(BuildTask(triomphe::Arc::new(BuildTaskI {
            archive: tr.archive.to_owned(),
            base_path: tr.base_path.to_owned(),
            steps: format.targets.iter().map(|tgt| {
                BuildStep {
                    target:tgt.id,
                    data:Lock::new(StepData {
                        done:None,
                        needs_repeating:false,
                        out:None,
                        yields:Box::new([]),
                        requires:Vec::new(),
                        dependents:Vec::new()
                    })
                }
            }).collect::<Vec<_>>().into_boxed_slice(),
            next: Some(0),
            format:tr.target,
            path:tr.rel_path.split('/').fold(tr.base_path.join("source"),|p,s| p.join(s)),
            rel_path:tr.rel_path.into()
        })))
    }
    #[cfg(not(feature="async"))]
    fn new<'a,Ctrl:Controller>(tr:TaskSpec<'a>,ctrl:&Ctrl) -> Option<Self> {
        todo!()
    }
}

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct TaskRef {
    pub archive:ArchiveId,
    pub rel_path:Box<str>,
    pub target:BuildTargetId
}

#[derive(Copy,Clone)]
pub(crate) struct TaskSpec<'a> {
    pub archive: ArchiveURIRef<'a>,
    pub base_path: &'a Path,
    pub rel_path: &'a str,
    pub target:FormatOrTarget
}
/*
impl<'a> Into<TaskRef> for TaskSpec<'a> {
    fn into(self) -> TaskRef {
        TaskRef {
            archive: self.archive.id().clone(),
            rel_path: self.rel_path.into(),
            target: self.target
        }
    }
}

 */

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Dependency {
    Physical{task:TaskRef,strict:bool},
    Logical{uri:ModuleURI,strict:bool},
    Resolved { task:BuildTask, step:u8, strict:bool }
}


#[derive(Debug)]
pub(crate) struct Queue {
    id:String,
    queue: Lock<VecDeque<BuildTask>>,
    tasks:Lock<HMap<(ArchiveId,String),Vec<BuildTask>>>,
    dependents:Lock<HMap<TaskRef,Vec<(BuildTask,u8)>>>,
    blocked:Lock<Vec<BuildTask>>,
    done:Lock<Vec<BuildTask>>,
}
impl Queue {
    pub(crate) fn new(id:String) -> Self {
        Self {
            id,
            queue: Lock::new(VecDeque::new()),
            tasks: Lock::new(HMap::default()),
            blocked: Lock::new(Vec::new()),
            dependents: Lock::new(HMap::default()),
            done: Lock::new(Vec::new())
        }
    }
    immt_core::asyncs!{ALT !pub(crate) fn enqueue
        <@s['a,Ctrl:Controller+'static,I:Iterator<Item=TaskSpec<'a>>]>
        <@a['a,Ctrl:ControllerAsync+Clone+'static,I:Iterator<Item=TaskSpec<'a>>]>
        (@s &self,mut tasks:I,ctrl:&Ctrl)
        (@a &self,mut tasks:I,ctrl:&Ctrl) {
            let span = tracing::info_span!(target:"buildqueue","queueing tasks");
            let _span = span.enter();

            #[cfg(feature="async")]
            let mut alltasks = tokio::task::JoinSet::new();

            let mut taskmap = wait!(self.tasks.write());
            let mut dependents = wait!(self.dependents.write());
            for t in tasks {
                let key = (t.archive.id().clone(),t.rel_path.into());
                //tracing::debug!("Queueing: {key:?}");
                let ret = match taskmap.entry(key) {
                    Entry::Occupied(mut e) if !e.get().iter().any(|e| e.0.format == t.target) => {
                        if let Some(task) = BuildTask::new(t,ctrl) {
                            e.get_mut().push(task.clone());
                            Some(task)
                        } else {None}
                    }
                    Entry::Vacant(e) => {
                        if let Some(task) = BuildTask::new(t,ctrl) {
                            e.insert(vec![task.clone()]);
                            Some(task)
                        } else { None }
                    }
                    _ => None
                };
                if let Some(task) = ret {
                    if let FormatOrTarget::Format(fmt) = t.target {
                        #[cfg(not(feature="async"))]
                        Self::get_deps(ctrl,fmt,task,&*taskmap,&mut dependents);
                        #[cfg(feature="async")]
                        alltasks.spawn(Self::get_deps(ctrl.clone(),fmt,task).in_current_span());
                    }
                }
            }
            #[cfg(feature="async")]
            { while let Some(Ok(t)) = alltasks.join_next().await {
                if let Some(task) = t {
                    Self::process_deps(task,&*taskmap,&mut dependents).in_current_span().await
                }
            } }
            span.in_scope(||
                tracing::info!("queued {} tasks with {} steps",taskmap.values().map(|v| v.len()).sum::<usize>(),taskmap.values().map(|v| v.iter().map(|t| t.0.steps.len()).sum::<usize>()).sum::<usize>())
            );
            drop(_span);drop(span);
        }
    }

    #[cfg(not(feature = "async"))]
    fn get_deps<Ctrl:Controller+'static>(ctrl:&Ctrl, fmt:SourceFormatId, task:BuildTask,tasks:&HMap<(ArchiveId,String),Vec<BuildTask>>, deps:&mut HMap<TaskRef,Vec<TaskRef>>) {
        if let Some(fmt) = ctrl.get_format(fmt) {
            if let Some(ext) = fmt.extension {
                if let Some(ext) = ctrl.get_extension(ext) {
                    let ext = ext.as_formats().unwrap();
                    ext.get_deps(ctrl,&task);
                    Self::process_deps(task, tasks,deps)
                }
            }
        }
    }

    #[cfg(feature = "async")]
    async fn get_deps<Ctrl:ControllerAsync+'static>(mut ctrl:Ctrl,fmt:SourceFormatId,task:BuildTask) -> Option<BuildTask> {
        if let Some(fmt) = ctrl.get_format(fmt) {
            if let Some(ext) = fmt.extension {
                if let Some(ext) = ctrl.get_extension(ext) {
                    let ext = ext.as_formats().unwrap();
                    ext.get_deps_async(&ctrl,&task).in_current_span().await;
                    return Some(task)
                }
            }
        }
        None
    }

    immt_core::asyncs!{ALT fn process_deps(task:BuildTask,tasks:&HMap<(ArchiveId,String),Vec<BuildTask>>,deps:&mut HMap<TaskRef,Vec<(BuildTask,u8)>>) {
        //tracing::debug!("Processing [{}]/{}:{:?}", task.0.archive.id(),task.0.rel_path,task.0.format);
        for (num,s) in task.0.steps.iter().enumerate() {
            let key = TaskRef { archive:task.0.archive.id().clone(),rel_path:task.0.rel_path.clone(),target:s.target };
            if let Some(v) = deps.remove(&key) {
                for (d,i) in v.into_iter() {
                    if let Some(t) = d.0.steps.get(i as usize) {
                        let mut deps = wait!(t.data.write());
                        for d in deps.requires.iter_mut() { match d {
                            Dependency::Physical {task:ref t,strict} if t == &key => {
                                *d = Dependency::Resolved { task:task.clone(), step: num as u8, strict:*strict };
                                //tracing::debug!("Resolving dependency: [{}]/{}:{:?}", task.0.archive,task.0.rel_path,task.0.format);
                            }
                            _ => ()
                        }};
                    }
                }
            }
            let mut data = wait!(s.data.write());
            for dep in data.requires.iter_mut() {
                match dep {
                    Dependency::Physical { task:ref deptask, ref strict} => {
                        let key = (deptask.archive.clone(),deptask.rel_path.clone().into());
                        if let Some(deptasks) = tasks.get(&key) {
                            if let Some((i,deptask,step)) = deptasks.iter().find_map(|bt| bt.0.steps.iter().enumerate().find_map(|(i,tsk)|
                                if tsk.target == s.target {Some((i,bt,tsk))} else {None}
                            )) {
                                let deptask = deptask.clone();
                                wait!(step.data.write()).dependents.push((task.clone(),num as u8));
                                let step = i as u8;
                                *dep = Dependency::Resolved { task:deptask, step, strict: *strict };
                                //tracing::debug!("Resolving dependency: [{}]/{}:{:?}", task.0.archive,task.0.rel_path,task.0.format);
                                continue
                            }
                        }
                        //tracing::debug!("Not yet resolvable: {deptask:?}");
                        deps.entry(deptask.clone()).or_insert_with(Vec::new).push((task.clone(),num as u8));
                    }
                    _ => ()
                }
            }
        }
    }}


    /*
    #[cfg(feature = "tokio")]
    pub(crate) async fn enqueue<I:Iterator<Item=TaskRef>>(&mut self,mut tasks:I) {

    }

     */

    #[cfg(feature = "async")]
    pub(crate) async fn run(&self,sem:Arc<tokio::sync::Semaphore>) {
        todo!()
    }
}