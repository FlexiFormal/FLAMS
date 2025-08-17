/*
use std::{
    collections::VecDeque,
    fs::ReadDir,
    path::{Path, PathBuf},
};

pub struct ManifestIterator {
    stack: VecDeque<(PathBuf, ReadDir)>,
    curr: Option<ReadDir>,
    //curr_path: PathBuf,
}
impl ManifestIterator {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            stack: VecDeque::new(),
            curr: path.read_dir().ok(),
        }
    }
}
impl Iterator for ManifestIterator {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = match self.curr.as_mut().and_then(ReadDir::next) {
                None => {
                    if let Some((_, next)) = self.stack.pop_front() {
                        self.curr = Some(next);
                        //self.curr_path = path;
                        continue;
                    }
                    return None;
                }
                Some(Ok(d)) => d,
                _ => continue,
            };
            let Ok(md) = entry.metadata() else { continue };
            if !md.is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            if file_name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            if file_name.eq_ignore_ascii_case("meta-inf")
                && let Some(manifest) = find_manifest(&path)
            {
                // SAFETY: path is a child of self.curr
                let parent = unsafe { path.parent().unwrap_unchecked() };
                self.stack.retain(|(p, _)| !p.starts_with(parent));
                self.curr = None;
                return Some(manifest);
            }
            if let Ok(next) = path.read_dir() {
                self.stack.push_back((path, next));
            }
        }
    }
}
 */

use std::path::{Path, PathBuf};

pub(super) struct ManifestIterator {
    stack: Vec<Vec<PathBuf>>,
    curr: Option<std::fs::ReadDir>,
}

impl ManifestIterator {
    pub fn new(path: &Path) -> Self {
        Self {
            stack: vec![vec![]],
            curr: std::fs::read_dir(path)
                .map_err(|_| {
                    tracing::warn!("Could not read directory {}", path.display());
                })
                .ok(),
        }
    }

    fn next(curr: &mut Option<std::fs::ReadDir>, stack: &mut Vec<Vec<PathBuf>>) -> Option<PathBuf> {
        loop {
            let d = match curr.as_mut().and_then(std::fs::ReadDir::next) {
                None => {
                    if Self::next_dir(stack, curr) {
                        continue;
                    }
                    return None;
                }
                Some(Ok(d)) => d,
                _ => continue,
            };
            let Ok(md) = d.metadata() else { continue };
            let path = d.path();

            //let _span = tracing::debug_span!(target:"archives","checking","{}",path.display()).entered();
            if md.is_dir() {
                if d.file_name().to_str().is_none_or(|s| s.starts_with('.')) {
                    continue;
                } else if d.file_name().eq_ignore_ascii_case("meta-inf")
                    && let Some(path) = find_manifest(&path)
                {
                    stack.pop();
                    if !Self::next_dir(stack, curr) {
                        *curr = None;
                    }
                    return Some(path);
                }
                stack
                    .last_mut()
                    .unwrap_or_else(|| unreachable!())
                    .push(path);
            }
        }
    }

    fn next_dir(stack: &mut Vec<Vec<PathBuf>>, curr: &mut Option<std::fs::ReadDir>) -> bool {
        loop {
            match stack.last_mut() {
                None => return false,
                Some(s) => match s.pop() {
                    Some(e) => {
                        *curr = if let Ok(rd) = e.read_dir() {
                            Some(rd)
                        } else {
                            tracing::warn!(target:"archives","Could not read directory {}", e.display());
                            return false;
                        };
                        stack.push(Vec::new());
                        return true;
                    }
                    None => {
                        stack.pop();
                    }
                },
            }
        }
    }
}

impl Iterator for ManifestIterator {
    type Item = PathBuf;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Self::next(&mut self.curr, &mut self.stack)
    }
}

impl spliter::Spliterator for ManifestIterator {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 || self.stack[0].len() < 2 {
            return None;
        }
        let stacksplit = self.stack[0].len() / 2;
        let mut rightstack = self.stack[0].split_off(stacksplit);
        std::mem::swap(&mut self.stack[0], &mut rightstack);
        loop {
            match rightstack.pop() {
                None => return None,
                Some(e) => {
                    if let Ok(rd) = std::fs::read_dir(&e) {
                        return Some(Self {
                            curr: Some(rd),
                            stack: vec![rightstack, Vec::new()],
                        });
                    }
                }
            }
        }
    }
}

fn find_manifest(metainf: &Path) -> Option<PathBuf> {
    tracing::trace!("Checking manifest {}", metainf.display());
    if let Ok(rd) = metainf.read_dir() {
        for d in rd {
            let Ok(manifest) = d else {
                tracing::warn!("Could not read directory {}", metainf.display());
                continue;
            };
            if !manifest.file_name().eq_ignore_ascii_case("manifest.mf") {
                continue;
            }
            let path = manifest.path();
            if !path.is_file() {
                continue;
            }
            return Some(path);
        }
        tracing::trace!("not found");
    } else {
        tracing::warn!("Could not read directory {}", metainf.display());
    }
    None
}
