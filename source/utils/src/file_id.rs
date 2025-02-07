use std::{num::NonZeroUsize, path::PathBuf};

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct FileId(NonZeroUsize);

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct FileOrUrl {
  inner: triomphe::Arc<FileOrUrlI>
}

#[derive(PartialEq,Eq,Hash)]
enum FileOrUrlI {
  File(PathBuf),
  Url(url::Url)
}

struct FileOrUrlStore {
  inner:Vec<FileOrUrl>
}
impl FileOrUrlStore {
  fn get(&self,id:FileId) -> FileOrUrl {
    self.inner[id.0.get() - 1].clone()
  }
}