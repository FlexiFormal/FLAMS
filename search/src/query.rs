use flams_backend_types::search::{QueryFilter, SearchResult, SearchResultKind};

#[must_use]
pub fn build_query(
    query: &str,
    index: &tantivy::Index,
    filter: QueryFilter,
) -> Option<Box<dyn tantivy::query::Query>> {
    use std::fmt::Write;
    let QueryFilter {
        allow_documents,
        allow_paragraphs,
        allow_definitions,
        allow_examples,
        allow_assertions,
        allow_problems,
        definition_like_only,
    } = filter;
    let mut s = String::new();
    if !allow_documents
        || !allow_paragraphs
        || !allow_definitions
        || !allow_examples
        || !allow_assertions
        || !allow_problems
    {
        //s.push('(');
        let mut had_first = false;
        if allow_documents {
            had_first = true;
            s.push_str("(kind:0");
        }
        if allow_paragraphs {
            s.push_str(if had_first { " OR kind:1" } else { "(kind:1" });
            had_first = true;
        }
        if allow_definitions {
            s.push_str(if had_first { " OR kind:2" } else { "(kind:2" });
            had_first = true;
        }
        if allow_examples {
            s.push_str(if had_first { " OR kind:3" } else { "(kind:3" });
            had_first = true;
        }
        if allow_assertions {
            s.push_str(if had_first { " OR kind:4" } else { "(kind:4" });
            had_first = true;
        }
        if allow_problems {
            s.push_str(if had_first { " OR kind:5" } else { "(kind:5" });
        }
        if had_first {
            s.push_str(") AND ");
        }
    }
    if definition_like_only {
        s.push_str("deflike:true AND ");
    }
    write!(s, "({query})").ok()?;
    let schema = crate::schema::SearchSchema::get();
    let mut parser = tantivy::query::QueryParser::for_index(index, vec![schema.fors,schema.uri, schema.title, schema.body]);
    //parser.set_field_fuzzy(SCHEMA.body, false, 1, true);
    parser.set_conjunction_by_default();
    parser.parse_query(&s).ok()
}

#[derive(Debug)]
pub(crate) struct Wrapper<T>(pub T);

impl tantivy::schema::document::ValueDeserialize for Wrapper<bool> {
    fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::ValueDeserializer<'de>,
    {
        Ok(Self(deserializer.deserialize_bool()?))
    }
}

impl tantivy::schema::document::ValueDeserialize for Wrapper<SearchResultKind> {
    fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::ValueDeserializer<'de>,
    {
        deserializer
            .deserialize_u64()?
            .try_into()
            .map(Wrapper)
            .map_err(|()| tantivy::schema::document::DeserializeError::custom(""))
    }
}

impl tantivy::schema::document::DocumentDeserialize for Wrapper<SearchResult> {
    fn deserialize<'de, D>(
        mut deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::DocumentDeserializer<'de>,
    {
        macro_rules! next {
            () => {{
                let Some((_, r)) = deserializer.next_field()? else {
                    return Err(tantivy::schema::document::DeserializeError::custom(
                        "Missing value",
                    ));
                };
                r
            }};
            (!) => {{
                let Some((_, Wrapper(r))) = deserializer.next_field()? else {
                    return Err(tantivy::schema::document::DeserializeError::custom(
                        "Missing value",
                    ));
                };
                r
            }};
        }
        let Wrapper(kind) = next!();
        match kind {
            SearchResultKind::Document => Ok(Self(SearchResult::Document(next!()))),
            kind => {
                let uri = next!();
                let def_like = next!(!);
                let mut fors = Vec::new();
                while let Some((_, s)) = deserializer.next_field()? {
                    fors.push(s);
                }
                Ok(Self(SearchResult::Paragraph {
                    uri,
                    def_like,
                    kind,
                    fors,
                }))
            }
        }
    }
}
