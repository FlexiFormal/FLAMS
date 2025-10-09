use regex::Regex;
use std::path::{Path, PathBuf};

use crate::utils::path_ext::PathExt;

/// A regular expression (using [`Regex`]) used to ignore source files specified as
/// relative to the source directory of some [`MathArchive`](crate::MathArchive)
///
/// # Example
/// ```
/// # use flams_math_archives::utils::ignore_source::IgnoreSource;
/// # use std::path::Path;
/// // The source directory of an archive:
/// let source_path = Path::new("/home/user/MathHub/FTML/doc/source");
/// let ignore = IgnoreSource::new("*/code/*|*/tikz/*|*/tutorial/solution/*", source_path);
/// let path = Path::new("/home/user/MathHub/FTML/doc/source/tutorial/solution/preamble.tex");
/// assert!(ignore.ignores(path));
/// let path = Path::new("/home/user/MathHub/FTML/doc/source/tutorial/math/assertions.en.tex");
/// assert!(!ignore.ignores(path));
/// ```
#[derive(Default, Clone, Debug)]
pub struct IgnoreSource(Option<Regex>);
impl std::fmt::Display for IgnoreSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(r) => r.fmt(f),
            None => write!(f, "(None)"),
        }
    }
}

impl PartialEq for IgnoreSource {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a.as_str() == b.as_str(),
            (None, None) => true,
            _ => false,
        }
    }
}

impl IgnoreSource {
    #[must_use]
    pub fn new(regex: &str, source_path: &Path) -> Self {
        if regex.is_empty() {
            return Self::default();
        }
        //#[cfg(target_os = "windows")]
        //let regex = regex.replace('/', PathBuf::PATH_SEPARATOR);
        let s = regex.replace('.', r"\.").replace('*', ".*");
        let s = s
            .split('|')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("|");
        let p = source_path.display();
        #[cfg(target_os = "windows")]
        let p = p.to_string()[3..].replace(PathBuf::PATH_SEPARATOR,"/");
        let s = format!("{p}(/)?({s})");
        Self(Regex::new(&s).ok())
    }

    #[must_use]
    pub fn ignores(&self, p: &Path) -> bool {
        let Some(p) = p.to_str() else { return false };
        if cfg!(target_os = "windows") {
            let p = p[3..].replace(PathBuf::PATH_SEPARATOR,"/");
            self.0.as_ref().is_some_and(|r| r.is_match(&p))
        } else {
            self.0.as_ref().is_some_and(|r| r.is_match(p))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_ignore(source: &Path) -> IgnoreSource {
        IgnoreSource::new("*/code/*|*/tikz/*|*/tutorial/solution/*", source)
    }

    #[test]
    fn ignore_test() {
        let source = Path::new(if cfg!(target_os = "windows") {"C:\\home\\user\\MathHub\\sTeX\\Documentation\\source"} else {"/home/user/MathHub/sTeX/Documentation/source"});
        let ignore = get_ignore(source);
        let path = Path::new(
            if cfg!(target_os = "windows") {
                "c:\\home\\user\\MathHub\\sTeX\\Documentation\\source\\tutorial/solution\\preamble.tex"
            } else {"/home/user/MathHub/sTeX/Documentation/source/tutorial/solution/preamble.tex"},
        );
        assert!(ignore.ignores(path),"{ignore} matches {}",path.display());
        let path = Path::new(
            if cfg!(target_os = "windows") {
                "C:\\home\\user\\MathHub\\sTeX\\Documentation\\source\\tutorial\\math\\assertions.en.tex"
            } else {"/home/user/MathHub/sTeX/Documentation/source/tutorial/math/assertions.en.tex"},
        );
        assert!(!ignore.ignores(path),"{ignore} matches {}",path.display());
    }
}
