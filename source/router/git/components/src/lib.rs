#![recursion_limit = "256"]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

use flams_ontology::uris::ArchiveId;
use flams_router_buildqueue_base::select_queue;
use flams_router_git_base::{
    GitState,
    server_fns::{clone_to_queue, get_archives, get_branches, update_from_branch},
};
use flams_utils::vecmap::VecMap;
use flams_web_utils::components::{Spinner, display_error};
use leptos::{
    either::{Either, EitherOf4},
    prelude::*,
};
use std::num::NonZeroU32;

#[component]
pub fn Archives() -> impl IntoView {
    let r = Resource::new(|| (), |()| get_archives());
    view! {<Suspense fallback = || view!(<Spinner/>)>{move ||
      match r.get() {
        Some(Ok(projects)) if projects.is_empty() => EitherOf4::A("(No archives)"),
        Some(Err(e)) => EitherOf4::B(
          display_error(e.to_string().into())
        ),
        None => EitherOf4::C(view!(<Spinner/>)),
        Some(Ok(projects)) => EitherOf4::D(do_projects(projects))
      }
    }</Suspense>}
}

#[derive(Debug, Copy, Clone)]
struct QueueSignal(RwSignal<Option<NonZeroU32>>, RwSignal<()>);
#[derive(Debug, Clone, Default)]
struct ProjectTree {
    pub children: Vec<Either<Project, ProjectGroup>>,
}

#[derive(Debug, Clone)]
struct Project {
    pub id: u64,
    pub name: ArchiveId,
    pub url: String,
    pub path: String,
    pub state: RwSignal<GitState>,
    pub default_branch: Option<String>,
}
impl Eq for Project {}
impl PartialEq for Project {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
struct ProjectGroup {
    pub name: String,
    pub children: ProjectTree,
}

impl ProjectTree {
    fn add(&mut self, repo: flams_git::Project, id: ArchiveId, state: GitState) {
        let mut steps = id.steps().enumerate().peekable();
        let mut current = self;
        while let Some((i, step)) = steps.next() {
            macro_rules! insert {
                ($j:ident) => {
                    if steps.peek().is_none() {
                        current.children.insert(
                            $j,
                            Either::Left(Project {
                                url: repo.url,
                                id: repo.id,
                                path: repo.path,
                                name: id,
                                default_branch: repo.default_branch,
                                state: RwSignal::new(state),
                            }),
                        );
                        return;
                    } else {
                        current.children.insert(
                            $j,
                            Either::Right(ProjectGroup {
                                name: step.to_string(),
                                children: ProjectTree::default(),
                            }),
                        );
                        let Either::Right(e) = &mut current.children[$j] else {
                            unreachable!()
                        };
                        current = &mut e.children;
                    }
                };
            }
            match current.children.binary_search_by_key(&step, |e| match e {
                Either::Left(p) => p.name.steps().nth(i).unwrap_or_else(|| unreachable!()),
                Either::Right(g) => &g.name,
            }) {
                Err(j) => insert!(j),
                Ok(j) => {
                    let cont = match &current.children[j] {
                        Either::Left(_) => false,
                        Either::Right(_) => true,
                    };
                    if cont {
                        let Either::Right(e) = &mut current.children[j] else {
                            unreachable!()
                        };
                        current = &mut e.children;
                    } else {
                        insert!(j)
                    }
                }
            }
        }
    }
}

fn do_projects(vec: Vec<(flams_git::Project, ArchiveId, GitState)>) -> impl IntoView {
    use flams_web_utils::components::{Header, Leaf, Subtree, Tree};
    use thaw::Caption1Strong;

    let queue = RwSignal::new(None);
    let get_queues = RwSignal::new(());
    provide_context(QueueSignal(queue, get_queues));

    let mut tree = ProjectTree::default();
    for (p, id, state) in vec {
        tree.add(p, id, state);
    }
    fn inner_tree(tree: ProjectTree) -> impl IntoView {
        tree.children.into_iter().map(|c| match c {
        Either::Left(project) => Either::Left(view!{<Leaf><div>{move || project.state.with(|state| {
          if matches!(state,GitState::None){
            let state = project.state;
            Either::Right(unmanaged(project.name.clone(),project.id,state,project.path.clone(),project.url.clone()))
          } else {
            Either::Left(managed(project.name.clone(),project.id,state,project.default_branch.clone(),project.path.clone(),project.url.clone(),project.state))
          }
        })
      }</div></Leaf>}),
      Either::Right(group) => {
        Either::Right(view!{
          <Subtree><Header slot><div>{group.name}</div></Header>{inner_tree(group.children)}</Subtree>
        }.into_any())
      }
    }).collect_view()
    }
    view! {
      <Caption1Strong>"Archives on GitLab"</Caption1Strong>
      {move || {get_queues.get(); select_queue(queue)}}
      <Tree>{inner_tree(tree)}</Tree>
    }
}

fn managed(
    name: ArchiveId,
    _id: u64,
    state: &GitState,
    default_branch: Option<String>,
    path: String,
    git_url: String,
    and_then: RwSignal<GitState>,
) -> impl IntoView + use<> {
    use thaw::{Button, ButtonSize, Combobox, ComboboxOption};
    match state {
        GitState::Queued { commit, .. } => leptos::either::EitherOf3::A(view! {
          {path}
          " (commit "{commit[..8].to_string()}" currently queued)"
        }),
        GitState::Live { commit, updates } if updates.is_empty() => {
            leptos::either::EitherOf3::B(view! {
              {path}
              " (commit "{commit[..8].to_string()}" up to date)"
            })
        }
        GitState::Live { commit, updates } => leptos::either::EitherOf3::C({
            let mut updates = updates.clone();
            if let Some(branch) = default_branch {
                if let Some(main) = updates.iter().position(|(b, _)| b == &branch) {
                    let main = updates.remove(main);
                    updates.insert(0, main)
                }
            }
            let first = updates
                .first()
                .map(|(name, _)| name.clone())
                .unwrap_or_default();
            let branch = RwSignal::new(first.clone());
            let _ = Effect::new(move || {
                if branch.with(|s| s.is_empty()) {
                    branch.set(first.clone());
                }
            });
            let QueueSignal(queue, get_queues) = expect_context();
            let commit_map: VecMap<_, _> = updates.clone().into();
            let namecl = name.clone();
            let (act, v) = flams_web_utils::components::waiting_message_action(
                move |()| {
                    update_from_branch(
                        queue.get_untracked(),
                        namecl.clone(),
                        git_url.clone(),
                        branch.get_untracked(),
                    )
                },
                move |(i, q)| {
                    let commit = commit_map
                        .get(&branch.get_untracked())
                        .unwrap_or_else(|| unreachable!())
                        .clone();
                    get_queues.set(());
                    and_then.set(GitState::Queued {
                        commit: commit.id,
                        queue: q,
                    });
                    format!("{i} jobs queued")
                },
            );

            view! {
              {v}
              <span style="color:green">{path}
                " (commit "{commit[..8].to_string()}") Updates available: "
              </span>
              <div style="margin-left:10px">
                <Button size=ButtonSize::Small on_click=move |_| {act.dispatch(());}>"Update"</Button>
                " from branch: "
                <div style="display:inline-block;"><Combobox value=branch>{
                  updates.into_iter().map(|(name,commit)| {let vname = name.clone(); view!{
                    <ComboboxOption text=vname.clone() value=vname>
                      {name}<span style="font-size:x-small">" (Last commit "{commit.id[..8].to_string()}" at "{commit.created_at.to_string()}" by "{commit.author_name}")"</span>
                    </ComboboxOption>
                  }}).collect_view()
                }</Combobox></div>
              </div>
            }
        }),
        _ => unreachable!(),
    }
}

fn unmanaged(
    name: ArchiveId,
    id: u64,
    and_then: RwSignal<GitState>,
    path: String,
    git_url: String,
) -> impl IntoView {
    use thaw::{Button, ButtonSize, Combobox, ComboboxOption};
    let r = Resource::new(
        || (),
        move |()| async move {
            get_branches(id).await.map(|mut branches| {
                let main = branches.iter().position(|b| b.default);
                let main = main.map(|i| branches.remove(i));
                if let Some(b) = main {
                    branches.insert(0, b);
                }
                let release = branches.iter().position(|b| b.name == "release");
                let release = release.map(|i| branches.remove(i));
                (branches, release.is_some())
            })
        },
    );
    view! {
      <span style="color:grey">{path}" (unmanaged) "</span>
      <Suspense fallback=|| view!(<flams_web_utils::components::Spinner/>)>{move ||
        match r.get() {
          Some(Err(e)) => leptos::either::EitherOf3::B(flams_web_utils::components::display_error(e.to_string().into())),
          None => leptos::either::EitherOf3::C(view!(<flams_web_utils::components::Spinner/>)),
          Some(Ok((branches,has_release))) => leptos::either::EitherOf3::A({
            let first = branches.first().map(|f| f.name.clone()).unwrap_or_default();
            let branch = RwSignal::new(first.clone());
            let _ = Effect::new(move || if branch.with(|s| s.is_empty()) {
              branch.set(first.clone());
            });
            let QueueSignal(queue,get_queues) = expect_context();
            let name = name.clone();
            let git_url = git_url.clone();
            let commit_map : VecMap<_,_> = branches.iter().map(|b| (b.name.clone(),b.commit.clone())).collect();
            let (act,v) = flams_web_utils::components::waiting_message_action(
              move |()| clone_to_queue(queue.get_untracked(),name.clone(),git_url.clone(),branch.get_untracked(),has_release),
              move |(i,q)| {
                let commit = commit_map.get(&branch.get_untracked()).unwrap_or_else(|| unreachable!()).clone();
                get_queues.set(());
                and_then.set(GitState::Queued{commit:commit.id,queue:q});
                format!("{i} jobs queued")
              }
            );
            view!{<div style="margin-left:10px">{v}
            <Button size=ButtonSize::Small on_click=move |_| {act.dispatch(());}>"Add"</Button>
              " from branch: "<div style="display:inline-block;"><Combobox value=branch>{
                branches.into_iter().map(|b| {let name = b.name.clone(); view!{
                  <ComboboxOption value=name.clone() text=name>
                    {b.name}<span style="font-size:x-small">" (Last commit "{b.commit.id[..8].to_string()}" at "{b.commit.created_at.to_string()}" by "{b.commit.author_name}")"</span>
                  </ComboboxOption>
                }}).collect_view()
              }</Combobox></div></div>
            }
          })
        }
      }</Suspense>
    }
}
