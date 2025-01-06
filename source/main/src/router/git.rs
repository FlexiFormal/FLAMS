use std::num::NonZeroU32;

use immt_ontology::uris::ArchiveId;
use immt_utils::vecmap::VecMap;
use leptos::prelude::*;
use either::Either;

use crate::users::LoginState;

#[cfg(feature="ssr")]
pub(crate) fn get_oauth() -> Result<(immt_git::gl::auth::GitLabOAuth,String),ServerFnError<String>> {
  use immt_git::gl::auth::GitLabOAuth;
  let Some(session)= use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
    return Err("Internal Error".to_string().into())
  };
  let Some(user) = session.user else {
    return Err("Not logged in".to_string().into())
  };
  let Some(oauth):Option<GitLabOAuth> = expect_context() else {
    return Err("Not Gitlab integration set up".to_string().into())
  };
  Ok((oauth,user.secret))
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum GitState {
  None,
  Queued {
    commit:String,
    queue:NonZeroU32
  },
  Live {
    commit:String,
    updates:Vec<(String,immt_git::Commit)>
  }
}


#[server(
  prefix="/api/gitlab",
  endpoint="get_archives",
)]
pub async fn get_archives() -> Result<Vec<(immt_git::Project,ArchiveId,GitState)>,ServerFnError<String>> {
  use immt_git::gl::auth::GitLabOAuth;
  use immt_git::gl::auth::AccessToken;
  use immt_system::backend::{Backend,AnyBackend,GlobalBackend,archives::Archive,SandboxedRepository};
  let (oauth,secret) = get_oauth()?;
  let r = oauth.get_projects(secret.clone()).await
  .map_err(|e| ServerFnError::WrappedServerError(e.to_string()))?;
  let mut r2 = Vec::new();
  for p in r {
    if let Some(branch) = &p.default_branch {
      if let Some(id) = oauth.get_archive_id(p.id, secret.clone(), branch).await
      .map_err(|e| ServerFnError::WrappedServerError(e.to_string()))? {
        r2.push((p,id));
      }
    }
  }
  tokio::task::spawn_blocking(move || {
    let backend = GlobalBackend::get();
    let gitlab_url = &**immt_system::settings::Settings::get().gitlab_url.as_ref().unwrap_or_else(|| unreachable!());
    let mut manageds = backend.all_archives().iter().filter_map(|a| {
      let Archive::Local(a) = a else {return None};
      if !r2.iter().any(|(_,id)| id == a.id()) { return None }
      immt_git::repos::GitRepo::open(a.path()).ok().and_then(|git| {
        git.get_origin_url().ok().and_then(|url| {
          if url.starts_with(gitlab_url) {
            let newer = match git.get_new_commits_with_oauth(&secret) {
              Ok(v) => v,
              Err(e) => {
                println!("{e}");
                Vec::new()
              }
            };
            git.release_commit_id().ok().map(|id| 
              (a.id().clone(),(url,id,newer))
            )
          } else {None}
        })
      })
    }).collect::<VecMap<_,_>>();

    use immt_system::building::{queue_manager::QueueManager,QueueName};
    let mut building = VecMap::new();
    QueueManager::get().with_all_queues(|qs| {
      for (qid,q) in qs {
        if let AnyBackend::Sandbox(sb) = q.backend() {
          sb.with_repos(|rp| 
            for rep in rp {
              if let SandboxedRepository::Git { id, commit,remote,.. } = rep {
                building.insert(id.clone(),(remote.to_string(),(*qid).into(),commit.id.clone()));
              }
            }
          )
        }
      }
    });
    let ret = r2.into_iter().map(|(p,id)| {
      let state = building.remove(&id).map(|(_,id,commit)|
        GitState::Queued { commit, queue:id }
      ).or_else(|| manageds.remove(&id).map(|(_,commit,updates)|
        GitState::Live { commit,updates }
      )).unwrap_or(GitState::None);
      (p,id,state)
    }).collect();
    Ok(ret)
  }).await.map_err(|e| ServerFnError::WrappedServerError(e.to_string()))?
}

#[server(
  prefix="/api/gitlab",
  endpoint="get_branches",
)]
pub async fn get_branches(id:u64) -> Result<Vec<immt_git::Branch>,ServerFnError<String>> {
  let (oauth,secret) = get_oauth()?;
  oauth.get_branches(id, secret).await
  .map_err(|e| ServerFnError::WrappedServerError(e.to_string()))
}

#[server(
  prefix="/api/gitlab",
  endpoint="clone_to_queue",
)]
pub async fn clone_to_queue(id:Option<NonZeroU32>,archive:ArchiveId,url:String,branch:String,has_release:bool) -> Result<(usize,NonZeroU32),ServerFnError<String>> {
  use immt_system::building::{queue_manager::QueueManager,QueueName};
  use immt_system::backend::{AnyBackend,SandboxedRepository,Backend,archives::Archive};
  use immt_system::formats::FormatOrTargets;
  let (oauth,secret) = get_oauth()?;
  let login = LoginState::get_server();
  let ret = tokio::task::spawn_blocking(move || {
    let queues = QueueManager::get();
    let queue_id = match (&login,id) {
      (LoginState::NoAccounts,_) => return Err("Only allowed in public mode".to_string().into()),
      (LoginState::None | LoginState::Loading,_) => return Err("Not logged in".to_string().into()),
      (LoginState::User{name,..},Some(q)) => {
        let allowed = queues.with_queue(q.into(), |q| {
          q.is_some_and(|q| matches!(q.name(),immt_system::building::QueueName::Sandbox{name:qname,..} if &**qname == name))
        });
        if !allowed {
          return Err(format!("Not allowed to edit queue {q}").into())
        }
        q.into()
      }
      (LoginState::Admin,Some(q)) => q.into(),
      (LoginState::Admin,_) => queues.new_queue("admin"),
      (LoginState::User{name,..},_) => queues.new_queue(&name)
    };
    let (backend,path) = queues.with_queue(queue_id, |q| {
      let Some(queue) = q else {
        return Err(format!("Queue does not exist"))
      };
      let AnyBackend::Sandbox(sb) = queue.backend() else {
        return Err(format!("Not a sandboxed queue"))
      };
      Ok((sb.clone(),sb.path_for(&archive)))
    })?;
    if path.exists() {
      let _ = std::fs::remove_dir_all(&path);
    }
    let commit = if has_release {
      let repo = immt_git::repos::GitRepo::clone_from_oauth(&secret, &url, "release", &path, true)
        .map_err(|e| e.to_string())?;
      repo.fetch_branch_from_oauth(&secret,&branch,false).map_err(|e| e.to_string())?;
      let commit = repo.current_commit_on(&branch).map_err(|e| e.to_string())?;
      repo.merge(&commit.id).map_err(|e| e.to_string())?;
      repo.mark_managed().map_err(|e| e.to_string())?;
      repo.current_commit().map_err(|e| e.to_string())?
    } else {
      let repo = immt_git::repos::GitRepo::clone_from_oauth(&secret,&url, &branch, &path,true)
        .map_err(|e| e.to_string())?;
      let commit = repo.current_commit().map_err(|e| e.to_string())?;
      repo.new_branch("release").map_err(|e| e.to_string())?;
      repo.mark_managed().map_err(|e| e.to_string())?;
      commit
    };
    backend.add(SandboxedRepository::Git { id:archive.clone(),commit,branch:branch.into(),remote:url.into() },|| ());
    let formats = backend.with_archive(&archive, |a| {
      let Some(Archive::Local(a)) = a else {
        return Err("Archive not found".to_string())
      };
      Ok(a.file_state().formats.iter().map(|(k,_)| *k).collect::<Vec<_>>())
    })?;
    queues.with_queue(queue_id, move |queue| {
      let Some(queue) = queue else { 
        return Err("Queue not found".to_string())
      };
      let mut u = 0;
      for f in formats {
        u += queue.enqueue_archive(&archive, FormatOrTargets::Format(f), false, None);
      }
      Ok((u,queue_id.into()))
    })
  }).await.map_err(|e| e.to_string())??;
  Ok(ret)
}

#[component]
pub fn Archives() -> impl IntoView {
  let r = Resource::new(|| (),|()| get_archives());
  view!{<Suspense fallback = || view!(<immt_web_utils::components::Spinner/>)>{move ||
    match r.get() {
      Some(Ok(projects)) if projects.is_empty() => leptos::either::EitherOf4::A("(No archives)"),
      Some(Err(e)) => leptos::either::EitherOf4::B(
        immt_web_utils::components::error_toast(e.to_string().into())
      ),
      None => leptos::either::EitherOf4::C(view!(<immt_web_utils::components::Spinner/>)),
      Some(Ok(projects)) => leptos::either::EitherOf4::D(do_projects(projects))
    }
  }</Suspense>}
}

#[derive(Debug,Copy,Clone)]
struct QueueSignal(RwSignal<Option<NonZeroU32>>,RwSignal<()>);

#[derive(Debug,Clone,Default)]
pub struct ProjectTree {
  pub children:Vec<Either<Project,ProjectGroup>>
}

#[derive(Debug,Clone)]
struct Project {
  pub id:u64,
  pub name: ArchiveId,
  pub url:String,
  pub state:RwSignal<GitState>,
  pub default_branch: Option<String>
}
impl Eq for Project {}
impl PartialEq for Project {
  #[inline]
  fn eq(&self,other:&Self) -> bool {
    self.id == other.id
  }
}

#[derive(Debug,Clone)]
pub struct ProjectGroup {
  pub name:String,
  pub children:ProjectTree
}

impl ProjectTree {
  #[inline]
  pub fn is_empty(&self) -> bool { self.children.is_empty()}
}

impl ProjectTree {
  fn add(&mut self,repo: immt_git::Project,id:ArchiveId,state:GitState) {
    use thaw::ToasterInjection;
    let mut steps = id.steps().enumerate().peekable();
    let mut current = self;
    while let Some((i,step)) = steps.next() {
      match current.children.binary_search_by_key(&step, |e| match e {
        Either::Left(p) => p.name.steps().nth(i).unwrap_or_else(|| unreachable!()), 
        Either::Right(g) => &g.name
      }) {
        Err(j) => {
          if steps.peek().is_none() {
            current.children.insert(j, 
              Either::Left(Project {
                url: repo.url,
                id: repo.id,
                name: id,
                default_branch:repo.default_branch,
                state:RwSignal::new(state)
              })
            );
            return
          } else {
            current.children.insert(j,
              Either::Right(ProjectGroup {
                name: step.to_string(),
                children: ProjectTree::default()
              })
            );
            let Either::Right(e) = &mut current.children[j] else {unreachable!()};
            current = &mut e.children;
          }
        }
        Ok(j) => {
          let Either::Right(e) = &mut current.children[j] else {unreachable!()};
          current = &mut e.children;
        }
      }
    }
  }
}

fn do_projects(vec:Vec<(immt_git::Project,ArchiveId,GitState)>) -> impl IntoView {
  use immt_web_utils::components::{Tree,Subtree,Leaf,Header};
  use thaw::Caption1Strong;

  let queue = RwSignal::new(None);
  let get_queues = RwSignal::new(());
  provide_context(QueueSignal(queue,get_queues));

  let mut tree = ProjectTree::default();
  for (p,id,state) in vec {
    tree.add(p,id,state);
  }
  fn inner_tree(tree:ProjectTree) -> impl IntoView {
    tree.children.into_iter().map(|c| match c {
        Either::Left(project) => leptos::either::Either::Left(view!{<Leaf><div>{move || project.state.with(|state| {
          if matches!(state,GitState::None){
            let state = project.state;
            leptos::either::Either::Right(unmanaged(project.name.clone(),project.id,state,project.url.clone()))
          } else {
            leptos::either::Either::Left(managed(project.name.clone(),project.id,state))
          }
        })
      }</div></Leaf>}),
      Either::Right(group) => {
        leptos::either::Either::Right(view!{
          <Subtree><Header slot><div>{group.name}</div></Header>{inner_tree(group.children)}</Subtree>
        }.into_any())
      }
    }).collect_view()
  }
  view!{
    <Caption1Strong>"Archives on GitLab"</Caption1Strong>
    {move || {get_queues.get(); super::backend::select_queue(queue)}}
    <Tree>{inner_tree(tree)}</Tree>
  }
}

fn managed(name:ArchiveId,id:u64,state:&GitState) -> impl IntoView {
  
  let (commit,queue) = match state {
    GitState::Live{commit,..} => (commit[..8].to_string(),None),
    GitState::Queued { commit, queue } => (commit[..8].to_string(),Some(*queue)),
    _ => unreachable!()
  };
  view!{
    {name.to_string()}
    {match state {
      GitState::Queued { commit, queue } =>
        leptos::either::Either::Left(format!(" (commit {} currently queued)",&commit[..8])),
      GitState::Live{commit,updates} => leptos::either::Either::Right(view!{
        " on commit "{commit[..8].to_string()}
        {if updates.is_empty() {leptos::either::Either::Left(" (no updates available) ")} else {
          leptos::either::Either::Right("Updates available! (TODO)")
        }}
      }),
      GitState::None => unreachable!()
    }}
  }
}

fn unmanaged(name:ArchiveId,id:u64,and_then:RwSignal<GitState>,git_url:String) -> impl IntoView {
  use thaw::{Button,ButtonSize,Select,ToasterInjection};
  let name_str = name.to_string();
  let r = Resource::new(|| (), move |()| async move {
    get_branches(id).await.map(|mut branches| {
      let main = branches.iter().position(|b| b.default);
      let main = main.map(|i| branches.remove(i));
      if let Some(b) = main {
        branches.insert(0,b);
      }
      let release = branches.iter().position(|b| b.name == "release");
      let release = release.map(|i| branches.remove(i));
      (branches,release.is_some())
    })
  });
  view!{
    <span style="color:grey">{name_str}" (unmanaged) "</span>
    <Suspense fallback=|| view!(<immt_web_utils::components::Spinner/>)>{move ||
      match r.get() {
        Some(Err(e)) => leptos::either::EitherOf3::B(immt_web_utils::components::error_toast(e.to_string().into())),
        None => leptos::either::EitherOf3::C(view!(<immt_web_utils::components::Spinner/>)),
        Some(Ok((branches,has_release))) => leptos::either::EitherOf3::A({
          let branch = RwSignal::new(branches.first().map(|f| f.name.clone()).unwrap_or_default());
          let toaster = ToasterInjection::expect_context();
          let QueueSignal(queue,get_queues) = expect_context();
          let name = name.clone();
          let git_url = git_url.clone();
          let commit_map : VecMap<_,_> = branches.iter().map(|b| (b.name.clone(),b.commit.clone())).collect();
          let act = Action::new(move |()| {
            use thaw::{MessageBar,MessageBarIntent,MessageBarBody,ToastOptions,ToastPosition};
            let name = name.clone();
            let commit = commit_map.get(&branch.get_untracked()).unwrap_or_else(|| unreachable!()).clone();
            let git_url = git_url.clone();
            async move {
              match clone_to_queue(queue.get_untracked(),name,git_url,branch.get_untracked(),has_release).await {
                Ok((i,q)) => {
                  toaster.dispatch_toast(
                    move || view!{
                      <MessageBar intent=MessageBarIntent::Success><MessageBarBody>
                          {format!("{i} jobs queued")}
                      </MessageBarBody></MessageBar>}, 
                    ToastOptions::default().with_position(ToastPosition::Top)
                  );
                  get_queues.set(());
                  and_then.set(GitState::Queued{commit:commit.id,queue:q});
                }
                Err(e) => toaster.dispatch_toast(
                  || view!{
                    <MessageBar intent=MessageBarIntent::Error><MessageBarBody>
                        {e.to_string()}
                    </MessageBarBody></MessageBar>}, 
                  ToastOptions::default().with_position(ToastPosition::Top)
                )
              }
            }
          });
          view!{<div style="margin-left:10px">
          <Button size=ButtonSize::Small on_click=move |_| {act.dispatch(());}>"Add"</Button>
            " from branch: "<div style="display:inline-block;"><Select value=branch>{
              branches.into_iter().map(|b| {let name = b.name.clone(); view!{
                <option value=name>
                  {b.name}" (Last commit "{b.commit.id[..8].to_string()}" at "{b.commit.created_at.to_string()}" by "{b.commit.author_name}")"
                </option>
              }}).collect_view()
            }</Select></div></div>
          }
        })
      }
    }</Suspense>
  }
}

/*
fn unmanaged(name:ArchiveId,id:u64,url:String,parents:Vec<Project>) -> impl IntoView {
  use thaw::{Button,ButtonSize,Caption1Strong};
  let r = Resource::new(|| (),move |()| get_branches(id));
  view!{
    <span style="color:grey">{name.to_string()}" (unmanaged) "</span>
    <Suspense fallback=|| view!(<immt_web_utils::components::Spinner/>)>{move ||
      match r.get() {
        Some(Ok(b)) => leptos::either::EitherOf3::A(branches(b,name.clone(),url.clone(),parents.clone())),
        Some(Err(e)) => leptos::either::EitherOf3::B(immt_web_utils::components::error_toast(e.to_string().into())),
        None => leptos::either::EitherOf3::C(view!(<immt_web_utils::components::Spinner/>))
      }
    }</Suspense>
  }
}

fn branches(mut branches:Vec<immt_git::Branch>,name:ArchiveId,url:String,parents:Vec<Project>) -> impl IntoView {
  use thaw::{Select,Divider,Button,ButtonSize,ToasterInjection,MessageBar,MessageBarIntent,MessageBarBody,ToastOptions,ToastPosition,Dialog,DialogSurface,DialogBody,DialogContent};
  tracing::info!("{name} - parents: {parents:?}");

  let main = branches.iter().position(|b| b.default);
  let main = main.map(|i| branches.remove(i));
  if let Some(b) = main {
    branches.insert(0,b);
  }
  let release = branches.iter().position(|b| b.name == "release");
  let release = release.map(|i| branches.remove(i));
  let has_release = release.is_some();
  let branch = RwSignal::new(branches.first().unwrap_or_else(|| unreachable!()).name.clone());
  let toaster = ToasterInjection::expect_context();
  let queue_signal: QueueSignal = expect_context();
  let queue = queue_signal.0;
  let get_queues = queue_signal.1;
  let act = Action::new(move |()| {let name = name.clone(); let path = url.clone(); async move {
      match clone_to_queue(queue.get_untracked(),name,path,branch.get_untracked(),has_release).await {
        Ok(i) => {
          toaster.dispatch_toast(
            move || view!{
              <MessageBar intent=MessageBarIntent::Success><MessageBarBody>
                  {format!("{i} jobs queued")}
              </MessageBarBody></MessageBar>}, 
            ToastOptions::default().with_position(ToastPosition::Top)
          );
          //clicked.set(false);
          get_queues.set(());
        }
        Err(e) => toaster.dispatch_toast(
          || view!{
            <MessageBar intent=MessageBarIntent::Error><MessageBarBody>
                {e.to_string()}
            </MessageBarBody></MessageBar>}, 
          ToastOptions::default().with_position(ToastPosition::Top)
        )
      }
    }}
  );
  let dialog_clicked = RwSignal::new(false);
  let preact = move || {
    let parents = parents.get_untracked();
    if parents.is_empty() {
      act.dispatch(());
    } else {
      todo!()
    }
  };
  view!{<div style="margin-left:10px">
    <Button size=ButtonSize::Small on_click=move |_| preact()>"Add"</Button>
    " from branch: "<div style="display:inline-block;"><Select value=branch>{
      branches.into_iter().map(|b| {let name = b.name.clone(); view!{
        <option value=name>
          {b.name}" (Last commit "{b.commit.id[..8].to_string()}" at "{b.commit.created_at.to_string()}" by "{b.commit.author_name}")"
        </option>
      }}).collect_view()
    }</Select></div>
  </div>}
}
 */



