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
use std::sync::Arc;
use flams_lsp::documents::LSPDocument;
use flams_lsp::state::DocData::{Data, Doc};
use flams_lsp::state::UrlOrFile::File;

use serde::Serialize;

extern crate tokio;

#[unsafe(no_mangle)]
pub extern "C" fn hello_world(arg: usize) {
    // use this as a test to see if the FFI works
    println!("Hi from Rust! arg: {}", arg);
}

static GLOBAL_STATE: std::sync::OnceLock<LSPState> = std::sync::OnceLock::new();

pub fn to_json<T: Serialize>(data: &T) -> *const libc::c_char {
    CString::new(serde_json::to_string(data).unwrap()).unwrap().into_raw()
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_file_json(path: *const libc::c_char) -> *const libc::c_char {
    let path_str: &str = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let state = GLOBAL_STATE.get().unwrap();
    let binding = state.documents.read();
    let doc = binding.get(&File(Path::new(path_str).into()));
    match doc {
        Some(Data(data, _)) => {
            to_json(&data.lock().annotations)
        }
        Some(Doc(_lspdoc)) => { CString::new("").unwrap().into_raw() }
        None => { CString::new("").unwrap().into_raw() }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_all_files() -> *const libc::c_char {
    let state = GLOBAL_STATE.get().unwrap();
    let binding = state.documents.read();
    let paths: Vec<&str> = binding.keys().map(|k| {
        match k {
            File(p) => p.as_ref().to_str().unwrap(),
            _ => ""
        }
    }).filter(|s| !s.is_empty()).collect();
    to_json(&paths)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_string(s: *mut libc::c_char) {
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
    tracing::info!("FINISHED");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn reload_document(path: *const libc::c_char) {
    let path_str: &str = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
    let state = GLOBAL_STATE.get().unwrap();
    let lspdoc = LSPDocument::new("".to_string(), File(Path::new(path_str).into()));
    state.load_all(
        vec!((Path::new(path_str).into(), lspdoc.document_uri().unwrap().clone())).into_iter(), |_,_| {});
}

async fn linter() {
    tracing_subscriber::fmt().init();
    let _ce = color_eyre::install();
    let mut spec = flams_system::settings::SettingsSpec::default();
    spec.lsp = true;
    flams_system::settings::Settings::initialize(spec);
    GlobalBackend::initialize();
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
