use flams_backend_types::search::{SearchIndex, SearchResultKind};
use ftml_ontology::{
    narrative::{
        documents::Document,
        elements::{DocumentElementRef, LogicalParagraph},
    },
    utils::RefTree,
};

pub trait SearchIndexExt {
    fn to_document(self) -> tantivy::TantivyDocument;
}

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

pub fn index_document(doc: &Document, html: &str) -> impl Iterator<Item = SearchIndex> {
    let elems = doc.dfs().filter_map(|e| {
        if let DocumentElementRef::Paragraph(p) = e {
            index_paragraph(p, html)
        } else {
            None
        }
    });
    if let Some(s) = index_document_html(doc, html) {
        either::Left(std::iter::once(s).chain(elems))
    } else {
        either::Right(elems)
    }
}

#[must_use]
pub fn index_document_html(doc: &Document, html: &str) -> Option<SearchIndex> {
    let title = doc.title.as_ref().map(|s| html_to_search_text(s));
    let body = html_to_search_text(html);
    Some(SearchIndex::Document {
        uri: doc.uri.clone(),
        title,
        body,
    })
}

pub fn index_paragraph(para: &LogicalParagraph, html: &str) -> Option<SearchIndex> {
    let title = para.title.as_ref().map(|s| html_to_search_text(s));
    let Some(body) = html.get(para.range.start..para.range.end) else {
        tracing::error!("Failed to plain textify body of {}", para.uri);
        return None;
    };
    let body = html_to_search_text(body);
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
}

#[must_use]
pub fn html_to_search_text(html: &str) -> String {
    fn replacer(s: &mut String) {
        let mut i = 0;
        loop {
            match s.as_bytes().get(i..i + 2) {
                None => return,
                Some(b".\n" | b"!\n" | b":\n" | b";\n") => i += 2,
                Some(b) if b[0] == b'\n' => {
                    s.remove(i);
                }
                _ => i += 1,
            }
        }
    }
    let Ok(mut s) = html2text::from_read(html.as_bytes(), usize::MAX / 3) else {
        return html.to_string();
    };
    replacer(&mut s);
    s
}
