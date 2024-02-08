use std::fmt::Debug;
use std::path::{Path, PathBuf};
use immt_api::formats::FormatStore;
use immt_api::archives::{ArchiveData, ArchiveGroupBase, ArchiveGroupT, ArchiveId, ArchiveT, IgnoreSource};
use immt_api::source_files::{ParseError, SerializeError, SourceDir};
use immt_api::utils::problems::{ProblemHandler as PHandlerT};
use tracing::{event, instrument};

#[derive(Debug)]
pub struct Archive {
    pub(in crate::backend) manifest:ArchiveData,
    path:PathBuf,
    state:parking_lot::RwLock<ArchiveState>,
    //watcher: Option<RecommendedWatcher>
}
impl ArchiveT for Archive {
    #[instrument(level = "info", name = "initialize", target = "backend::archive", skip(data, handler, formats))]
    fn new_from<P: PHandlerT>(data: ArchiveData, path: &Path, handler: &P, formats: &FormatStore) -> Self {
        let mut state = ArchiveState::default();
        state.initialized = true;
        event!(tracing::Level::DEBUG,"Initializing archive {}",data.id);
        if !Self::ls_f(&mut state, path, &data.ignores, handler, formats) && !data.is_meta {
            handler.add("Missing source", format!("Archive has no source directory: {}",data.id));
        }
        event!(tracing::Level::DEBUG,"Done");
        Self {
            manifest:data,
            path:path.to_path_buf(),
            state:parking_lot::RwLock::new(state),
            //watcher:None
        }
    }
    #[inline]
    fn data(&self) -> &ArchiveData { &self.manifest }
}

#[derive(Debug)]
struct ArchiveState {
    initialized:bool,
    source_dir:SourceDir
}
impl Default for ArchiveState {
    fn default() -> Self {
        Self {
            initialized:false,
            source_dir:SourceDir{name:"source".into(),children:Vec::new().into()}
        }
    }
}
// use notify::{Watcher, RecommendedWatcher, RecursiveMode};
impl Archive {
    fn ls_f<P: PHandlerT>(state:&mut ArchiveState, path:&Path,ignore:&IgnoreSource, handler:&P,formats:&FormatStore) -> bool {
        let dirfile = path.join(".out").join("ls_f.db");
        if dirfile.exists() {
            match SourceDir::parse(&dirfile) {
                Ok(v) => state.source_dir.children = v,
                Err(ParseError::DecodingError) => handler.add("ArchiveManager",format!("Error decoding {}",dirfile.display())),
                Err(ParseError::FileError) => handler.add("ArchiveManager",format!("Error reading {}",dirfile.display()))
            }
        }
        let source = path.join("source");
        if source.exists() {
            SourceDir::update(&source, &mut state.source_dir.children, handler,ignore, &|s| formats.from_ext(s));
            match state.source_dir.write_to(&dirfile) {
                Ok(_) => {},
                Err(SerializeError::EncodingError) => handler.add("ArchiveManager",format!("Error encoding {}",dirfile.display())),
                Err(SerializeError::IOError) => handler.add("ArchiveManager",format!("Error writing to {}",dirfile.display()))
            }
            true
        } else {false}
    }
/*
    pub(in crate::backend) fn watch(&mut self,handler:&ProblemHandler) {
        if self.watcher.is_none() {
            if let Ok(watcher) = Self::new_watcher(self.state.clone(), &self.path.join("source"), handler) {
                self.watcher = Some(watcher);
            }
        }
    }
    pub(in crate::backend) fn unwatch(&mut self) {
        self.watcher = None;
    }

    fn new_watcher(state:Arc<parking_lot::RwLock<ArchiveState>>,source:&Path,handler:&ProblemHandler) -> Result<RecommendedWatcher,notify::Error> {
        let ih = handler.clone();
        match notify::recommended_watcher(move |res:Result<notify::Event,notify::Error>| {
            match res {
                Ok(event) => {
                    let state = state.write();
                    match event.kind {
                        notify::EventKind::Create(_) => {
                            todo!()
                        }
                        notify::EventKind::Modify(_) => {
                            todo!()
                        }
                        notify::EventKind::Remove(_) => {
                            todo!()
                        }
                        _ => {}
                    }
                }
                Err(e) => ih.add("file watch",format!("Error: {:?}", e))
            }
        }) {
            Err(e) => {
                handler.add("file watch",format!("Error: {:?}", e));
                Err(e)
            },
            Ok(mut w) => {
                match w.watch(source, RecursiveMode::Recursive) {
                    Err(e) => {
                        handler.add("file watch",format!("Error: {:?}", e));
                        Err(e)
                    },
                    Ok(_) => {
                        event!(tracing::Level::INFO,"Watching {}",source.display());
                        Ok(w)
                    }
                }
            }
        }
    }

    fn iter_sources<R,F:FnMut(&SourceFile,&mut R)>(&self,mut init:R,mut f:F) -> R {
        let state = self.state.read();
        let i = state.source_dir.iter();//TreeIter::new(, |s:&SourceDir| s.children.iter(), |e| e.as_ref());
        for fl in i { f(fl,&mut init) }
        init
    }

 */
}

#[derive(Debug)]
pub struct ArchiveGroup {
    base:ArchiveGroupBase<Archive,Self>
}
impl ArchiveGroupT<Archive> for ArchiveGroup {
    fn new(id:ArchiveId) -> Self {
        Self {
            base:ArchiveGroupBase {
                id,meta:None,archives:Vec::new()
            }
        }
    }
    fn base(&self) -> &ArchiveGroupBase<Archive,Self> { &self.base }
    fn base_mut(&mut self) -> &mut ArchiveGroupBase<Archive,Self> {
        &mut self.base
    }
}