use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use immt_core::building::formats::{ShortId, SourceFormatId};
use immt_core::ontology::archives::{MathArchiveSpec, MathArchiveSpecRef, StorageSpecRef};
use immt_core::utils::filetree::{Dir, FileChange, SourceDir, SourceDirEntry};
use immt_core::utils::triomphe::Arc;
use crate::building::targets::SourceFormat;
use crate::core::uris::archives::{ArchiveId, ArchiveURIRef};
use crate::core::utils::VecMap;
use crate::utils::asyncs::ChangeSender;
use immt_core::building::buildstate::AllStates;

pub trait Storage:std::fmt::Debug {
    fn spec(&self) -> StorageSpecRef<'_>;
    #[inline]
    fn uri(&self) -> ArchiveURIRef<'_> { self.spec().uri }
    #[inline]
    fn id(&self) -> &ArchiveId { self.uri().id() }
    #[inline]
    fn parents(&self) -> std::str::Split<'_,char> {
        self.uri().id().steps()
    }
    #[inline]
    fn is_meta(&self) -> bool {
        self.id().is_meta()
    }
    #[inline]
    fn attributes(&self) -> &VecMap<Box<str>,Box<str>> {
        self.spec().attributes
    }
    #[inline]
    fn formats(&self) -> &[SourceFormatId] {
        self.spec().formats
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
pub struct MathArchive {
    spec:MathArchiveSpec,
    #[cfg_attr(feature = "serde",serde(skip))]
    pub source:Option<Vec<SourceDirEntry>>,
    state: AllStates
}
impl MathArchive {

    pub fn out_dir(&self) -> PathBuf { self.path().join(".immt") }
    pub fn source_files(&self) -> Option<&[SourceDirEntry]> { self.source.as_deref() }

    pub fn update_sources(&mut self, formats:&[SourceFormat], on_change:&ChangeSender<FileChange>) {
        let path = self.out_dir().join("ls_f.db");
        let mut dir =  if path.exists() {
            SourceDir::parse(&path).unwrap_or_else(|_| vec![])
        } else { let _ = std::fs::create_dir_all(path.parent().unwrap()); vec![] };
        let source = self.path().join("source");
        if source.is_dir() {
            let formats = formats.iter().flat_map(|f| 
                f.file_extensions.iter().filter(|_| self.formats().contains(&f.id)).map(|e| (*e,f.id))
            ).collect::<Vec<_>>();
            let (b,state) = SourceDir::update(&source,&mut dir,&formats,&self.spec.ignore_source,|c| on_change.send(c));
            self.state = state;
            if b {
                let _ = Dir::write_to(&dir,&path);
            }
        }
        self.source = Some(dir)
    }
}

impl std::hash::Hash for MathArchive {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri().hash(state);
    }
}
impl PartialEq for MathArchive {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.uri().eq(&other.uri()) }
}
impl Eq for MathArchive {}

impl PartialOrd<Self> for MathArchive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id().cmp(other.id()))
    }
}
impl Ord for MathArchive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(other.id())
    }
}

impl MathArchive {
    #[inline]
    pub fn archive_spec(&self) -> MathArchiveSpecRef<'_> { self.spec.as_ref() }
    #[inline]
    pub fn path(&self) -> &Path { &self.spec.path }
    
    pub fn new_from(spec:MathArchiveSpec) -> Self {
        Self { spec,source:None,state: AllStates::default() }
    }
    pub fn state(&self) -> &AllStates { &self.state }
}
impl Storage for MathArchive {
    #[inline]
    fn spec(&self) -> StorageSpecRef<'_> { self.archive_spec().storage }
}

pub trait VirtualArchive:Storage+Send+Sync {
    fn kind(&self) -> &str;
}

#[derive(Debug)]
pub enum Archive {
    Physical(MathArchive),
    Virtual(Arc<dyn VirtualArchive>)
}
impl Storage for Archive {
    #[inline]
    fn spec(&self) -> StorageSpecRef<'_> { 
        match self {
            Self::Physical(a) => a.spec(),
            Self::Virtual(a) => a.spec()
        }
    }
}

impl std::hash::Hash for Archive {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri().hash(state);
    }
}
impl PartialEq for Archive {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.uri().eq(&other.uri()) }
}
impl Eq for Archive {}

impl PartialOrd<Self> for Archive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id().cmp(other.id()))
    }
}
impl Ord for Archive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(other.id())
    }
}