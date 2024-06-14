use std::any::Any;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use parking_lot::RwLock;
use immt_core::building::formats::ShortId;
use immt_core::content::Module;
use immt_core::narration::Document;
use immt_core::uris::archives::ArchiveURI;
use immt_core::utils::triomphe::Arc;
use immt_core::utils::VecMap;

#[derive(Copy,Clone,Debug)]
pub struct BuildDataFormat {
    pub id:ShortId,
    pub description: &'static str,
    pub file_extensions: &'static [&'static str],
}

#[cfg(feature="serde")]
#[derive(serde::Serialize,serde::Deserialize,Debug,Clone)]
pub struct BuildDataFormatOwned {
    pub id:ShortId,
    pub description: String,
    pub file_extensions: Vec<String>,
}

impl BuildDataFormat {
    pub const CONTENT_OMDOC : BuildDataFormat = BuildDataFormat {id:ShortId::new("comdoc"),file_extensions:&[],
        description: "(Flexi-)formal representation of knowledge corresponding to the (flexi)formal fragment of the OMDoc ontology"
    };
    pub const NARRATIVE_OMDOC: BuildDataFormat = BuildDataFormat {id:ShortId::new("nomdoc"),file_extensions:&[],
        description: "Informal representation of the narrative structure of some document corresponding to the narrative fragment of the OMDoc ontology"
    };
    pub const PDF : BuildDataFormat = BuildDataFormat {id:ShortId::new("pdf"),file_extensions:&["pdf"],
        description: "PDF"
    };

    #[cfg(feature="serde")]
    pub fn to_owned(&self) -> BuildDataFormatOwned {
        BuildDataFormatOwned {
            id:self.id,
            description:self.description.into(),
            file_extensions:self.file_extensions.iter().map(|s| s.to_string()).collect()
        }
    }
}

#[derive(Copy,Clone,Debug)]
pub struct BuildTarget {
    pub id: ShortId,
    pub description: &'static str,
    pub requires: &'static [BuildDataFormat],
    pub produces: &'static [BuildDataFormat],
    pub extension: Option<ShortId>
}

#[cfg(feature="serde")]
#[derive(serde::Serialize,serde::Deserialize,Debug,Clone)]
pub struct BuildTargetOwned {
    pub id:ShortId,
    pub description: String,
    pub requires: Vec<ShortId>,
    pub produces: Vec<ShortId>,
}

impl BuildTarget {
    #[cfg(feature="serde")]
    pub fn to_owned(&self) -> BuildTargetOwned {
        BuildTargetOwned {
            id:self.id,
            description:self.description.into(),
            requires:self.requires.iter().map(|f| f.id).collect(),
            produces:self.produces.iter().map(|f| f.id).collect()
        }
    }
    pub const CHECK: BuildTarget = BuildTarget {id:ShortId::CHECK,
        requires:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
        produces:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
        description: "Type check OMDoc content",
        extension:None
    };
}

#[derive(Copy,Clone,Debug)]
pub struct SourceFormat {
    pub id: ShortId,
    pub file_extensions: &'static [&'static str],
    pub description: &'static str,
    pub targets:&'static [BuildTarget],
    pub extension: Option<ShortId>
}

impl SourceFormat {

    #[cfg(feature="serde")]
    pub fn to_owned(&self) -> SourceFormatOwned {
        SourceFormatOwned {
            id: self.id,
            file_extensions: self.file_extensions.iter().map(|s| s.to_string()).collect(),
            description: self.description.into(),
            targets: self.targets.iter().map(|t| t.id).collect()
        }
    }
}

#[cfg(feature="serde")]
#[derive(serde::Serialize,serde::Deserialize,Debug,Clone)]
pub struct SourceFormatOwned {
    pub id:ShortId,
    pub file_extensions: Vec<String>,
    pub description: String,
    pub targets: Vec<ShortId>,
}

struct BuildData {
    path: Option<PathBuf>,
    rel_path: Box<str>,
    archive: ArchiveURI,
    source: OnceCell<Option<Box<str>>>,
    data: RwLock<VecMap<&'static str, Box<dyn Any>>>,
    document: Option<(Document,Box<str>)>,
    modules: Vec<Module>,
    format: ShortId
}
pub struct BuildJob(Arc<BuildData>);
impl BuildJob {
    pub fn new(path:&Path, archive:ArchiveURI, rel_path:&str, format: ShortId) -> Self {
        Self(Arc::new(BuildData {
            path: Some(path.to_owned()),
            rel_path: rel_path.into(),
            archive,
            source: OnceCell::new(),
            data: RwLock::new(VecMap::default()),
            document: None,
            modules: Vec::new(),
            format
        }))
    }
    pub fn source(&self) -> Option<&str> {
        self.0.source
            .get_or_init(|| match &self.0.path {
                None => None,
                Some(p) => std::fs::read_to_string(p).ok().map(|s| s.into()),
            })
            .as_deref()
    }
    pub fn path(&self) -> Option<&Path> {
        self.0.path.as_deref()
    }
}

#[derive(Clone)]
pub enum Dependency {
    Physical {
        id: ShortId,
        archive: ArchiveURI,
        filepath: Arc<str>,
        strong: bool,
    },
    Logical, // TODO
}
