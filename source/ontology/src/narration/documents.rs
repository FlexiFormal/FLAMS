use super::{checking::DocumentChecker, DocumentElement, NarrationTrait};
use crate::{uris::DocumentURI, Checked, CheckingState, Resolvable, Unchecked};
use core::str;
use std::{borrow::Cow, fmt::Debug};
use triomphe::Arc;

#[derive(Debug)]
pub struct OpenDocument<State:CheckingState> {
    pub uri: DocumentURI,
    pub title: Option<Box<str>>,
    pub elements: State::Seq<DocumentElement<State>>,
}
crate::serde_impl!{mod serde_doc =
    struct OpenDocument[uri,title,elements]
}


#[derive(Debug, Clone)]
pub struct Document(pub(super) Arc<OpenDocument<Checked>>);
impl Resolvable for Document {
    type From = DocumentURI;
    #[inline]
    fn id(&self) -> Cow<'_,Self::From> {
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
}

impl NarrationTrait for Document {
    #[inline]
    #[must_use]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.0.elements
    }
    #[inline]
    fn from_element(_: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        None
    }
}

impl NarrationTrait for OpenDocument<Checked> {
    #[inline]
    #[must_use]
    fn children(&self) -> &[DocumentElement<Checked>] {
        &self.elements
    }
    #[inline]
    fn from_element(_: &DocumentElement<Checked>) -> Option<&Self> where Self: Sized {
        None
    }
}

#[cfg(feature="serde")]
mod serde_impl {
    impl serde::Serialize for super::Document {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
            self.0.serialize(serializer)
        }
    }
}

pub type UncheckedDocument = OpenDocument<Unchecked>;


impl UncheckedDocument {
    pub fn check(self, checker: &mut impl DocumentChecker) -> Document {
        let elements = super::checking::DocumentCheckIter::go(self.elements, checker, &self.uri)
            .into_boxed_slice();
        Document(Arc::new(OpenDocument {
            uri: self.uri,
            title: self.title,
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
                DocumentElement::Exercise(e) => {
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
      17 /*Exercise*/ => todo!(),
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