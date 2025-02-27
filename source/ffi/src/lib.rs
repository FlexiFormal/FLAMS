use flams_lsp::state::LSPState;
use flams_system::backend::GlobalBackend;
use flams_system::backend::archives::Archive;
use flams_system::backend::archives::source_files::{SourceDir, SourceEntry};
use flams_utils::prelude::TreeChildIter;
use flams_utils::time::measure;
use flams_ontology::uris::DocumentURI;
use flams_ontology::uris::URIRefTrait;

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use flams_lsp::documents::LSPDocument;
use flams_lsp::state::DocData::{Data, Doc};
use flams_lsp::state::UrlOrFile::File;

extern crate tokio;

#[unsafe(no_mangle)]
pub extern "C" fn hello_world(arg: usize) {
    println!("Hi from Rust! arg: {}", arg);
}

static GLOBAL_STATE: std::sync::OnceLock<LSPState> = std::sync::OnceLock::new();


#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_file_json(path: *const libc::c_char) -> *const libc::c_char {
    let path_str: &str = std::ffi::CStr::from_ptr(path).to_str().unwrap();
    let state = GLOBAL_STATE.get().unwrap();
    let binding = state.documents.read();
    let doc = binding.get(&File(Path::new(path_str).into()));
    match doc {
        Some(Data(data, _)) => {
            // lspdoc.compute_annots(state.clone());
            CString::new(serde_json::to_string(&data.lock().annotations).unwrap()).unwrap().into_raw()
        }
        Some(Doc(lspdoc)) => {
            // println!("is doc");
            CString::new("").unwrap().into_raw()
        }
        None => {
            // println!("No document found for {:?}", path_str);
            CString::new("").unwrap().into_raw()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut libc::c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        drop(CString::from_raw(s));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn initialize_lspstate() {
    let mut rt = tokio::runtime::Builder::new_multi_thread();
    // rt.worker_threads(16);
    rt.enable_all();
    rt.thread_stack_size(4 * 1024 * 1024);
    rt.build().expect("Failed to initialize Tokio runtime").block_on(linter());
    println!("FINISHED");
}

async fn linter() {
    tracing_subscriber::fmt().init();
    let _ce = color_eyre::install();
    let mut spec = flams_system::settings::SettingsSpec::default();
    spec.lsp = true;
    flams_system::initialize(spec);
    let state = LSPState::default();
    let _ = GLOBAL_STATE.set(state.clone());
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
        state.load_all(files.into_iter(), |_,_| {});
    });
    tracing::info!("initialized after {t}");
}


/*
#[unsafe(no_mangle)]
pub extern "C" fn print_docs() {
    let state = GLOBAL_STATE.get().unwrap();
    for (uri,doc) in state.documents.read().iter() {
        println!("{:?} -> ...",uri);
        match state.documents.read().get(uri) {
            Some(lspdoc) => {
                println!("  YES1");
            }
            None => {
                println!("  NO1");
            }
        }
        match doc {
            Doc(lspdoc) => {
                println!("  (doc) ");
            }
            Data(stpd, _) => {
                println!("  (data)");
                // println!("      {:?}", serde_json::json!(stpd.lock().annotations[0]).to_string())
                // println!("      {:?}", stpd.lock().annotations[0])
                /*{
                    println!("    {:?}",a);
                }*/
            }
        }
    }
    // println!("{:?}", state.get(File(Cow::Borrowed("file:///home/alex/Downloads/Flams/Flams/flams-system/src/settings.rs"))).);
}
*/
