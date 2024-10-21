use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Language {
    #[default]
    English,
    German,
    French,
    Romanian,
    Arabic,
    Bulgarian,
    Russian,
    Finnish,
    Turkish,
    Slovenian,
}
impl Language {
    pub const SEPARATOR: char = 'l';
    #[inline]
    fn check(s: &str) -> Self {
        match &s[s.len() - 3..] {
            ".en" => Self::English,
            ".de" => Self::German,
            ".fr" => Self::French,
            ".ro" => Self::Romanian,
            ".ar" => Self::Arabic,
            ".bg" => Self::Bulgarian,
            ".ru" => Self::Russian,
            ".fi" => Self::Finnish,
            ".tr" => Self::Turkish,
            ".sl" => Self::Slovenian,
            _ => Self::default(),
        }
    }
    #[must_use]
    pub fn from_rel_path(mut s: &str) -> Self {
        s = s.strip_suffix(".tex").unwrap_or(s);
        Self::check(s)
    }
    pub fn from_file(path: &Path) -> Self {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map_or_else(Self::default, Self::check)
    }
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}
impl From<Language> for &'static str {
    fn from(value: Language) -> Self {
        match value {
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Romanian => "ro",
            Language::Arabic => "ar",
            Language::Bulgarian => "bg",
            Language::Russian => "ru",
            Language::Finnish => "fi",
            Language::Turkish => "tr",
            Language::Slovenian => "sl",
        }
    }
}
impl FromStr for Language {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "en" => Ok(Self::English),
            "de" => Ok(Self::German),
            "fr" => Ok(Self::French),
            "ro" => Ok(Self::Romanian),
            "ar" => Ok(Self::Arabic),
            "bg" => Ok(Self::Bulgarian),
            "ru" => Ok(Self::Russian),
            "fi" => Ok(Self::Finnish),
            "tr" => Ok(Self::Turkish),
            "sl" => Ok(Self::Slovenian),
            _ => Err(()),
        }
    }
}
