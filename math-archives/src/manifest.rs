use crate::{
    Archive, ArchiveKind, LocalArchive,
    formats::{SourceFormat, SourceFormatId},
    source_files::SourceDir,
    utils::{errors::ManifestParseError, ignore_source::IgnoreSource, path_ext::RelPath},
};
use flams_backend_types::archive_json::{ArchiveDatum, ArchiveIndex, Institution};
use ftml_uris::{ArchiveId, ArchiveUri, BaseUri, UriWithArchive};
use std::path::Path;

#[derive(Debug)]
pub struct RepositoryData {
    pub uri: ArchiveUri,
    pub attributes: Vec<(Box<str>, Box<str>)>,
    pub formats: smallvec::SmallVec<SourceFormatId, 1>,
    //pub dependencies: Box<[ArchiveId]>,
    pub institutions: Box<[Institution]>,
    pub index: Box<[ArchiveIndex]>,
}

#[allow(clippy::too_many_lines)]
/// # Errors
pub fn parse_manifest(
    path: &Path,
    id: RelPath,
    external_url: &str,
) -> Result<Archive, ManifestParseError> {
    use std::io::BufRead;
    let Some(top_dir) = path.parent().and_then(Path::parent) else {
        return Err(ManifestParseError::NoParent);
    };
    let out_path = LocalArchive::out_dir_of(top_dir);
    let reader = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(reader);
    let mut lines = reader.lines();

    let mut source = None;
    let mut formats = smallvec::SmallVec::<_, 1>::new();
    let mut url_base: Option<BaseUri> = None;
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
                    return Err(ManifestParseError::IdMismatch(v.to_string()));
                } else if v.is_empty() {
                    return Err(ManifestParseError::EmptyId);
                }
                real_id = Some(
                    v.parse()
                        .map_err(|_| ManifestParseError::InvalidId(v.to_string()))?,
                );
            }
            "format" => {
                for f in v.split(',') {
                    formats.push(
                        SourceFormat::get(f)
                            .ok_or_else(|| ManifestParseError::UnknownFormat(f.to_string()))?,
                    );
                }
            }
            "url-base" => {
                url_base = Some(
                    v.parse()
                        .map_err(|e| ManifestParseError::InvalidUrlBase(v.to_string(), e))?,
                );
            }
            "ignore" => {
                ignore = IgnoreSource::new(v, &top_dir.join("source")); //Some(v.into());
            }
            "source" => source = Some(v.to_string().into_boxed_str()),
            "kind" => {
                if let Some(k) = ArchiveKind::get(v) {
                    kind = Some(k);
                } else {
                    return Err(ManifestParseError::UnknownKind(v.to_string()));
                }
            }
            _ => {
                attributes.push((k.into(), v.into()));
            }
        }
    }
    let Some(id) = real_id else {
        return Err(ManifestParseError::EmptyId);
    };
    if formats.is_empty() && !id.is_meta() && kind.is_none() {
        return Err(ManifestParseError::NoFormatOrKind);
    }
    let Some(dom_uri) = url_base else {
        return Err(ManifestParseError::NoUrlBase);
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
        (kind.make_new)(data, top_dir).map_or_else(
            |e| Err(ManifestParseError::InvalidKind(kind.name, e)),
            |r| Ok(Archive::Ext(kind, r)),
        )
    } else {
        Ok(Archive::Local(Box::new(LocalArchive {
            uri,
            //attributes,
            formats,
            institutions,
            index,
            ignore,
            out_path,
            source,
            //ignore,
            file_state: parking_lot::RwLock::new(SourceDir::default()),
            #[cfg(feature = "git")]
            is_managed: std::sync::OnceLock::new(),
        })))
    }
}

pub fn read_archive_json(
    archive: &ArchiveUri,
    path: &Path,
    external_url: &str,
) -> (Box<[Institution]>, Box<[ArchiveIndex]>) {
    if !path.exists() {
        return (Vec::new().into(), Vec::new().into());
    }
    let reader = match std::fs::File::open(path) {
        Ok(reader) => reader,
        Err(e) => {
            tracing::error!("Could not read index file {}: {e}", path.display());
            return (Vec::new().into(), Vec::new().into());
        }
    };
    let reader = std::io::BufReader::new(reader);
    let v = match serde_json::from_reader::<_, Vec<ArchiveDatum>>(reader) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Invalid JSON file {}: {e}", path.display());
            return (Vec::new().into(), Vec::new().into());
        }
    };
    let mut insts = Vec::new();
    let mut idxs = Vec::new();
    for d in v {
        match d {
            ArchiveDatum::Document(mut d) => {
                if d.teaser().is_none() {
                    let desc = path.with_file_name("desc.html");
                    if desc.exists()
                        && let Ok(s) = std::fs::read_to_string(desc)
                    {
                        d.set_teaser(s.into_boxed_str());
                    }
                }
                match ArchiveIndex::from_kind(d, archive, |i| {
                    format!(
                        "{external_url}/img?a={}&rp=source/{i}",
                        archive.archive_id()
                    )
                    .into_boxed_str()
                }) {
                    Ok(e) => idxs.push(e),
                    Err(e) => tracing::error!("Error in index file {}: {e:#}", path.display()),
                }
            }
            ArchiveDatum::Institution(i) => insts.push(match i {
                Institution::University {
                    title,
                    place,
                    country,
                    url,
                    acronym,
                    logo,
                } => Institution::University {
                    title,
                    place,
                    country,
                    url,
                    acronym,
                    logo: format!(
                        "{external_url}/img?a={}&rp=source/{logo}",
                        archive.archive_id()
                    )
                    .into_boxed_str(),
                },
                Institution::School {
                    title,
                    place,
                    country,
                    url,
                    acronym,
                    logo,
                } => Institution::School {
                    title,
                    place,
                    country,
                    url,
                    acronym,
                    logo: format!(
                        "{external_url}/img?a={}&rp=source/{logo}",
                        archive.archive_id()
                    )
                    .into_boxed_str(),
                },
            }),
        }
    }
    (insts.into(), idxs.into())
}
