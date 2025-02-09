use flams_ontology::{archive_json::{ArchiveDatum, Institution}, uris::{ArchiveId, ArchiveURI, ArchiveURITrait, BaseURI, DocumentURI}};
use flams_utils::vecmap::{VecMap, VecSet};
use parking_lot::RwLock;
use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

use crate::{backend::archives::{
    ignore_regex::IgnoreSource, source_files::SourceDir, RepositoryData,
}, formats::SourceFormat};

use super::{ArchiveIndex, LocalArchive};

pub(super) struct ArchiveIterator<'a> {
    path: &'a Path,
    stack: Vec<Vec<(PathBuf, String)>>,
    curr: Option<std::fs::ReadDir>,
    currp: String,
    in_span: tracing::span::Span,
}

impl<'a> ArchiveIterator<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self {
            stack: vec![vec![]],
            curr: std::fs::read_dir(path)
                .map_err(|_| {
                    tracing::warn!(target:"archives","Could not read directory {}", path.display());
                })
                .ok(),
            path,
            currp: String::new(),
            in_span: tracing::Span::current(),
        }
    }

    fn next(
        curr: &mut Option<ReadDir>,
        stack: &mut Vec<Vec<(PathBuf, String)>>,
        currp: &mut String,
    ) -> Option<LocalArchive> {
        loop {
            let d = match curr.as_mut().and_then(ReadDir::next) {
                None => {
                    if Self::next_dir(stack, curr, currp) {
                        continue;
                    }
                    return None;
                }
                Some(Ok(d)) => d,
                _ => continue,
            };
            let Ok(md) = d.metadata() else { continue };
            let path = d.path();

            //let _span = tracing::debug_span!(target:"archives","checking","{}",path.display()).entered();
            if md.is_dir() {
                if d.file_name().to_str().map_or(true, |s| s.starts_with('.')) {
                    continue;
                } else if d.file_name().eq_ignore_ascii_case("meta-inf") {
                    if let Some(path) = Self::find_manifest(&path) {
                        stack.pop();
                        return if let Some(m) = Self::do_manifest(&path, currp) {
                            if !Self::next_dir(stack, curr, currp) {
                                *curr = None;
                            }
                            Some(m)
                        } else {
                            if Self::next_dir(stack, curr, currp) {
                                continue;
                            }
                            None
                        };
                    }
                }
                let mut ins = currp.clone();
                if !ins.is_empty() {
                    ins.push('/');
                }
                ins.push_str(d.file_name().to_str().unwrap_or_else(|| unreachable!()));
                stack
                    .last_mut()
                    .unwrap_or_else(|| unreachable!())
                    .push((path, ins));
            }
        }
    }

    fn next_dir(
        stack: &mut Vec<Vec<(PathBuf, String)>>,
        curr: &mut Option<std::fs::ReadDir>,
        currp: &mut String,
    ) -> bool {
        loop {
            match stack.last_mut() {
                None => return false,
                Some(s) => match s.pop() {
                    Some((e, s)) => {
                        *curr = if let Ok(rd) = e.read_dir() {
                            Some(rd)
                        } else {
                            tracing::warn!(target:"archives","Could not read directory {}", e.display());
                            return false;
                        };
                        *currp = s;
                        stack.push(Vec::new());
                        return true;
                    }
                    None => {
                        stack.pop();
                    }
                },
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn find_manifest(metainf: &Path) -> Option<PathBuf> {
        tracing::trace!("Checking for manifest");
        if let Ok(rd) = metainf.read_dir() {
            for d in rd {
                let d = match d {
                    Err(_) => {
                        tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
                        continue;
                    }
                    Ok(d) => d,
                };
                if !d.file_name().eq_ignore_ascii_case("manifest.mf") {
                    continue;
                }
                let path = d.path();
                if !path.is_file() {
                    continue;
                }
                return Some(path);
            }
            tracing::trace!("not found");
        } else {
            tracing::warn!(target:"archives","Could not read directory {}", metainf.display());
        }
        None
    }

    #[allow(clippy::cognitive_complexity)]
    fn do_manifest(path: &Path, id: &str) -> Option<LocalArchive> {
        use std::io::BufRead;
        let Some(top_dir) = path.parent().and_then(Path::parent) else {
            tracing::warn!(target:"archives","Could not find parent directory of {}", path.display());
            return None;
        };
        let out_path = top_dir.join(".flams");
        let Ok(reader) = std::fs::File::open(path) else {
            tracing::warn!(target:"archives","Could not open manifest {}", path.display());
            return None;
        };
        let reader = std::io::BufReader::new(reader);
        let mut lines = reader.lines();

        let mut formats = VecSet::default();
        let mut dom_uri: String = String::new();
        let mut dependencies = Vec::new();
        let mut ignore = IgnoreSource::default();
        let mut attributes: VecMap<Box<str>, Box<str>> = VecMap::default();
        let mut had_id: bool = false;
        loop {
            let line = match lines.next() {
                Some(Err(_)) => continue,
                Some(Ok(l)) => l,
                _ => break,
            };
            let (k, v) = match line.split_once(':') {
                Some((k, v)) => (k.trim(), v.trim()),
                _ => continue,
            };
            match k {
                "id" => {
                    if v != id {
                        tracing::warn!(target:"archives","Archive {v}'s id does not match its location ({id})");
                        return None;
                    } else if v.is_empty() {
                        tracing::warn!(target:"archives","Archive {v} has an empty id");
                        return None;
                    }
                    had_id = true;
                }
                "format" => {
                    formats = v
                        .split(',')
                        .filter_map(SourceFormat::get_from_str)
                        .collect();
                }
                "url-base" => dom_uri = v.into(),
                //"ns" => dom_uri = v.into(),
                "dependencies" => {
                    for d in v
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty() && *s != id)
                    {
                        dependencies.push(ArchiveId::new(d));
                    }
                }
                "ignore" => {
                    ignore = IgnoreSource::new(v, &top_dir.join("source")); //Some(v.into());
                }
                _ => {
                    attributes.insert(k.into(), v.into());
                }
            }
        }
        if !had_id {
            tracing::warn!(target:"archives","Archive {id} has no id");
            return None;
        }
        /*if dom_uri.ends_with(id) {
            dom_uri.split_off(id.len() + 1);
        }*/
        let id = ArchiveId::new(id);
        if formats.is_empty() && !id.is_meta() {
            tracing::warn!(target:"archives","No formats found for archive {}",id);
            return None;
        }
        if dom_uri.is_empty() {
            tracing::warn!(target:"archives","Archive {} has no URL base", id);
            return None;
        }
        let dom_uri: BaseURI = match dom_uri.parse() {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(target:"archives","Archive {} has an invalid URL base: {}", id, e);
                return None;
            }
        };
        let uri = dom_uri & id;
        let (institutions,index) = read_index_file(&uri,&path.with_file_name("archive.json"));
        Some(LocalArchive {
            out_path: out_path.into(),
            ignore,
            file_state: RwLock::new(SourceDir::default()),
            data: RepositoryData {
                uri,
                attributes,
                formats,
                institutions,index,
                dependencies: dependencies.into(),
            },
        })
    }
}

fn read_index_file(archive:&ArchiveURI,path:&Path) -> (Box<[Institution]>,Box<[ArchiveIndex]>) {
    if !path.exists() {
        return (Vec::new().into(),Vec::new().into())
    }
    let reader = match std::fs::File::open(path) {
        Ok(reader) => reader,
        Err(e) => {
            tracing::error!("Could not read index file {}: {e}", path.display());
            return (Vec::new().into(),Vec::new().into())
        }
    };
    let reader = std::io::BufReader::new(reader);
    let v = match serde_json::from_reader::<_,Vec<ArchiveDatum>>(reader) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Invalid JSON file {}: {e}", path.display());
            return (Vec::new().into(),Vec::new().into())
        }
    };
    let mut insts = Vec::new();
    let mut idxs = Vec::new();
    for d in v {
        match d {
            ArchiveDatum::Document(d) => idxs.push(ArchiveIndex::from_kind(d,archive,
                |i| format!("{}/img?a={}&rp=source/{i}",crate::settings::Settings::get().external_url().unwrap_or(""),archive.archive_id()).into_boxed_str()
            )),
            ArchiveDatum::Institution(i) => insts.push(match i {
                Institution::University { title, place, country, url, acronym, logo }
                    => Institution::University { title, place, country, url, acronym, 
                        logo: format!("{}/img?a={}&rp=source/{logo}",crate::settings::Settings::get().external_url().unwrap_or(""),archive.archive_id()).into_boxed_str()
                    },
                Institution::School { title, place, country, url, acronym, logo }
                    => Institution::School { title, place, country, url, acronym, 
                        logo: format!("{}/img?a={}&rp=source/{logo}",crate::settings::Settings::get().external_url().unwrap_or(""),archive.archive_id()).into_boxed_str()
                    }
            }),
        }
    }
    (insts.into(),idxs.into())
}

impl Iterator for ArchiveIterator<'_> {
    type Item = LocalArchive;
    fn next(&mut self) -> Option<Self::Item> {
        let _span = self.in_span.enter();
        Self::next(&mut self.curr, &mut self.stack, &mut self.currp)
    }
}

impl spliter::Spliterator for ArchiveIterator<'_> {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 || self.stack[0].len() < 2 {
            return None;
        }
        let stacksplit = self.stack[0].len() / 2;
        let mut rightstack = self.stack[0].split_off(stacksplit);
        std::mem::swap(&mut self.stack[0], &mut rightstack);
        loop {
            match rightstack.pop() {
                None => return None,
                Some((e, s)) => {
                    if let Ok(rd) = std::fs::read_dir(&e) {
                        return Some(Self {
                            path: self.path,
                            curr: Some(rd),
                            stack: vec![rightstack, Vec::new()],
                            currp: s,
                            in_span: self.in_span.clone(),
                        });
                    }
                }
            }
        }
    }
}
