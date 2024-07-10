use std::fs::Metadata;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use crate::building::buildstate::{BuildState, AllStates};
use crate::building::formats::{ShortId, SourceFormatId};
use crate::utils::ignore_regex::IgnoreSource;

pub trait FileLike<Data> {
    fn relative_path(&self) -> &str;
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct SourceFile {
    pub relative_path: String,
    pub format: SourceFormatId,
    pub build_state: BuildState
}

impl FileLike<AllStates> for SourceFile {
    fn relative_path(&self) -> &str {
        &self.relative_path
    }
}

#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub enum FSEntry<D,F:FileLike<D>> {
    Dir(Dir<D,F>),
    File(F)
}
impl<D,F:FileLike<D>> FSEntry<D,F> {
    pub fn relative_path(&self) -> &str {
        match self {
            FSEntry::Dir(d) => &d.relative_path,
            FSEntry::File(f) => f.relative_path()
        }
    }
}


#[derive(Debug)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct Dir<D,F:FileLike<D>> {
    pub relative_path: String,
    pub data: D,
    pub children: Vec<FSEntry<D,F>>
}

#[cfg(feature="serde")]
pub enum ParseError {
    DecodingError,
    FileError
}

#[cfg(feature="serde")]
pub enum SerializeError {
    EncodingError,
    IOError
}

#[cfg(feature = "serde")]
impl<D:for <'a> serde::Deserialize<'a>+serde::Serialize,F:FileLike<D>+for <'a> serde::Deserialize<'a>+serde::Serialize> Dir<D,F> {
    pub fn parse(file:&Path) -> Result<Vec<FSEntry<D,F>>,ParseError> {
        match std::fs::File::open(file) {
            Ok(mut f) => match bincode::serde::decode_from_std_read(&mut f,bincode::config::standard()) {
                Ok(v) => Ok(v),
                _ => Err(ParseError::DecodingError)
            },
            _ => Err(ParseError::FileError)
        }
    }
    #[cfg(feature="async")]
    pub async fn parse_async(file:&Path) -> Result<Vec<FSEntry<D,F>>,ParseError> {
        match tokio::fs::File::open(file).await {
            Ok(mut f) => {
                let s = tokio::fs::read(file).await.map_err(|_| ParseError::FileError)?;
                match bincode::serde::decode_from_slice(&s,bincode::config::standard()) {
                    Ok((v,_)) => Ok(v),
                    _ => Err(ParseError::DecodingError)
                }
            },
            _ => Err(ParseError::FileError)
        }
    }
    crate::asyncs!{!pub fn write_to(tree:&[FSEntry<D,F>],file:&Path) -> Result<(),SerializeError> {
        if let Some(p) = file.parent() {
            let _ = switch!(
                (std::fs::create_dir_all(p))
                (tokio::fs::create_dir_all(p))
            );
        };
        let mut f = match switch!(
            (std::fs::File::create(file))
            (tokio::fs::File::create(file))
        ) {
            Ok(f) => f,
            _ => return Err(SerializeError::IOError)
        };
        let ret = switch!(
            (bincode::serde::encode_into_std_write(tree,&mut f,bincode::config::standard()))
            (async {
                match bincode::serde::encode_to_vec(tree,bincode::config::standard()) {
                    Ok(v) => Ok(tokio::io::AsyncWriteExt::write_all(&mut f,&v).await),
                    Err(e) => Err(e)
                }
            })
        );
        if ret.is_err() {
            Err(SerializeError::EncodingError)
        } else { Ok(()) }
    }}
}

pub type SourceDir = Dir<AllStates,SourceFile>;
pub type SourceDirEntry = FSEntry<AllStates,SourceFile>;


#[derive(Clone,Debug)]
pub struct FileChange{
    pub previous: Option<BuildState>,
    pub new: BuildState
}

impl SourceDir {
    fn delete<O:FnMut(FileChange) + Copy>(&mut self,mut on_change:O) -> bool {
        self.children.retain_mut(|c| match c {
            SourceDirEntry::File(f) => {
                if f.build_state == BuildState::Deleted { false }
                else {
                    on_change(FileChange { previous: Some(f.build_state.clone()), new: BuildState::Deleted });
                    f.build_state != BuildState::New
                }
            }
            SourceDirEntry::Dir(d) => !d.delete(on_change),
        });
        self.children.is_empty()
    }

    crate::asyncs!{fn do_file(path:PathBuf, current:&mut StackItem, md:Metadata, changed:&mut bool,
               from_ext:impl Fn(&str) -> Option<SourceFormatId>,
               mut on_change:impl FnMut(FileChange) + Copy) {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let rel_path:String = if current.rel_path.is_empty() {filename.into()} else {format!("{}/{}",current.rel_path,filename).into() };
        if md.is_dir() {
            let old = if let Some(i) = current.old.iter().enumerate().rfind(|s| match s {
                (_,SourceDirEntry::Dir(s)) => s.relative_path == rel_path,
                _ => false
            }).map(|(i,_)| i) {
                if let SourceDirEntry::Dir(old) = current.old.remove(i) {old.children.into()} else {unreachable!()}
            } else {
                Vec::new()
            };
            current.stack.push(StackItem {
                old,
                dones: Vec::new(),
                path,
                parent: None,
                rel_path,
                state: AllStates::default(),
                stack:Vec::new()
            });
        } else {
            let format = match path.extension() {
                Some(ext) => match from_ext(ext.to_str().unwrap()) {
                    Some(f) => f,
                    _ => return
                }
                _ => return
            };
            if let Some(i) = current.old.iter().enumerate().rfind(|s| match s {
                (_,SourceDirEntry::File(s)) => s.relative_path == rel_path,
                _ => false
            }).map(|(i,_)| i) {
                let mut old = if let SourceDirEntry::File(f) = current.old.remove(i) {f} else {unreachable!()};
                *changed = switch!(
                    (old.build_state.update(&path,md,on_change))
                    (old.build_state.update_async(&path,md,on_change))
                ) || *changed;
                current.state.merge(&old.build_state,format);
                current.dones.push(SourceDirEntry::File(old))
            } else {
                *changed = true;
                on_change(FileChange{ previous: None, new: BuildState::New });
                current.state.merge(&BuildState::New,format);
                current.dones.push(SourceDirEntry::File(SourceFile{
                    relative_path:rel_path,
                    build_state:BuildState::New,
                    format
                }));
            }
        }
    }}

    crate::asyncs!{!pub fn update<[O:FnMut(FileChange) + Copy]>(path:&Path, old_ref:&mut Vec<SourceDirEntry>, exts:&[(&str, SourceFormatId)], ignore:&IgnoreSource, mut on_change:O) -> (bool,AllStates) {
        let mut top_state: AllStates = Default::default();
        if ignore.ignores(path) {
            tracing::trace!("Ignoring {} because of {}",path.display(),ignore);
            return (false,top_state)
        }
        let mut curr = match read_dir!(path) {
            Ok(d) => d,
            _ => {
                tracing::warn!(target:"archives","Could not read directory {}",path.display());
                return (false,top_state)
            }
        };
        let mut old = std::mem::take(old_ref);
        old.reverse();
        let mut current = StackItem {old,
            dones:Vec::new(),
            path:path.into(),
            parent:None,
            rel_path: "".into(),
            state:top_state.clone(),
            stack: Vec::new()
        };
        let mut top = true;
        let mut changed = false;
        let mut pstack = Vec::new();
        let from_ext = |s:&str| exts.iter().find(|(e,_)| *e == s).map(|(_,f)| *f);
        'top:loop {
            let d = match next_file!(curr) {
                Some(Ok(d)) => {
                    let path = d.path();
                    if ignore.ignores(&path) {
                        tracing::trace!("Ignoring {} because of {}",path.display(),ignore);
                        continue
                    }
                    let md = match wait!(d.metadata()) {
                        Ok(d) => d,
                        _ => {
                            tracing::warn!(target:"archives","Could not read metadata of file {}",path.display());
                            continue
                        }
                    };
                    switch!(
                        (Self::do_file(path,&mut current,md,&mut changed,from_ext,on_change))
                        (Self::do_file_async(path,&mut current,md,&mut changed,from_ext,on_change))
                    );
                }
                None  => {
                    for mut o in std::mem::take(&mut current.old).into_iter() {
                        match &mut o {
                            SourceDirEntry::File(f) if matches!(f.build_state,BuildState::Stale {..} | BuildState::UpToDate {..}) => {
                                on_change(FileChange{ previous: Some(f.build_state.clone()), new: BuildState::Deleted });
                                f.build_state = BuildState::Deleted;
                                current.state.merge(&BuildState::Deleted,f.format)
                            },
                            SourceDirEntry::Dir(l) => if l.delete(on_change) { continue },
                            SourceDirEntry::File(f) => {
                                current.state.merge(&f.build_state,f.format);
                                continue
                            }
                        }
                        changed = true;
                        current.dones.push(o);
                    }
                    loop {
                        while current.stack.is_empty() {
                            if let Some(mut next) = pstack.pop() {
                                std::mem::swap(&mut next,&mut current);
                                let state = next.state.clone();
                                if let Some(i) = next.parent {
                                    let entry = SourceDirEntry::Dir(SourceDir{
                                        relative_path:next.rel_path,
                                        children:next.dones.into(),
                                        data:next.state
                                    });
                                    if i == pstack.len() {
                                        current.state.merge_cum(&state);
                                        current.dones.push(entry);
                                    } else {
                                        pstack[i].state.merge_cum(&state);
                                        pstack[i].dones.push(entry);
                                    }
                                } else if top {
                                    top_state.merge_cum(&state);
                                    *old_ref = next.dones.into();
                                    top = false;
                                } else {
                                    let entry = SourceDirEntry::Dir(SourceDir{
                                        relative_path:next.rel_path,
                                        children:next.dones.into(),
                                        data:next.state
                                    });
                                    top_state.merge_cum(&state);
                                    old_ref.push(entry);
                                }
                            } else {
                                top_state.merge_cum(&current.state);
                                if current.rel_path == "" {
                                    old_ref.extend(current.dones);
                                } else {
                                    let entry = SourceDirEntry::Dir(SourceDir{
                                        relative_path:current.rel_path,
                                        children:current.dones.into(),
                                        data:current.state
                                    });
                                    old_ref.push(entry);
                                }
                                break 'top
                            }
                        }
                        let mut next = current.stack.pop().unwrap();
                        curr = match read_dir!(&next.path) {
                            Ok(d) => d,
                            _ => {
                                tracing::warn!(target:"archives","Could not read directory {}",path.display());
                                continue
                            }
                        };
                        next.parent = Some(pstack.len());
                        std::mem::swap(&mut current,&mut next);
                        pstack.push(next);
                        break
                    }
                }
                _ => {
                    tracing::warn!(target:"archives","Error when reading directory {}",path.display());
                }
            };
        }
        (changed,top_state)
    }}
}

struct StackItem {
    old: Vec<SourceDirEntry>,
    dones: Vec<SourceDirEntry>,
    parent: Option<usize>,
    path:PathBuf,
    rel_path: String,
    state: AllStates,
    stack: Vec<StackItem>
}

pub struct DirIter<'a,D,F:FileLike<D>> {
    current: std::slice::Iter<'a,FSEntry<D,F>>,
    stack: Vec<std::slice::Iter<'a,FSEntry<D,F>>>
}
#[cfg(feature="rayon")]
impl<'a,D,F:FileLike<D>> spliter::Spliterator for DirIter<'a,D,F> {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 {return None}
        let split = self.stack.len() / 2;
        let mut new_stack = self.stack.split_off(split);
        let mut new_curr = new_stack.pop().unwrap();
        Some(DirIter {
            current: new_curr,
            stack: new_stack
        })
    }
}

impl<'a,D,F:FileLike<D>> Iterator for DirIter<'a,D,F> {
    type Item = &'a FSEntry<D,F>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(c) = self.current.next() {
                match c {
                    FSEntry::Dir(d) => self.stack.push(d.children.iter()),
                    _ => {}
                }
                return Some(c)
            } else {
                if let Some(mut s) = self.stack.pop() {
                    self.current = s;
                } else {
                    return None
                }
            }
        }
    }
}

pub trait DirLike<D,F:FileLike<D>> {
    fn find_entry<S:AsRef<str>>(&self,s:S) -> Option<&FSEntry<D,F>>;
    fn dir_iter<'a>(&'a self) -> DirIter<'a,D,F>;
}
impl<'a,D,F:FileLike<D>> DirLike<D,F> for &'a [FSEntry<D,F>] {
    fn find_entry<S:AsRef<str>>(&self,s:S) -> Option<&FSEntry<D,F>> {
        let _s = s.as_ref();
        let mut ls = *self;
        let mut ret = None;
        let mut curr = None;
        for step in _s.split('/') {
            if curr.is_none() { curr = Some(step.to_string())} else { curr = Some(format!("{}/{}",curr.as_deref().unwrap(),step)) }
            let elem = ls.iter().find(|e| e.relative_path() == curr.as_deref().unwrap())?;
            ret = Some(elem);
            ls = match elem {
                FSEntry::Dir(Dir { children, .. }) => children.as_slice(),
                _ => &[]
            }
        }
        ret
    }

    fn dir_iter<'b>(&'b self) -> DirIter<'b,D,F> {
        DirIter { current: self.iter(), stack: Vec::new() }
    }
}