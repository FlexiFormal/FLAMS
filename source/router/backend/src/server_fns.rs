use flams_ontology::{
    archive_json::{
        ArchiveData, ArchiveGroupData, ArchiveIndex, DirectoryData, FileData, Institution,
    },
    uris::ArchiveId,
};
use leptos::prelude::*;

use crate::FileStates;

#[server(prefix = "/api/backend", endpoint = "group_entries")]
pub async fn group_entries(
    r#in: Option<ArchiveId>,
) -> Result<(Vec<ArchiveGroupData>, Vec<ArchiveData>), ServerFnError<String>> {
    server::group_entries(r#in).await
}

#[server(prefix = "/api/backend", endpoint = "archive_entries")]
pub async fn archive_entries(
    archive: ArchiveId,
    path: Option<String>,
) -> Result<(Vec<DirectoryData>, Vec<FileData>), ServerFnError<String>> {
    server::archive_entries(archive, path).await
}

#[server(prefix = "/api/backend", endpoint = "archive_dependencies")]
pub async fn archive_dependencies(
    archives: Vec<ArchiveId>,
) -> Result<Vec<ArchiveId>, ServerFnError<String>> {
    server::archive_dependencies(archives).await
}

#[server(prefix = "/api/backend", endpoint = "build_status")]
pub async fn build_status(
    archive: ArchiveId,
    path: Option<String>,
) -> Result<FileStates, ServerFnError<String>> {
    server::build_status(archive, path).await
}

#[server(prefix="/api/backend",endpoint="download",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Streaming
)]
pub async fn archive_stream(
    id: ArchiveId,
) -> Result<leptos::server_fn::codec::ByteStream, ServerFnError> {
    server::archive_stream(id).await
}

#[server(
  prefix="/api",
  endpoint="index",
  output=server_fn::codec::Json
)]
pub async fn index() -> Result<(Vec<Institution>, Vec<ArchiveIndex>), ServerFnError<String>> {
    use flams_system::backend::GlobalBackend;
    flams_web_utils::blocking_server_fn(|| {
        let (a, b) = GlobalBackend::get().with_archive_tree(|t| t.index.clone());
        Ok((a.0, b.0))
    })
    .await
}

#[cfg(feature = "ssr")]
mod server {
    use flams_ontology::{
        archive_json::{ArchiveData, ArchiveGroupData, DirectoryData, FileData},
        uris::{ArchiveId, ArchiveURI, ArchiveURITrait, URIOrRefTrait},
    };
    use flams_router_base::LoginState;
    use flams_system::backend::{
        Backend, GlobalBackend,
        archives::{Archive, ArchiveOrGroup as AoG},
    };
    use flams_utils::vecmap::VecSet;
    use flams_web_utils::blocking_server_fn;
    use leptos::prelude::*;

    use crate::FileStates;

    pub async fn group_entries(
        id: Option<ArchiveId>,
    ) -> Result<(Vec<ArchiveGroupData>, Vec<ArchiveData>), ServerFnError<String>> {
        let login = LoginState::get_server();
        blocking_server_fn(move || {
            let allowed = matches!(
                login,
                LoginState::Admin
                    | LoginState::NoAccounts
                    | LoginState::User { is_admin: true, .. }
            );
            flams_system::backend::GlobalBackend::get().with_archive_tree(|tree| {
                let v = match id {
                    None => &tree.groups,
                    Some(id) => match tree.find(&id) {
                        Some(AoG::Group(g)) => &g.children,
                        _ => return Err(format!("Archive Group {id} not found").into()),
                    },
                };
                let mut groups = Vec::new();
                let mut archives = Vec::new();
                for a in v {
                    match a {
                        AoG::Archive(id) => {
                            let (summary, git) = if !allowed
                                && flams_system::settings::Settings::get().gitlab_url.is_none()
                            {
                                (None, None)
                            } else {
                                tree.get(id)
                                    .map(|a| {
                                        if let Archive::Local(a) = a {
                                            (
                                                if allowed {
                                                    Some(a.state_summary())
                                                } else {
                                                    None
                                                },
                                                a.is_managed().map(ToString::to_string),
                                            )
                                        } else {
                                            (None, None)
                                        }
                                    })
                                    .unwrap_or_default()
                            };
                            archives.push(ArchiveData {
                                id: id.clone(),
                                summary,
                                git,
                            });
                        }
                        AoG::Group(g) => {
                            let summary = if allowed {
                                Some(g.state.summarize())
                            } else {
                                None
                            };
                            groups.push(ArchiveGroupData {
                                id: g.id.clone(),
                                summary,
                            });
                        }
                    }
                }
                Ok((groups, archives))
            })
        })
        .await
    }
    pub async fn archive_entries(
        archive: ArchiveId,
        path: Option<String>,
    ) -> Result<(Vec<DirectoryData>, Vec<FileData>), ServerFnError<String>> {
        use either::Either;
        use flams_system::backend::{Backend, archives::source_files::SourceEntry};
        let login = LoginState::get_server();

        blocking_server_fn(move || {
            let allowed = matches!(
                login,
                LoginState::Admin
                    | LoginState::NoAccounts
                    | LoginState::User { is_admin: true, .. }
            );
            flams_system::backend::GlobalBackend::get().with_local_archive(&archive, |a| {
                let Some(a) = a else {
                    return Err(format!("Archive {archive} not found").into());
                };
                a.with_sources(|d| {
                    let d = match path {
                        None => d,
                        Some(p) => match d.find(&p) {
                            Some(Either::Left(d)) => d,
                            _ => {
                                return Err(format!(
                                    "Directory {p} not found in archive {archive}"
                                )
                                .into());
                            }
                        },
                    };
                    let mut ds = Vec::new();
                    let mut fs = Vec::new();
                    for d in &d.children {
                        match d {
                            SourceEntry::Dir(d) => ds.push(DirectoryData {
                                rel_path: d.relative_path.to_string(),
                                summary: if allowed {
                                    Some(d.state.summarize())
                                } else {
                                    None
                                },
                            }),
                            SourceEntry::File(f) => fs.push(FileData {
                                rel_path: f.relative_path.to_string(),
                                format: f.format.to_string(),
                            }),
                        }
                    }
                    Ok((ds, fs))
                })
            })
        })
        .await
    }

    pub async fn archive_dependencies(
        archives: Vec<ArchiveId>,
    ) -> Result<Vec<ArchiveId>, ServerFnError<String>> {
        use flams_system::backend::archives::ArchiveOrGroup;
        let mut archives: VecSet<_> = archives.into_iter().collect();
        blocking_server_fn(move || {
            let mut ret = VecSet::new();
            let mut dones = VecSet::new();
            let backend = flams_system::backend::GlobalBackend::get();
            while let Some(archive) = archives.0.pop() {
                if dones.0.contains(&archive) {
                    continue;
                }
                dones.insert(archive.clone());
                let Some(iri) = backend.with_archive_tree(|tree| {
                    let mut steps = archive.steps();
                    if let Some(mut n) = steps.next() {
                        let mut curr = tree.groups.as_slice();
                        while let Some(g) = curr.iter().find_map(|a| match a {
                            ArchiveOrGroup::Group(g) if g.id.last_name() == n => Some(g),
                            _ => None,
                        }) {
                            curr = g.children.as_slice();
                            if let Some(a) = curr.iter().find_map(|a| match a {
                                ArchiveOrGroup::Archive(a) if a.is_meta() => Some(a),
                                _ => None,
                            }) {
                                if !ret.0.contains(a) {
                                    ret.insert(a.clone());
                                    archives.insert(a.clone());
                                }
                            }
                            if let Some(m) = steps.next() {
                                n = m;
                            } else {
                                break;
                            }
                        }
                    }
                    tree.get(&archive).map(|a| a.uri().to_iri())
                }) else {
                    return Err(format!("Archive {archive} not found"));
                };
                let res = flams_system::backend::GlobalBackend::get()
                    .triple_store()
                    .query_str(format!(
                        "SELECT DISTINCT ?a WHERE {{
                            <{}> ulo:contains ?d.
                            ?d rdf:type ulo:document .
                            ?d ulo:contains* ?x.
                            ?x (dc:requires|ulo:imports|dc:hasPart) ?m.
                            ?e ulo:contains? ?m.
                            ?e rdf:type ulo:document.
                            ?a ulo:contains ?e.
                        }}",
                        iri.as_str()
                    ))
                    .map_err(|e| e.to_string())?;
                for i in res.into_uris::<ArchiveURI>() {
                    let id = i.archive_id();
                    if !ret.0.contains(&id) {
                        archives.insert(id.clone());
                        ret.insert(id.clone());
                    }
                }
            }
            Ok(ret.0)
        })
        .await
    }
    pub async fn build_status(
        archive: ArchiveId,
        path: Option<String>,
    ) -> Result<FileStates, ServerFnError<String>> {
        use either::Either;
        use flams_system::backend::Backend;
        use flams_system::backend::archives::{Archive, ArchiveOrGroup as AoG};
        let login = LoginState::get_server();

        blocking_server_fn(move || {
            let allowed = matches!(
                login,
                LoginState::Admin
                    | LoginState::NoAccounts
                    | LoginState::User { is_admin: true, .. }
            );
            if !allowed {
                return Err("Not logged in".to_string().into());
            }
            path.map_or_else(
                || {
                    GlobalBackend::get().with_archive_tree(|tree| match tree.find(&archive) {
                        None => Err(format!("Archive {archive} not found").into()),
                        Some(AoG::Archive(id)) => {
                            let Some(Archive::Local(archive)) = tree.get(id) else {
                                return Err(format!("Archive {archive} not found").into());
                            };
                            Ok(archive.file_state().into())
                        }
                        Some(AoG::Group(g)) => Ok(g.state.clone().into()),
                    })
                },
                |path| {
                    GlobalBackend::get().with_local_archive(&archive, |a| {
                        let Some(a) = a else {
                            return Err(format!("Archive {archive} not found").into());
                        };
                        a.with_sources(|d| match d.find(&path) {
                            Some(Either::Left(d)) => Ok(d.state.clone().into()),
                            Some(Either::Right(f)) => Ok((&f.target_state).into()),
                            None => {
                                Err(format!("Directory {path} not found in archive {archive}")
                                    .into())
                            }
                        })
                    })
                },
            )
        })
        .await
    }
    pub async fn archive_stream(
        id: ArchiveId,
    ) -> Result<leptos::server_fn::codec::ByteStream, ServerFnError> {
        use futures::TryStreamExt;
        let stream = GlobalBackend::get()
            .with_local_archive(&id, |a| a.map(|a| a.zip()))
            .ok_or_else(|| ServerFnError::new(format!("No archive with id {id} found!")))?;
        Ok(leptos::server_fn::codec::ByteStream::new(
            stream.map_err(|e| ServerFnError::new(e.to_string())),
        ))
    }
}
