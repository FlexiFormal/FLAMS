use std::path::Path;

use flams_ontology::{
    narration::{
        documents::Document,
        paragraphs::{LogicalParagraph, ParagraphKind},
        DocumentElement, NarrationTrait,
    },
    search::{QueryFilter, SearchResult, SearchResultKind, SearchSchema},
    uris::{DocumentElementURI, DocumentURI, SymbolURI, URIRefTrait},
    Checked,
};
use flams_system::{
    backend::{
        archives::{source_files::SourceDir, Archive},
        Backend, GlobalBackend,
    },
    search::Searcher,
};
use flams_utils::{prelude::TreeChildIter, unwrap};
use rstest::{fixture, rstest};

#[fixture]
fn do_tracing() {
    use flams_stex::*;
    tracing_subscriber::fmt().init();
    color_eyre::install().expect("failed");
}

#[fixture]
fn setup(do_tracing: ()) {
    let mut spec = flams_system::settings::SettingsSpec::default();
    spec.lsp = true;
    flams_system::settings::Settings::initialize(spec);
    GlobalBackend::initialize();
}

fn all_documents() -> Vec<DocumentURI> {
    let mut uris = Vec::new();
    for a in GlobalBackend::get().all_archives().iter().filter_map(|a| {
        if let Archive::Local(a) = a {
            Some(a)
        } else {
            None
        }
    }) {
        a.with_sources(|f| {
            for c in <_ as TreeChildIter<SourceDir>>::dfs(f.children.iter()) {
                uris.push(unwrap!(DocumentURI::from_archive_relpath(
                    a.uri().owned(),
                    &c.relative_path()
                )
                .ok()));
            }
        })
    }
    uris
}

fn stex_documents() -> Vec<DocumentURI> {
    let mut uris = Vec::new();
    for a in GlobalBackend::get()
        .all_archives()
        .iter()
        .filter(|a| a.id().steps().next() == Some("sTeX"))
        .filter_map(|a| {
            if let Archive::Local(a) = a {
                Some(a)
            } else {
                None
            }
        })
    {
        a.with_sources(|f| {
            for c in <_ as TreeChildIter<SourceDir>>::dfs(f.children.iter()) {
                uris.push(unwrap!(DocumentURI::from_archive_relpath(
                    a.uri().owned(),
                    &c.relative_path()
                )
                .ok()));
            }
        })
    }
    uris
}

fn do_document(doc: &Document) -> Vec<tantivy::TantivyDocument> {
    let Some(html) = GlobalBackend::get().get_html_full(doc.uri()) else {
        return Vec::new();
    };
    doc.all_searches(&html)
        .into_iter()
        .map(Into::into)
        .collect()
}

#[rstest]
#[tokio::test]
async fn single_test(do_tracing: ()) {
    let path = Path::new("/home/jazzpirate/work/MathHub/sTeX/ComputerScience/Software/.flams/mod/systems/tex/sTeX.en.tex/ftml_doc");
    macro_rules! LB {
        () => {
            "%*~LINEBREAK~*%"
        };
    }
    fn replacer(s: &mut String) {
        let mut i = 0;
        macro_rules! curr {
            () => {
                s.as_str()[i..]
            };
        }
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
    let html = std::fs::read_to_string(path).expect("exists");
    let mut txt = html2text::from_read(html.as_bytes(), usize::MAX / 3).expect("duh");
    tracing::info!("txt:\n\n{txt}\n");
    replacer(&mut txt);
    tracing::info!("txt:\n\n{txt}\n");
    //let index = tantivy::index::Index::create_in_ram(SearchSchema::get().schema.clone());
    //index.writer(50_000_000).expect("wut");
}

#[rstest]
#[tokio::test]
async fn search_test(setup: ()) {
    tracing::info!("Henlo!");
    //let path = Path::from("/home/jazzpirate/.flams/search");
    //std::fs::create_dir_all(path);
    //let index = tantivy::index::Index::create_in_dir(path, SCHEMA.clone()).expect("Failed to create serach directory");
    //let index = tantivy::index::Index::create_in_ram(SCHEMA.schema.clone());
    //let mut index_writer = index.writer(50_000_000).expect("Failed to index write");
    //let mut js = tokio::task::JoinSet::new();
    let searcher = Searcher::get();
    searcher
        .with_writer(|index_writer| {
            for d in stex_documents() {
                if let Some(doc) = GlobalBackend::get().get_document(&d) {
                    for d in do_document(&doc) {
                        index_writer.add_document(d).expect("adding document error");
                    }
                }
            }
            Ok(())
        })
        .expect("adding document error");

    let res = searcher
        .query("directed graph", QueryFilter::default(), 10)
        .expect("Query failed");
    for (score, doc) in res {
        tracing::info!("{score}: {:?}", doc);
    }
}
