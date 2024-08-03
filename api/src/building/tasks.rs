use std::any::Any;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use oxigraph::model::Triple;
use immt_core::building::buildstate::QueueEntry;
use immt_core::building::formats::{BuildTargetId, FormatOrTarget};
use immt_core::content::Module;
use immt_core::narration::{Document, HTMLDocSpec};
use immt_core::uris::archives::{ArchiveId, ArchiveURI};
use immt_core::uris::documents::DocumentURI;
use immt_core::uris::modules::ModuleURI;
use immt_core::utils::triomphe;
use crate::backend::archives::{Archive, Storage};
use crate::building::targets::BuildDataFormat;
use crate::controller::Controller;

#[derive(Debug)]
pub(super) struct StepData {
    pub(super) done:Option<bool>,
    pub(super) needs_repeating:bool,
    pub(super) out:Option<Box<dyn Any+Send+Sync>>,
    pub(super) yields:Box<[ModuleURI]>,
    pub(super) requires:Vec<Dependency>,
    pub(super) dependents:Vec<(BuildTask,u8)>
}

#[derive(Debug)]
pub struct BuildStep {
    pub(super)target:BuildTargetId,
    pub(super)data:parking_lot::RwLock<StepData>
}

pub struct BuildStepResult<'a> {
    path:PathBuf,data:&'a parking_lot::RwLock<StepData>,target:BuildTargetId,ctrl:&'a dyn Controller
}
impl BuildStepResult<'_> {

    pub fn set_relational(&self,doc:&HTMLDocSpec, mods:&[Module]) {
        let iter = doc.doc.triples().chain(mods.iter().flat_map(|m| m.triples(doc.doc.uri)));
        let iter2 = iter.clone().map(|t| Triple::from(t));
        self.ctrl.relations().add_quads(iter);
        let out = self.path.join("index.ttl");
        self.ctrl.relations().export(iter2,&out,doc.doc.uri);
    }
    pub fn set_narrative(&self, doc:HTMLDocSpec) {
        let out = self.path.join("index.nomd");
        doc.write(&out);
    }
    pub fn set_content(&self, mods:Vec<Module>) {
        if let Some(path) = self.path.parent() {
            let path = path.join(".modules");
            let _ = std::fs::create_dir(&path);
            for m in mods {
                let out = path.join(m.uri.name().as_ref()).with_extension("comd");
                if let Ok(mut f) = std::fs::File::create(out) {
                    let _ = bincode::serde::encode_into_std_write(m,&mut f,bincode::config::standard());
                } else {
                    todo!()
                }
            }
        }
    }
    pub fn set_artifact_str(&self, format: BuildDataFormat, s:&str) {
        let ext = if let Some(e) = format.file_extensions.first() {
            *e
        } else {return };
        let out = self.path.join("index").with_extension(ext);
        std::fs::write(out,s.as_bytes()).unwrap();
    }
    pub fn set_artifact_path(&self,format: BuildDataFormat, path:&Path) {
        let ext = if let Some(e) = format.file_extensions.first() {
            *e
        } else { return };
        let out = self.path.join("index").with_extension(ext);
        let _ = std::fs::rename(path, out);
    }
    pub fn set_log_str(&self, success:bool, s:String) {
        let out = self.path.join(self.target.to_string()).with_extension("log");
        std::fs::write(out,s.as_bytes()).unwrap();
        let mut data = self.data.write();
        data.done = Some(success);
    }
    pub fn set_log_path(&self, success:bool, path:&Path) {
        let out = self.path.join(self.target.to_string()).with_extension("log");
        let _ = std::fs::rename(path,out);
        let mut data = self.data.write();
        data.done = Some(success);
    }
}

impl BuildStep {
    pub fn result<'a>(&'a self,ctrl:&'a dyn Controller,task:&BuildTask) -> Option<BuildStepResult> {
        ctrl.archives().with_archives(|a|
            if let Some(ma) = a.iter().find_map(|a| match a {
                Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                _ => None
            }) {
                let path = task.rel_path().split('/').fold(ma.out_dir(),|p,s| p.join(s));
                let _ = std::fs::create_dir_all(&path);
                Some(path)
            } else {None}
        ).map(|path| BuildStepResult { path,data:&self.data,target:self.target,ctrl })
    }
    pub fn push_dependency(&self,dependency: Dependency) {
        let mut data = self.data.write();
        data.requires.push(dependency);
    }
    /*
    pub fn set_relational(&self,ctrl:&dyn Controller, task:&BuildTask,doc:&HTMLDocSpec, mods:&[Module]) {
        let iter = doc.doc.triples().chain(mods.iter().flat_map(|m| m.triples(doc.doc.uri)));
        ctrl.relations().add_quads(iter);
    }
    pub fn set_narrative(&self, ctrl:&dyn Controller, task:&BuildTask,doc:HTMLDocSpec) {
        //println!("{}",doc.doc);
        ctrl.archives().with_archives(|a|
            if let Some(ma) = a.iter().find_map(|a| match a {
                Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                _ => None
            }) {
                let p = task.rel_path().split('/').fold(ma.out_dir(),|p,s| p.join(s));
                let _ = std::fs::create_dir_all(&p);
                let out = p.join("index.nomd");
                doc.write(&out);
            });
    }
    pub fn set_content(&self, ctrl:&dyn Controller, task:&BuildTask,mods:Vec<Module>) {
        // TODO export relations somewhere
        ctrl.archives().with_archives(|a|
            if let Some(ma) = a.iter().find_map(|a| match a {
                Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                _ => None
            }) {
                let p = task.rel_path().split('/').fold(ma.out_dir(),|p,s| p.join(s));
                let _ = std::fs::create_dir_all(&p);
                for m in mods {
                    let out = p.join(m.uri.name().as_ref()).with_extension("comd");
                    if let Ok(mut f) = std::fs::File::create(out) {
                        let _ = bincode::serde::encode_into_std_write(m,&mut f,bincode::config::standard());
                    } else {todo!()}
                }
            });
    }
    pub fn set_artifact_str(&self, ctrl:&dyn Controller, task:&BuildTask,format: BuildDataFormat, s:&str) {
        let ext = if let Some(e) = format.file_extensions.first() {
            *e
        } else {return };
        ctrl.archives().with_archives(|a|
            if let Some(ma) = a.iter().find_map(|a| match a {
                Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                _ => None
            }) {
                let p = task.rel_path().split('/').fold(ma.out_dir(),|p,s| p.join(s));
                let _ = std::fs::create_dir_all(&p);
                let out = p.join("index").with_extension(ext);
                std::fs::write(out,s.as_bytes()).unwrap();
            });
    }
    pub fn set_artifact_path(&self, ctrl:&dyn Controller, task:&BuildTask,format: BuildDataFormat, path:&Path) {
        if path.exists() {
            let ext = if let Some(e) = format.file_extensions.first() {
                *e
            } else { return };
            ctrl.archives().with_archives(|a|
                if let Some(ma) = a.iter().find_map(|a| match a {
                    Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                    _ => None
                }) {
                    let p = task.rel_path().split('/').fold(ma.out_dir(), |p, s| p.join(s));
                    let _ = std::fs::create_dir_all(&p);
                    let out = p.join("index").with_extension(ext);
                    let _ = std::fs::copy(path, out);
                });
        }
    }
    pub fn set_log_str(&self, success:bool, ctrl:&dyn Controller, task:&BuildTask, s:String) {
        ctrl.archives().with_archives(|a|
            if let Some(ma) = a.iter().find_map(|a| match a {
                Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                _ => None
            }) {
                let p = task.rel_path().split('/').fold(ma.out_dir(),|p,s| p.join(s));
                let _ = std::fs::create_dir_all(&p);
                let out = p.join(self.target.to_string()).with_extension("log");
                std::fs::write(out,s.as_bytes()).unwrap();
            });
        let mut data = self.data.write();
        data.done = Some(success);
    }
    pub fn set_log_path(&self, success:bool, ctrl:&dyn Controller, task:&BuildTask, path:&Path) {
        if path.exists() {
            ctrl.archives().with_archives(|a|
                if let Some(ma) = a.iter().find_map(|a| match a {
                    Archive::Physical(ma) if ma.id() == task.archive().id() => Some(ma),
                    _ => None
                }) {
                    let p = task.rel_path().split('/').fold(ma.out_dir(), |p, s| p.join(s));
                    let _ = std::fs::create_dir_all(&p);
                    let out = p.join(self.target.to_string()).with_extension("log");
                    let _ = std::fs::copy(path,out);
                })
        }
        let mut data = self.data.write();
        data.done = Some(success);
    }

     */
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TaskState {
    Running,Queued,Blocked,Done,Failed,None
}

#[derive(Debug)]
pub(super) struct BuildTaskI {
    pub(super)archive: ArchiveURI,
    pub(super)base_path:PathBuf,
    pub(super)steps: Box<[BuildStep]>,
    pub(super)next: parking_lot::Mutex<Option<u8>>,
    pub(super)path:PathBuf,
    pub(super)status:parking_lot::Mutex<TaskState>,
    pub(super)rel_path:Box<str>,
    pub(super)format:FormatOrTarget,
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
pub struct BuildTask(pub(super) triomphe::Arc<BuildTaskI>);
impl BuildTask {
    pub fn as_entry(&self) -> QueueEntry {
        QueueEntry {
            archive: self.archive().id().clone(),
            rel_path: self.rel_path().to_string(),
            target: self.0.format,
            step:(self.0.next.lock().unwrap_or(self.0.steps.len() as u8 - 1),self.0.steps.len() as u8)
        }
    }
    pub fn archive(&self) -> ArchiveURI {
        self.0.archive
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

    pub(super) fn new<Ctrl:Controller>(tr:TaskSpec<'_>,ctrl:&Ctrl) -> Option<Self> {
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
            next: Some(0).into(),
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
    pub archive: ArchiveURI,
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
