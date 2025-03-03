use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[non_exhaustive]
pub enum Language {
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "en"))]
    English,
    #[cfg_attr(feature = "serde", serde(rename = "de"))]
    German,
    #[cfg_attr(feature = "serde", serde(rename = "fr"))]
    French,
    #[cfg_attr(feature = "serde", serde(rename = "ro"))]
    Romanian,
    #[cfg_attr(feature = "serde", serde(rename = "ar"))]
    Arabic,
    #[cfg_attr(feature = "serde", serde(rename = "bg"))]
    Bulgarian,
    #[cfg_attr(feature = "serde", serde(rename = "ru"))]
    Russian,
    #[cfg_attr(feature = "serde", serde(rename = "fi"))]
    Finnish,
    #[cfg_attr(feature = "serde", serde(rename = "tr"))]
    Turkish,
    #[cfg_attr(feature = "serde", serde(rename = "sl"))]
    Slovenian,
}
impl Language {
    pub const SEPARATOR: char = 'l';
    #[inline]
    fn check(s: &str) -> Self {
        if s.len() < 3 { return Self::default()}
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

    #[must_use]
    pub const fn flag_unicode(&self) -> &'static str {
        match self {
            Self::English => "🇬🇧",
            Self::German => "🇩🇪",
            Self::French => "🇫🇷",
            Self::Romanian => "🇷🇴",
            Self::Arabic => "🇦🇪",
            Self::Bulgarian => "🇧🇬",
            Self::Russian => "🇷🇺",
            Self::Finnish => "🇫🇮",
            Self::Turkish => "🇹🇷",
            Self::Slovenian => "🇸🇮",
        }
    }

    #[must_use]
    pub const fn flag_svg(&self) -> &'static str {
        // https://flagicons.lipis.dev/
        match self {
            Self::English => r##"<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-gb" viewBox="0 0 640 480">
  <path fill="#012169" d="M0 0h640v480H0z"/>
  <path fill="#FFF" d="m75 0 244 181L562 0h78v62L400 241l240 178v61h-80L320 301 81 480H0v-60l239-178L0 64V0z"/>
  <path fill="#C8102E" d="m424 281 216 159v40L369 281zm-184 20 6 35L54 480H0zM640 0v3L391 191l2-44L590 0zM0 0l239 176h-60L0 42z"/>
  <path fill="#FFF" d="M241 0v480h160V0zM0 160v160h640V160z"/>
  <path fill="#C8102E" d="M0 193v96h640v-96zM273 0v480h96V0z"/>
</svg>"##,
            Self::German => r##"<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-de" viewBox="0 0 640 480">
  <path fill="#fc0" d="M0 320h640v160H0z"/>
  <path fill="#000001" d="M0 0h640v160H0z"/>
  <path fill="red" d="M0 160h640v160H0z"/>
</svg>
"##,
            Self::French => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-fr" viewBox="0 0 640 480">
  <path fill="#fff" d="M0 0h640v480H0z"/>
  <path fill="#000091" d="M0 0h213.3v480H0z"/>
  <path fill="#e1000f" d="M426.7 0H640v480H426.7z"/>
</svg>
"##,
            Self::Romanian => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-ro" viewBox="0 0 640 480">
  <g fill-rule="evenodd" stroke-width="1pt">
    <path fill="#00319c" d="M0 0h213.3v480H0z"/>
    <path fill="#ffde00" d="M213.3 0h213.4v480H213.3z"/>
    <path fill="#de2110" d="M426.7 0H640v480H426.7z"/>
  </g>
</svg>
"##,
            Self::Arabic => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-ae" viewBox="0 0 640 480">
  <path fill="#00732f" d="M0 0h640v160H0z"/>
  <path fill="#fff" d="M0 160h640v160H0z"/>
  <path fill="#000001" d="M0 320h640v160H0z"/>
  <path fill="red" d="M0 0h220v480H0z"/>
</svg>
"##,
            Self::Bulgarian => r##"<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-bg" viewBox="0 0 640 480">
  <path fill="#fff" d="M0 0h640v160H0z"/>
  <path fill="#00966e" d="M0 160h640v160H0z"/>
  <path fill="#d62612" d="M0 320h640v160H0z"/>
</svg>"##,
            Self::Russian => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-ru" viewBox="0 0 640 480">
  <path fill="#fff" d="M0 0h640v160H0z"/>
  <path fill="#0039a6" d="M0 160h640v160H0z"/>
  <path fill="#d52b1e" d="M0 320h640v160H0z"/>
</svg>"##,
            Self::Finnish => r##"<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-fi" viewBox="0 0 640 480">
  <path fill="#fff" d="M0 0h640v480H0z"/>
  <path fill="#002f6c" d="M0 174.5h640v131H0z"/>
  <path fill="#002f6c" d="M175.5 0h130.9v480h-131z"/>
</svg>"##,
            Self::Turkish => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-tr" viewBox="0 0 640 480">
  <g fill-rule="evenodd">
    <path fill="#e30a17" d="M0 0h640v480H0z"/>
    <path fill="#fff" d="M407 247.5c0 66.2-54.6 119.9-122 119.9s-122-53.7-122-120 54.6-119.8 122-119.8 122 53.7 122 119.9"/>
    <path fill="#e30a17" d="M413 247.5c0 53-43.6 95.9-97.5 95.9s-97.6-43-97.6-96 43.7-95.8 97.6-95.8 97.6 42.9 97.6 95.9z"/>
    <path fill="#fff" d="m430.7 191.5-1 44.3-41.3 11.2 40.8 14.5-1 40.7 26.5-31.8 40.2 14-23.2-34.1 28.3-33.9-43.5 12-25.8-37z"/>
  </g>
</svg>
"##,
            Self::Slovenian => r##"
<svg width="1.5em" xmlns="http://www.w3.org/2000/svg" id="flag-icons-si" viewBox="0 0 640 480">
  <defs>
    <clipPath id="si-a">
      <path fill-opacity=".7" d="M-15 0h682.6v512H-15.1z"/>
    </clipPath>
  </defs>
  <g fill-rule="evenodd" stroke-width="1pt" clip-path="url(#si-a)" transform="translate(14.1)scale(.9375)">
    <path fill="#fff" d="M-62 0H962v512H-62z"/>
    <path fill="#d50000" d="M-62 341.3H962V512H-62z"/>
    <path fill="#0000bf" d="M-62 170.7H962v170.6H-62z"/>
    <path fill="#d50000" d="M228.4 93c-4 61.6-6.4 95.4-15.7 111-10.2 16.8-20 29.1-59.7 44-39.6-14.9-49.4-27.2-59.6-44-9.4-15.6-11.7-49.4-15.7-111l5.8-2c11.8-3.6 20.6-6.5 27.1-7.8 9.3-2 17.3-4.2 42.3-4.7 25 .4 33 2.8 42.3 4.8 6.4 1.4 15.6 4 27.3 7.7z"/>
    <path fill="#0000bf" d="M222.6 91c-3.8 61.5-7 89.7-12 103.2-9.6 23.2-24.8 35.9-57.6 48-32.8-12.1-48-24.8-57.7-48-5-13.6-8-41.7-11.8-103.3 11.6-3.6 20.6-6.4 27.1-7.7 9.3-2 17.3-4.3 42.3-4.7 25 .4 33 2.7 42.3 4.7a284 284 0 0 1 27.4 7.7z"/>
    <path fill="#ffdf00" d="m153 109.8 1.5 3.7 7 1-4.5 2.7 4.3 2.9-6.3 1-2 3.4-2-3.5-6-.8 4-3-4.2-2.7 6.7-1z"/>
    <path fill="#fff" d="m208.3 179.6-3.9-3-2.7-4.6-5.4-4.7-2.9-4.7-5.4-4.9-2.6-4.7-3-2.3-1.8-1.9-5 4.3-2.6 4.7-3.3 3-3.7-2.9-2.7-4.8-10.3-18.3-10.3 18.3-2.7 4.8-3.7 2.9-3.3-3-2.7-4.7-4.9-4.3-1.9 1.8-2.9 2.4-2.6 4.7-5.4 4.9-2.9 4.7-5.4 4.7-2.7 4.6-3.9 3a65.8 65.8 0 0 0 18.6 36.3 107 107 0 0 0 36.6 20.5 104.1 104.1 0 0 0 36.8-20.5c5.8-6 16.6-19.3 18.6-36.3"/>
    <path fill="#ffdf00" d="m169.4 83.9 1.6 3.7 7 1-4.6 2.7 4.4 2.9-6.3 1-2 3.4-2-3.5-6-.8 4-3-4.2-2.7 6.6-1zm-33 0 1.6 3.7 7 .9-4.5 2.7 4.3 2.9-6.3 1-2 3.4-2-3.4-6-.9 4-3-4.2-2.7 6.7-1z"/>
    <path fill="#0000bf" d="M199.7 203h-7.4l-7-.5-8.3-4h-9.4l-8.1 4-6.5.6-6.4-.6-8.1-4H129l-8.4 4-6.9.6-7.6-.1-3.6-6.2.1-.2 11.2 1.9 6.9-.5 8.3-4.1h9.4l8.2 4 6.4.6 6.5-.6 8.1-4h9.4l8.4 4 6.9.6 10.8-2 .2.4zm-86.4 9.5 7.4-.5 8.3-4h9.4l8.2 4 6.4.5 6.4-.5 8.2-4h9.4l8.3 4 7.5.5 4.8-6h-.1l-5.2 1.4-6.9-.5-8.3-4h-9.4l-8.2 4-6.4.6-6.5-.6-8.1-4H129l-8.4 4-6.9.6-5-1.3v.2l4.5 5.6z"/>
  </g>
</svg>
"##,
        }
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
