use std::{io::Read, path::Path};

use flams_ontology::{content::modules::OpenModule, narration::documents::UncheckedDocument, DocumentRange, Unchecked};
pub use flams_utils::global;
use flams_utils::CSS;

use crate::{backend::AnyBackend, building::{BuildArtifact, BuildResult, BuildTask}};

global! {SER SourceFormat {name,
  description: &'static str,
  file_exts: &'static [&'static str],
  targets: &'static [BuildTargetId],
  dependencies:fn(&AnyBackend,task:&BuildTask)
}}

#[macro_export]
macro_rules! source_format {
  ($name:ident[$($ft:literal),+] [$($tgt:expr)=>*] @ $desc:literal = $deps:expr) => {
    $crate::formats::global!{NEW {$crate::formats}SourceFormat; $name [
      $desc,&[$($ft),+],&[$($tgt),*],$deps
    ]
    }
  }
}

global! {SER BuildTarget {name,
  description: &'static str,
  dependencies: &'static [BuildArtifactTypeId],
  yields: &'static [BuildArtifactTypeId],
  run: fn(&AnyBackend,task:&BuildTask) -> BuildResult
}}


#[macro_export]
macro_rules! build_target {
  ($name:ident [$($dep:expr),*] => [$($yield:expr),*] @ $desc:literal = $f:expr) => {
    $crate::formats::global!{NEW {$crate::formats}BuildTarget; $name [
      $desc,&[$($dep),*],&[$($yield),*],$f
    ]
    }
  }
}

build_target!(check [UNCHECKED_OMDOC] => [OMDOC]
  @ "Resolve OMDoc dependencies and type check"
  = |_,_| { BuildResult::empty() }
);

global! {SER BuildArtifactType {name,
  description: &'static str
}}

#[macro_export]
macro_rules! build_result {
  ($name:ident @ $desc:literal) => {
    $crate::formats::global!{NEW {$crate::formats}BuildArtifactType; $name [$desc]
    }
  }
}

build_result!(unchecked_omdoc @ "OMDoc ready to be checked");
build_result!(omdoc @ "OMDoc document and modules; fully checked");
build_result!(pdf @ "PDF document (semantically opaque)");

pub struct OMDocResult {
  pub document: UncheckedDocument,
  pub html:HTMLData,
  pub modules:Vec<OpenModule<Unchecked>>,
}

#[derive(Debug)]
pub struct HTMLData {
  pub html:String,
  pub css:Vec<CSS>,
  pub body:DocumentRange,
  pub inner_offset:usize,
  pub refs:Vec<u8>
}

impl OMDocResult {

  pub(crate) fn load_html_body(path:&Path,full:bool) -> Option<(Vec<CSS>,String)> {
    use std::io::{Seek,SeekFrom};
    let file = std::fs::File::open(path).ok()?;
    let mut file = std::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let body_start = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
    let body_start = 20 + css_offset + css_len + body_start;
    let body_len = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    let inner_offset = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let mut css = vec![0;css_len as usize];
    file.seek(SeekFrom::Start(u64::from(css_offset + 20))).ok()?;
    file.read_exact(&mut css).ok()?;
    let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
    if full {
      file.seek(SeekFrom::Start(u64::from(body_start))).ok()?;
      let mut html = vec![0;body_len as usize];
      file.read_exact(&mut html).ok()?;
      String::from_utf8(html).ok().map(|html| (css,html))
    } else {
      let inner_offset_real = body_start + inner_offset;
      file.seek(SeekFrom::Start(u64::from(inner_offset_real))).ok()?;
      let mut html = vec![0;((body_len - inner_offset) as usize) - "</body>".len()];
      file.read_exact(&mut html).ok()?;
      String::from_utf8(html).ok().map(|html| (css,html))

    }
  }

  #[cfg(feature="tokio")]
  pub(crate) async fn load_html_body_async(path:std::path::PathBuf,full:bool) -> Option<(Vec<CSS>,String)> {
    use std::io::SeekFrom;
    use tokio::io::{AsyncSeekExt,AsyncReadExt};
    //println!("loading {path:?}");
    let file = tokio::fs::File::open(path).await.ok()?;
    let mut file = tokio::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).await.ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let body_start = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
    let body_start = 20 + css_offset + css_len + body_start;
    let body_len = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    let inner_offset = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    //println!("css_offset: {}, css_len: {}, body_start: {}, body_len: {}, inner_offset: {}",css_offset,css_len,body_start,body_len,inner_offset);
    let mut css = vec![0;css_len as usize];
    file.seek(SeekFrom::Start(u64::from(css_offset + 20))).await.ok()?;
    file.read_exact(&mut css).await.ok()?;
    let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
    if full {
      file.seek(SeekFrom::Start(u64::from(body_start))).await.ok()?;
      let mut html = vec![0;body_len as usize];
      file.read_exact(&mut html).await.ok()?;
      String::from_utf8(html).ok().map(|html| (css,html))
    } else {
      let inner_offset_real = body_start + inner_offset;
      //println!("Seek");
      file.seek(SeekFrom::Start(u64::from(inner_offset_real))).await.ok()?;
      let mut html = vec![0;((body_len - inner_offset) as usize) - "</body>".len()];
      //println!("Reading {}",html.len());
      file.read_exact(&mut html).await.ok()?;
      //println!("Decode");
      String::from_utf8(html).ok().map(|html| (css,html))
    }
  }

  #[cfg(feature="tokio")]
  pub(crate) async fn load_html_full_async(path:std::path::PathBuf) -> Option<String> {
    use std::io::SeekFrom;
    use tokio::io::{AsyncSeekExt,AsyncReadExt};
    //println!("loading {path:?}");
    let file = tokio::fs::File::open(path).await.ok()?;
    let mut file = tokio::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).await.ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let html_start = 20 + css_offset + css_len;
    
    file.seek(SeekFrom::Start(u64::from(html_start))).await.ok()?;
    let mut ret = String::new();
    file.read_to_string(&mut ret).await.ok()?;
    Some(ret)
  }

  pub(crate) fn load_html_full(path:std::path::PathBuf) -> Option<String> {
    use std::io::{Seek,SeekFrom};
    let file = std::fs::File::open(path).ok()?;
    let mut file = std::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let html_start = 20 + css_offset + css_len;
    
    file.seek(SeekFrom::Start(u64::from(html_start))).ok()?;
    let mut ret = String::new();
    file.read_to_string(&mut ret).ok()?;
    Some(ret)
  }

  pub(crate) fn load_html_fragment(path:&Path,range:DocumentRange) -> Option<(Vec<CSS>,String)> {
    use std::io::{Seek,SeekFrom};
    let file = std::fs::File::open(path).ok()?;
    let mut file = std::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    //let body_start = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
    let html_start = 20 + css_offset + css_len;
    //let body_len = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    //let inner_offset = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let mut css = vec![0;css_len as usize];
    file.seek(SeekFrom::Start(u64::from(css_offset + 20))).ok()?;
    file.read_exact(&mut css).ok()?;
    let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
    file.seek(SeekFrom::Start(u64::from(html_start) + range.start as u64)).ok()?;
    let mut html = vec![0;range.end - range.start];
    file.read_exact(&mut html).ok()?;
    String::from_utf8(html).ok().map(|html| (css,html))
  }

  #[allow(clippy::cast_possible_wrap)]
  pub(crate) fn load_reference<T:flams_ontology::Resourcable>(path:&Path,range:DocumentRange) -> Option<T> {
    use std::io::{Seek,SeekFrom};
    let file = std::fs::File::open(path).ok()?;
    let mut file = std::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).ok()?;
    file.seek(SeekFrom::Current(range.start as i64)).ok()?;
    let len = range.end - range.start;
    let mut bytes = vec![0;len];
    file.read_exact(&mut bytes).ok()?;
    bincode::serde::decode_from_slice(&bytes,bincode::config::standard()).ok().map(|(a,_)| a)
  }


  #[cfg(feature="tokio")]
  pub(crate) async fn load_html_fragment_async(path:std::path::PathBuf,range:DocumentRange) -> Option<(Vec<CSS>,String)> {
    use std::io::SeekFrom;
    use tokio::io::{AsyncSeekExt,AsyncReadExt};
    let file = tokio::fs::File::open(path).await.ok()?;
    let mut file = tokio::io::BufReader::new(file);
    let mut buf = [0;20];
    file.read_exact(&mut buf).await.ok()?;
    let css_offset = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let css_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let html_start = 20 + css_offset + css_len;
    //let body_start = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
    //let body_start = 20 + css_offset + css_len + body_start;
    //let body_len = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    //let inner_offset = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let mut css = vec![0;css_len as usize];
    file.seek(SeekFrom::Start(u64::from(css_offset + 20))).await.ok()?;
    file.read_exact(&mut css).await.ok()?;
    let css = bincode::serde::decode_from_slice(&css,bincode::config::standard()).ok()?.0;
    file.seek(SeekFrom::Start(u64::from(html_start)  + range.start as u64)).await.ok()?;
    let mut html = vec![0;range.end - range.start];
    file.read_exact(&mut html).await.ok()?;
    String::from_utf8(html).ok().map(|html| (css,html))
  }

  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::cognitive_complexity)]
  pub(crate) fn write(&self,path:&Path) {
    use std::io::Write;
    macro_rules! err {
      ($e:expr) => {
          match $e {
              Ok(r) => r,
              Err(e) => {
                  tracing::error!("Failed to save {}: {}", path.display(), e);
                  return
              }
          }
      }
    }        
    macro_rules! er {
        ($e:expr) => {
            if let Err(e) = $e {
                tracing::error!("Failed to save {}: {}", path.display(), e);
                return
            }
        }
    }
    let file = err!(std::fs::File::create(path));
    let mut buf = std::io::BufWriter::new(file);
    let Self {html:HTMLData{css,body,inner_offset,refs,html},..} = self;
    let css_offset = (refs.len() as u32).to_be_bytes();
    let css = err!(bincode::serde::encode_to_vec(css, bincode::config::standard()));
    let css_len = (css.len() as u32).to_be_bytes();
    let body_start = (body.start as u32).to_be_bytes();
    let body_len = ((body.end - body.start) as u32).to_be_bytes();
    let inner_offset = (*inner_offset as u32).to_be_bytes();
    er!(buf.write_all(&css_offset));
    er!(buf.write_all(&css_len));
    er!(buf.write_all(&body_start));
    er!(buf.write_all(&body_len));
    er!(buf.write_all(&inner_offset));
    er!(buf.write_all(refs));
    er!(buf.write_all(&css));
    er!(buf.write_all(html.as_bytes()));
    er!(buf.flush());
  }
}

impl BuildArtifact for OMDocResult {
  #[inline]
  fn get_type(&self) -> BuildArtifactTypeId {
    UNCHECKED_OMDOC
  }
  #[inline]
  fn get_type_id() -> BuildArtifactTypeId where Self:Sized {
      UNCHECKED_OMDOC
  }
  fn write(&self,_path:&std::path::Path) -> Result<(),std::io::Error> {
      unreachable!()
  }
  fn load(_p:&std::path::Path) -> Result<Self,std::io::Error> where Self:Sized {
      unreachable!()
  }
  #[inline]
  fn as_any(&self) -> &dyn std::any::Any {self}
}


global! {FLAMSExtension {name,
  on_start: fn()
}}

#[macro_export]
macro_rules! flams_extension {
  ($name:ident = $f:expr) => {
    $crate::formats::global!{NEW {$crate::formats} FLAMSExtension; $name [$f]}
  }
}

impl FLAMSExtension {
  pub fn initialize() {
    for e in Self::all() {
      let f = move || {
          tracing::info_span!("Initializing",extension=e.name()).in_scope(||
              (e.on_start())()
          );
      };
      #[cfg(feature="tokio")]
      flams_utils::background(f);
      #[cfg(not(feature="tokio"))]
      f();
    }
  }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum FormatOrTargets<'a> {
  Format(SourceFormatId),
  Targets(&'a[BuildTargetId])
}