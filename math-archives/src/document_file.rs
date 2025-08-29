use std::path::PathBuf;

use crate::utils::{
    AsyncEngine,
    errors::ReadError,
    lazy_file::{BytesField, EagerField, LazyField, LazyFile, StreamField},
};
use ftml_ontology::{
    narrative::{DocumentRange, documents::Document},
    utils::Css,
};

#[derive(Debug)]
pub struct DocumentFile {
    reader: LazyFile<6>,
    body: EagerField<DocumentRange, 0>,
    inner_offset: EagerField<u32, 1>,
    css: EagerField<Box<[Css]>, 2>,
    data: BytesField<3>,
    document: LazyField<Document, 4>,
    html: StreamField<5>,
}
impl DocumentFile {
    /// # Errors
    #[inline]
    pub fn get_document(&self) -> Result<Document, ReadError> {
        self.document.get(&self.reader)
    }

    /// # Errors
    #[inline]
    #[allow(clippy::future_not_send)]
    pub async fn get_document_async<A: AsyncEngine>(&self) -> Result<Document, ReadError> {
        self.document.get_async::<A, _>(&self.reader).await
    }

    /// # Errors
    #[inline]
    pub fn get_html(&self) -> Result<Box<str>, ReadError> {
        self.html.get(&self.reader)
    }

    /// # Errors
    #[inline]
    pub fn get_css(&self) -> Box<[Css]> {
        self.css.get().clone()
    }

    /// # Errors
    pub fn get_html_body(&self) -> Result<Box<str>, ReadError> {
        self.get_html_range(*self.body.get())
    }

    /// # Errors
    pub fn get_html_body_inner(&self) -> Result<Box<str>, ReadError> {
        let mut range = *self.body.get();
        range.start += *self.inner_offset.get() as usize;
        range.end -= "</body>".len();
        self.get_html_range(range)
    }

    /// # Errors
    #[inline]
    pub fn get_html_range(&self, range: DocumentRange) -> Result<Box<str>, ReadError> {
        self.html.get_range(&self.reader, range.start, range.end)
    }

    /// # Errors
    #[inline]
    pub fn get_data<T: serde::de::DeserializeOwned>(
        &self,
        start: usize,
        end: usize,
    ) -> Result<T, ReadError> {
        self.data.deserialize_range(&self.reader, start, end) //.get_range(&self.reader, start, end)
    }

    /// # Errors
    pub fn from_file(path: PathBuf) -> Result<Self, ReadError> {
        let data = BytesField;
        let document = LazyField::default();
        let html = StreamField;
        let (reader, (body, css, inner_offset)) = LazyFile::new_and_then(path, |mut reader| {
            let body = EagerField::new(&mut reader)?;
            let inner_offset = EagerField::new(&mut reader)?;
            let css = EagerField::new(&mut reader)?;
            Ok((body, css, inner_offset))
        })?;
        Ok(Self {
            reader,
            body,
            inner_offset,
            css,
            data,
            document,
            html,
        })
    }
}
#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for DocumentFile {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.css
            .get()
            .iter()
            .map(|c| match c {
                Css::Class { name, css } => name.len() + css.len(),
                Css::Inline(s) | Css::Link(s) => s.len(),
            })
            .sum::<usize>()
            + self.document.deep_size_of_children(context)
    }
}
