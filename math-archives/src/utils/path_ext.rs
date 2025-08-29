use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use ftml_uris::{ArchiveId, UriPath};

use crate::utils::errors::FileError;

/// A relative path normalized and always displayed with `/` as component separator
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct RelPath<'p>(&'p Path);

impl std::ops::Deref for RelPath<'_> {
    type Target = Path;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl AsRef<Path> for RelPath<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0
    }
}

impl std::fmt::Display for RelPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(target_os = "windows")]
        {
            let mut first = true;
            for c in self.0.components() {
                if let std::path::Component::Normal(s) = c {
                    if !first {
                        f.write_char('/')?;
                    }
                    s.display().fmt(f)?;
                    first = false;
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.0.as_os_str().display().fmt(f)
        }
    }
}
impl<'s> RelPath<'s> {
    /// # Errors
    pub fn parse<T: std::str::FromStr>(&self) -> Result<T, T::Err> {
        #[cfg(target_os = "windows")]
        {
            self.to_string().parse()
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.0.as_os_str().to_str().unwrap_or("").parse()
        }
    }

    #[must_use]
    pub fn steps(self) -> impl DoubleEndedIterator<Item = &'s str> {
        self.0.components().filter_map(|s| match s {
            Component::Normal(n) => n.to_str(),
            _ => None,
        })
    }

    #[must_use]
    pub fn split_last(self) -> Option<(Self, &'s str)> {
        let last = self.steps().last()?;
        let first = self.0.parent()?;
        Some((Self(first), last))
    }

    #[must_use]
    pub fn from_id(id: &'s ArchiveId) -> Self {
        Self(Path::new(id.as_ref()))
    }

    #[must_use]
    pub fn from_path(path: &'s UriPath) -> Self {
        Self(Path::new(path.as_ref()))
    }
    #[must_use]
    pub fn new(path: &'s str) -> Self {
        Self(Path::new(path))
    }
}

/*
/// A relative path normalized and always displayed with `/` as component separator
#[derive(Clone, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct RelPathBuf(PathBuf);
impl RelPathBuf {
    #[inline]
    #[must_use]
    pub fn borrow(&self) -> RelPath<'_> {
        RelPath(&self.0)
    }
}
impl RelPath<'_> {
    #[inline]
    #[must_use]
    pub fn to_buf(&self) -> RelPathBuf {
        RelPathBuf(self.0.to_path_buf())
    }
}

impl std::fmt::Display for RelPathBuf {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl std::ops::Deref for RelPathBuf {
    type Target = Path;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Path> for RelPathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
 */

impl PartialEq<str> for RelPath<'_> {
    fn eq(&self, o: &str) -> bool {
        let mut others = o.split('/');
        for s in self.0.components() {
            let std::path::Component::Normal(s) = s else {
                continue;
            };
            let Some(o) = others.next() else { return false };
            if s.to_str().is_none_or(|s| s != o) {
                return false;
            }
        }
        others.next().is_none()
    }
}

pub trait PathExt {
    const PATH_SEPARATOR: char;
    //fn as_slash_str(&self) -> Cow<'_, str>;
    fn relative_to<'s, P: AsRef<std::path::Path>>(&'s self, ancestor: &P) -> Option<RelPath<'s>>;
    fn same_fs_as<P: AsRef<std::path::Path>>(&self, other: &P) -> bool;
    /// ### Errors
    fn rename_safe<P: AsRef<std::path::Path>>(&self, target: &P) -> Result<(), FileError>;
    /// ### Errors
    fn copy_dir_all<P: AsRef<std::path::Path>>(&self, target: &P) -> Result<(), FileError>;
    fn join_uri_path(&self, path: &UriPath) -> PathBuf;
    fn as_slash_str(&self) -> Cow<'_, str>;
}
impl<T: AsRef<std::path::Path>> PathExt for T {
    #[cfg(target_os = "windows")]
    const PATH_SEPARATOR: char = '\\';
    #[cfg(not(target_os = "windows"))]
    const PATH_SEPARATOR: char = '/';
    fn relative_to<'s, P: AsRef<std::path::Path>>(&'s self, ancestor: &P) -> Option<RelPath<'s>> {
        self.as_ref()
            .strip_prefix(ancestor.as_ref())
            .ok()
            .map(RelPath)
    }
    fn join_uri_path(&self, path: &UriPath) -> PathBuf {
        let mut steps = path.steps();
        // SAFETY: UriPaths are non-empty
        let ret = self
            .as_ref()
            .join(unsafe { steps.next().unwrap_unchecked() });
        steps.fold(ret, |p, n| p.join(n))
    }

    fn as_slash_str(&self) -> Cow<'_, str> {
        // SAFTEY: don't run this on weird OSes with entirely nonstandard filepaths
        if cfg!(windows) {
            Cow::Owned(unsafe {
                self.as_ref()
                    .as_os_str()
                    .to_str()
                    .unwrap_unchecked()
                    .replace('\\', "/")
            })
        } else {
            Cow::Borrowed(unsafe { self.as_ref().as_os_str().to_str().unwrap_unchecked() })
        }
    }

    #[cfg(target_os = "windows")]
    fn same_fs_as<P: AsRef<std::path::Path>>(&self, other: &P) -> bool {
        let Some(p1) = self
            .as_ref()
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
        else {
            return false;
        };
        let Some(p2) = other
            .as_ref()
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
        else {
            return false;
        };
        p1 == p2
    }
    #[cfg(target_arch = "wasm32")]
    fn same_fs_as<P: AsRef<std::path::Path>>(&self, other: &P) -> bool {
        impossible!()
    }

    #[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
    fn same_fs_as<P: AsRef<std::path::Path>>(&self, other: &P) -> bool {
        use std::os::unix::fs::MetadataExt;
        fn existent_parent(p: &std::path::Path) -> &std::path::Path {
            if p.exists() {
                return p;
            }
            existent_parent(p.parent().unwrap_or_else(|| unreachable!()))
        }
        let p1 = existent_parent(self.as_ref());
        let p2 = existent_parent(other.as_ref());
        let md1 = p1.metadata().unwrap_or_else(|_| unreachable!());
        let md2 = p2.metadata().unwrap_or_else(|_| unreachable!());
        md1.dev() == md2.dev()
    }

    fn rename_safe<P: AsRef<std::path::Path>>(&self, target: &P) -> Result<(), FileError> {
        if self.same_fs_as(target) {
            std::fs::rename(self.as_ref(), target.as_ref())
                .map_err(|e| FileError::Rename(self.as_ref().to_path_buf(), e))
        } else {
            self.copy_dir_all(target)
        }
    }

    /// #### Errors
    fn copy_dir_all<P: AsRef<std::path::Path>>(&self, target: &P) -> Result<(), FileError> {
        let dst = target.as_ref();
        let src = self.as_ref();
        std::fs::create_dir_all(dst).map_err(|e| FileError::Creation(dst.to_path_buf(), e))?;
        for entry in src
            .read_dir()
            .map_err(|e| FileError::ReadDir(src.to_path_buf(), e))?
        {
            let entry = entry.map_err(|e| FileError::ReadEntry(src.to_path_buf(), e))?;
            let ty = entry
                .file_type()
                .map_err(|e| FileError::FileType(entry.path(), e))?;
            let target = dst.join(entry.file_name());
            if ty.is_dir() {
                entry.path().copy_dir_all(&target)?;
            } else {
                let md = entry
                    .metadata()
                    .map_err(|e| FileError::MetaData(entry.path(), e))?;
                std::fs::copy(entry.path(), &target).map_err(|e| FileError::Copying {
                    from: entry.path(),
                    to: target.clone(),
                    error: e,
                })?;
                let mtime = filetime::FileTime::from_last_modification_time(&md);
                filetime::set_file_mtime(&target, mtime)
                    .map_err(|e| FileError::SetFileModTime(entry.path(), e))?;
            }
        }
        Ok(())
    }
}
