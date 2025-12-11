#![allow(clippy::wildcard_imports)]

#[cfg(feature = "vectorsearch")]
use ftml_ontology::narrative::documents::Document;
use ftml_ontology::narrative::elements::paragraphs::ParagraphKind;
#[cfg(feature = "vectorsearch")]
use ftml_uris::Language;
use ftml_uris::{DocumentElementUri, DocumentUri, SymbolUri};
#[cfg(feature = "vectorsearch")]
use smallvec::SmallVec;

#[allow(dead_code)]
const fn get_true() -> bool {
    true
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct QueryFilter {
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_documents: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_paragraphs: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_definitions: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_examples: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_assertions: bool,
    #[cfg_attr(feature = "serde", serde(default = "get_true"))]
    pub allow_problems: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub definition_like_only: bool,
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            allow_documents: true,
            allow_paragraphs: true,
            allow_definitions: true,
            allow_examples: true,
            allow_assertions: true,
            allow_problems: true,
            definition_like_only: false,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResult {
    Document(DocumentUri),
    Paragraph {
        uri: DocumentElementUri,
        fors: Vec<SymbolUri>,
        def_like: bool,
        kind: SearchResultKind,
    },
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResultKind {
    Document = 0,
    Paragraph = 1,
    Definition = 2,
    Example = 3,
    Assertion = 4,
    Problem = 5,
}
impl SearchResultKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Paragraph => "Paragraph",
            Self::Definition => "Definition",
            Self::Example => "Example",
            Self::Assertion => "Assertion",
            Self::Problem => "Problem",
        }
    }
}

impl From<SearchResultKind> for u64 {
    fn from(value: SearchResultKind) -> Self {
        match value {
            SearchResultKind::Document => 0,
            SearchResultKind::Paragraph => 1,
            SearchResultKind::Definition => 2,
            SearchResultKind::Example => 3,
            SearchResultKind::Assertion => 4,
            SearchResultKind::Problem => 5,
        }
    }
}

impl TryFrom<u64> for SearchResultKind {
    type Error = ();
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Document,
            1 => Self::Paragraph,
            2 => Self::Definition,
            3 => Self::Example,
            4 => Self::Assertion,
            5 => Self::Problem,
            _ => return Err(()),
        })
    }
}
impl TryFrom<ParagraphKind> for SearchResultKind {
    type Error = ();
    fn try_from(value: ParagraphKind) -> Result<Self, Self::Error> {
        Ok(match value {
            ParagraphKind::Assertion => Self::Assertion,
            ParagraphKind::Definition => Self::Definition,
            ParagraphKind::Example => Self::Example,
            ParagraphKind::Paragraph => Self::Paragraph,
            _ => return Err(()),
        })
    }
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum SearchIndex {
    Document {
        uri: DocumentUri,
        title: Option<String>,
        body: String,
    },
    Paragraph {
        uri: DocumentElementUri,
        kind: SearchResultKind,
        definition_like: bool,
        title: Option<String>,
        fors: Vec<SymbolUri>,
        body: String,
    },
}

#[cfg(feature = "vectorsearch")]
const LEN: usize = 384;

#[cfg(feature = "vectorsearch")]
#[derive(bincode::Encode, bincode::Decode, Debug, Clone)]
pub struct Embedding(pub(crate) [f32; LEN]);

#[cfg(feature = "vectorsearch")]
impl Embedding {
    #[must_use]
    #[inline]
    pub const fn zero() -> Self {
        Self([0.0; LEN])
    }

    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 4 * LEN] {
        unsafe {
            self.0
                .as_ptr()
                .cast::<[u8; 4 * LEN]>()
                .as_ref()
                .unwrap_unchecked()
        }
    }
    #[inline]
    #[must_use]
    pub const fn new(vec: [f32; LEN]) -> Self {
        Self(vec)
    }
}

#[cfg(feature = "vectorsearch")]
impl<'b> std::ops::Rem<&'b Embedding> for &Embedding {
    type Output = f64;
    fn rem(self, rhs: &'b Embedding) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        simsimd::SpatialSimilarity::cos(&self.0, &rhs.0).map_or(0.0, |r| 1.0 - r)
    }
}

#[cfg(feature = "vectorsearch")]
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub enum SearchIndex {
    Document {
        uri: DocumentUri,
        title: Option<Embedding>,
        body: Embedding,
    },
    Paragraph {
        uri: DocumentElementUri,
        kind: SearchResultKind,
        definition_like: bool,
        title: Option<Embedding>,
        fors: Vec<SymbolUri>,
        body: Embedding,
    },
}

#[cfg(feature = "vectorsearch")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryFilter {
    SymbolsOnly,
    Any(FragmentQueryFilter),
}

#[cfg(feature = "vectorsearch")]
impl Default for QueryFilter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(feature = "vectorsearch")]
impl QueryFilter {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::Any(FragmentQueryFilter::new())
    }
}

#[cfg(feature = "vectorsearch")]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FragmentQueryFilter {
    #[cfg_attr(feature = "serde", serde(default))]
    pub in_documents: SmallVec<DocumentUri, 2>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub languages: SmallVec<Language, 2>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub flags: QueryFilterFlags,
}
#[cfg(feature = "vectorsearch")]
impl FragmentQueryFilter {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_documents: SmallVec::new(),
            languages: SmallVec::new(),
            flags: QueryFilterFlags::new(),
        }
    }

    pub fn close(&mut self, mut get: impl FnMut(&DocumentUri) -> Option<Document>) {
        if self.in_documents.is_empty() {
            return;
        }
        let mut dones = Vec::new();
        let mut todos: Vec<_> = std::mem::take(&mut self.in_documents).into_vec();
        while let Some(uri) = todos.pop() {
            if dones.contains(&uri) {
                continue;
            }
            let d = get(&uri);
            dones.push(uri);
            if let Some(d) = d {
                use ftml_ontology::utils::RefTree;

                for e in d.dfs() {
                    use ftml_ontology::narrative::elements::DocumentElementRef;

                    if let DocumentElementRef::DocumentReference { target, .. } = e {
                        todos.push(target.clone());
                    }
                }
            }
        }
        self.in_documents = dones.into();
    }
}

#[cfg(feature = "vectorsearch")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QueryFilterFlags(u8);
#[cfg(feature = "vectorsearch")]
impl Default for QueryFilterFlags {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(feature = "vectorsearch")]
impl QueryFilterFlags {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(0b0111_1111)
    }
    #[inline]
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_documents(self) -> Self {
        Self(self.0 | 0b0000_0001)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_documents(self) -> Self {
        Self(self.0 & 0b1111_1110)
    }
    #[inline]
    #[must_use]
    pub const fn allow_documents(self) -> bool {
        self.0 % 2 == 1
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_paragraphs(self) -> Self {
        Self(self.0 | 0b0000_0010)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_paragraphs(self) -> Self {
        Self(self.0 & 0b1111_1101)
    }
    #[inline]
    #[must_use]
    pub const fn allow_paragraphs(self) -> bool {
        (self.0 & 0b0000_0010) >> 1 == 1
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_definitions(self) -> Self {
        Self(self.0 | 0b0000_0100)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_definitions(self) -> Self {
        Self(self.0 & 0b1111_1011)
    }
    #[inline]
    #[must_use]
    pub const fn allow_definitions(self) -> bool {
        (self.0 & 0b0000_0100) >> 2 == 1
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_examples(self) -> Self {
        Self(self.0 | 0b0000_1000)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_examples(self) -> Self {
        Self(self.0 & 0b1111_0111)
    }
    #[inline]
    #[must_use]
    pub const fn allow_examples(self) -> bool {
        (self.0 & 0b0000_1000) >> 3 == 1
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_assertions(self) -> Self {
        Self(self.0 | 0b0001_0000)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_assertions(self) -> Self {
        Self(self.0 & 0b1110_1111)
    }
    #[inline]
    #[must_use]
    pub const fn allow_assertions(self) -> bool {
        (self.0 & 0b0001_0000) >> 4 == 1
    }
    #[inline]
    #[must_use]
    pub const fn set_allow_problems(self) -> Self {
        Self(self.0 | 0b0010_0000)
    }
    #[inline]
    #[must_use]
    pub const fn unset_allow_problems(self) -> Self {
        Self(self.0 & 0b1101_1111)
    }
    #[inline]
    #[must_use]
    pub const fn allow_problems(self) -> bool {
        (self.0 & 0b0010_0000) >> 5 == 1
    }
    #[inline]
    #[must_use]
    pub const fn definition_like_only() -> Self {
        Self(0b1000_0000).set_allow_definitions()
    }
}
