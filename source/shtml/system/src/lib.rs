#![feature(string_from_utf8_lossy_owned)]

mod parser;

use either::Either;
use immt_ontology::uris::ArchiveURITrait;
use immt_system::{backend::{AnyBackend, Backend}, build_result, build_target, building::{BuildArtifact, BuildResult, BuildResultArtifact, BuildTask}, formats::{BuildArtifactTypeId, CHECK, UNCHECKED_OMDOC}, source_format};

source_format!(shtml ["html","xhtml","htm"] [SHTML_IMPORT => SHTML_OMDOC => CHECK] @
  "Semantically annotated HTML"
  = |_,_| todo!()
);

build_target!(
  shtml_import [] => [SHTML_DOC] 
  @ "Import existing sHTML"
  = |_,_| todo!()
);

build_target!(
  shtml_omdoc [SHTML_DOC] => [UNCHECKED_OMDOC] 
  @ "Extract OMDoc from sHTML" 
  = extract
);

build_result!(shtml_doc @ "Semantically annotated HTML");

fn extract(backend:&AnyBackend,task:&BuildTask) -> BuildResult {
  let html:Result<HTMLString,_> = backend.with_archive(task.archive().archive_id(), |a| {
    let Some(a) = a else {return Err(BuildResult::err())};
    a.load(task.rel_path()).map_err(|e| BuildResult {
      log:Either::Left(format!("Error loading html data for {}/{}",task.archive().archive_id(),task.rel_path())),
      result:Err(())
    })
  });
  let html = match html {
    Err(e) => return e,
    Ok(h) => h
  };
  parser::HTMLParser::run(&html.0,task.document_uri(),task.rel_path(),backend)
}

pub struct HTMLString(pub String);
impl BuildArtifact for HTMLString {
  #[inline] fn get_type_id() -> BuildArtifactTypeId where Self:Sized {
      SHTML_DOC
  }
  #[inline]
  fn get_type(&self) -> BuildArtifactTypeId {
      SHTML_DOC
  }
  fn write(&self,path:&std::path::Path) -> Result<(),std::io::Error> {
    std::fs::write(path, &self.0)
  }
  fn load(p:&std::path::Path) -> Result<Self,std::io::Error> where Self:Sized {
    let s = std::fs::read_to_string(p)?;
    Ok(Self(s))
  }

  #[inline]
  fn as_any(&self) -> &dyn std::any::Any {self}
}
impl HTMLString {
  #[must_use]
  pub fn create(s:String) -> BuildResultArtifact {
    BuildResultArtifact::Data(Box::new(Self(s)))
  }
}