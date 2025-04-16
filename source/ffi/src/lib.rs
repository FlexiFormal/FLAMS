#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use flams_lsp::state::{DocData, UrlOrFile};
use flams_ontology::uris::DocumentURI;
use flams_ontology::uris::URIRefTrait;
use flams_system::backend::archives::source_files::{SourceDir, SourceEntry};
use flams_system::backend::archives::Archive;
use flams_system::backend::GlobalBackend;

use flams_lsp::documents::LSPDocument;
use flams_lsp::state::DocData::{Data, Doc};
use flams_lsp::state::UrlOrFile::File;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};

use flams_lsp::LSPStore;
use flams_utils::prelude::{HMap, TreeChildIter};
use serde::Serialize;

extern crate tokio;

#[unsafe(no_mangle)]
pub extern "C" fn hello_world(arg: usize) {
    // use this as a test to see if the FFI works
    println!("Hi from Rust! arg: {}", arg);
}

pub fn to_json<T: Serialize>(data: &T) -> *const libc::c_char {
    CString::new(serde_json::to_string(data).unwrap())
        .unwrap()
        .into_raw()
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

static GLOBAL_STATE: LazyLock<Mutex<HMap<UrlOrFile, DocData>>> =
    LazyLock::new(|| Mutex::new(HMap::default()));

#[unsafe(no_mangle)]
pub extern "C" fn initialize() {
    tracing_subscriber::fmt().init();
    let _ce = color_eyre::install();
    let spec = flams_system::settings::SettingsSpec::default();
    // spec.lsp = true;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime")
        .block_on(async {
            flams_system::settings::Settings::initialize(spec);
            GlobalBackend::initialize();
        });
}

#[unsafe(no_mangle)]
pub extern "C" fn load_all_files() {
    let mut files: Vec<(Arc<Path>, DocumentURI)> = Vec::new();
    for a in GlobalBackend::get().all_archives().iter() {
        if let Archive::Local(a) = a {
            a.with_sources(|d| {
                for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
                    match e {
                        SourceEntry::File(f) => {
                            let Ok(uri) = DocumentURI::from_archive_relpath(a.uri().owned(), &f.relative_path) else { continue};
                            files.push((
                                f.relative_path
                                    .split('/')
                                    .fold(a.source_dir(), |p, s| p.join(s))
                                    .into(),
                                uri
                            ));
                        }
                        _ => {}
                    }
                }
            })
        }
    }
    let len = files.len();
    tracing::info!("Linting {len} files");

    let mut state = GLOBAL_STATE.lock().unwrap();
    state.clear();
    // let mut lspstore = LSPStore::<true>::new(&mut state);
    for (p, uri) in files {
        if let Some(ret) = LSPStore::<true>::new(&mut state).load(p.as_ref(), &uri) {
            state.insert(File(p.clone()), Data(ret, true));
        }
    }
    tracing::info!("Finished linting {len} files");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_of_loaded_files() -> *const libc::c_char {
    let state = GLOBAL_STATE.lock().unwrap();
    let paths: Vec<&str> = state
        .keys()
        .map(|k| match k {
            File(p) => p.as_ref().to_str().unwrap(),
            x => {
                tracing::warn!("Unexpected key: {:?}", x);
                ""
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    to_json(&paths)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_file_annotations(path: *const libc::c_char) -> *const libc::c_char {
    let path_str: &str = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let state = GLOBAL_STATE.lock().unwrap();
    let doc = state.get(&File(Path::new(path_str).into()));
    match doc {
        Some(Data(data, _)) => to_json(&data.lock().annotations),
        Some(Doc(_lspdoc)) => CString::new("").unwrap().into_raw(),
        None => CString::new("").unwrap().into_raw(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unload_file(path: *const libc::c_char) {
    let path_str: &str = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.remove(&File(Path::new(path_str).into()));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_file(path: *const libc::c_char) {
    let path_str: &str = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    let mut state = GLOBAL_STATE.lock().unwrap();

    let lspdoc = LSPDocument::new("".to_string(), File(Path::new(path_str).into()));
    let p = Path::new(path_str);
    let uri: &DocumentURI = lspdoc.document_uri().unwrap();
    if let Some(ret) = LSPStore::<true>::new(&mut state).load(p.as_ref(), &uri) {
        state.insert(File(p.into()), Data(ret, true));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn unload_all_files() {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.clear();
}
