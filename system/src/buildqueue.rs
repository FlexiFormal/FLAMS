use crate::backend::archive_manager::ArchiveManager;
use crate::controller::Controller;
use crate::settings::SettingsValue;
use immt_api::archives::ArchiveId;
use immt_api::archives::ArchiveT;
use immt_api::formats::building::{
    BuildData, BuildInfo, BuildResult, BuildStep, BuildStepKind, Dependency,
};
use immt_api::formats::Id;
use immt_api::source_files::BuildState;
use immt_api::utils::{HMap, HSet};
use immt_api::CloneStr;
use rayon::prelude::*;
use std::any::Any;
use std::cell::OnceCell;
use std::collections::hash_map::Entry;
use std::collections::VecDeque;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, info_span, instrument, trace, warn};

#[derive(Default)]
pub struct BuildQueue {
    inner: parking_lot::RwLock<BuildQueueI>,
}

#[derive(Default)]
struct BuildQueueI {
    pub stale: Vec<(ArchiveId, Id, CloneStr, u64)>,
    pub new: Vec<(ArchiveId, Id, CloneStr)>,
    pub deleted: Vec<(ArchiveId, CloneStr)>,
    todo: Vec<(ArchiveId, Id, CloneStr)>,
    queue: VecDeque<QueuedTask>,
    processes: Vec<BuildProcess>,
}

impl BuildQueue {
    pub fn unqueued(&self) -> (usize, usize, usize) {
        let lock = self.inner.read();
        (lock.stale.len(), lock.new.len(), lock.deleted.len())
    }
    pub(crate) fn init(controller: Controller) {
        controller.settings().set_default([(
            "build queue",
            "threads",
            SettingsValue::PositiveInteger(4),
        )]);
        let (stale, new, deleted) = controller
            .archives()
            .par_iter()
            .fold(
                || (vec![], vec![], vec![]),
                |(stale, new, deleted), a| {
                    a.iter_sources((stale, new, deleted), |f, (stale, new, deleted)| {
                        match f.state {
                            BuildState::Stale { last_built, .. } => stale.push((
                                a.id().to_owned(),
                                f.format,
                                f.rel_path.clone(),
                                last_built,
                            )),
                            BuildState::New => {
                                new.push((a.id().to_owned(), f.format, f.rel_path.clone()))
                            }
                            BuildState::Deleted => {
                                deleted.push((a.id().to_owned(), f.rel_path.clone()))
                            }
                            _ => (),
                        }
                    })
                },
            )
            .reduce(
                || (vec![], vec![], vec![]),
                |(mut stale, mut new, mut deleted), (s, n, d)| {
                    stale.extend(s);
                    new.extend(n);
                    deleted.extend(d);
                    (stale, new, deleted)
                },
            );
        {
            let mut lock = controller.build_queue().inner.write();
            lock.stale = stale;
            lock.new = new;
            lock.deleted = deleted;
        }
        rayon::spawn(move || Self::build_thread(controller));
        //tokio::task::spawn(async move {Self::build_thread(controller).await});
    }
    pub fn clean_deleted(&self, mgr: &ArchiveManager) {
        todo!()
    }
    pub fn run_all(&self) {
        let mut lock = self.inner.write();
        let inner = &mut *lock;
        inner.todo.extend(
            inner
                .stale
                .drain(..)
                .map(|(a, f, p, _)| (a, f, p))
                .chain(inner.new.drain(..)),
        );
    }

    fn build_thread(controller: Controller) {
        loop {
            loop {
                let (go, threads) = {
                    let lock = controller.build_queue().inner.read();
                    let change = controller
                        .settings()
                        .get([("build queue", "threads")], |v| {
                            if let [Some(SettingsValue::PositiveInteger(i))] = v {
                                if lock.processes.len() != *i as usize {
                                    Some(*i as usize)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        });
                    (!lock.todo.is_empty(), change)
                };
                if let Some(v) = threads {
                    let mut lock = controller.build_queue().inner.write();
                    let old = lock.processes.len();
                    if v > old {
                        lock.processes.extend(
                            (old..v).map(|i| BuildProcess::new(controller.clone(), i as u8)),
                        );
                    } else {
                        for p in &mut lock.processes[v..] {
                            p.shutdown();
                        }
                        lock.processes.truncate(v);
                    }
                }
                if go {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            let todos = {
                let mut lock = controller.build_queue().inner.write();
                std::mem::take(&mut lock.todo)
            };

            Self::sort_build_jobs(&controller, todos);
        }
    }

    #[instrument(level="info",name="sorting build jobs",skip_all,fields(num=todos.len()))]
    fn sort_build_jobs(controller: &Controller, todos: Vec<(ArchiveId, Id, CloneStr)>) {
        let pb =
            crate::utils::progress::in_progress2("Sorting build jobs...", todos.len(), false, "");
        let now = std::time::SystemTime::now();
        let span = tracing::Span::current();
        let mut qb = QueueBuilder::default();
        //let mut set = tokio::task::JoinSet::new();
        //for (a,id,path) in todos.into_iter() {

        //std::thread::sleep(Duration::from_secs(2));

        todos.into_iter().for_each(|(a, id, path)| {
            //std::thread::sleep(Duration::from_secs_f32(60.0 / 12000.0));

            let _span = span.enter();
            let controller = controller.clone();
            //set.spawn(async move {
            let apath = controller
                .archives()
                .find(a.clone())
                .and_then(|a| a.right().map(|a| a.path().clone()))
                .flatten();
            let bpath = apath
                .as_ref()
                .map(|a| PathBuf::from(format!("{}/source{}", a.display(), path)).into());
            let mut info = BuildInfo {
                archive_id: a,
                format: id,
                rel_path: path,
                archive_path: apath,
                state_data: BuildData::new(bpath),
            };
            //std::thread::sleep(Duration::from_secs_f32(0.05));
            if let Some(format) = controller.formats().get(id) {
                let v = format
                    .extension
                    .get_task(&mut info, &controller.as_backend());
                qb.add(v, info);
            }
            if let Some(pb) = pb {
                pb.tick();
            }
            //}.in_current_span());
        });
        /*
        while let Some(res) = set.join_next().await {
            res.unwrap();
            if let Some(pb) = pb { pb.tick(); }
        }*/
        let mut lock = controller.build_queue().inner.write();
        lock.queue = std::mem::take(&mut qb.0.queue); //.lock().queue);
        let elapsed = now.elapsed().unwrap();
        info!("Finished after {:?}; {} queued", elapsed, lock.queue.len());
    }
}

#[derive(Clone)]
struct QueuedTask {
    step: BuildStepKind,
    index: u8,
    of: u8,
    as_dep: PhysicalDependency,
    dependencies: Vec<Dependency>,
    state: Arc<parking_lot::Mutex<BuildData>>,
    //next: Option<usize>,
}

#[derive(Default)]
struct QueueBuilderI {
    queue: VecDeque<QueuedTask>,
    constraints: HMap<PhysicalDependency, Constraint>,
}
#[derive(Default)]
struct QueueBuilder(QueueBuilderI); //Arc<parking_lot::Mutex<QueueBuilderI>>);
impl QueueBuilder {
    pub fn add(&mut self, bt: Vec<BuildStep>, info: BuildInfo) {
        let state = Arc::new(parking_lot::Mutex::new(info.state_data));

        //let mut lock = self.0.lock();
        //let lock = &mut *lock;
        let lock = &mut self.0;

        let constraints = &mut lock.constraints;
        let queue = &mut lock.queue;
        let len = bt.len();
        //let mut last = None;

        for (j, step) in bt.into_iter().enumerate() {
            let mut strong_deps = vec![];
            let mut weak_deps = vec![];
            for d in step.dependencies.iter() {
                match d {
                    Dependency::Physical {
                        id,
                        archive,
                        filepath,
                        strong,
                    } => {
                        let dep = PhysicalDependency {
                            id,
                            archive: archive.clone(),
                            filepath: filepath.clone(),
                        };
                        if *strong {
                            strong_deps.push(dep)
                        } else {
                            weak_deps.push(dep)
                        }
                    }
                    Dependency::Logical => todo!(),
                }
            }
            let nd = PhysicalDependency {
                id: step.id,
                archive: info.archive_id.clone(),
                filepath: info.rel_path.clone(),
            };
            let constraint = Constraint::get(&nd, &strong_deps, &weak_deps, &*constraints);

            let mut i = constraint.strong_lower.unwrap_or_default();
            if let Some(upper) = constraint.strong_upper {
                if upper > i {
                    if let Some(weak_lower) = constraint.weak_lower {
                        if weak_lower < upper {
                            i = weak_lower
                        } else {
                            i = upper
                        }
                    } else {
                        i = upper
                    }
                    if let Some(weak) = constraint.weak_upper {
                        if weak < upper && weak > i {
                            i = weak
                        }
                    }
                } else {
                    i += 1;
                    queue.insert(
                        upper,
                        QueuedTask {
                            step: step.kind.clone(),
                            as_dep: nd.clone(),
                            index: j as u8 + 1,
                            of: len as u8,
                            dependencies: step.dependencies.clone(),
                            state: state.clone(),
                            //next: None,
                        },
                    );
                    for e in queue.iter().skip(upper + 1) {
                        if let Some(Constraint::Solved(j)) = constraints.get_mut(&e.as_dep) {
                            *j += 1
                        }
                    }
                }
            } else if let Some(upper) = constraint.weak_upper {
                if upper > i {
                    i = upper
                }
            } else {
                i = queue.len();
            }

            for ref d in strong_deps {
                match constraints.entry(d.clone()) {
                    Entry::Vacant(e) => {
                        let mut set = HSet::default();
                        set.insert(nd.clone());
                        e.insert(Constraint::Unsolved {
                            strong_upper: set,
                            weak_upper: HSet::default(),
                        });
                    }
                    Entry::Occupied(mut e) => {
                        if let Constraint::Unsolved { strong_upper, .. } = e.get_mut() {
                            strong_upper.insert(nd.clone());
                        }
                    }
                }
            }
            for ref d in weak_deps {
                match constraints.entry(d.clone()) {
                    Entry::Vacant(e) => {
                        let mut set = HSet::default();
                        set.insert(nd.clone());
                        e.insert(Constraint::Unsolved {
                            strong_upper: HSet::default(),
                            weak_upper: set,
                        });
                    }
                    Entry::Occupied(mut e) => {
                        if let Constraint::Unsolved { weak_upper, .. } = e.get_mut() {
                            weak_upper.insert(nd.clone());
                        }
                    }
                }
            }
            {
                trace!("Adding {:?} to queue at index {}", nd, i);
                trace!("Constraints: {constraint}");

                //println!("Adding {:?} to queue at index {}", nd, i);
                //println!("upper bounds: {:?}", upper_bounds);
                //println!("indices: {:?}", indices);
                //println!("Constraints: {constraint}");
            }
            constraints.insert(nd.clone(), Constraint::Solved(i));
            let t = QueuedTask {
                step: step.kind,
                as_dep: nd,
                index: j as u8 + 1,
                of: len as u8,
                dependencies: step.dependencies,
                state: state.clone(),
                //next: None,
            };
            if i == queue.len() {
                queue.push_back(t);
            } else {
                queue.insert(i, t);
                for e in queue.iter().skip(i + 1) {
                    if let Some(Constraint::Solved(j)) = constraints.get_mut(&e.as_dep) {
                        *j += 1
                    }
                }
            }
            //std::thread::sleep(Duration::from_millis(3000));
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, Debug)]
struct PhysicalDependency {
    id: &'static str,
    archive: ArchiveId,
    filepath: CloneStr,
}

enum Constraint {
    Solved(usize),
    Unsolved {
        strong_upper: HSet<PhysicalDependency>,
        weak_upper: HSet<PhysicalDependency>,
    },
}
impl Constraint {
    fn get(
        this: &PhysicalDependency,
        strong: &[PhysicalDependency],
        weak: &[PhysicalDependency],
        map: &HMap<PhysicalDependency, Constraint>,
    ) -> ConstraintNumber {
        let mut ret = ConstraintNumber::default();
        ret.strong_lower = strong
            .iter()
            .filter_map(|d| match map.get(d) {
                Some(Constraint::Solved(i)) => Some(*i),
                _ => None,
            })
            .max();
        ret.weak_lower = weak
            .iter()
            .filter_map(|d| match map.get(d) {
                Some(Constraint::Solved(i)) => Some(*i),
                _ => None,
            })
            .max();
        let mut to_check = match map.get(this) {
            None => return ret.close(),
            Some(Constraint::Unsolved {
                strong_upper: strong,
                weak_upper: weak,
            }) => strong
                .iter()
                .map(|d| (d, true))
                .chain(weak.iter().map(|d| (d, false)))
                .collect::<VecDeque<_>>(),
            _ => unreachable!(),
        };
        let mut considered: HSet<&PhysicalDependency> = HSet::default();
        while let Some((d, strong)) = to_check.pop_front() {
            if considered.contains(d) {
                continue;
            }
            considered.insert(d);
            match map.get(d) {
                Some(Constraint::Solved(i)) => {
                    if strong {
                        ret.strong_upper = ret.strong_upper.map(|j| j.min(*i)).or(Some(*i));
                    } else {
                        ret.weak_upper = ret.weak_upper.map(|j| j.min(*i)).or(Some(*i));
                    }
                }
                Some(Constraint::Unsolved {
                    strong_upper,
                    weak_upper,
                }) => {
                    if strong {
                        for d in strong_upper.iter().filter_map(|d| {
                            if !considered.contains(d) {
                                Some((d, true))
                            } else {
                                None
                            }
                        }) {
                            to_check.push_front(d);
                        }
                    } else {
                        to_check.extend(strong_upper.iter().chain(weak_upper.iter()).filter_map(
                            |d| {
                                if !considered.contains(d) {
                                    Some((d, false))
                                } else {
                                    None
                                }
                            },
                        ));
                    }
                }
                _ => (),
            }
        }
        ret.close()
    }
}
#[derive(Default)]
struct ConstraintNumber {
    strong_lower: Option<usize>,
    weak_lower: Option<usize>,
    strong_upper: Option<usize>,
    weak_upper: Option<usize>,
}
impl Display for ConstraintNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} <= x < {:?}",
            (self.weak_lower, self.strong_lower),
            (self.strong_upper, self.weak_upper)
        )
    }
}
impl ConstraintNumber {
    fn close(mut self) -> Self {
        if let Some(ref mut c) = self.strong_lower {
            *c += 1
        }
        if let Some(w) = self.weak_lower {
            if let Some(s) = self.strong_lower {
                if w <= s {
                    self.weak_lower = None;
                }
            }
        }
        if let Some(w) = self.weak_upper {
            if let Some(s) = self.strong_upper {
                if w >= s {
                    self.weak_upper = None;
                }
            }
        }
        self
    }
}

#[derive(Clone)]
struct BuildProcess {
    controller: Controller,
    inner: Arc<parking_lot::RwLock<BuildProcessI>>,
    index: u8,
}

struct BuildProcessI {
    current: Option<QueuedTask>,
    running: bool,
}
impl BuildProcess {
    fn new(controller: Controller, index: u8) -> Self {
        let ret = Self {
            controller,
            index,
            inner: Arc::new(parking_lot::RwLock::new(BuildProcessI {
                current: None,
                running: true,
            })),
        };
        let r2 = ret.clone();
        rayon::spawn(move || Self::run(r2));
        ret
    }
    fn shutdown(&mut self) {
        self.inner.write().running = false;
    }
    fn run(self) {
        loop {
            let next = {
                {
                    let lock = self.inner.read();
                    if !lock.running {
                        break;
                    }
                }
                let mut queue = self.controller.build_queue().inner.write();
                if queue.queue.is_empty() {
                    std::mem::drop(queue);
                    std::thread::sleep(Duration::from_secs_f32(0.5));
                    continue;
                }
                let mut running = queue
                    .processes
                    .iter()
                    .filter(|p| p.index != self.index)
                    .flat_map(|p| {
                        let read = p.inner.read();
                        read.current.as_ref().map(|c| c.as_dep.clone())
                    })
                    .collect::<Vec<_>>();
                let next = queue.queue.iter().enumerate().find_map(|(i, t)| {
                    if t.dependencies.iter().any(|d| match d {
                        Dependency::Physical {
                            id,
                            archive,
                            filepath,
                            ..
                        } => running.iter().any(|d2| {
                            d2.id == *id && d2.archive == *archive && d2.filepath == *filepath
                        }),
                        _ => false,
                    }) {
                        running.push(t.as_dep.clone());
                        None
                    } else {
                        Some(i)
                    }
                });
                if let Some(next) = next {
                    let next = queue.queue.remove(next).unwrap();
                    let mut lock = self.inner.write();
                    lock.current = Some(next.clone());
                    Some(next)
                } else {
                    None
                }
            };
            if let Some(next) = next {
                let span = info_span!(target:"build", "process", thread = self.index, step = next.index, of = next.of, 
                    archive=%next.as_dep.archive, format=%next.as_dep.id, path=%next.as_dep.filepath
                ).entered();
                match next.step {
                    BuildStepKind::Check => (),
                    BuildStepKind::Source(task) => {
                        let mut state = next.state.lock();
                        match task.run(&mut state, &self.controller.as_backend()) {
                            BuildResult::Success => {
                                info!("Success");
                            }
                            BuildResult::Err(e) => {
                                warn!("Error: {}", e);
                                let mut lock = self.controller.build_queue().inner.write();
                                let mut all_deps = VecDeque::new();
                                all_deps.push_back(next.as_dep.clone());
                                loop {
                                    let mut removed = false;
                                    let len = all_deps.len();
                                    lock.queue.retain(|tsk| {
                                        if tsk.dependencies.iter().any(|d| match d {
                                            Dependency::Physical {
                                                id,
                                                archive,
                                                filepath,
                                                ..
                                            } => all_deps.iter().any(|d2| {
                                                d2.id == *id
                                                    && d2.archive == *archive
                                                    && d2.filepath == *filepath
                                            }),
                                            _ => todo!(),
                                        }) {
                                            all_deps.push_back(tsk.as_dep.clone());
                                            removed = true;
                                            false
                                        } else {
                                            true
                                        }
                                    });
                                    if !removed {
                                        break;
                                    }
                                    all_deps.drain(0..len);
                                }
                                debug!("Removed {} tasks", all_deps.len());
                            }
                        }
                    }
                }
                continue;
            }
            std::thread::sleep(Duration::from_secs_f32(0.5));
        }
    }
}
