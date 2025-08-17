use crate::{
    Archive, ArchiveKind, LocalArchive,
    archive_json::{ArchiveIndex, Institution, read_archive_json},
    formats::{SourceFormat, SourceFormatId},
    source_files::SourceDir,
    utils::{ignore_source::IgnoreSource, path_ext::RelPath},
};
use ftml_uris::{ArchiveId, ArchiveUri, BaseUri};
use std::path::Path;

#[derive(Debug)]
pub struct RepositoryData {
    pub uri: ArchiveUri,
    pub attributes: Vec<(Box<str>, Box<str>)>,
    pub formats: Vec<SourceFormatId>,
    //pub dependencies: Box<[ArchiveId]>,
    pub institutions: Box<[Institution]>,
    pub index: Box<[ArchiveIndex]>,
}

#[allow(clippy::too_many_lines)]
pub fn parse_manifest(path: &Path, id: RelPath, external_url: &str) -> Option<Archive> {
    use std::io::BufRead;
    let Some(top_dir) = path.parent().and_then(Path::parent) else {
        tracing::warn!("Could not find parent directory of {}", path.display());
        return None;
    };
    let out_path = LocalArchive::out_dir_of(top_dir);
    let Ok(reader) = std::fs::File::open(path) else {
        tracing::warn!("Could not open manifest {}", path.display());
        return None;
    };
    let reader = std::io::BufReader::new(reader);
    let mut lines = reader.lines();

    let mut formats = Vec::default();
    let mut url_base: String = String::new();
    let mut ignore = IgnoreSource::default();
    let mut attributes: Vec<(Box<str>, Box<str>)> = Vec::new();
    let mut real_id: Option<ArchiveId> = None;
    let mut kind = None;
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
                if id != *v {
                    tracing::warn!("Archive {v}'s id does not match its location ({id})");
                    return None;
                } else if v.is_empty() {
                    tracing::warn!("Archive {v} has an empty id");
                    return None;
                }
                match v.parse() {
                    Ok(id) => real_id = Some(id),
                    Err(e) => {
                        tracing::warn!("Invalid archive id {v} in {}: {e}", path.display());
                        return None;
                    }
                }
            }
            "format" => {
                formats = v
                    .split(',')
                    .filter_map(|l| {
                        SourceFormat::get(l).or_else(|| {
                            tracing::warn!("Invalid format {l} in archive {v}");
                            None
                        })
                    })
                    .collect();
            }
            "url-base" => url_base = v.into(),
            "ignore" => {
                ignore = IgnoreSource::new(v, &top_dir.join("source")); //Some(v.into());
            }
            "kind" => {
                if let Some(k) = ArchiveKind::get(v) {
                    kind = Some(k);
                } else {
                    tracing::error!("Unknown archive kind {v}");
                    return None;
                }
            }
            _ => {
                attributes.push((k.into(), v.into()));
            }
        }
    }
    let Some(id) = real_id else {
        tracing::warn!("Archive {id} has no id");
        return None;
    };
    if formats.is_empty() && !id.is_meta() && kind.is_none() {
        tracing::warn!("No formats found for archive {id}");
        return None;
    }
    if url_base.is_empty() {
        tracing::warn!("Archive {id} has no URL base");
        return None;
    }
    let dom_uri: BaseUri = match url_base.parse() {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Archive {id} has an invalid URL base: {e}");
            return None;
        }
    };
    let uri = dom_uri & id;
    let (institutions, index) =
        read_archive_json(&uri, &path.with_file_name("archive.json"), external_url);
    if let Some(kind) = kind {
        let data = RepositoryData {
            uri,
            attributes,
            formats,
            institutions,
            index, //dependencies: dependencies.into(),
        };
        (kind.make_new)(data, top_dir).map(|r| Archive::Ext(kind, r))
    } else {
        Some(Archive::Local(Box::new(LocalArchive {
            uri,
            attributes,
            formats,
            institutions,
            index,
            ignore,
            out_path,
            //ignore,
            file_state: parking_lot::RwLock::new(SourceDir::default()),
            #[cfg(feature = "gitlab")]
            is_managed: std::sync::OnceLock::new(),
        })))
    }
}
