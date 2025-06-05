use flams_ontology::uris::ArchiveId;
use leptos::prelude::*;
use std::num::NonZeroU32;

use crate::GitState;

#[server(prefix = "/api/gitlab", endpoint = "get_archives")]
pub async fn get_archives()
-> Result<Vec<(flams_git::Project, ArchiveId, GitState)>, ServerFnError<String>> {
    server::get_archives().await
}

#[server(prefix = "/api/gitlab", endpoint = "get_branches")]
pub async fn get_branches(id: u64) -> Result<Vec<flams_git::Branch>, ServerFnError<String>> {
    let (oauth, secret) = flams_router_base::get_oauth()?;
    oauth
        .get_branches(id, secret)
        .await
        .map_err(|e| ServerFnError::WrappedServerError(e.to_string()))
}

#[server(prefix = "/api/gitlab", endpoint = "update_from_branch")]
pub async fn update_from_branch(
    id: Option<NonZeroU32>,
    archive: ArchiveId,
    url: String,
    branch: String,
) -> Result<(usize, NonZeroU32), ServerFnError<String>> {
    server::update_from_branch(id, archive, url, branch).await
}

#[server(prefix = "/api/gitlab", endpoint = "clone_to_queue")]
pub async fn clone_to_queue(
    id: Option<NonZeroU32>,
    archive: ArchiveId,
    url: String,
    branch: String,
    has_release: bool,
) -> Result<(usize, NonZeroU32), ServerFnError<String>> {
    server::clone_to_queue(id, archive, url, branch, has_release).await
}

#[server(prefix = "/api/gitlab", endpoint = "get_new_commits")]
pub async fn get_new_commits(
    queue: Option<NonZeroU32>,
    id: ArchiveId,
) -> Result<Vec<(String, flams_git::Commit)>, ServerFnError<String>> {
    server::get_new_commits(queue, id).await
}

#[cfg(feature = "ssr")]
mod server {
    use flams_ontology::uris::ArchiveId;
    use flams_router_base::{LoginState, get_oauth};
    use flams_router_buildqueue_base::LoginQueue;
    use flams_system::backend::archives::{ArchiveTrait, LocalOut};
    use flams_utils::{impossible, unwrap};
    use flams_web_utils::blocking_server_fn;
    use leptos::prelude::*;
    use std::num::NonZeroU32;

    use crate::GitState;

    #[allow(clippy::too_many_lines)]
    pub(super) async fn get_archives()
    -> Result<Vec<(flams_git::Project, ArchiveId, GitState)>, ServerFnError<String>> {
        use flams_git::gl::auth::GitLabOAuth;
        use flams_system::backend::{
            AnyBackend, GlobalBackend, SandboxedRepository, archives::Archive,
        };
        use leptos::either::Either::{Left, Right};
        async fn get(
            oauth: GitLabOAuth,
            secret: String,
            p: flams_git::Project,
        ) -> (
            flams_git::Project,
            Result<Option<ArchiveId>, flams_git::gl::Err>,
        ) {
            let id = if let Some(b) = &p.default_branch {
                oauth.get_archive_id(p.id, secret, b).await
            } else {
                return (p, Ok(None));
            };
            (p, id)
        }
        let (oauth, secret) = get_oauth()?;
        let r = oauth
            .get_projects(secret.clone())
            .await
            .map_err(|e| ServerFnError::WrappedServerError(e.to_string()))?;
        let mut r2 = Vec::new();
        let mut js = tokio::task::JoinSet::new();
        for p in r {
            js.spawn(get(oauth.clone(), secret.clone(), p));
        }
        while let Some(r) = js.join_next().await {
            match r {
                Err(e) => return Err(e.to_string().into()),
                Ok((p, Err(e))) => {
                    tracing::error!("error obtaining archive ID of {} ({}): {e}", p.path, p.id);
                }
                Ok((p, Ok(Some(id)))) => r2.push((p, id)),
                Ok((_, Ok(None))) => (),
            }
        }
        let r = blocking_server_fn(move || {
            use flams_system::building::queue_manager::QueueManager;
            let mut ret = Vec::new();
            let backend = GlobalBackend::get();
            let gitlab_url = unwrap!(flams_system::settings::Settings::get().gitlab_url.as_ref());
            for a in backend.all_archives().iter() {
                if let Archive::Local(a) = a {
                    if let Some((p, id)) = r2
                        .iter()
                        .position(|(_, id)| id == a.id())
                        .map(|i| r2.swap_remove(i))
                    {
                        if let Ok(git) = flams_git::repos::GitRepo::open(a.path()) {
                            if gitlab_url
                                .host_str()
                                .is_some_and(|s| git.is_managed(s).is_some())
                            {
                                ret.push((p, id, Left(git)));
                            } else {
                                ret.push((p, id, Right(GitState::None)));
                            }
                        } else {
                            ret.push((p, id, Right(GitState::None)));
                        }
                    }
                }
            }
            ret.extend(r2.into_iter().map(|(p, id)| (p, id, Right(GitState::None))));
            //let r2 = &mut ret;
            QueueManager::get().with_all_queues(|qs| {
                for (qid, q) in qs {
                    if let AnyBackend::Sandbox(sb) = q.backend() {
                        sb.with_repos(|rs| {
                            for r in rs {
                                match r {
                                    SandboxedRepository::Git {
                                        id: rid, commit, ..
                                    } => {
                                        if let Some(e) = ret.iter_mut().find_map(|(_, id, e)| {
                                            if id == rid { Some(e) } else { None }
                                        }) {
                                            *e = Right(GitState::Queued {
                                                commit: commit.id.clone(),
                                                queue: (*qid).into(),
                                            });
                                        }
                                        //return Some(GitState::Queued { commit:commit.id.clone(), queue:(*qid).into()})
                                    }
                                    SandboxedRepository::Copy(_) => (),
                                }
                            }
                        });
                    }
                }
            });

            Ok(ret)
        })
        .await?;

        let mut r2 = Vec::new();

        let mut js = tokio::task::JoinSet::new();
        for (p, id, e) in r {
            match e {
                Right(e) => r2.push((p, id, e)),
                Left(git) => {
                    let secret = secret.clone();
                    js.spawn_blocking(move || {
                        if let Ok(rid) = git.current_commit() {
                            let newer = git
                                .get_new_commits_with_oauth(&secret)
                                .ok()
                                .unwrap_or_default();
                            (
                                p,
                                id,
                                GitState::Live {
                                    commit: rid.id,
                                    updates: newer,
                                },
                            )
                        } else {
                            (p, id, GitState::None)
                        }
                    });
                }
            }
        }

        while let Some(r) = js.join_next().await {
            match r {
                Err(e) => return Err(e.to_string().into()),
                Ok((p, id, s)) => r2.push((p, id, s)),
            }
        }

        Ok(r2)
    }

    pub(super) async fn update_from_branch(
        id: Option<NonZeroU32>,
        archive: ArchiveId,
        url: String,
        branch: String,
    ) -> Result<(usize, NonZeroU32), ServerFnError<String>> {
        use flams_system::backend::{AnyBackend, Backend, SandboxedRepository, archives::Archive};
        use flams_system::formats::FormatOrTargets;
        let (_, secret) = get_oauth()?;
        let login = LoginState::get_server();
        if matches!(login, LoginState::NoAccounts) {
            return Err("Only allowed in public mode".to_string().into());
        }
        blocking_server_fn(move || {
            login.with_opt_queue(id, |queue_id, queue| {
                let AnyBackend::Sandbox(backend) = queue.backend() else {
                    unreachable!()
                };
                backend.require(&archive);
                let path = backend.path_for(&archive);
                if !path.exists() {
                    return Err(format!("Archive {archive} not found!"));
                }
                let repo = flams_git::repos::GitRepo::open(&path).map_err(|e| e.to_string())?;
                repo.fetch_branch_from_oauth(&secret, &branch, false)
                    .map_err(|e| e.to_string())?;
                let commit = repo
                    .current_remote_commit_on(&branch)
                    .map_err(|e| e.to_string())?;
                repo.force_checkout(&commit.id).map_err(|e| e.to_string())?;
                //repo.mark_managed(&branch,&commit.id).map_err(|e| e.to_string())?;
                backend.add(
                    SandboxedRepository::Git {
                        id: archive.clone(),
                        commit,
                        branch: branch.into(),
                        remote: url.into(),
                    },
                    || (),
                );
                let formats = backend.with_archive(&archive, |a| {
                    let Some(Archive::Local(a)) = a else {
                        return Err("Archive not found".to_string());
                    };
                    Ok(a.file_state()
                        .formats
                        .iter()
                        .map(|(k, _)| *k)
                        .collect::<Vec<_>>())
                })?;
                let mut u = 0;
                for f in formats {
                    u += queue.enqueue_archive(&archive, FormatOrTargets::Format(f), true, None);
                }
                Ok((u, queue_id.into()))
            })?
        })
        .await
    }

    pub async fn clone_to_queue(
        id: Option<NonZeroU32>,
        archive: ArchiveId,
        url: String,
        branch: String,
        _has_release: bool,
    ) -> Result<(usize, NonZeroU32), ServerFnError<String>> {
        use flams_system::backend::{AnyBackend, Backend, SandboxedRepository, archives::Archive};
        use flams_system::formats::FormatOrTargets;
        let (_, secret) = get_oauth()?;
        let login = LoginState::get_server();
        if matches!(login, LoginState::NoAccounts) {
            return Err("Only allowed in public mode".to_string().into());
        }

        tokio::task::spawn_blocking(move || {
            login.with_opt_queue(id, |queue_id, queue| {
                let AnyBackend::Sandbox(backend) = queue.backend() else {
                    unreachable!()
                };
                let path = backend.path_for(&archive);
                if path.exists() {
                    let _ = std::fs::remove_dir_all(&path);
                }
                let commit = {
                    let repo = flams_git::repos::GitRepo::clone_from_oauth(
                        &secret, &url, &branch, &path, false,
                    )
                    .map_err(|e| e.to_string())?;
                    repo.current_commit().map_err(|e| e.to_string())?
                    //repo.new_branch("release").map_err(|e| e.to_string())?;
                    //repo.mark_managed(&branch,&commit.id).map_err(|e| e.to_string())?;
                    //commit
                };
                backend.add(
                    SandboxedRepository::Git {
                        id: archive.clone(),
                        commit,
                        branch: branch.into(),
                        remote: url.into(),
                    },
                    || (),
                );
                let formats = backend.with_archive(&archive, |a| {
                    let Some(Archive::Local(a)) = a else {
                        return Err("Archive not found".to_string());
                    };
                    Ok(a.file_state()
                        .formats
                        .iter()
                        .map(|(k, _)| *k)
                        .collect::<Vec<_>>())
                })?;
                let mut u = 0;
                for f in formats {
                    u += queue.enqueue_archive(&archive, FormatOrTargets::Format(f), false, None);
                }
                Ok((u, queue_id.into()))
            })
        })
        .await
        .unwrap_or_else(|e| Err(e.to_string()))? //.map_err(Into::into)
    }

    pub(super) async fn get_new_commits(
        queue: Option<NonZeroU32>,
        id: ArchiveId,
    ) -> Result<Vec<(String, flams_git::Commit)>, ServerFnError<String>> {
        use flams_system::backend::AnyBackend;

        let (_, secret) = get_oauth()?;
        let login = LoginState::get_server();
        if matches!(login, LoginState::NoAccounts) {
            return Err("Only allowed in public mode".to_string().into());
        }
        blocking_server_fn(move || {
            login.with_opt_queue(queue, |_, queue| {
                let AnyBackend::Sandbox(backend) = queue.backend() else {
                    impossible!()
                };
                let path = backend.path_for(&id);
                let r = flams_git::repos::GitRepo::open(path)
                    .ok()
                    .and_then(|git| {
                        let gitlab_url =
                            unwrap!(flams_system::settings::Settings::get().gitlab_url.as_ref());
                        if gitlab_url
                            .host_str()
                            .is_some_and(|s| git.is_managed(s).is_some())
                        {
                            git.get_new_commits_with_oauth(&secret).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                Ok(r)
            })?
        })
        .await
    }
}
