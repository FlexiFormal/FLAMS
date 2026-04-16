#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
use flams_backend_types::search::FragmentQueryFilter;
use flams_backend_types::search::{QueryFilter, SearchResult, SearchResultKind};
#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
use ftml_uris::{DocumentElementUri, DocumentUri};

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
#[must_use]
pub fn build_query(
    query: &str,
    index: &tantivy::Index,
    filter: FragmentQueryFilter,
) -> Option<Box<dyn tantivy::query::Query>> {
    use std::fmt::Write;
    let mut s = String::new();
    if !filter.flags.allow_documents()
        || !filter.flags.allow_paragraphs()
        || !filter.flags.allow_definitions()
        || !filter.flags.allow_examples()
        || !filter.flags.allow_assertions()
        || !filter.flags.allow_problems()
    {
        //s.push('(');
        let mut had_first = false;
        if filter.flags.allow_documents() {
            had_first = true;
            s.push_str("(kind:0");
        }
        if filter.flags.allow_paragraphs() {
            s.push_str(if had_first { " OR kind:1" } else { "(kind:1" });
            had_first = true;
        }
        if filter.flags.allow_definitions() {
            s.push_str(if had_first { " OR kind:2" } else { "(kind:2" });
            had_first = true;
        }
        if filter.flags.allow_examples() {
            s.push_str(if had_first { " OR kind:3" } else { "(kind:3" });
            had_first = true;
        }
        if filter.flags.allow_assertions() {
            s.push_str(if had_first { " OR kind:4" } else { "(kind:4" });
            had_first = true;
        }
        if filter.flags.allow_problems() {
            s.push_str(if had_first { " OR kind:5" } else { "(kind:5" });
        }
        if had_first {
            s.push_str(") AND ");
        }
    }
    if filter.flags.is_definition_like() {
        s.push_str("deflike:true AND ");
    }
    write!(s, "({query})").ok()?;
    let schema = crate::schema::SearchSchema::get();
    let mut parser = tantivy::query::QueryParser::for_index(
        index,
        vec![schema.fors, schema.uri, schema.title, schema.body],
    );
    //parser.set_field_fuzzy(SCHEMA.body, false, 1, true);
    parser.set_conjunction_by_default();
    parser.parse_query(&s).ok()
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
#[derive(Debug)]
pub(crate) struct Wrapper<T>(pub T);

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
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

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
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
            .map_err(|()| {
                tantivy::schema::document::DeserializeError::custom(format_args!("weird"))
            })
    }
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
impl tantivy::schema::document::DocumentDeserialize for Wrapper<SearchResult> {
    fn deserialize<'de, D>(
        mut deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::DocumentDeserializer<'de>,
    {
        macro_rules! next {
            ($name:literal) => {{
                let Some((_, r)) = deserializer.next_field()?.map_err(|e| {
                    tantivy::schema::document::DeserializeError::custom(format_args!(
                        "weird A: {e} (in {})",
                        $name
                    ))
                }) else {
                    return Err(tantivy::schema::document::DeserializeError::custom(
                        format_args!("Missing value {}", $name),
                    ));
                };
                r
            }};
            ($name:literal!) => {{
                let Some((_, Wrapper(r))) = deserializer.next_field().map_err(|e| {
                    tantivy::schema::document::DeserializeError::custom(format_args!(
                        "weird A: {e} (in {})",
                        $name
                    ))
                })?
                else {
                    return Err(tantivy::schema::document::DeserializeError::custom(
                        format_args!("Missing value {}", $name),
                    ));
                };
                r
            }};
        }
        let kind = next!("kind"!);
        match kind {
            SearchResultKind::Document => Ok(Self(SearchResult::Document(next!("uri"!)))),
            kind => {
                let uri = next!("uri"!);
                let def_like = next!("deflike"!);
                let mut fors = Vec::new();
                while let Ok(Some((_, s))) = deserializer.next_field() {
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

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
impl tantivy::schema::document::ValueDeserialize for Wrapper<DocumentUri> {
    fn deserialize<'de, D>(
        mut deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::ValueDeserializer<'de>,
    {
        //SAFETY: it's a string
        unsafe { String::from_utf8_unchecked(deserializer.deserialize_bytes()?) }
            .parse()
            .map_or_else(
                |_| {
                    Err(tantivy::schema::document::DeserializeError::custom(
                        "Invalid DocumentUri",
                    ))
                },
                |u| Ok(Wrapper(u)),
            )
    }
}

#[cfg(all(feature = "tantivy", not(feature = "vectorsearch")))]
impl tantivy::schema::document::ValueDeserialize for Wrapper<DocumentElementUri> {
    fn deserialize<'de, D>(
        mut deserializer: D,
    ) -> Result<Self, tantivy::schema::document::DeserializeError>
    where
        D: tantivy::schema::document::ValueDeserializer<'de>,
    {
        //SAFETY: it's a string
        unsafe { String::from_utf8_unchecked(deserializer.deserialize_bytes()?) }
            .parse()
            .map_or_else(
                |_| {
                    Err(tantivy::schema::document::DeserializeError::custom(
                        "Invalid DocumentElementUri",
                    ))
                },
                |u| Ok(Wrapper(u)),
            )
    }
}
