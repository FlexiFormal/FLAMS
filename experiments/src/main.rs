use flams_lsp::state::LSPState;
use flams_ontology::uris::{DocumentURI, URIRefTrait};
use flams_system::backend::{archives::{source_files::{SourceDir, SourceEntry}, Archive}, GlobalBackend};
use flams_utils::{prelude::TreeChildIter, time::measure};


pub fn main() {
  tracing_subscriber::fmt().init();
  use git_url_parse::GitUrl;
  let url = GitUrl::parse("https://gl.mathhub.info/smglom/foo")
    .expect("Failed to parse URL");
  tracing::info!("HTTPS: {url}\n{url:?}");
  let url2 = GitUrl::parse("git@gl.mathhub.info:smglom/foo.git")
    .expect("Failed to parse URL");
  tracing::info!("HTTPS: {url2}\n{url2:?}");

}

pub fn linter() {
  /*
  let mut rt = tokio::runtime::Builder::new_multi_thread();
  rt.enable_all();
  rt.thread_stack_size(2 * 1024 * 1024);
  rt.build()
    .expect("Failed to initialize Tokio runtime")
    .block_on(linter_i());
   */
}

async fn linter_i() {
  tracing_subscriber::fmt().init();
  let _ce = color_eyre::install();
  let mut spec = flams_system::settings::SettingsSpec::default();
  spec.lsp = true;
  flams_system::settings::Settings::initialize(spec);
  flams_system::backend::GlobalBackend::initialize();
  //flams_system::initialize(spec);
  let state = LSPState::default();
  tracing::info!("Waiting for stex to load...");
  std::thread::sleep(std::time::Duration::from_secs(5));
  tracing::info!("Go!");
  let (_,t) = measure(move || {
    tracing::info!("Loading all archives");
    let mut files = Vec::new();
    for a in GlobalBackend::get().all_archives().iter() {
      if let Archive::Local(a) =a { 
        a.with_sources(|d| for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
          match e {
            SourceEntry::File(f) => files.push((
              f.relative_path.split('/').fold(a.source_dir(),|p,s| p.join(s)).into(),
              DocumentURI::from_archive_relpath(a.uri().owned(), &f.relative_path)
          )),
            _ => {}
          }
        })
      }
    }
    let len = files.len();
    tracing::info!("Linting {len} files");
    state.load_all(files.into_iter()/*.enumerate().map(|(i,(path,uri))| {
      tracing::info!("{}/{len}: {}",i+1,path.display());
      (path,uri)
    })*/, |_,_| {});
  });
  tracing::info!("initialized after {t}");
}