use crate::utils::{
    errors::{ArtifactSaveError, FileError, WriteError},
    lazy_file::LazyFileWriter,
    path_ext::PathExt,
};
use ftml_ontology::{
    domain::modules::Module,
    narrative::{DocumentRange, documents::Document},
    utils::Css,
};
use std::path::{Path, PathBuf};

pub enum FileOrString {
    File(PathBuf),
    Str(Box<str>),
}

pub trait Artifact: std::any::Any {
    fn kind(&self) -> &'static str;
    /// # Errors
    fn write(&self, into: &Path) -> Result<(), ArtifactSaveError>;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait FileArtifact: std::any::Any {
    fn kind(&self) -> &'static str;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn as_any(&self) -> &dyn std::any::Any;
    fn source(&self) -> &Path;
}
impl<F: FileArtifact> Artifact for F {
    #[inline]
    fn kind(&self) -> &'static str {
        <Self as FileArtifact>::kind(self)
    }
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        <Self as FileArtifact>::as_any_mut(self)
    }
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        <Self as FileArtifact>::as_any(self)
    }
    fn write(&self, into: &Path) -> Result<(), ArtifactSaveError> {
        self.source().rename_safe(&into)?;
        Ok(())
    }
}

pub struct FtmlString(pub Box<str>);
impl Artifact for FtmlString {
    #[inline]
    fn kind(&self) -> &'static str {
        "ftml"
    }
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as _
    }
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
    fn write(&self, into: &Path) -> Result<(), ArtifactSaveError> {
        std::fs::write(into, self.0.as_bytes())
            .map_err(|e| ArtifactSaveError::Fs(FileError::Write(into.to_path_buf(), e)))
    }
}

#[derive(Debug)]
pub struct ContentResult {
    pub document: Document,
    pub modules: Vec<Module>,
    pub data: Box<[u8]>,
    pub body: DocumentRange,
    pub inner_offset: u32,
    pub css: Box<[Css]>,
    pub ftml: Box<str>,
    #[cfg(feature = "rdf")]
    pub triples: Vec<ulo::rdf_types::Triple>,
}
impl Artifact for ContentResult {
    #[inline]
    fn kind(&self) -> &'static str {
        "content"
    }
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as _
    }
    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
    fn write(&self, into: &Path) -> Result<(), ArtifactSaveError> {
        let mut writer = match LazyFileWriter::<6>::new(into) {
            Ok(w) => w,
            Err(e) => {
                return Err(ArtifactSaveError::Fs(FileError::Write(
                    into.to_path_buf(),
                    e,
                )));
            }
        };
        macro_rules! err {
            ($e:expr) => {
                if let Err(e) = $e {
                    match e {
                        WriteError::Io(e) => {
                            return Err(ArtifactSaveError::Fs(FileError::Write(
                                into.to_path_buf(),
                                e,
                            )));
                        }
                        WriteError::Encode(e) => return Err(ArtifactSaveError::Encode(e)),
                        _ => unreachable!(),
                    }
                }
            };
        }
        err!(writer.write(&self.body));
        err!(writer.write(&self.inner_offset));
        err!(writer.write(&self.css));
        err!(writer.write_bytes(&self.data));
        err!(writer.write(&self.document));
        err!(writer.write_string(&self.ftml));
        Ok(())
    }
}
