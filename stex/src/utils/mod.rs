use std::path::{Path, PathBuf};
use immt_api::backend::archives::Archive;
use immt_api::backend::manager::ArchiveTree;
use immt_api::core::uris::archives::ArchiveId;

pub(crate) fn file_path_from_archive(
    current: &Path,
    id:&ArchiveId,
    module:&str,
    tree:&ArchiveTree,
    yields:&[&str]
) -> Option<Box<str>> {
    let lang = if current.extension().and_then(|s| s.to_str()) == Some("tex") {
        match current.file_stem().and_then(|s| s.to_str()) {
            Some(s) if s.ends_with(".ru") => "ru",
            Some(s) if s.ends_with(".de") => "de",
            Some(s) if s.ends_with(".fr") => "fr",
            // TODO etc
            _ => "en",
        }
    } else {
        "en"
    };
    if let Some(Archive::Physical(a)) = tree.find_archive(id) {
        let archive_path = a.path();
        let (path, mut module) = if let Some((a,b)) = module.split_once('?') {
            (a,b)
        } else {("",module)};
        module = module.split('/').next().unwrap_or(module);
        let p = PathBuf::from(format!(
            "{}/source/{path}/{module}.{lang}.tex",
            archive_path.display()
        ));
        if p.exists() {
            return Some(format!("{path}/{module}.{lang}.tex").into())
        }
        let p = PathBuf::from(format!(
            "{}/source/{path}/{module}.en.tex",
            archive_path.display()
        ));
        if p.exists() {
            return Some(format!("{path}/{module}.en.tex").into());
        }
        let p = PathBuf::from(format!(
            "{}/source/{path}/{module}.tex",
            archive_path.display()
        ));
        if p.exists() {
            return Some(format!("{path}/{module}.tex").into())
        }
        let p = PathBuf::from(format!(
            "{}/source/{path}.{lang}.tex",
            archive_path.display()
        ));
        if p.exists() {
            return Some(format!("{path}.{lang}.tex").into())
        }
        let p = PathBuf::from(format!("{}/source/{path}.en.tex", archive_path.display()));
        if p.exists() {
            return Some(format!("{path}.en.tex").into())
        }
        let p = PathBuf::from(format!("{}/source/{path}.tex", archive_path.display()));
        if p.exists() {
            return Some(format!("{path}.tex").into())
        }
        if yields.contains(&module) {
            return None
        }
    }
    None
}