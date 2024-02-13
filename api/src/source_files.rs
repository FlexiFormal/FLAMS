#[cfg(feature="fs")]
use std::path::Path;
use std::path::PathBuf;
#[cfg(feature="fs")]
use crate::utils::problems::ProblemHandler;
#[cfg(feature="fs")]
use crate::archives::IgnoreSource;
use either::Either;
use spliter::ParallelSpliterator;
use crate::formats::FormatId;
use crate::{Seq, Str};
use crate::utils::iter::{HasChildren, HasChildrenMut, HasChildrenRef, LeafIterator, TreeLike, TreeMutLike, TreeRefLike};

#[derive(Debug)]
#[cfg_attr(feature = "serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub enum FileLike {
    Dir(SourceDir),
    File(SourceFile)
}
impl FileLike {
    fn rel_path(&self) -> &str {
        match self {
            FileLike::Dir(d) => &d.rel_path,
            FileLike::File(f) => &f.rel_path
        }
    }
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

#[cfg(feature = "pariter")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "pariter")]
use spliter::ParSpliter;

#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct SourceDir{pub rel_path:Str,pub children:Seq<FileLike>}
impl SourceDir {
    #[cfg(feature = "pariter")]
    pub fn par_iter(&self) -> ParSpliter<LeafIterator<&FileLike>> { self.iter_leafs().par_split().into_par_iter() }
    pub fn iter(&self) -> LeafIterator<&FileLike> { self.iter_leafs() }
    pub fn iter_mut(&mut self) -> LeafIterator<&mut FileLike> { self.iter_leafs() }
    #[cfg(all(feature="bincode",feature="fs"))]
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
    #[cfg(all(feature="bincode",feature="fs"))]
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
    #[cfg(feature="fs")]
    pub fn update<F:AsRef<Path>,P:ProblemHandler>(in_dir:F,rel_path:&str,old:&mut Seq<FileLike>,handler:&P,ignore:&IgnoreSource,from_ext:&impl Fn(&str) -> Option<FormatId>) {
        use tracing::trace;
        let mut dones_v = Vec::new();
        let mut todos = Vec::new();
        let mut oldv: Vec<_> = std::mem::take(old).into();
        let path = in_dir.as_ref();
        if ignore.ignores(path) {
            trace!("Ignoring {} because of {}",path.display(),ignore);
            return
        }
        oldv.reverse();
        let curr = match std::fs::read_dir(path) {
            Ok(d) => d,
            _ => {
                handler.add("archives",format!("Could not read directory {}",path.display()));
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
            //std::thread::sleep(std::time::Duration::from_secs_f32(0.001));
            if ignore.ignores(&path) {
                trace!("Ignoring {} because of {}",path.display(),ignore);
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
                let rel_path:Str = format!("{}/{}",rel_path,path.file_name().unwrap().to_str().unwrap()).into();
                let old = oldv.iter().enumerate().rfind(|s| match s {
                    (_,FileLike::Dir(s)) => s.rel_path == rel_path,
                    _ => false
                }).map(|(i,_)| i);
                if let Some(i) = old {
                    let old = oldv.remove(i);
                    dones_v.push(old);
                } else {
                    dones_v.push(FileLike::Dir(SourceDir{
                        rel_path:rel_path.clone(),
                        children:Vec::new().into()
                    }));
                }
                todos.push((path,rel_path,dones_v.len()-1));
            } else {
                let rel_path:Str = format!("{}/{}",rel_path,path.file_name().unwrap().to_str().unwrap()).into();
                let format = match path.extension() {
                    Some(ext) => match from_ext(ext.to_str().unwrap()) {
                        Some(f) => f,
                        _ => continue
                    }
                    _ => continue
                };
                let old = oldv.iter().enumerate().rfind(|s| match s {
                    (_,FileLike::File(s)) => s.rel_path == rel_path,
                    _ => false
                }).map(|(i,_)| i);
                if let Some(i) = old {
                    let mut old = oldv.remove(i).into_either().unwrap_right();
                    if let BuildState::UpToDate { last_built,md5 } = &mut old.state {
                        let changed = md.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        if changed > *last_built {
                            old.state = BuildState::Stale { last_built: changed,md5:std::mem::take(md5) };
                        }
                    }
                    dones_v.push(FileLike::File(old));
                } else {
                    dones_v.push(FileLike::File(SourceFile{
                        rel_path,
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
        for (p,r,i) in todos {
            Self::update(&p,&r,&mut old[i].as_either_mut().unwrap_left().children,handler,ignore,from_ext);
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub struct SourceFile{pub rel_path:Str,pub state:BuildState,pub format: FormatId }
impl SourceFile {
    pub fn path_in_archive(&self,archive_path:&Path) -> PathBuf {
        archive_path.join("source").join(&self.rel_path[1..])
    }
}


#[derive(Debug,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="bincode",derive(bincode::Encode,bincode::Decode))]
pub enum BuildState {
    Deleted,
    New,
    Stale{last_built:u64,md5:Str},
    UpToDate{last_built:u64,md5:Str}
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