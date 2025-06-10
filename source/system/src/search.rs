use flams_ontology::{
    search::{QueryFilter, SearchIndex, SearchResult, SearchSchema},
    uris::SymbolURI,
};
use flams_utils::vecmap::VecMap;

use crate::backend::{
    archives::{Archive, HasLocalOut},
    GlobalBackend,
};

const MEMORY_SIZE: usize = 50_000_000;

pub struct Searcher {
    index: parking_lot::RwLock<tantivy::index::Index>,
    reader: parking_lot::RwLock<tantivy::IndexReader>,
    writer: parking_lot::Mutex<()>,
}
impl Searcher {
    fn new() -> Self {
        let index = tantivy::index::Index::create_in_ram(SearchSchema::get().schema.clone());
        Self {
            reader: parking_lot::RwLock::new(index.reader().expect("Failed to build reader")),
            index: parking_lot::RwLock::new(index),
            writer: parking_lot::Mutex::new(()),
        }
    }
}

lazy_static::lazy_static! {
  static ref SEARCHER : Searcher = Searcher::new();
  static ref SPAN: tracing::Span = tracing::info_span!(target:"tantivy",parent:None,"search");
}

struct WriterWrapper(tantivy::IndexWriter);
impl Drop for WriterWrapper {
    fn drop(&mut self) {
        match self.0.commit() {
            Ok(i) => tracing::info!("Loaded {i} entries"),
            Err(e) => tracing::error!("Error: {e}"),
        }
    }
}

impl Searcher {
    #[inline]
    #[must_use]
    pub fn get() -> &'static Self {
        &SEARCHER
    }

    /// #### Panics
    pub fn reload(&self) {
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
        SPAN.in_scope(move || {
            let index = tantivy::index::Index::create_in_ram(SearchSchema::get().schema.clone());
            let writer = WriterWrapper(
                index
                    .writer(MEMORY_SIZE)
                    .expect("Failed to instantiate search writer"),
            );
            tracing::info_span!("Loading search indices").in_scope(move || {
                GlobalBackend::get()
                    .all_archives()
                    .par_iter()
                    .filter_map(|a| match a {
                        Archive::Local(a) => Some(a),
                        #[allow(unreachable_patterns)]
                        _ => None,
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
                                    tracing::error!(
                                        "error deserializing file {}",
                                        e.path().display()
                                    );
                                    return;
                                };
                                for d in v {
                                    let d: tantivy::TantivyDocument = d.into();
                                    if let Err(e) = writer.0.add_document(d) {
                                        tracing::error!("{e}");
                                    }
                                }
                            }
                        }
                    });
            });
            let writer = self.writer.lock();
            let mut old_index = self.index.write();
            let mut reader = self.reader.write();
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

    /// #### Errors
    #[allow(clippy::result_unit_err)]
    pub fn with_writer<R>(
        &self,
        f: impl FnOnce(&mut tantivy::IndexWriter) -> Result<R, ()>,
    ) -> Result<R, ()> {
        SPAN.in_scope(move || {
            let lock = self.writer.lock();
            let mut write = self.index.read().writer(MEMORY_SIZE).map_err(|_| ())?;
            let r = f(&mut write)?;
            let i = write.commit().map_err(|_| ())?;
            tracing::info!("Added {i} documents to search index");
            *self.reader.write() = self.index.read().reader().map_err(|_| ())?;
            drop(lock);
            Ok(r)
        })
    }

    pub fn query(
        &self,
        s: &str,
        opts: QueryFilter,
        num_results: usize,
    ) -> Option<Vec<(f32, SearchResult)>> {
        SPAN.in_scope(move || {
            let searcher = self.reader.read().searcher();
            let query = opts.to_query(s, &self.index.read())?;
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
                let r = searcher.doc(a).ok()?;
                ret.push((s, r));
            }
            Some(ret)
        })
    }
    pub fn query_symbols(
        &self,
        s: &str,
        num_results: usize,
    ) -> Option<VecMap<SymbolURI, Vec<(f32, SearchResult)>>> {
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
            let query = FILTER.to_query(s, &self.index.read())?;
            let top_num = if num_results == 0 {
                usize::MAX / 2
            } else {
                num_results
            };
            let mut ret = VecMap::new();
            for (s, a) in searcher
                .search(
                    &*query,
                    &tantivy::collector::TopDocs::with_limit(top_num * 2),
                )
                .ok()?
            {
                let r: SearchResult = searcher.doc(a).ok()?;
                if let SearchResult::Paragraph { fors, .. } = &r {
                    for sym in fors {
                        ret.get_or_insert_mut(sym.clone(), Vec::new)
                            .push((s, r.clone()));
                    }
                }
            }
            if ret.0.len() > num_results {
                let _ = ret.0.split_off(num_results);
            }
            Some(ret)
        })
    }
}
