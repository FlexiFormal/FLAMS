use regex::Regex;
use std::fmt::Display;
use std::path::Path;
#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = "\\\\";
#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR: char = '/';

#[derive(Default, Clone, Debug)]
pub struct IgnoreSource(Option<Regex>);
impl Display for IgnoreSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(r) => Display::fmt(r, f),
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
    pub fn new(regex: &str, source_path: &Path) -> Self {
        if regex.is_empty() {
            return Self::default();
        }
        #[cfg(target_os = "windows")]
        let regex = regex.replace('/', PATH_SEPARATOR);
        let s = regex.replace('.', r"\.").replace('*', ".*"); //.replace('/',r"\/");
        let s = s
            .split('|')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("|");
        let p = source_path.display(); //path.to_str().unwrap().replace('/',r"\/");
        #[cfg(target_os = "windows")]
        let p = p.to_string().replace('\\', PATH_SEPARATOR);
        let s = format!("{p}({PATH_SEPARATOR})?({s})");
        Self(regex::Regex::new(&s).ok())
    }
    pub fn ignores(&self, p: &Path) -> bool {
        let Some(p) = p.to_str() else { return false };
        self.0.as_ref().map_or(false, |r| r.is_match(p))
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
        let source = Path::new("/home/jazzpirate/work/MathHub/sTeX/Documentation/source");
        let ignore = get_ignore(source);
        tracing::info!("Ignore: {ignore}");

        let path = Path::new("/home/jazzpirate/work/MathHub/sTeX/Documentation/source/tutorial/solution/preamble.tex");

        assert!(ignore.ignores(path));
        let path = Path::new("/home/jazzpirate/work/MathHub/sTeX/Documentation/source/tutorial/math/assertions.en.tex");
        assert!(!ignore.ignores(path));
    }
    
}
