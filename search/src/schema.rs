#[cfg(feature = "tantivy")]
pub struct SearchSchema {
    #[allow(dead_code)]
    pub schema: tantivy::schema::Schema,
    pub uri: tantivy::schema::Field,
    pub uri_str: tantivy::schema::Field,
    pub kind: tantivy::schema::Field,
    pub title: tantivy::schema::Field,
    pub body: tantivy::schema::Field,
    pub fors: tantivy::schema::Field,
    pub def_like: tantivy::schema::Field,
}

#[cfg(feature = "tantivy")]
impl SearchSchema {
    #[inline]
    #[must_use]
    pub fn get() -> &'static Self {
        static SCHEMA: std::sync::LazyLock<SearchSchema> = std::sync::LazyLock::new(|| {
            use tantivy::schema::{FAST, INDEXED, STORED, Schema, TEXT};
            /*
            let text_field_indexing = tantivy::schema::TextFieldIndexing::default()
              .set_tokenizer("ngram3")
              .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions);
            let txt_opts = tantivy::schema::TextOptions::default().set_indexing_options(text_field_indexing);
             */

            let mut schema = Schema::builder();
            let kind = schema.add_u64_field("kind", INDEXED | STORED);
            let uri = schema.add_bytes_field("uri", FAST);
            let uri_str = schema.add_text_field("uri_str", STORED);
            let def_like = schema.add_bool_field("deflike", INDEXED | STORED);
            let fors = schema.add_text_field("for", STORED);
            let title = schema.add_text_field("title", TEXT);
            let body = schema.add_text_field("body", TEXT); //txt_opts);//TEXT);

            let schema = schema.build();
            SearchSchema {
                schema,
                uri,
                uri_str,
                kind,
                title,
                body,
                fors,
                def_like,
            }
        });

        &SCHEMA
    }
}
