use std::{borrow::Cow};

use flams_utils::{
    CharExt,
    parsing::{SourceParser, StrParser},
    sourcerefs::{StringPosition, StringRange},
};
use ftml_uris::{Language, SymbolUri};
use radix_trie::NibbleVec;
use smallvec::SmallVec;

#[derive(Clone, Debug, Default)]
pub struct VerbalizationTrie(
    pub(crate) std::sync::Arc<parking_lot::Mutex<radix_trie::Trie<Verbalization, SmallVec<SymbolUri, 1>>>>,
);
pub struct UnlockedVerbalizationTrie<'a>(parking_lot::MutexGuard<'a,radix_trie::Trie<Verbalization, SmallVec<SymbolUri, 1>>>);
impl UnlockedVerbalizationTrie<'_> {
    pub fn insert(&mut self, language: Language, text: &str, uri: &SymbolUri) {
        if text.len() <= 3 {return;}
        let stem = Verbalization::new(text, language);
        if let Some(e) = self.0.get_mut(&stem) {
            if !e.contains(uri) {
                e.push(uri.clone());
            }
        } else {
            self.0.insert(stem, smallvec::smallvec_inline![uri.clone()]);
        }
    }
    pub fn find_all_in<P: StringPosition>(
        &self,
        language: Language,
        mut text: StrParser<P>,
        needs_usemodule: &dyn Fn(&SymbolUri) -> bool
    ) -> Vec<(StringRange<P>, SmallVec<(SymbolUri,bool), 1>)> {
        use radix_trie::TrieCommon;
        struct StackElem<P: StringPosition> {
            verb: Verbalization,
            range: StringRange<P>,
            separator: Option<char>,
            has_singleton: bool,
            merged: Option<(Verbalization, bool)>,
        }
        fn clear<P:StringPosition>(stack:&mut Vec<StackElem<P>>,ret:&mut Vec<(StringRange<P>, SmallVec<(SymbolUri,bool), 1>)>,
            needs_usemodule: &dyn Fn(&SymbolUri) -> bool,
            trie:&radix_trie::Trie<Verbalization,SmallVec<SymbolUri, 1>>) {
                let mut readd = Vec::new();
                if let Some(start) = stack.first().map(|e| e.range.start) {
                    for e in std::mem::take(stack).into_iter().rev() {
                        if let Some((merged, true)) = e.merged {
                            let range = StringRange {
                                start,
                                end: e.range.end,
                            };
                            // SAFETY: e.merged.1 == true
                            readd.push((range, unsafe {
                                trie.get(&merged).unwrap_unchecked().iter().map(|s| (s.clone(),needs_usemodule(s))).collect()
                            }));
                            break;
                        }
                        if e.has_singleton {
                            // SAFETY: has_singleton == true
                            readd.push((e.range, unsafe {
                                trie.get(&e.verb).unwrap_unchecked().iter().map(|s| (s.clone(),needs_usemodule(s))).collect()
                            }));
                        }
                    }
                    ret.extend(readd.into_iter().rev());
                }
            }
        let mut stack: Vec<StackElem<P>> = Vec::with_capacity(1);
        let mut ret = Vec::new();
        let trie = &self.0;

        while !text.rest().is_empty() {
            text.trim_start();
            let start = text.pos;
            let next = text.read_until(|c| Verbalization::BREAKS.contains(&c));
            let end = text.pos;
            let separator = text.next_char();
            let verb = Verbalization::new(next, language);
            let range = StringRange { start, end };
            let has_singleton = trie.get(&verb).is_some();
            if let Some(last) = stack.last() {
                let merger = last
                    .merged
                    .as_ref()
                    .map_or_else(|| last.verb.clone(), |(m, _)| m.clone());
                let merged = merger.merge(&verb, last.separator);
                if let Some(sub) = trie.subtrie(&verb)
                    && !sub.is_empty()
                {
                    if sub.is_leaf() {
                        if let Ok(Some(v)) = sub.get(&merged) {
                            // SAFETY: known to be non-empty
                            let start = unsafe { stack.first().unwrap_unchecked() }.range.start;
                            ret.push((
                                StringRange {
                                    start,
                                    end: range.end,
                                },
                                v.iter().map(|s| (s.clone(),needs_usemodule(s))).collect(),
                            ));
                            stack.clear();
                            continue;
                        }
                    } else {
                        let can_merge = trie.get(&merged).is_some();
                        stack.push(StackElem {
                            verb,
                            range,
                            separator,
                            has_singleton,
                            merged: Some((merged, can_merge)),
                        });
                        continue;
                    }
                } else {
                    clear(&mut stack,&mut ret,needs_usemodule,trie);
                }
            }

            if let Some(sub) = trie.subtrie(&verb)
                && !sub.is_empty()
            {
                if sub.is_leaf() {
                    if let Ok(Some(v)) = sub.get(&verb) {
                        ret.push((range, v.iter().map(|s| (s.clone(),needs_usemodule(s))).collect()));
                    }
                } else {
                    stack.push(StackElem {
                        verb,
                        range,
                        separator,
                        has_singleton,
                        merged: None,
                    });
                }
            }
        }
        clear(&mut stack,&mut ret,needs_usemodule,trie);
        ret
    }
}
impl VerbalizationTrie {
    #[must_use]
    pub fn lock(&self) -> UnlockedVerbalizationTrie<'_> {
        UnlockedVerbalizationTrie(self.0.lock())
    }
    pub fn insert(&self, language: Language, text: &str, uri: &SymbolUri) {
        self.lock().insert(language, text, uri);
    }
    //pub fn get()
    pub fn find_all_in<P: StringPosition>(
        &self,
        language: Language,
        text: StrParser<P>,
        needs_usemodule: &dyn Fn(&SymbolUri) -> bool
    ) -> Vec<(StringRange<P>, SmallVec<(SymbolUri,bool), 1>)> {
        self.lock().find_all_in(language, text,needs_usemodule)
    }
    pub fn merge(&self,other:Self) {
        use radix_trie::TrieCommon;
        let mut trie = self.0.lock();
        for (k,v) in other.0.lock().iter() {
            // not nice :(
            trie.insert(k.clone(), v.clone());
        }
    }
}

#[derive(Clone)]
pub struct Verbalization {
    pub lang: Language,
    inner: Nibble,
}
#[derive(Clone)]
struct Nibble(radix_trie::NibbleVec<[u8; 64]>);
impl Nibble {
    fn push(&mut self, mut c: u8) {
        c.make_ascii_lowercase();
        self.0.push(c >> 4);
        self.0.push(c);
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::hash::Hash for Verbalization {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.0.as_bytes().hash(state);
    }
}
impl AsRef<str> for Verbalization {
    fn as_ref(&self) -> &str {
        // SAFETY: by construction, always valid utf8
        unsafe { std::str::from_utf8_unchecked(self.inner.0.as_bytes()) }
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
    const BREAKS: [char; 17] = [
        ' ', '\t', '\r', '\n', '.', ',', ';', '!', ':', '-', '(', ')', '[', ']', '{', '}', '$',
    ];
    fn merge(mut self, other: &Self, sep: Option<char>) -> Self {
        if let Some(sep) = sep
            && !sep.is_ascii_whitespace()
        {
            for u in sep.into_bytes() {
                self.inner.push(u);
            }
        } else {
            self.inner.push(b' ');
        }
        for b in other.inner.0.as_bytes() {
            self.inner.push(*b);
        }
        self
    }
    fn stem(s: &str, lang: Language) -> Cow<'_, str> {
        Self::stemmer(lang).map_or_else(|| Cow::Borrowed(s), |stem| stem.stem(s))
    }
    fn stemmer(lang: Language) -> Option<waken_snowball::Stemmer> {
        Some(
            (match lang {
                Language::Arabic => waken_snowball::Algorithm::Arabic,
                Language::English => waken_snowball::Algorithm::English,
                Language::Finnish => waken_snowball::Algorithm::Finnish,
                Language::French => waken_snowball::Algorithm::French,
                Language::German => waken_snowball::Algorithm::German,
                Language::Romanian => waken_snowball::Algorithm::Romanian,
                Language::Russian => waken_snowball::Algorithm::Russian,
                Language::Turkish => waken_snowball::Algorithm::Turkish,
                Language::Slovenian | Language::Bulgarian | _ => {
                    return None;
                }
            })
            .stemmer(),
        )
    }
    #[must_use]
    pub fn new(txt: &str, lang: Language) -> Self {
        let mut ret = Nibble(NibbleVec::new());
        let stemmer = Self::stemmer(lang);

        /*let Some(stemmer) = Self::stemmer(lang) else {
            for b in txt.as_bytes() {
                ret.push(*b);
            }
            return Self { lang, inner: ret };
        };*/
        for e in txt.trim().split(|c: char| c.is_ascii_whitespace()) {
            if e.is_empty() {
                continue;
            }
            if !ret.is_empty() {
                ret.push(b' ');
            }
            if let Some(stemmer) = &stemmer {
                for b in stemmer.stem(e).as_bytes() {
                    ret.push(*b);
                }
            } else {
                for b in e.as_bytes() {
                    ret.push(*b);
                }
            }
        }
        Self { lang, inner: ret }
    }
}
impl PartialEq for Verbalization {
    fn eq(&self, other: &Self) -> bool {
        self.inner.0 == other.inner.0
    }
}
impl Eq for Verbalization {}
impl radix_trie::TrieKey for Verbalization {
    fn encode(&self) -> radix_trie::NibbleVec<[u8; 64]> {
        self.inner.0.clone()
    }
}
