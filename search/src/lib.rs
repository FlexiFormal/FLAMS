#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

use flams_backend_types::search::{QueryFilter, SearchIndex, SearchResult};
use flams_math_archives::{
    Archive, LocallyBuilt,
    artifacts::{Artifact, ContentResult, FileOrString},
    backend::{AnyBackend, GlobalBackend, LocalBackend},
    build_target,
    formats::BuildResult,
    utils::errors::{ArtifactSaveError, FileError},
};
use flams_system::FlamsExtension;
use ftml_uris::{DocumentUri, SymbolUri, UriPath, UriWithArchive};

use crate::{index::SearchIndexExt, schema::SearchSchema};

pub mod index;
pub mod query;
pub mod schema;

flams_system::register_exension!(FlamsExtension {
    name: "tantivy_search",
    on_start: initialize,
    on_build_result: |b, uri, rel_path, a| if let Some(content) =
        a.as_any().downcast_ref::<ContentResult>()
    {
        index(b, uri, rel_path, content);
    }
});

build_target!(TANTIVY {
    name: "tantivy_search",
    description: "search index",
    run: |_| BuildResult::default()
});

const MEMORY_SIZE: usize = 50_000_000;
static SEARCHER: std::sync::LazyLock<Searcher> = std::sync::LazyLock::new(Searcher::new);
static SPAN: std::sync::LazyLock<tracing::Span> =
    std::sync::LazyLock::new(|| tracing::info_span!(target:"search",parent:None,"search"));

pub struct Searcher {
    index: parking_lot::RwLock<tantivy::index::Index>,
    reader: parking_lot::RwLock<tantivy::IndexReader>,
    writer: parking_lot::Mutex<()>,
}
impl Searcher {
    #[inline]
    #[must_use]
    pub fn get() -> &'static Self {
        &SEARCHER
    }

    fn new() -> Self {
        let index =
            tantivy::index::Index::create_in_ram(schema::SearchSchema::get().schema.clone());
        Self {
            reader: parking_lot::RwLock::new(index.reader().expect("Failed to build reader")),
            index: parking_lot::RwLock::new(index),
            writer: parking_lot::Mutex::new(()),
        }
    }

    pub fn query(
        &self,
        s: &str,
        opts: QueryFilter,
        num_results: usize,
    ) -> Option<Vec<(f32, SearchResult)>> {
        SPAN.in_scope(move || {
            let searcher = self.reader.read().searcher();
            let query = query::build_query(s, &self.index.read(), opts)?;
            let top_num = if num_results == 0 {
                usize::MAX / 2
            } else {
                num_results
            };
            let mut ret = Vec::new();
            for (s, a) in searcher
                .search(&*query, &tantivy::collector::TopDocs::with_limit(top_num))
                .ok()?
            {
                let query::Wrapper(r) = searcher.doc(a).ok()?;
                ret.push((s, r));
            }
            Some(ret)
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn query_symbols(
        &self,
        s: &str,
        num_results: usize,
    ) -> Option<Vec<(SymbolUri, Vec<(f32, SearchResult)>)>> {
        SPAN.in_scope(move || {
            const FILTER: QueryFilter = QueryFilter {
                allow_documents: false,
                allow_paragraphs: true,
                allow_definitions: true,
                allow_examples: false,
                allow_assertions: true,
                allow_problems: false,
                definition_like_only: true,
            };
            let searcher = self.reader.read().searcher();

            let query = query::build_query(s, &self.index.read(), FILTER)?;
            let top_num = if num_results == 0 {
                usize::MAX / 2
            } else {
                num_results
            };
            let mut ret: Vec<(SymbolUri, Vec<(f32, SearchResult)>)> = Vec::new();
            for (s, a) in searcher
                .search(
                    &*query,
                    &tantivy::collector::TopDocs::with_limit(top_num * 2),
                )
                .ok()?
            {
                let query::Wrapper(r): query::Wrapper<SearchResult> = searcher.doc(a).ok()?;
                if let SearchResult::Paragraph { fors, .. } = &r {
                    for sym in fors {
                        if let Some(v) = ret
                            .iter_mut()
                            .find_map(|(k, v)| if *k == *sym { Some(v) } else { None })
                        {
                            v.push((s, r.clone()));
                        } else {
                            ret.push((sym.clone(), vec![(s, r.clone())]));
                        }
                    }
                }
            }
            if ret.len() > num_results {
                let _ = ret.split_off(num_results);
            }
            Some(ret)
        })
    }
}

fn index(backend: &AnyBackend, uri: &DocumentUri, rel_path: &UriPath, result: &ContentResult) {
    backend.with_buildable_archive(uri.archive_id(), |a| {
        if let Some(a) = a {
            let it: Vec<_> = index::index_document(&result.document, &result.ftml).collect();
            let _ = a.save(
                uri,
                Some(rel_path),
                FileOrString::Str(String::new().into_boxed_str()),
                TANTIVY.id(),
                Some(Box::new(IndexFile(it)) as _),
                GlobalBackend.triple_store(),
                false,
            );
        }
    });
}

struct IndexFile(Vec<SearchIndex>);
impl Artifact for IndexFile {
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as _
    }
    fn kind(&self) -> &'static str {
        "tantivy"
    }
    fn write(&self, into: &std::path::Path) -> Result<(), ArtifactSaveError> {
        let file = std::fs::File::create(into)
            .map_err(|e| ArtifactSaveError::Fs(FileError::Creation(into.to_path_buf(), e)))?;
        bincode::serde::encode_into_std_write(
            &self.0,
            &mut std::io::BufWriter::new(file),
            bincode::config::standard(),
        )
        .map_err(ArtifactSaveError::Encode)?;
        Ok(())
    }
}

fn initialize() {
    SPAN.in_scope(|| {
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
        let index = tantivy::index::Index::create_in_ram(SearchSchema::get().schema.clone());
        let mut writer = index
            .writer(MEMORY_SIZE)
            .expect("Failed to instantiate search writer");
        let wr = &writer;
        tracing::info_span!("Loading search indices").in_scope(move || {
            GlobalBackend
                .all_archives()
                .par_iter()
                .filter_map(|a| match a {
                    Archive::Local(a) => Some(a),
                    Archive::Ext(_, _) => None,
                })
                .for_each(|a| {
                    let out = a.out_dir();
                    if out.exists() && out.is_dir() {
                        for e in walkdir::WalkDir::new(out)
                            .into_iter()
                            .filter_map(Result::ok)
                            .filter(|entry| entry.file_name() == "tantivy")
                        {
                            let Ok(f) = std::fs::File::open(e.path()) else {
                                tracing::error!("error reading file {}", e.path().display());
                                return;
                            };
                            let file = std::io::BufReader::new(f);

                            let Ok(v): Result<Vec<SearchIndex>, _> =
                                bincode::serde::decode_from_reader(
                                    file,
                                    bincode::config::standard(),
                                )
                            else {
                                tracing::error!("error deserializing file {}", e.path().display());
                                return;
                            };
                            for d in v {
                                let d: tantivy::TantivyDocument = d.to_document();
                                if let Err(e) = wr.add_document(d) {
                                    tracing::error!("{e}");
                                }
                            }
                        }
                    }
                });
        });
        match writer.commit() {
            Ok(i) => tracing::info!("Loaded {i} entries"),
            Err(e) => tracing::error!("Error: {e}"),
        }
        let slf = Searcher::get();
        let writer = slf.writer.lock();
        let mut old_index = slf.index.write();
        let mut reader = slf.reader.write();
        let Ok(r) = index.reader() else {
            tracing::error!("Failed to instantiate search reader");
            return;
        };
        *reader = r;
        *old_index = index;
        drop(reader);
        drop(old_index);
        drop(writer);
    });
}
