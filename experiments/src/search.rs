use flams_ontology::{narration::{documents::Document, paragraphs::{LogicalParagraph, ParagraphKind}, DocumentElement, NarrationTrait}, search::{QueryFilter, SearchResult, SearchResultKind}, uris::{DocumentElementURI, DocumentURI, SymbolURI, URIRefTrait}, Checked};
use flams_system::{backend::{archives::{source_files::SourceDir, Archive}, Backend, GlobalBackend}, search::Searcher};
use flams_utils::prelude::TreeChildIter;
use rstest::{fixture,rstest};

#[fixture]
fn do_tracing() {
  use flams_stex::*;
  tracing_subscriber::fmt().init();
  color_eyre::install().expect("failed");
}

#[fixture]
fn setup(do_tracing:()) {
  let mut spec = flams_system::settings::SettingsSpec::default();
  spec.lsp = true;
  flams_system::settings::Settings::initialize(spec);
  GlobalBackend::initialize();
}

fn all_documents() -> Vec<DocumentURI> {
  let mut uris = Vec::new();
  for a in GlobalBackend::get().all_archives().iter().filter_map(|a|
    if let Archive::Local(a) = a { Some(a) } else { None }
  ) {
    a.with_sources(|f| {
      for c in <_ as TreeChildIter<SourceDir>>::dfs(f.children.iter()) {
        uris.push(DocumentURI::from_archive_relpath(a.uri().owned(),&c.relative_path()));
      }
    })
  }
  uris
}

fn stex_documents() -> Vec<DocumentURI> {
  let mut uris = Vec::new();
  for a in GlobalBackend::get().all_archives().iter().filter(|a| a.id().steps().next() == Some("sTeX")).filter_map(|a|
    if let Archive::Local(a) = a { Some(a) } else { None }
  ) {
    a.with_sources(|f| {
      for c in <_ as TreeChildIter<SourceDir>>::dfs(f.children.iter()) {
        uris.push(DocumentURI::from_archive_relpath(a.uri().owned(),&c.relative_path()));
      }
    })
  }
  uris
}

fn do_document(doc:&Document) -> Vec<tantivy::TantivyDocument> {

  let Some(html) = GlobalBackend::get().get_html_full(doc.uri()) else {return Vec::new() };
  doc.all_searches(&html).into_iter().map(Into::into).collect()
}








#[rstest]
#[tokio::test]
async fn search_test(setup:()) {
  use std::path::Path;
  tracing::info!("Henlo!");
  //let path = Path::from("/home/jazzpirate/.flams/search");
  //std::fs::create_dir_all(path);
  //let index = tantivy::index::Index::create_in_dir(path, SCHEMA.clone()).expect("Failed to create serach directory");
  //let index = tantivy::index::Index::create_in_ram(SCHEMA.schema.clone());
  //let mut index_writer = index.writer(50_000_000).expect("Failed to index write");
  //let mut js = tokio::task::JoinSet::new();
  let searcher = Searcher::get();
  searcher.with_writer(|index_writer| {
    for d in stex_documents() {
      if let Some(doc) = GlobalBackend::get().get_document(&d) {
        for d in do_document(&doc) {
          index_writer.add_document(d).expect("adding document error");
        }
      } 
    }
    Ok(())
  }).expect("adding document error");

  let res = searcher.query("directed graph",QueryFilter::default(),10).expect("Query failed");
  for (score,doc) in res {
    tracing::info!("{score}: {:?}",doc);
  }
}