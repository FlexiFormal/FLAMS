use std::path::Path;
use either::Either;
use tracing::event;
use crate::formats::FormatId;
use crate::{Seq, Str};
use crate::archives::IgnoreSource;
use crate::utils::iter::{HasChildren, HasChildrenMut, HasChildrenRef, LeafIterator, TreeLike, TreeMutLike, TreeRefLike};
use crate::utils::problems::ProblemHandler;

#[derive(Debug)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub enum FileLike {
    Dir(SourceDir),
    File(SourceFile)
}

#[cfg(feature="bincode")]
pub enum ParseError {
    DecodingError,
    FileError
}

#[cfg(feature="bincode")]
pub enum SerializeError {
    EncodingError,
    IOError
}

#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct SourceDir{pub name:Str,pub children:Seq<FileLike>}
impl SourceDir {
    pub fn iter(&self) -> LeafIterator<&FileLike> { self.iter_leafs() }
    pub fn iter_mut(&mut self) -> LeafIterator<&mut FileLike> { self.iter_leafs() }
    #[cfg(feature="bincode")]
    pub fn parse<F:AsRef<Path>>(file:F) -> Result<Seq<FileLike>,ParseError> {
        let file = file.as_ref();
        match std::fs::File::open(file) {
            Ok(mut f) => match bincode::decode_from_std_read(&mut f,bincode::config::standard()) {
                Ok(v) => Ok(v),
                _ => Err(ParseError::DecodingError)
            },
            _ => Err(ParseError::FileError)
        }
    }
    #[cfg(feature="bincode")]
    pub fn write_to<F:AsRef<Path>>(&self,file:F) -> Result<(),SerializeError> {
        let file = file.as_ref();
        file.parent().map(std::fs::create_dir_all);
        let mut f = match std::fs::File::create(file) {
            Ok(f) => f,
            _ => return Err(SerializeError::IOError)
        };
        if bincode::encode_into_std_write(&self.children,&mut f,bincode::config::standard()).is_err() {
            Err(SerializeError::EncodingError)
        } else { Ok(()) }
    }
    pub fn update<F:AsRef<Path>,P:ProblemHandler>(in_dir:F,old:&mut Seq<FileLike>,handler:&P,ignore:&IgnoreSource,from_ext:&impl Fn(&str) -> Option<FormatId>) {
        let mut dones_v = Vec::new();
        let mut todos = Vec::new();
        let mut oldv: Vec<_> = std::mem::take(old).into();
        let path = in_dir.as_ref();
        if ignore.ignores(path) {
            event!(tracing::Level::TRACE,"Ignoring {} because of {}",path.display(),ignore);
            return
        }
        oldv.reverse();
        let curr = match std::fs::read_dir(path) {
            Ok(d) => d,
            _ => {
                handler.add("ArchiveManager",format!("Could not read directory {}",path.display()));
                return
            }
        };
        for d in curr {
            let d = match d {
                Ok(d) => d,
                _ => {
                    handler.add("ArchiveManager",format!("Error when reading directory {}",path.display()));
                    continue
                }
            };
            let path = d.path();
            if ignore.ignores(&path) {
                event!(tracing::Level::TRACE,"Ignoring {} because of {}",path.display(),ignore);
                return
            }
            let md = match d.metadata() {
                Ok(d) => d,
                _ => {
                    handler.add("ArchiveManager",format!("Could not read metadata of file {}",path.display()));
                    continue
                }
            };
            if md.is_dir() {
                let old = oldv.iter().enumerate().rfind(|s| match s {
                    (_,FileLike::Dir(s)) => &*s.name == path.file_name().unwrap(),
                    _ => false
                }).map(|(i,_)| i);
                if let Some(i) = old {
                    let old = oldv.remove(i);
                    dones_v.push(old);
                } else {
                    dones_v.push(FileLike::Dir(SourceDir{
                        name:path.file_name().unwrap().to_str().unwrap().to_string().into(),
                        children:Vec::new().into()
                    }));
                }
                todos.push((path,dones_v.len()-1));
            } else {
                let format = match path.extension() {
                    Some(ext) => match from_ext(ext.to_str().unwrap()) {
                        Some(f) => f,
                        _ => continue
                    }
                    _ => continue
                };
                let old = oldv.iter().enumerate().rfind(|s| match s {
                    (_,FileLike::File(s)) => &*s.name == path.file_name().unwrap(),
                    _ => false
                }).map(|(i,_)| i);
                if let Some(i) = old {
                    let mut old = oldv.remove(i).into_either().unwrap_right();
                    if let BuildState::UpToDate { last_built } = old.state {
                        let changed = md.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        if changed > last_built {
                            old.state = BuildState::Stale { last_built: changed };
                        }
                    }
                    dones_v.push(FileLike::File(old));
                } else {
                    dones_v.push(FileLike::File(SourceFile{
                        name:path.file_name().unwrap().to_str().unwrap().to_string().into(),
                        state:BuildState::New,
                        format
                    }));
                }
            }
        }
        for mut c in oldv {
            match &mut c {
                FileLike::File(f) => {
                    f.state = BuildState::Deleted;
                },
                FileLike::Dir(l) => for c in l.iter_mut() {
                    c.state = BuildState::Deleted;
                }
            }
            dones_v.push(c);
        }
        *old = dones_v.into();
        for (p,i) in todos {
            Self::update(&p,&mut old[i].as_either_mut().unwrap_left().children,handler,ignore,from_ext);
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct SourceFile{name:Str,state:BuildState,format: FormatId }


#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub enum BuildState {
    Deleted,
    New,
    Stale{last_built:u64},
    UpToDate{last_built:u64}
}

impl TreeLike for FileLike {
    type Leaf = SourceFile;
    type Node = SourceDir;
    fn into_either(self) -> Either<Self::Node,Self::Leaf> {
        match self {
            FileLike::Dir(d) => Either::Left(d),
            FileLike::File(f) => Either::Right(f)
        }
    }
}
impl TreeRefLike for FileLike {
    type Leaf = SourceFile;
    type Node = SourceDir;
    fn as_either(&self) -> Either<&Self::Node,&Self::Leaf> {
        match self {
            FileLike::Dir(d) => Either::Left(d),
            FileLike::File(f) => Either::Right(f)
        }
    }
}
impl TreeMutLike for FileLike {
    type Leaf = SourceFile;
    type Node = SourceDir;
    fn as_either_mut(&mut self) -> Either<&mut Self::Node,&mut Self::Leaf> {
        match self {
            FileLike::Dir(d) => Either::Left(d),
            FileLike::File(f) => Either::Right(f)
        }
    }
}
impl HasChildren<FileLike> for SourceDir {
    type ChildIter = std::vec::IntoIter<FileLike>;
    fn into_children(self) -> Self::ChildIter {
        Into::<Vec<_>>::into(self.children).into_iter()
    }
}
impl HasChildrenRef<FileLike> for SourceDir {
    type ChildRefIter<'a> = std::slice::Iter<'a,FileLike>;
    fn as_children(&self) -> Self::ChildRefIter<'_> {
        self.children.iter()
    }
}
impl HasChildrenMut<FileLike> for SourceDir {
    type ChildMutIter<'a> = std::slice::IterMut<'a,FileLike>;
    fn as_children_mut(&mut self) -> Self::ChildMutIter<'_> {
        self.children.iter_mut()
    }
}