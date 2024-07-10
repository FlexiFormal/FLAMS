use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::{Instrument, instrument};
use immt_core::building::buildstate::{QueueEntry, QueueMessage};
use immt_core::building::formats::{BuildTargetId, FormatOrTarget, SourceFormatId};
use immt_core::uris::archives::{ArchiveId, ArchiveURI, ArchiveURIRef};
use immt_core::uris::modules::ModuleURI;
use immt_core::utils::triomphe;
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::Storage;
use crate::building::targets::BuildFormatId;
use crate::controller::Controller;
use crate::extensions::FormatExtension;
use crate::HMap;
use crate::utils::asyncs::ChangeSender;

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
    data:parking_lot::RwLock<StepData>
}
impl BuildStep {
    pub fn push_dependency(&self,dependency: Dependency) {
        let mut data = self.data.write();
        data.requires.push(dependency);
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TaskState {
    Running,Queued,Blocked,Done,Failed,None
}

#[derive(Debug)]
struct BuildTaskI {
    archive: ArchiveURI,
    base_path:PathBuf,
    steps: Box<[BuildStep]>,
    next: Option<u8>,
    path:PathBuf,
    status:parking_lot::Mutex<TaskState>,
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
    pub fn as_entry(&self) -> QueueEntry {
        QueueEntry {
            archive: self.archive().id().clone(),
            rel_path: self.rel_path().to_string(),
            target: self.0.format
        }
    }
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

    fn new<Ctrl:Controller>(tr:TaskSpec<'_>,ctrl:&Ctrl) -> Option<Self> {
        let format = match tr.target {
            FormatOrTarget::Target(_) => todo!(),
            FormatOrTarget::Format(f) => ctrl.get_format(f)?
        };
        Some(BuildTask(triomphe::Arc::new(BuildTaskI {
            archive: tr.archive.to_owned(),
            base_path: tr.base_path.to_owned(),
            status:parking_lot::Mutex::new(TaskState::None),
            steps: format.targets.iter().map(|tgt| {
                BuildStep {
                    target:tgt.id,
                    data:parking_lot::RwLock::new(StepData {
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
struct QueueInner {
    queue: VecDeque<BuildTask>,
    blocked:Vec<BuildTask>,
    done:Vec<BuildTask>,
    running:Vec<BuildTask>
}


#[derive(Debug)]
struct QueueI {
    id:String,
    tasks:parking_lot::RwLock<HMap<(ArchiveId,String),Vec<BuildTask>>>,
    dependents:parking_lot::RwLock<HMap<TaskRef,Vec<(BuildTask,u8)>>>,
    sender:ChangeSender<QueueMessage>,
    inner: parking_lot::RwLock<QueueInner>
}

#[derive(Debug,Clone)]
pub struct Queue(triomphe::Arc<QueueI>);

impl Queue {
    pub fn id(&self) -> &str { &self.0.id }
    pub(crate) fn new(id:String,sender:ChangeSender<QueueMessage>) -> Self {
        Self(Arc::new(QueueI {
            id,sender,
            inner: parking_lot::RwLock::new(QueueInner {
                queue: VecDeque::new(),
                blocked: Vec::new(),
                done: Vec::new(),
                running: Vec::new()
            }),
            tasks: parking_lot::RwLock::new(HMap::default()),
            dependents: parking_lot::RwLock::new(HMap::default()),
        }))
    }
    pub(crate) fn enqueue<'a,Ctrl:Controller+'static,I:rayon::iter::ParallelIterator<Item=TaskSpec<'a>>>(&self,mut tasks:I,ctrl:&Ctrl) {
        use rayon::prelude::*;
        let span = tracing::info_span!(target:"buildqueue","queueing tasks");
        tasks.for_each(move |t| {
            let _span = span.enter();
            let mut taskmap = self.0.tasks.write();
            let mut dependents = self.0.dependents.write();
            let key = (t.archive.id().clone(),t.rel_path.into());
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
                    Self::get_deps(ctrl,fmt,task,&*taskmap,&mut *dependents);
                }
            }
        })
    }

    fn  get_deps<Ctrl:Controller+'static>(ctrl:&Ctrl, fmt:SourceFormatId, task:BuildTask,tasks:&HMap<(ArchiveId,String),Vec<BuildTask>>, deps:&mut HMap<TaskRef,Vec<(BuildTask,u8)>>) {
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

    fn process_deps(task:BuildTask,tasks:&HMap<(ArchiveId,String),Vec<BuildTask>>,deps:&mut HMap<TaskRef,Vec<(BuildTask,u8)>>) {
        //tracing::debug!("Processing [{}]/{}:{:?}", task.0.archive.id(),task.0.rel_path,task.0.format);
        for (num,s) in task.0.steps.iter().enumerate() {
            let key = TaskRef { archive:task.0.archive.id().clone(),rel_path:task.0.rel_path.clone(),target:s.target };
            if let Some(v) = deps.remove(&key) {
                for (d,i) in v.into_iter() {
                    if let Some(t) = d.0.steps.get(i as usize) {
                        let mut deps = t.data.write();
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
            let mut data = s.data.write();
            for dep in data.requires.iter_mut() {
                match dep {
                    Dependency::Physical { task:ref deptask, ref strict} => {
                        let key = (deptask.archive.clone(),deptask.rel_path.clone().into());
                        if let Some(deptasks) = tasks.get(&key) {
                            if let Some((i,deptask,step)) = deptasks.iter().find_map(|bt| bt.0.steps.iter().enumerate().find_map(|(i,tsk)|
                                if tsk.target == s.target {Some((i,bt,tsk))} else {None}
                            )) {
                                let deptask = deptask.clone();
                                step.data.write().dependents.push((task.clone(),num as u8));
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
    }

    pub(crate) fn run(&self,sem:Arc<crate::building::buildqueue::Semaphore>) {
        let span = tracing::info_span!(target:"buildqueue","Sorting queue",id=self.0.id);
        let _span = span.enter();
        let tasks = self.0.tasks.read();
        let mut tasks = tasks.values().flatten().cloned().collect::<Vec<_>>();
        let mut inner = self.0.inner.write();
        let QueueInner {ref mut blocked, ref mut done, ref mut queue,..} = *inner;
        let mut weak = true;
        while !tasks.is_empty() {
            let mut changed = false;
            for t in &tasks {
                if let Some(i) = t.0.next {
                    let task = t.0.steps.get(i as usize).unwrap();
                    let data = task.data.read();
                    let mut newstate = TaskState::Queued;
                    for d in data.requires.iter() { match d {
                        Dependency::Resolved { task, strict, .. } if *strict || weak => {
                            match *task.0.status.lock() {
                                TaskState::Done | TaskState::Queued | TaskState::Failed | TaskState::Running => (),
                                TaskState::Blocked => {
                                    newstate = TaskState::Blocked;
                                }
                                TaskState::None => {
                                    newstate = TaskState::None;
                                    break
                                }
                            }
                        }
                        _ => ()
                    }}
                    match newstate {
                        TaskState::Blocked => {
                            changed = true;
                            *t.0.status.lock() = TaskState::Blocked;
                            blocked.push(t.clone());
                        }
                        TaskState::Queued => {
                            changed = true;
                            *t.0.status.lock() = TaskState::Queued;
                            queue.push_back(t.clone());
                        }
                        _ => ()
                    }
                } else {
                    *t.0.status.lock() = TaskState::Done;
                    done.push(t.clone());
                }
            }
            if changed {
                tasks.retain(|t| t.0.status.lock().deref() == &TaskState::None)
            } else if weak {
                weak = false
            } else {
                let tasks = std::mem::take(&mut tasks);
                for t in tasks {
                    *t.0.status.lock() = TaskState::Blocked;
                    blocked.push(t)
                }
            }
        }
        tracing::info!("Done.");
        drop(_span);drop(span);
        self.0.sender.lazy_send(|| QueueMessage::Start {
            id:self.0.id.clone(),
            queue:queue.iter().map(|t| t.as_entry()).collect(),
            blocked:blocked.iter().map(|t| t.as_entry()).collect(),
            failed:Vec::new(),
            done:done.iter().map(|t| t.as_entry()).collect()
        });
        tokio::task::spawn(self.clone().go(sem));
    }

    pub fn state(&self) -> QueueMessage {
        let mut inner = self.0.inner.write();
        let QueueInner {ref mut blocked, ref mut done, ref mut queue,..} = *inner;
        QueueMessage::Start {
            id:self.0.id.clone(),
            queue:queue.iter().map(|t| t.as_entry()).collect(),
            blocked:blocked.iter().map(|t| t.as_entry()).collect(),
            failed:Vec::new(),
            done:done.iter().map(|t| t.as_entry()).collect()
        }
    }

    async fn go(self,sem:Arc<crate::building::buildqueue::Semaphore>) {
        loop {
            let _res = match &*sem {
                crate::building::buildqueue::Semaphore::Stepwise { recv,..} => {
                    match recv.clone().changed().await {
                        Ok(_) => None,
                        _ => return
                    }
                },
                crate::building::buildqueue::Semaphore::Counting { inner,..} => {
                    match inner.acquire().await {
                        Ok(r) => Some(r),
                        _ => return
                    }
                }
            };
            self.next().await
        }
    }
        async fn next(&self) {

        }


    pub fn running(&self) -> bool {
        !self.0.inner.read().queue.is_empty()
    }

    pub fn get_list(&self) -> Vec<QueueEntry> {
        self.0.tasks.read().values().flat_map(|e| e.iter().map(|e| e.as_entry())).collect()
    }
}