use super::{
    checking::DocumentChecker, paragraphs::ParagraphKind, sections::SectionLevel, DocumentElement,
    NarrationTrait,
};
use crate::{
    uris::{DocumentURI, Name},
    Checked, CheckingState, Resolvable, Unchecked,
};
use core::str;
use flams_utils::prelude::{TreeChild, TreeChildIter, TreeLike};
use std::{borrow::Cow, fmt::Debug, str::FromStr};
use triomphe::Arc;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentStyles {
    pub counters: Vec<SectionCounter>,
    pub styles: Vec<DocumentStyle>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentStyle {
    pub kind: ParagraphKind,
    pub name: Option<Name>,
    pub counter: Option<Name>,
}
impl FromStr for DocumentStyle {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((a, b)) = s.split_once('-') {
            let kind = ParagraphKind::from_str(a)?;
            let name = Some(Name::from_str(b).map_err(|_| ())?);
            return Ok(Self {
                kind,
                name,
                counter: None,
            });
        }
        let kind = ParagraphKind::from_str(s)?;
        Ok(Self {
            kind,
            name: None,
            counter: None,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SectionCounter {
    pub name: Name,
    pub parent: Option<SectionLevel>,
}

#[derive(Debug)]
pub struct OpenDocument<State: CheckingState> {
    pub uri: DocumentURI,
    pub title: Option<Box<str>>,
    pub elements: State::Seq<DocumentElement<State>>,
    pub styles: DocumentStyles,
}
crate::serde_impl! {mod serde_doc =
    struct OpenDocument[uri,title,elements,styles]
}

#[derive(Debug, Clone)]
pub struct Document(pub(super) Arc<OpenDocument<Checked>>);
impl Resolvable for Document {
    type From = DocumentURI;
    #[inline]
    fn id(&self) -> Cow<'_, Self::From> {
        Cow::Borrowed(&self.0.uri)
    }
}
impl Document {
    #[inline]
    #[must_use]
    pub fn uri(&self) -> &DocumentURI {
        &self.0.uri
    }
    #[must_use]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
    #[inline]
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.0.title.as_deref()
    }
    #[inline]
    #[must_use]
    pub fn styles(&self) -> &DocumentStyles {
        &self.0.styles
    }
    #[inline]
    pub fn dfs(&self) -> impl Iterator<Item = &DocumentElement<Checked>> {
        <_ as TreeChildIter<Self>>::dfs(NarrationTrait::children(self).iter())
    }
}

impl NarrationTrait for Document {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.0.elements
    }
    #[inline]
    fn from_element(_: &DocumentElement<Checked>) -> Option<&Self>
    where
        Self: Sized,
    {
        None
    }
}

impl NarrationTrait for OpenDocument<Checked> {
    #[inline]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.elements
    }
    #[inline]
    fn from_element(_: &DocumentElement<Checked>) -> Option<&Self>
    where
        Self: Sized,
    {
        None
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    impl serde::Serialize for super::Document {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
}

pub type UncheckedDocument = OpenDocument<Unchecked>;

impl UncheckedDocument {
    #[inline]
    pub fn dfs(&self) -> impl Iterator<Item = &DocumentElement<Unchecked>> {
        <_ as TreeChildIter<Self>>::dfs(self.elements.iter())
    }
    pub fn check(self, checker: &mut impl DocumentChecker) -> Document {
        let elements = super::checking::DocumentCheckIter::go(self.elements, checker, &self.uri)
            .into_boxed_slice();
        Document(Arc::new(OpenDocument {
            uri: self.uri,
            title: self.title,
            styles: self.styles,
            elements,
        }))
    }

    /*
    /// #### Errors
    pub fn from_byte_stream(bytes: &mut impl BinaryReader) -> Result<Self, DecodeError> {
        let version = bytes.pop()?;
        let uri = bytes.read_string(DocumentURI::from_str)??;
        let title: Option<Box<str>> =
            bytes.read_string(|s| if s.is_empty() { None } else { Some(s.into()) })?;
        let elements = ByteReadState::new(bytes,version).go()?;
        Ok(Self {
            uri,
            title,
            elements,
        })
    }
     */
}

impl TreeLike for Document {
    type Child<'a> = &'a DocumentElement<Checked>;
    type RefIter<'a> = std::slice::Iter<'a, DocumentElement<Checked>>;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        Some(NarrationTrait::children(self).iter())
    }
}
impl<'a> TreeChild<Document> for &'a DocumentElement<Checked> {
    fn children<'b>(&self) -> Option<std::slice::Iter<'a, DocumentElement<Checked>>>
    where
        Self: 'b,
    {
        Some(NarrationTrait::children(*self).iter())
    }
}

impl TreeLike for UncheckedDocument {
    type Child<'a> = &'a DocumentElement<Unchecked>;
    type RefIter<'a> = std::slice::Iter<'a, DocumentElement<Unchecked>>;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        Some(self.elements.iter())
    }
}
impl<'a> TreeChild<UncheckedDocument> for &'a DocumentElement<Unchecked> {
    fn children<'b>(&self) -> Option<std::slice::Iter<'a, DocumentElement<Unchecked>>>
    where
        Self: 'b,
    {
        match self {
            DocumentElement::Section(s) => Some(s.children.iter()),
            DocumentElement::Paragraph(p) => Some(p.children.iter()),
            DocumentElement::Problem(e) => Some(e.children.iter()),
            DocumentElement::Module { children, .. }
            | DocumentElement::Morphism { children, .. }
            | DocumentElement::MathStructure { children, .. }
            | DocumentElement::Extension { children, .. }
            | DocumentElement::SkipSection(children)
            | DocumentElement::Slide { children, .. } => Some(children.iter()),
            _ => None,
        }
    }
}

/*
impl<State:CheckingState> OpenDocument<State> {
    /// #### Errors
    pub fn into_byte_stream(&self,bytes: &mut impl BinaryWriter) -> Result<(),std::io::Error> {
        bytes.write_all(&[1])?;
        bytes.write_string(&self.uri.to_string())?;
        bytes.write_string(self.title.as_ref().map_or("",|b| &**b))?;
        for e in &self.elements {
            match e {
                DocumentElement::SetSectionLevel(lvl) => {
                    bytes.write_all(&[0,lvl.into()])?;
                },
                DocumentElement::Section(s) => {
                    // 1
                    todo!()
                }
                DocumentElement::Module { range, module, children } => {
                    // 2
                    todo!()
                }
                DocumentElement::Morphism { range, morphism, children } => {
                    // 3
                    todo!()
                }
                DocumentElement::MathStructure { range, structure, children } => {
                    // 4
                    todo!()
                }
                DocumentElement::DocumentReference { id, range, target } => {
                    // 5
                    todo!()
                }
                DocumentElement::SymbolDeclaration(s) => {
                    // 6
                    todo!()
                }
                DocumentElement::Notation { symbol, id, notation } => {
                    // 7
                    todo!()
                }
                DocumentElement::VariableNotation { variable, id, notation } => {
                    // 8
                    todo!()
                }
                DocumentElement::Variable(v) => {
                    // 9
                    todo!()
                }
                DocumentElement::Definiendum { range, uri } => {
                    // 10
                    todo!()
                }
                DocumentElement::SymbolReference { range, uri, notation } => {
                    // 11
                    todo!()
                }
                DocumentElement::VariableReference { range, uri, notation } => {
                    // 12
                    todo!()
                }
                DocumentElement::TopTerm { uri, term } => {
                    // 13
                    todo!()
                }
                DocumentElement::UseModule(uri) => {
                    // 14
                    todo!()
                }
                DocumentElement::ImportModule(uri) => {
                    // 15
                    todo!()
                }
                DocumentElement::Paragraph(p) => {
                    // 16
                    todo!()
                }
                DocumentElement::Problem(e) => {
                    // 17
                    todo!()
                }
            }
        }
        Ok(())
    }
}
*/
/*
#[allow(clippy::type_complexity)]
struct ByteReadState<'a, B: BinaryReader> {
    stack: Vec<(
        DocumentElement<Unchecked>,
        Vec<DocumentElement<Unchecked>>,
        u16,
        u16,
    )>,
    curr: Vec<DocumentElement<Unchecked>>,
    max: u16,
    idx: u16,
    version:u8,
    bytes: &'a mut B,
}
impl<'a, B: BinaryReader> ByteReadState<'a, B> {
    fn new(bytes: &'a mut B,version:u8) -> Self {
        Self {
            stack: Vec::new(),
            curr: Vec::new(),
            max: 0,
            idx: 0,
            version,
            bytes,
        }
    }

    fn do_elem(&mut self) -> Result<(), DecodeError> {
        match self.bytes.pop()? {
      0 /*SetSectionLevel*/ => if let Ok(s) = self.bytes.pop()?.try_into() {
          self.curr.push(DocumentElement::SetSectionLevel(s));
          Ok(())
        } else {
          Err(DecodeError::UnknownDiscriminant)
        },
      1 /*Section*/ => todo!(),
      2 /*Module*/ => todo!(),
      3 /*Morphism*/ => todo!(),
      4 /*MathStructure*/ => todo!(),
      5 /*DocumentReference*/ => todo!(),
      6 /*SymbolDeclaration*/ => todo!(),
      7 /*Notation*/ => todo!(),
      8 /*VariableNotation*/ => todo!(),
      9 /*Variable*/ => todo!(),
      10 /*Definiendum*/ => todo!(),
      11 /*SymbolReference*/ => todo!(),
      12 /*VariableReference*/ => todo!(),
      13 /*TopTerm*/ => todo!(),
      14 /*UseModule*/ => todo!(),
      15 /*ImportModule*/ => todo!(),
      16 /*Paragraph*/ => todo!(),
      17 /*Problem*/ => todo!(),
      _ => Err(DecodeError::UnknownDiscriminant)
    }
    }

    fn go(mut self) -> Result<Vec<DocumentElement<Unchecked>>, DecodeError> {
        self.max = self.bytes.read_u16()?;
        loop {
            while self.idx < self.max {
                self.idx += 1;
                self.do_elem()?;
            }
            if let Some((mut last, next, idx, max)) = self.stack.pop() {
                let done = std::mem::replace(&mut self.curr, next);
                last.set_children(done).unwrap_or_else(|_| unreachable!());
                self.curr.push(last);
                self.idx = idx;
                self.max = max;
            } else {
                return Ok(self.curr);
            }
        }
    }
    /*
    #[cfg(feature="tokio")]
    async fn go_async(&mut self,bytes:&mut impl flams_utils::binary::AsyncBinaryReader) -> Result<Vec<UncheckedDocumentElement>,DecodeError> {
      use tokio::io::{AsyncBufRead,AsyncBufReadExt,AsyncRead,AsyncReadExt};
      match bytes.read_u8().await? {
        0 => todo!(),
        1 => todo!(),
        2 => todo!(),
        _ => return Err(DecodeError::UnknownDiscriminant)
      }
    }
    */
}
 */
