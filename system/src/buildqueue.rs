use crate::backend::archive_manager::ArchiveManager;
use crate::controller::Controller;
use crate::utils::measure;
use immt_api::archives::ArchiveId;
use immt_api::archives::ArchiveT;
use immt_api::formats::building::{BuildInfo, BuildStep, BuildStepKind, BuildTask, Dependency};
use immt_api::formats::Id;
use immt_api::source_files::BuildState;
use immt_api::utils::{HMap, HSet};
use immt_api::CloneStr;
use rayon::prelude::*;
use std::any::Any;
use std::cell::OnceCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, info_span, instrument, trace, Instrument};

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
}
impl BuildQueue {
    pub fn unqueued(&self) -> (usize, usize, usize) {
        let lock = self.inner.read();
        (lock.stale.len(), lock.new.len(), lock.deleted.len())
    }
    pub(crate) fn init(controller: Controller) {
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
        std::thread::spawn(move || Self::build_thread(controller));
        //tokio::task::spawn(async move {Self::build_thread(controller).await});
    }
    pub fn clean_deleted(&self, mgr: &ArchiveManager) {
        todo!()
    }
    pub fn run_all(&self, mgr: &ArchiveManager) {
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
                let go = {
                    let lock = controller.build_queue().inner.read();
                    !lock.todo.is_empty()
                };
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
        let qb = QueueBuilder::default();
        //let mut set = tokio::task::JoinSet::new();
        //for (a,id,path) in todos.into_iter() {
        todos.into_par_iter().for_each(|(a, id, path)| {
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
            let info = BuildInfo {
                archive_id: a,
                format: id,
                rel_path: path,
                archive_path: apath,
                build_path: bpath,
                source: OnceCell::new(),
            };
            //std::thread::sleep(Duration::from_secs_f32(0.05));
            if let Some(format) = controller.formats().get(id) {
                if let Some(task) = format
                    .extension
                    .get_task(&info, &(&controller.as_backend()).into())
                {
                    qb.add(task, &info);
                }
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
        let elapsed = now.elapsed().unwrap();
        info!("Finished after {:?}", elapsed);
    }
}

struct QueuedTask {
    step: BuildStepKind,
    index: u8,
    of: u8,
    dependencies: Vec<Dependency>,
    state: Option<Box<dyn Any + Send>>,
    next: Option<usize>,
}

#[derive(Default)]
struct QueueBuilderI {
    queue: VecDeque<QueuedTask>,
    indices: HMap<PhysicalDependency, usize>,
    upper_bounds: HMap<PhysicalDependency, (HSet<usize>, HSet<usize>)>,
}
#[derive(Default)]
struct QueueBuilder(Arc<parking_lot::Mutex<QueueBuilderI>>);
impl QueueBuilder {
    pub fn add(&self, mut bt: BuildTask, info: &BuildInfo) {
        let mut lock = self.0.lock();
        let lock = &mut *lock;
        let indices = &mut lock.indices;
        let upper_bounds = &mut lock.upper_bounds;
        let queue = &mut lock.queue;
        let len = bt.steps.len();
        let mut last = None;
        for (j, step) in bt.steps.into_iter().enumerate() {
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
            let strong_lower = strong_deps
                .iter()
                .map(|a| indices.get(a).copied().map(|i| i + 1).unwrap_or_default())
                .max()
                .unwrap_or_default();
            let weak_lower = weak_deps
                .iter()
                .map(|a| indices.get(a).copied().map(|i| i + 1).unwrap_or_default())
                .max()
                .unwrap_or_default();
            let strong_upper = strong_deps
                .iter()
                .filter_map(|a| upper_bounds.get(a).and_then(|(s, _)| s.iter().min()))
                .min()
                .copied();
            let weak_upper = weak_deps
                .iter()
                .filter_map(|a| {
                    upper_bounds
                        .get(a)
                        .and_then(|(s, w)| s.iter().chain(w.iter()).min())
                })
                .min()
                .copied();
            let mut i = strong_lower;
            if let Some(upper) = strong_upper {
                if upper > i {
                    if weak_lower < upper {
                        i = weak_lower
                    } else {
                        i = upper
                    }
                    if let Some(weak) = weak_upper {
                        if weak < upper && weak > i {
                            i = weak
                        }
                    }
                } else {
                    todo!()
                }
            } else if let Some(upper) = weak_upper {
                if upper > i {
                    i = upper
                }
            } else {
                i = queue.len();
            }
            if let Some(last) = last {
                queue[last].next = Some(i);
            }
            last = Some(i);
            for ref d in strong_deps {
                let (set, _) = upper_bounds.entry(d.clone()).or_default();
                set.insert(i);
            }
            for ref d in weak_deps {
                let (_, set) = upper_bounds.entry(d.clone()).or_default();
                set.insert(i);
            }
            let nd = PhysicalDependency {
                id: step.id,
                archive: info.archive_id.clone(),
                filepath: info.rel_path.clone(),
            };
            {
                debug!("Adding {:?} to queue at index {}", nd, i);
                trace!("upper bounds: {:?}", upper_bounds);
                trace!("indices: {:?}", indices);
                trace!(
                    "Constraints: {:?} <= x < {:?}",
                    (weak_lower, strong_lower),
                    (strong_upper, weak_upper)
                );

                //println!("Adding {:?} to queue at index {}", nd, i);
                //println!("upper bounds: {:?}", upper_bounds);
                //println!("indices: {:?}", indices);
                println!(
                    "Constraints: {:?} <= x < {:?}",
                    (weak_lower, strong_lower),
                    (strong_upper, weak_upper)
                );
            }
            indices.insert(nd, i);
            let t = QueuedTask {
                step: step.kind,
                index: j as u8 + 1,
                of: len as u8,
                dependencies: step.dependencies,
                state: if j == 0 {
                    std::mem::take(&mut bt.state)
                } else {
                    None
                },
                next: None,
            };
            if i == queue.len() {
                queue.push_back(t);
            } else {
                queue.insert(i, t);
                todo!()
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
