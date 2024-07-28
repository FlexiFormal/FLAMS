use std::any::Any;
use std::cmp::PartialEq;
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
use immt_core::utils::time::{Delta, Timestamp};
use immt_core::utils::triomphe;
use immt_core::utils::triomphe::Arc;
use crate::backend::archives::{Archive, Storage};
use crate::building::targets::{BuildDataFormat, BuildFormatId};
use crate::building::tasks::{BuildTask, Dependency, TaskRef, TaskSpec, TaskState};
use crate::controller::Controller;
use crate::extensions::FormatExtension;
use crate::HMap;
use crate::utils::asyncs::ChangeSender;

#[derive(Default,Debug)]
struct Timer {
    last:Option<Timestamp>,
    average:Option<Delta>,
    steps:usize,
    done:usize
}
impl Timer {
    fn init(&mut self,queue:&VecDeque<BuildTask>) {
        self.last = Some(Timestamp::now());
        self.average = None;
        self.steps = queue.iter().map(|q| q.0.steps.len()).sum();
        self.done = 0;
    }
    fn update(&mut self,dones:u8) {
        let dur = self.last.unwrap().since_now();
        self.last = Some(Timestamp::now());
        let avg = self.average.get_or_insert_with(|| Delta::new());
        avg.update_average(self.done as f64 / (self.done as f64 + dones as f64),dur);
        self.steps -= dones as usize;
        self.done += dones as usize;
    }
    fn eta(&self) -> Delta {
        self.average.map(|a| a * (self.steps as f64)).unwrap_or(Delta::new())
    }
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
    inner: parking_lot::RwLock<QueueInner>,
    timer:parking_lot::RwLock<Timer>
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
            timer: parking_lot::RwLock::new(Timer::default()),
            tasks: parking_lot::RwLock::new(HMap::default()),
            dependents: parking_lot::RwLock::new(HMap::default()),
        }))
    }
    #[instrument(level = "info",
        target = "buildqueue",
        name = "Queueing tasks",
        skip_all
    )]
    pub(crate) fn enqueue<'a,Ctrl:Controller+'static,I:rayon::iter::ParallelIterator<Item=TaskSpec<'a>>>(&self,mut tasks:I,ctrl:&Ctrl) {
        use rayon::prelude::*;
        let span = tracing::Span::current();
        tasks.for_each(move |t| {
            let _span = span.enter();
            let mut taskmap = self.0.tasks.write();
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
            drop(taskmap);
            if let Some(task) = ret {
                if let FormatOrTarget::Format(fmt) = t.target {
                    self.get_deps(ctrl,fmt,task);
                }
            }
        })
    }

    fn get_deps<Ctrl:Controller+'static>(&self,ctrl:&Ctrl, fmt:SourceFormatId, task:BuildTask) {
        if let Some(fmt) = ctrl.get_format(fmt) {
            if let Some(ext) = fmt.extension {
                if let Some(ext) = ctrl.get_extension(ext) {
                    let ext = ext.as_formats().unwrap();
                    ext.get_deps(ctrl,&task);
                    self.process_deps(task)
                }
            }
        }
    }

    fn process_deps(&self,task:BuildTask) {
        tracing::debug!("Processing [{}]/{}:{:?}", task.0.archive.id(),task.0.rel_path,task.0.format);
        //tracing::debug!("[{}]/{}: Getting dependents",task.0.archive.id(),task.0.rel_path);
        let mut deps = self.0.dependents.write();
        //tracing::debug!("[{}]/{}: Getting tasks",task.0.archive.id(),task.0.rel_path);
        let tasks = self.0.tasks.read();
        //tracing::debug!("[{}]/{}: Got both",task.0.archive.id(),task.0.rel_path);
        for (num,s) in task.0.steps.iter().enumerate() {
            let key = TaskRef { archive:task.0.archive.id().clone(),rel_path:task.0.rel_path.clone(),target:s.target };
            if let Some(v) = deps.remove(&key) {
                for (d,i) in v.into_iter() {
                    if let Some(t) = d.0.steps.get(i as usize) {
                        //tracing::debug!("[{}]/{}: Getting data for [{}]/{}:{}",task.0.archive.id(),task.0.rel_path,d.0.archive,d.0.rel_path,t.target);
                        let mut deps = t.data.write();
                        //tracing::debug!("[{}]/{}: Got it",task.0.archive.id(),task.0.rel_path);
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
            //tracing::debug!("[{}]/{}: Getting data for self:{}",task.0.archive.id(),task.0.rel_path,s.target);
            let mut data = s.data.write();
            //tracing::debug!("[{}]/{}: Got it",task.0.archive.id(),task.0.rel_path);
            for dep in data.requires.iter_mut() {
                match dep {
                    Dependency::Physical { task:ref deptask, ref strict} => {
                        if &deptask.archive == task.0.archive.id() && deptask.rel_path == task.0.rel_path {
                            continue
                            // TODO check for more
                        }
                        let key = (deptask.archive.clone(),deptask.rel_path.clone().into());
                        if let Some(deptasks) = tasks.get(&key) {
                            if let Some((i,deptask,step)) = deptasks.iter().find_map(|bt| bt.0.steps.iter().enumerate().find_map(|(i,tsk)|
                                if tsk.target == s.target {Some((i,bt,tsk))} else {None}
                            )) {
                                let deptask = deptask.clone();
                                //tracing::debug!("[{}]/{}: Getting data for [{}]/{}:{}",task.0.archive.id(),task.0.rel_path,deptask.0.archive,deptask.0.rel_path,step.target);
                                step.data.write().dependents.push((task.clone(),num as u8));
                                //tracing::debug!("[{}]/{}: Got it",task.0.archive.id(),task.0.rel_path);
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
        //tracing::debug!("Done processing [{}]/{}:{:?}", task.0.archive.id(),task.0.rel_path,task.0.format);
    }
    #[instrument(level = "info",
        target = "buildqueue",
        name = "Running buildqueue",
        fields(id = %self.0.id),
        skip_all
    )]
    pub(crate) fn run<Ctrl:Controller+Clone+'static>(&self,sem:&crate::building::buildqueue::Semaphore,ctrl:Ctrl) {
        let span = tracing::info_span!(target:"buildqueue","Sorting");
        let _span = span.enter();
        let tasks = self.0.tasks.read();
        let mut tasks = tasks.values().flatten().filter(|t| &*t.0.status.lock() != &TaskState::Done).cloned().collect::<Vec<_>>();
        let mut inner = self.0.inner.write();
        let QueueInner {ref mut blocked, ref mut done, ref mut queue,..} = *inner;
        let mut weak = true;
        while !tasks.is_empty() {
            let mut changed = false;
            for t in &tasks {
                if let Some(i) = t.0.next.lock().clone() {
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
        self.0.sender.lazy_send(|| QueueMessage::Started {
            id:self.0.id.clone(),
            queue:queue.iter().map(|t| t.as_entry()).collect(),
            blocked:blocked.iter().map(|t| t.as_entry()).collect(),
            failed:Vec::new(),
            done:done.iter().map(|t| t.as_entry()).collect(),
            eta:self.0.timer.read().eta()
        });
        tokio::task::spawn(self.clone().go(sem.clone(),ctrl).in_current_span());
    }

    async fn go<Ctrl:Controller+Clone+'static>(self,sem:crate::building::buildqueue::Semaphore,ctrl:Ctrl) {
        {
            let inner = self.0.inner.read();
            let queue = &inner.queue;
            self.0.timer.write().init(queue);
        }
        loop {
            let permit = match &sem {
                crate::building::buildqueue::Semaphore::Stepwise { recv,..} => {
                    match recv.clone().changed().await {
                        Ok(_) => None,
                        _ => return
                    }
                },
                crate::building::buildqueue::Semaphore::Counting { inner,..} => {
                    match tokio::sync::Semaphore::acquire_owned(inner.clone()).await {
                        Ok(r) => Some(r),
                        _ => return
                    }
                }
            };
            if !self.next(permit,&ctrl) {
                break
            }
        }
    }
    fn next<Ctrl:Controller+Clone+'static>(&self,permit:Option<tokio::sync::OwnedSemaphorePermit>,ctrl:&Ctrl) -> bool {
        let mut inner = self.0.inner.write();
        let QueueInner {ref mut queue, ref mut running, ref mut blocked,..} = *inner;
        if queue.is_empty() && blocked.is_empty() && running.is_empty() { return false }
        if let Some(i) = queue.iter().enumerate().find_map(|(next,e)| {
            if let Some(i) = e.0.next.lock().clone() {
                let task = e.0.steps.get(i as usize).unwrap();
                for d in task.data.read().requires.iter() {
                    match d {
                        Dependency::Resolved { task, strict, .. } => {
                            if *strict && *task.0.status.lock() == TaskState::Running {
                                return None
                            }
                        }
                        _ => ()
                    }
                }
                Some(next)
            } else { None }
        }) {
            let task = queue.remove(i).unwrap();
            *task.0.status.lock() = TaskState::Running;
            running.push(task.clone());
            drop(inner);
            self.0.sender.lazy_send(|| QueueMessage::TaskStarted{
                id:self.id().to_string(),
                entry:task.as_entry(),
                eta:self.0.timer.read().eta()
            });
            let ctrl = ctrl.clone();
            let span = tracing::Span::current();
            let s = self.clone();
            tokio::task::spawn_blocking(move || {let _span = span.enter(); s.run_task(task,permit,ctrl)});
            return true
        }
        if !running.is_empty() {
            drop(inner);
            std::thread::sleep(std::time::Duration::from_secs(1));
            true
        } else if !blocked.is_empty() {
            println!("TODO: Blocked");
            todo!()
        } else {
            println!("WAAAAAAAAAAAAH!");
            todo!()
        }
    }

    fn run_task<Ctrl:Controller>(self,task:BuildTask,permit:Option<tokio::sync::OwnedSemaphorePermit>,ctrl:Ctrl) {
        let start = std::time::Instant::now();
        if let Some(i) = {
            let lock = task.0.next.lock();
            let r = lock.deref().clone();
            r
        } {
            if let Some(step) = task.0.steps.get(i as usize) {
                if let Some(tgt) = ctrl.get_target(step.target) {
                    if let Some(ext) = tgt.extension {
                        if let Some(e) = ctrl.get_extension(ext).map(|e| e.as_formats()).flatten() {
                            let span = tracing::info_span!(target:"buildqueue","Running task",archive = %task.0.archive.id(),rel_path = %task.0.rel_path,format = %step.target);
                            let _span = span.enter();
                            let r = e.build(&ctrl,&task,step.target,i);
                            let mut inner = self.0.inner.write();
                            let QueueInner {ref mut queue, ref mut running, ref mut blocked,ref mut done} = *inner;
                            running.retain(|t| t != &task);
                            if r {
                                {self.0.timer.write().update(1);}
                                if task.0.steps.len() > (i + 1) as usize {
                                    *task.0.next.lock() = Some(i + 1);
                                    {*task.0.status.lock() = TaskState::Queued;}
                                    self.0.sender.lazy_send(|| QueueMessage::TaskDoneRequeued {
                                        id:self.id().to_string(),
                                        entry:task.as_entry(),
                                        index:0,
                                        eta:self.0.timer.read().eta()
                                    });
                                    queue.push_front(task);
                                } else {
                                    *task.0.next.lock() = None;
                                    {*task.0.status.lock() = TaskState::Done;}
                                    self.0.sender.lazy_send(|| QueueMessage::TaskDoneFinished {
                                        id:self.id().to_string(),
                                        entry:task.as_entry(),
                                        eta:self.0.timer.read().eta()
                                    });
                                    done.push(task);
                                }
                            } else {
                                {*task.0.status.lock() = TaskState::Failed;}
                                {
                                    let dones = task.0.steps.len() as u8 - i;
                                    self.0.timer.write().update(dones);
                                }
                                self.0.sender.lazy_send(|| QueueMessage::TaskFailed {
                                    id:self.id().to_string(),
                                    entry:task.as_entry(),
                                    eta:self.0.timer.read().eta()
                                });
                            }
                        }
                    }
                }
            }
        }
        drop(permit);
        let dur = start.elapsed();
        // TODO update average runtime
    }


    pub fn running(&self) -> bool {
        !self.0.inner.read().queue.is_empty()
    }

    pub fn get_list(&self) -> Vec<QueueEntry> {
        self.0.tasks.read().values().flat_map(|e| e.iter().map(|e| e.as_entry())).collect()
    }

    pub fn state(&self) -> QueueMessage {
        let mut inner = self.0.inner.write();
        let QueueInner {ref mut blocked, ref mut done, ref mut queue,..} = *inner;
        QueueMessage::Started {
            id:self.0.id.clone(),
            queue:queue.iter().map(|t| t.as_entry()).collect(),
            blocked:blocked.iter().map(|t| t.as_entry()).collect(),
            failed:Vec::new(),
            done:done.iter().map(|t| t.as_entry()).collect(),
            eta:self.0.timer.read().eta()
        }
    }
}