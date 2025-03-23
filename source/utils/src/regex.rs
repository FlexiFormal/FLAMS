#[cfg(target_family = "wasm")]
use js_regexp::RegExp as JRegex;
#[cfg(not(target_family = "wasm"))]
use regex::Regex as IRegex;

#[cfg(not(target_family = "wasm"))]
#[derive(Debug, Clone)]
pub struct Regex(IRegex);
#[cfg(target_family = "wasm")]
#[derive(Debug, Clone)]
pub struct Regex(String);

impl std::fmt::Display for Regex {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Regex {
    pub fn is_match(&self, text: &str) -> bool {
        #[cfg(not(target_family = "wasm"))]
        {
            self.0.is_match(text)
        }
        #[cfg(target_family = "wasm")]
        {
            crate::unwrap!(JRegex::new(&self.0, js_regexp::flags!("")).ok())
                .exec(text)
                .is_some()
        }
    }
    pub fn as_str(&self) -> &str {
        #[cfg(not(target_family = "wasm"))]
        {
            self.0.as_str()
        }
        #[cfg(target_family = "wasm")]
        {
            &self.0
        }
    }
    pub fn new(s: &str) -> Result<Self, String> {
        #[cfg(not(target_family = "wasm"))]
        {
            IRegex::new(s).map(Self).map_err(|e| format!("{}", e))
        }
        #[cfg(target_family = "wasm")]
        {
            JRegex::new(s, js_regexp::flags!(""))
                .map(|_| Self(s.to_string()))
                .map_err(|_| "Invalid Regular Expression".to_string())
        }
        // https://docs.rs/js-regexp/0.2.1/js_regexp/struct.FlagSets.html
    }
}

#[cfg(feature = "serde")]
mod regex_serde {
    use super::Regex;
    impl serde::Serialize for Regex {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> serde::Deserialize<'de> for Regex {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Regex::new(&s).map_err(serde::de::Error::custom)
        }
    }
}
