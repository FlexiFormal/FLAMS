use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use immt_core::building::formats::{ShortId, SourceFormatId};
use immt_core::ontology::archives::{MathArchiveSpec, MathArchiveSpecRef, StorageSpecRef};
use immt_core::utils::filetree::{Dir, FileChange, SourceDir, SourceDirEntry};
use immt_core::utils::triomphe::Arc;
use crate::building::targets::SourceFormat;
use crate::core::uris::archives::{ArchiveId};
use crate::core::utils::VecMap;
use crate::utils::asyncs::ChangeSender;
use immt_core::building::buildstate::AllStates;
use immt_core::narration::Language;
use immt_core::uris::archives::ArchiveURI;
use immt_core::uris::Name;

pub trait Storage:std::fmt::Debug {
    fn spec(&self) -> StorageSpecRef<'_>;
    #[inline]
    fn uri(&self) -> ArchiveURI { self.spec().uri }
    #[inline]
    fn id(&self) -> ArchiveId { self.uri().id() }
    #[inline]
    fn parents(&self) -> std::str::Split<'static,char> {
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
    /*
    pub fn doc_dir(&self,path:Option<Name>,lang:Language,name:Name) -> Option<PathBuf> {
        let mut p = self.out_dir();
        if let Some(n) = path { p = p.join(n.as_ref()) }
        let name = name.as_ref();
        for d in std::fs::read_dir(&p).ok()? {
            let d = if let Ok(d) = d {d} else {continue};
            let f = d.file_name();
            let f = if let Some(f) = f.to_str() {f} else {continue};
            let f = if let Some(f) = f.strip_prefix(name) {f} else {continue};
            if f.is_empty() { return Some(d.path()) }
            if f.as_bytes()[0] != b'.' {continue}
            let f = &f[1..];
            let lstr:&'static str = lang.into();
            if f.contains('.') {
                if f.starts_with(lstr) {
                    return Some(d.path())
                }
            } else {return Some(d.path())}
        }
        None
    }

     */
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
        Some(self.id().cmp(&other.id()))
    }
}
impl Ord for MathArchive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(&other.id())
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
        Some(self.id().cmp(&other.id()))
    }
}
impl Ord for Archive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(&other.id())
    }
}