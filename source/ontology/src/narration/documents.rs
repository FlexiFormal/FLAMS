use super::{checking::DocumentChecker, DocumentElement, UncheckedDocumentElement};
use crate::{uris::DocumentURI, DecodeError};
use core::str;
use immt_utils::binary::BinaryReader;
use std::{fmt::Debug, str::FromStr};
use triomphe::Arc;

#[derive(Debug)]
struct DocumentI {
    uri: DocumentURI,
    title: Option<Box<str>>,
    pub elements: Box<[DocumentElement]>,
}

#[derive(Debug, Clone)]
pub struct Document(Arc<DocumentI>);
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
    pub fn children(&self) -> &[DocumentElement] {
        &self.0.elements
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncheckedDocument {
    pub uri: DocumentURI,
    pub title: Option<Box<str>>,
    pub elements: Vec<UncheckedDocumentElement>,
}

impl UncheckedDocument {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_byte_stream(bytes: &mut impl BinaryReader) -> Result<Self, DecodeError> {
        let uri = bytes.read_string(DocumentURI::from_str)??;
        let title: Option<Box<str>> =
            bytes.read_string(|s| if s.is_empty() { None } else { Some(s.into()) })?;
        let elements = ByteReadState::new(bytes).go()?;
        Ok(Self {
            uri,
            title,
            elements,
        })
    }

    pub fn check(self, checker: &mut impl DocumentChecker) -> Document {
        let elements = super::checking::DocumentCheckIter::go(self.elements, checker, &self.uri)
            .into_boxed_slice();
        Document(Arc::new(DocumentI {
            uri: self.uri,
            title: self.title,
            elements,
        }))
    }
    /*
    #[cfg(feature="tokio")]
    #[allow(clippy::missing_errors_doc)]
    pub async fn from_byte_stream_async(bytes:&mut impl immt_utils::binary::AsyncBinaryReader) -> Result<Self,DecodeError> {
      let uri = bytes.read_string(DocumentURI::from_str).await??;
      let title:Option<Box<str>> = bytes.read_string(|s|
        if s.is_empty() {None} else {Some(s.into())}
      ).await?;
      let elements = ByteReadState::default().go_async(bytes).await?;
      Ok(Self {
        uri,title,elements
      })
    }
    */
}

struct ByteReadState<'a, B: BinaryReader> {
    stack: Vec<(
        UncheckedDocumentElement,
        Vec<UncheckedDocumentElement>,
        u16,
        u16,
    )>,
    curr: Vec<UncheckedDocumentElement>,
    max: u16,
    idx: u16,
    bytes: &'a mut B,
}
impl<'a, B: BinaryReader> ByteReadState<'a, B> {
    fn new(bytes: &'a mut B) -> Self {
        Self {
            stack: Vec::new(),
            curr: Vec::new(),
            max: 0,
            idx: 0,
            bytes,
        }
    }

    fn do_elem(&mut self) -> Result<(), DecodeError> {
        match self.bytes.pop()? {
      0 /*SetSectionLevel*/ => if let Ok(s) = self.bytes.pop()?.try_into() {
          self.curr.push(UncheckedDocumentElement::SetSectionLevel(s));
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
      17 /*Exercise*/ => todo!(),
      _ => Err(DecodeError::UnknownDiscriminant)
    }
    }

    fn go(mut self) -> Result<Vec<UncheckedDocumentElement>, DecodeError> {
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
    async fn go_async(&mut self,bytes:&mut impl immt_utils::binary::AsyncBinaryReader) -> Result<Vec<UncheckedDocumentElement>,DecodeError> {
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
