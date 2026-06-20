use ftml_uris::{Language, SymbolUri};
use smallvec::SmallVec;

#[derive(Clone, Debug, Default)]
pub struct VerbalizationTrie(
    std::sync::Arc<parking_lot::Mutex<radix_trie::Trie<Verbalization, SmallVec<SymbolUri, 1>>>>,
);
impl VerbalizationTrie {
    pub fn insert(&self, language: Language, text: &str, uri: &SymbolUri) {
        let stem = Verbalization::new(text, language);
        let mut slock = self.0.lock();
        if let Some(e) = slock.get_mut(&stem) {
            if !e.contains(uri) {
                e.push(uri.clone());
            }
        } else {
            slock.insert(stem, smallvec::smallvec_inline![uri.clone()]);
        }
        drop(slock);
    }
    //pub fn get()
}

#[derive(Clone)]
pub struct Verbalization {
    pub lang: Language,
    inner: radix_trie::NibbleVec<[u8; 64]>,
}

impl std::hash::Hash for Verbalization {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.as_bytes().hash(state);
    }
}
impl AsRef<str> for Verbalization {
    fn as_ref(&self) -> &str {
        // SAFETY: by construction, always valid utf8
        unsafe { std::str::from_utf8_unchecked(self.inner.as_bytes()) }
    }
}
impl std::fmt::Display for Verbalization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl std::fmt::Debug for Verbalization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl Verbalization {
    #[must_use]
    pub fn new(txt: &str, lang: Language) -> Self {
        let mut ret = radix_trie::NibbleVec::new();
        let stemmer = match lang {
            Language::Arabic => waken_snowball::Algorithm::Arabic,
            Language::English => waken_snowball::Algorithm::English,
            Language::Finnish => waken_snowball::Algorithm::Finnish,
            Language::French => waken_snowball::Algorithm::French,
            Language::German => waken_snowball::Algorithm::German,
            Language::Romanian => waken_snowball::Algorithm::Romanian,
            Language::Russian => waken_snowball::Algorithm::Russian,
            Language::Turkish => waken_snowball::Algorithm::Turkish,
            Language::Slovenian | Language::Bulgarian | _ => {
                for b in txt.as_bytes() {
                    ret.push(*b >> 4);
                    ret.push(*b);
                }
                return Self { lang, inner: ret };
            }
        }
        .stemmer();
        for e in txt.split(|c: char| c.is_ascii_whitespace()) {
            if e.is_empty() {
                continue;
            }
            if !ret.is_empty() {
                ret.push(b' ' >> 4);
                ret.push(b' ');
            }
            for b in stemmer.stem(e).as_bytes() {
                let b = b.to_ascii_lowercase();
                ret.push(b >> 4);
                ret.push(b);
            }
        }
        Self { lang, inner: ret }
    }
}
impl PartialEq for Verbalization {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for Verbalization {}
impl radix_trie::TrieKey for Verbalization {
    fn encode(&self) -> radix_trie::NibbleVec<[u8; 64]> {
        self.inner.clone()
    }
}
