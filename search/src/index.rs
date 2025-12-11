use flams_backend_types::search::{SearchIndex, SearchResultKind};
use ftml_ontology::{
    narrative::{
        documents::Document,
        elements::{DocumentElementRef, LogicalParagraph},
    },
    utils::RefTree,
};

use crate::textify::textify;

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
pub trait SearchIndexExt {
    fn to_document(self) -> tantivy::TantivyDocument;
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
impl SearchIndexExt for SearchIndex {
    fn to_document(self) -> tantivy::TantivyDocument {
        let mut ret = tantivy::TantivyDocument::default();
        let schema = crate::schema::SearchSchema::get();
        match self {
            Self::Document { uri, title, body } => {
                ret.add_u64(schema.kind, SearchResultKind::Document.into());
                ret.add_text(schema.uri, uri.to_string());
                if let Some(t) = title {
                    ret.add_text(schema.title, t);
                }
                ret.add_text(schema.body, body);
            }
            Self::Paragraph {
                uri,
                kind,
                definition_like,
                title,
                fors,
                body,
            } => {
                ret.add_u64(schema.kind, kind.into());
                ret.add_text(schema.uri, uri.to_string());
                ret.add_bool(schema.def_like, definition_like);
                for f in fors {
                    //write!(trace,"\n   FOR: {}",f);
                    ret.add_text(schema.fors, f.to_string());
                }
                if let Some(t) = title {
                    ret.add_text(schema.title, t);
                }
                ret.add_text(schema.body, body);
            }
        }
        ret
    }
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
pub fn index_document(doc: &Document, html: &str) -> Vec<SearchIndex> {
    let elems = doc.dfs().filter_map(|e| {
        if let DocumentElementRef::Paragraph(p) = e {
            index_paragraph(p, html)
        } else {
            None
        }
    });
    if let Some(s) = index_document_html(doc, html) {
        std::iter::once(s).chain(elems).collect()
    } else {
        elems.collect()
    }
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
#[must_use]
pub fn index_document_html(doc: &Document, html: &str) -> Option<SearchIndex> {
    let title = doc.title.as_ref().map(|s| textify(s, true));
    let body = textify(html, false);
    Some(SearchIndex::Document {
        uri: doc.uri.clone(),
        title,
        body,
    })
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
pub fn index_paragraph(para: &LogicalParagraph, html: &str) -> Option<SearchIndex> {
    crate::SPAN.in_scope(move || {
        let title = para.title.as_ref().map(|s| textify(s, true));
        let Some(body) = html.get(para.range.start..para.range.end) else {
            tracing::error!(
                "Failed to plain textify body of {}: Error getting HTML range in document",
                para.uri
            );
            return None;
        };
        let body = textify(body, true);
        let fors = para.fors.iter().map(|(f, _)| f.clone()).collect();

        let Ok(kind) = para.kind.try_into() else {
            return None;
        };
        let definition_like = para.kind.is_definition_like(&para.styles);

        Some(SearchIndex::Paragraph {
            uri: para.uri.clone(),
            kind,
            definition_like,
            title,
            fors,
            body,
        })
    })
}

#[cfg(feature = "vectorsearch")]
#[must_use]
pub fn index_document(doc: &Document, html: &str) -> Vec<SearchIndex> {
    use flams_backend_types::search::Embedding;

    let mut indexes = vec![SearchIndex::Document {
        uri: doc.uri.clone(),
        title: None,
        body: Embedding::zero(),
    }];
    let txt = textify(html, false);
    if txt.is_empty() {
        return Vec::new();
    }
    let mut texts = vec![txt];
    if let Some(ttl) = doc.title.as_ref() {
        let SearchIndex::Document { title, .. } = &mut indexes[0] else {
            unreachable!()
        };
        let txt = textify(ttl, true);
        if !txt.is_empty() {
            *title = Some(Embedding::zero());
            texts.push(txt);
        }
    }

    for e in doc.dfs() {
        if let DocumentElementRef::Paragraph(para) = e
            && let Some(body) = html.get(para.range.start..para.range.end)
            && let Ok(kind) = para.kind.try_into()
        {
            let mut txt = textify(body, false);
            if txt.is_empty() {
                continue;
            }
            if !para.fors.is_empty() {
                txt.push_str("\nKEYWORDS: ");
                let mut first = true;
                for (uri, _) in &para.fors {
                    if !first {
                        txt.push_str(", ");
                    }
                    txt.push_str(uri.name().as_ref());
                    first = false;
                }
            }
            texts.push(txt);
            let title = para.title.as_ref().and_then(|ttl| {
                let txt = textify(ttl, true);
                if txt.is_empty() {
                    None
                } else {
                    texts.push(txt);
                    Some(Embedding::zero())
                }
            });

            indexes.push(SearchIndex::Paragraph {
                uri: para.uri.clone(),
                kind,
                definition_like: para.kind.is_definition_like(&para.styles),
                title,
                fors: para.fors.iter().map(|(u, _)| u).cloned().collect(),
                body: Embedding::zero(),
            });
        }
    }
    let Ok(results) = crate::Embedder::embed(&texts) else {
        todo!()
    };
    drop(texts);
    let mut results = results.into_iter();
    let mut idx_iter = indexes.iter_mut();
    let Some(SearchIndex::Document { title, body, .. }) = idx_iter.next() else {
        // SAFETY: we know that it contains at least once document at the start
        unsafe {
            use std::hint::unreachable_unchecked;
            unreachable_unchecked()
        }
    };
    // SAFETY: results.len() == texts.len()
    *body = unsafe { results.next().unwrap_unchecked() };
    if title.is_some() {
        // SAFETY: result.len() == texts.len() && title.is_some() iff there is a title in texts
        *title = Some(unsafe { results.next().unwrap_unchecked() });
    }
    for index in idx_iter {
        let SearchIndex::Paragraph { title, body, .. } = index else {
            // SAFETY: only the first element is a Document
            unsafe {
                use std::hint::unreachable_unchecked;
                unreachable_unchecked()
            }
        };
        // SAFETY: results.len() == texts.len()
        *body = unsafe { results.next().unwrap_unchecked() };
        if title.is_some() {
            // SAFETY: result.len() == texts.len() && title.is_some() iff there is a title in texts
            *title = Some(unsafe { results.next().unwrap_unchecked() });
        }
    }
    indexes
}
