use std::fmt::Display;
use std::path::Path;
use regex::*;
#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = "\\\\";
#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR: char = '/';


#[derive(Default, Clone, Debug)]
pub struct IgnoreSource(Option<Regex>);
impl Display for IgnoreSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(r) => write!(f, "{}", r),
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
    pub fn new(regex: &str, source_path: &Path) -> IgnoreSource {
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
        let s = format!("{}{}({})", p, PATH_SEPARATOR, s);
        Self(regex::Regex::new(&s).ok())
    }
    pub fn ignores(&self, p: &Path) -> bool {
        match &self.0 {
            Some(r) => r.is_match(p.to_str().unwrap()),
            None => false,
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for IgnoreSource {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match &self.0 {
                None => Option::<&str>::None.serialize(serializer),
                Some(r) => Some(r.as_str()).serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for IgnoreSource {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(IgnoreSource(
                match Option::<&'de str>::deserialize(deserializer)? {
                    None => None,
                    Some(s) => Some(
                        Regex::new(s).map_err(|_| serde::de::Error::custom("Invalid regex"))?,
                    ),
                },
            ))
        }
    }
}