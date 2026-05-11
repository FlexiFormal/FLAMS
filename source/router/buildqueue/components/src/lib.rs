#![recursion_limit = "256"]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

use flams_router_base::{LoginState, require_login, ws::WebSocket};
use flams_router_base::{maybe_lazy, ws};
#[cfg(feature = "hydrate")]
use flams_router_buildqueue_base::server_fns::get_log;
use flams_router_buildqueue_base::{QueueInfo, RepoInfo, server_fns};
use flams_router_content::checks::{DocumentCheckResult, ResultExt};
use flams_router_git_base::server_fns::{get_new_commits, update_from_branch};
use flams_utils::vecmap::VecMap;
use flams_web_utils::components::wait_and_then_fn;
use ftml_component_utils::{Collapsible, Header, LazyCollapsible};
use ftml_dom::utils::css::inject_css;
use ftml_ontology::utils::time::{Delta, Eta};
use ftml_uris::{ArchiveId, DocumentUri};
use leptos::{either::EitherOf4, prelude::*};
use leptos_router::hooks::use_params_map;
use std::num::NonZeroU32;

#[derive(Copy, Clone)]
struct UpdateQueues(RwSignal<()>);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Entry {
    id: u32,
    archive: ArchiveId,
    rel_path: String,
    #[cfg(feature = "hydrate")]
    steps: RwSignal<VecMap<String, TaskState>>,
    #[cfg(not(feature = "hydrate"))]
    steps: VecMap<String, TaskState>,
}

impl Entry {
    #[cfg(not(feature = "hydrate"))]
    fn as_view(&self) -> impl IntoView + use<> {
        view! {
          <li>{format!("[{}]{}",self.archive,self.rel_path)}</li>
        }
    }

    #[cfg(feature = "hydrate")]
    fn as_view(&self) -> AnyView {
        use flams_router_base::vscode_link;

        let vscode = vscode_link(&self.archive, &self.rel_path);

        let title = format!("[{}]{}", self.archive, self.rel_path);
        let total = self.steps.with_untracked(|v| v.0.len());
        let steps = self.steps;
        let current = move || {
            steps.with(|v| {
                v.iter()
                    .enumerate()
                    .find_map(|(i, (e, s))| {
                        if *s == TaskState::Done {
                            None
                        } else {
                            Some((i + 1, e.clone()))
                        }
                    })
                    .unwrap_or_else(|| (total, "Done".to_string()))
            })
        };
        let rel_path = self.rel_path.clone();
        let archive = self.archive.clone();
        view! {
          <li><Collapsible>
            <Header slot>
              <b>
                {title}
                {move || {let (i,s) = current(); format!(" ({i}/{total}) {s}")}}
                " "{vscode}
              </b>
            </Header>
            <ol>
            {let rel_path = rel_path.clone();
              let archive = archive.clone();
              move || steps.get().iter().map(|(t,e)|
              view!(<li>{e.into_view(t.clone(),&archive,&rel_path)}</li>)
            ).collect_view()}
            </ol>
          </Collapsible></li>
        }
        .into_any()
    }
}

#[cfg(feature = "ssr")]
impl From<flams_system::building::QueueEntry> for Entry {
    fn from(e: flams_system::building::QueueEntry) -> Self {
        #[cfg(feature = "hydrate")]
        {
            unreachable!()
        }
        #[cfg(not(feature = "hydrate"))]
        Self {
            id: e.id.into(),
            archive: e.archive,
            rel_path: e.rel_path.to_string(),
            steps: e
                .steps
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskState {
    Running,
    Queued,
    Blocked,
    Done,
    Failed,
    None,
}
impl TaskState {
    #[cfg(feature = "hydrate")]
    fn into_view(self, t: String, archive: &ArchiveId, rel_path: &str) -> AnyView {
        match self {
            Self::Running => view! {<i style="color:yellow">{t}" (Running)"</i>}.into_any(),
            Self::Queued | Self::Blocked | Self::None => {
                view! {<span style="color:gray">{t}" (...)"</span>}.into_any()
            }
            Self::Done => {
                let archive = archive.clone();
                let rel_path = rel_path.to_string();
                let tc = t.clone();
                view! {
                  <LazyCollapsible>
                    <Header slot><span style="color:green">{t}" (Done)"</span></Header>
                    {
                      let archive = archive.clone();
                      let rel_path = rel_path.clone();
                      let tc = tc.clone();
                      let queue = expect_context::<AllQueues>().selected.get_untracked();
                      require_login(Box::new(move || wait_and_then_fn(
                          move || get_log(queue,archive.clone(),rel_path.clone(),tc.clone()),
                          |s| do_log(s)
                      )))
                    }
                  </LazyCollapsible>
                }
                .into_any()
            }
            Self::Failed => {
                let archive = archive.clone();
                let rel_path = rel_path.to_string();
                let tc = t.clone();
                view! {
                  <LazyCollapsible>
                    <Header slot><span style="color:red">{t}" (Failed)"</span></Header>
                    {
                      let archive = archive.clone();
                      let rel_path = rel_path.clone();
                      let tc = tc.clone();
                      let queue = expect_context::<AllQueues>().selected.get_untracked();
                      require_login(Box::new(move || wait_and_then_fn(
                          move || get_log(queue,archive.clone(),rel_path.clone(),tc.clone()),
                          do_log
                      )))
                    }
                  </LazyCollapsible>
                }
                .into_any()
            }
        }
    }
}

fn do_log(s: either::Either<String, String>) -> AnyView {
    use ftml_component_utils::Scrollbar;
    view! {<Scrollbar style="max-height: 160px;max-width:80vw;border:2px solid black;padding:5px;">{
        match s {
            either::Left(s) => leptos::either::Either::Left(view!{
                <pre style="width:fit-content;font-size:smaller;">{s}</pre>
            }),
            either::Right(v) => leptos::either::Either::Right({
                ftml_solver_trace::results::DocumentCheckResult::from_json(&v)
                    .map_or_else(|_| view!{<pre>{v}</pre>}.into_any(),|e| e.render())
            })
        }
    }</Scrollbar>}
    .into_any()
}

#[cfg(feature = "ssr")]
impl From<flams_system::building::TaskState> for TaskState {
    fn from(e: flams_system::building::TaskState) -> Self {
        use flams_system::building::TaskState;
        match e {
            TaskState::Running => Self::Running,
            TaskState::Queued => Self::Queued,
            TaskState::Blocked => Self::Blocked,
            TaskState::Done => Self::Done,
            TaskState::Failed => Self::Failed,
            TaskState::None => Self::None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum QueueMessage {
    Idle(Vec<Entry>),
    Started {
        running: Vec<Entry>,
        queue: Vec<Entry>,
        blocked: Vec<Entry>,
        failed: Vec<Entry>,
        done: Vec<Entry>,
    },
    Finished {
        failed: Vec<Entry>,
        done: Vec<Entry>,
    },
    TaskStarted {
        id: u32,
        target: String,
    },
    TaskSuccess {
        id: u32,
        target: String,
        eta: Eta,
    },
    TaskFailed {
        id: u32,
        target: String,
        eta: Eta,
    },
    TaskBlocked {
        id: u32,
        target: String,
        eta: Eta,
    },
}
#[cfg(feature = "ssr")]
impl From<flams_system::building::QueueMessage> for QueueMessage {
    fn from(e: flams_system::building::QueueMessage) -> Self {
        use flams_system::building::QueueMessage;
        match e {
            QueueMessage::Idle(v) => Self::Idle(v.into_iter().map(Into::into).collect()),
            QueueMessage::Started {
                running,
                queue,
                blocked,
                failed,
                done,
            } => Self::Started {
                running: running.into_iter().map(Into::into).collect(),
                queue: queue.into_iter().map(Into::into).collect(),
                blocked: blocked.into_iter().map(Into::into).collect(),
                failed: failed.into_iter().map(Into::into).collect(),
                done: done.into_iter().map(Into::into).collect(),
            },
            QueueMessage::Finished { failed, done } => Self::Finished {
                failed: failed.into_iter().map(Into::into).collect(),
                done: done.into_iter().map(Into::into).collect(),
            },
            QueueMessage::TaskStarted { id, target } => Self::TaskStarted {
                id: id.into(),
                target: target.to_string(),
            },
            QueueMessage::TaskSuccess { id, target, eta } => Self::TaskSuccess {
                id: id.into(),
                target: target.to_string(),
                eta,
            },
            QueueMessage::TaskFailed { id, target, eta } => Self::TaskFailed {
                id: id.into(),
                target: target.to_string(),
                eta,
            },
            QueueMessage::TaskBlocked { id, target, eta } => Self::TaskBlocked {
                id: id.into(),
                target: target.to_string(),
                eta,
            },
        }
    }
}

// ----------------------------------------------------------------------------------

maybe_lazy!(QueuesTop = queues_top());

//#[component]
pub fn queues_top() -> AnyView {
    use flams_web_utils::components::{Layout, LayoutHeader};
    use ftml_component_utils::{Divider, Spinner, Tab, TabList};

    let update = UpdateQueues(RwSignal::new(()));
    provide_context(update);
    (move || {
        let () = update.0.get();
        let params = use_params_map();
        let id = move || params.read().get("queue");

        require_login(Box::new(move || {
            wait_and_then_fn(server_fns::get_queues, move |v| {
                if v.is_empty() {
                    return view!(<div>"(No running queues)"</div>).into_any();
                }
                let queues = AllQueues::new(v);
                if let Some(id) = params.read_untracked().get("queue") && let Ok(id) = id.parse() {
                    queues.selected.update_untracked(|v| *v = id);
                }
                provide_context(queues);
                let selected_value = RwSignal::new(queues.selected.get_untracked().to_string());
                let _ = Effect::new(move |_| {
                    let value = selected_value.get();
                    let selected = queues.selected.get_untracked();
                    let value = value.parse().unwrap_or_else(|_| unreachable!());
                    if selected != value {
                        queues.selected.set(value);
                    }
                });
                view! {<Layout>
                    <LayoutHeader slot>
                        <TabList selected_value>
                            <For each=move || queues.queues.get() key=|e| e.0 children=move |(i,_)| view!{
                            <Tab value=i.to_string()>{
                                queues.queue_names.with_untracked(|m| m.get(&i).cloned()).unwrap_or_else(|| unreachable!())
                            }</Tab>
                            }/>
                        </TabList>
                        <div style="margin:10px"><Divider/></div>
                    </LayoutHeader>
                    {move || {
                        //let curr = queues.selected.get();
                        queues.show.update_untracked(|v| *v = false);
                        QueueSocket::run(queues);
                        move || view! {
                        <Show when=move || queues.show.get() fallback=|| view!(<Spinner/>)>{
                            let ls = move || {
                                let curr = queues.selected.get();
                                (curr,queues.queues.with(|m| m.get(&curr).copied()).unwrap_or_else(|| unreachable!()))
                            };
                            move || {
                                let (curr,ls) = ls();
                                match ls.get() {
                                    QueueData::Idle(v) => {
                                        idle(curr,v)
                                    },
                                    QueueData::Running(r) => {
                                        running(curr,r)
                                    },
                                    QueueData::Finished(failed,done) => finished(curr,failed,done),
                                    QueueData::Empty => view!(<div>"Other"</div>).into_any()
                                }
                            }
                        }</Show>
                        }
                    }}
                </Layout>}.into_any()
            })
        }) as _)
    }).into_any()
}

#[allow(clippy::too_many_lines)]
fn repos(queue_id: NonZeroU32, allowed: bool) -> AnyView {
    use ftml_component_utils::{BoldCaption, Table, TableCell, TableHeader, TableRow};
    if matches!(LoginState::get(), LoginState::NoAccounts) {
        return ().into_any();
    }
    let queues: AllQueues = expect_context();
    let Some(repos) = queues
        .queue_repos
        .with_untracked(|v| v.get(&queue_id).cloned())
        .flatten()
    else {
        return ().into_any();
    };
    if repos.is_empty() {
        return ().into_any();
    }
    let style = if allowed { "" } else { "color:gray;" };
    inject_css("flams-repo-table", include_str!("repo-table.css"));
    view! {<div style="margin-left:45px;width:fit-content;"><Collapsible>
          <Header slot><BoldCaption>"Archives"</BoldCaption></Header>
          <Table class="flams-repo-table">
            <TableHeader slot>
              <TableCell><BoldCaption>"Archive"</BoldCaption></TableCell>
              <TableCell><BoldCaption>"Branch"</BoldCaption></TableCell>
              <TableCell><BoldCaption>"Commit"</BoldCaption></TableCell>
            </TableHeader>
            {
              repos.into_iter().map(|d| match d {
                RepoInfo::Copy(id) => leptos::either::Either::Left(view!{
                  <TableRow>
                    <TableCell><span style=style>{id.to_string()}</span></TableCell>
                    <TableCell>"(Copied from MathHub)"</TableCell>
                  </TableRow>
                }),
                RepoInfo::Git{id,branch,commit,remote/*,updates */} => leptos::either::Either::Right({
                  let updates = RwSignal::<Option<Vec<(String,flams_backend_types::git::Commit)>>>::new(None);
                  let style = move || if allowed {
                    updates.with(|updates| updates.as_ref().map_or("",|updates| if updates.is_empty() {
                      "color:green;"
                    } else {
                      "color:yellowgreen;"
                    }))
                  } else {style};
                  let idstr = id.to_string();
                  view!{
                    <TableRow>
                      <TableCell><span style=style>{idstr}</span></TableCell>
                      <TableCell>{branch}</TableCell>
                      <TableCell>
                        {commit.id[..8].to_string()}" at "{commit.created_at.to_string()}" by "{commit.author_name}
                        {if allowed {Some(move || updates.with(|up| {
                          let Some(up) = up else {
                            let aid = id.clone();
                            let toaster = ftml_component_utils::toasts::ToasterInjection::expect_context();
                            let get_updates = Action::new(move |()| {
                              let id = aid.clone();
                              async move {
                                match get_new_commits(Some(queue_id),id).await {
                                  Ok(v) => updates.set(Some(v)),
                                  Err(e) => flams_web_utils::components::error_with_toaster(e, toaster),
                                }
                              }
                            });
                            return leptos::either::EitherOf3::A(view!{
                              <button on:click=move |_| {get_updates.dispatch(());}>"Check for updates"</button>
                            });
                          };
                          if up.is_empty() {
                            return leptos::either::EitherOf3::B(view!(<span style=style>" (already up-to-date)"</span>))
                          }
                          let updates = up.clone();

                          leptos::either::EitherOf3::C({
                            use ftml_component_utils::{Button,ButtonSize,Combobox,ComboboxOption};
                            let first = updates.first().map(|(name,_)| name.clone()).unwrap_or_default();
                            let branch = RwSignal::new(first.clone());
                            let _ = Effect::new(move || if branch.with(String::is_empty) {
                              branch.set(first.clone());
                            });
                            let update : UpdateQueues = expect_context();
                            //let commit_map:VecMap<_,_> = updates.clone().into();
                            let archive = id.clone();
                            let remote = remote.clone();
                            let act = flams_web_utils::components::message_action(
                              move |()| update_from_branch(Some(queue_id),archive.clone(),remote.clone(),branch.get_untracked()),
                              move |(i,_)| {
                                update.0.set(());
                                format!("{i} jobs queued")
                              }
                            );
                            view!{
                              <span style="color:green">
                                " Updates available: "
                              </span>
                              <div style="margin-left:10px">
                                <Button size=ButtonSize::Small on_click=move |_| {act.dispatch(());}>"Update"</Button>
                                " from branch: "
                                <div style="display:inline-block;"><Combobox selected_options=branch>{
                                  updates.into_iter().map(|(name,commit)| {let vname = name.clone(); view!{
                                    <ComboboxOption text=vname.clone() value=vname>
                                      {name}<span style="font-size:x-small">" (Last commit "{commit.id[..8].to_string()}" at "{commit.created_at.to_string()}" by "{commit.author_name}")"</span>
                                    </ComboboxOption>
                                  }}).collect_view()
                                }</Combobox></div>
                              </div>
                            }
                          })
                        }))} else {None}}
                      </TableCell>
                    </TableRow>
                  }
                }),
              }).collect_view()
            }
          </Table>
        </Collapsible></div>
    }.into_any()
}

fn delete_action(id: NonZeroU32) -> Action<(), ()> {
    use ftml_component_utils::toasts::ToasterInjection;
    let update: UpdateQueues = expect_context();
    let toaster = ToasterInjection::expect_context();
    Action::new(move |()| async move {
        match flams_router_buildqueue_base::server_fns::delete(id).await {
            Ok(()) => update.0.set(()),
            Err(e) => flams_web_utils::components::error_with_toaster(e.to_string(), toaster),
        }
    })
}

fn idle(id: NonZeroU32, ls: RwSignal<Vec<Entry>>) -> AnyView {
    use ftml_component_utils::Button;
    let act = Action::<(), Result<(), ServerFnError<String>>>::new(move |()| {
        flams_router_buildqueue_base::server_fns::run(id)
    });
    let del = delete_action(id);
    view! {
      <div style="width:100%"><div style="position:fixed;right:20px">
          <Button on_click=move |_| {act.dispatch(());}>"Run"</Button>
          <Button on_click=move |_| {del.dispatch(());}>"Delete"</Button>
      </div></div>
      {repos(id,true)}
      <ol reversed style="margin-left:30px">
        <For each=move || ls.get() key=|e| e.id children=|e| e.as_view()/>
      </ol>
    }
    .into_any()
}

fn running(id: NonZeroU32, queue: RunningQueue) -> AnyView {
    use ftml_component_utils::{AnchorMenu, AnchorMenuEntry, Button};
    let del = delete_action(id);
    let RunningQueue {
        running,
        queue,
        blocked,
        failed,
        done,
        eta,
    } = queue;
    view! {
      <div style="position:fixed;right:20px;z-index:5"><AnchorMenu>
          <AnchorMenuEntry href="#running">"Running"</AnchorMenuEntry>
          <AnchorMenuEntry href="#queued">"Queued"</AnchorMenuEntry>
          <AnchorMenuEntry href="#blocked">"Blocked"</AnchorMenuEntry>
          <AnchorMenuEntry href="#failed">"Failed"</AnchorMenuEntry>
          <AnchorMenuEntry href="#finished">"Finished"</AnchorMenuEntry>
      </AnchorMenu></div>
      {repos(id,false)}
      <div style="text-align:left;">
          {eta.into_view()}
          <div style="width:100%"><div style="position:fixed;right:20px">
              <Button on_click=move |_| {del.dispatch(());}>"Abort and Delete"</Button>
          </div></div>
          <h3 id="running">"Running ("{move || running.with(Vec::len)}")"</h3>
          <ul style="margin-left:30px"><For each=move || running.get() key=|e| e.id children=|e| e.as_view()/></ul>
          <h3 id="queued">"Queued ("{move || queue.with(Vec::len)}")"</h3>
          <ul style="margin-left:30px"><For each=move || queue.get() key=|e| e.id children=|e| e.as_view()/></ul>
          <h3 id="blocked">"Blocked ("{move || blocked.with(Vec::len)}")"</h3>
          <ul style="margin-left:30px"><For each=move || blocked.get() key=|e| e.id children=|e| e.as_view()/></ul>
          <h3 id="failed">"Failed ("{move || failed.with(Vec::len)}")"</h3>
          <ul style="margin-left:30px"><For each=move || failed.get() key=|e| e.id children=|e| e.as_view()/></ul>
          <h3 id="finished">"Finished ("{move || done.with(Vec::len)}")"</h3>
          <ul style="margin-left:30px"><For each=move || done.get() key=|e| e.id children=|e| e.as_view()/></ul>
      </div>
    }.into_any()
}

fn finished(id: NonZeroU32, failed: Vec<Entry>, done: Vec<Entry>) -> AnyView {
    use ftml_component_utils::{AnchorMenu, AnchorMenuEntry, Button};
    let requeue = Action::new(move |()| flams_router_buildqueue_base::server_fns::requeue(id));
    let num_failed = failed.len();
    let num_done = done.len();
    let del = delete_action(id);
    view! {
      <div style="width:100%"><div style="position:fixed;right:120px;z-index:10">
          {if num_failed > 0 {Some(view!(
            <Button on_click=move |_| {requeue.dispatch(());}>"Requeue Failed"</Button>
          ))} else { None }}
          {migrate_button(id,num_failed)}
          <Button on_click=move |_| {del.dispatch(());}>"Delete"</Button>
      </div></div>
      <div style="position:fixed;right:20px;z-index:5"><AnchorMenu>
          <AnchorMenuEntry href="#failed">"Failed"</AnchorMenuEntry>
          <AnchorMenuEntry href="#finished">"Finished"</AnchorMenuEntry>
      </AnchorMenu></div>
      {repos(id,true)}
      <div style="text-align:left;">
          <h3 id="failed">"Failed ("{num_failed}")"</h3>
          <ul style="margin-left:30px">{
            failed.iter().map(Entry::as_view).collect_view()
          }</ul>
          <h3 id="finished">"Finished ("{num_done}")"</h3>
          <ul style="margin-left:30px">{
            done.iter().map(Entry::as_view).collect_view()
          }</ul>
      </div>
    }
    .into_any()
}

fn migrate_button(id: NonZeroU32, num_failed: usize) -> AnyView {
    use ftml_component_utils::{
        BoldCaption, Button, Dialog, DialogBody, DialogContent, DialogSurface, Divider,
    };
    use leptos::either::EitherOf3;
    if matches!(LoginState::get(), LoginState::NoAccounts) {
        return ().into_any();
    }
    let update: UpdateQueues = expect_context();
    let migrate = flams_web_utils::components::message_action(
        move |()| flams_router_buildqueue_base::server_fns::migrate(id),
        move |i| {
            update.0.set(());
            format!("{i} archives migrated")
        },
    );
    if num_failed == 0 {
        view! {
          <Button on_click=move |_| {migrate.dispatch(());}>"Migrate"</Button>
        }
        .into_any()
    } else {
        let clicked = RwSignal::new(false);
        view! {
          <Button on_click=move |_| {clicked.set(true);}>"Migrate"</Button>
          <Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
            <BoldCaption><span style="color:red">WARNING</span></BoldCaption>
            <Divider/>
            <p>{num_failed}" jobs have failed to build!"<br/>"Migrate anyway?"</p>
            <div>
              <div style="width:min-content;margin-left:auto;">
                <Button on_click=move |_| {migrate.dispatch(());}>"Force Migration"</Button>
              </div>
            </div>
          </DialogContent></DialogBody></DialogSurface></Dialog>
        }
        .into_any()
    }
}

#[derive(Clone)]
pub struct QueueSocket {
    #[cfg(feature = "ssr")]
    listener:
        Option<flams_utils::change_listener::ChangeListener<flams_system::building::QueueMessage>>,
    #[cfg(all(not(feature = "docs-only"), feature = "hydrate"))]
    socket: leptos::web_sys::WebSocket,
    #[cfg(feature = "docs-only")]
    socket: (),
    #[cfg(feature = "hydrate")]
    _running: RwSignal<bool>,
}
impl WebSocket<NonZeroU32, QueueMessage> for QueueSocket {
    const SERVER_ENDPOINT: &'static str = "/ws/queue";
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl ws::WebSocketServer<NonZeroU32, QueueMessage> for QueueSocket {
    async fn new(account: LoginState, _db: ws::DBBackend) -> Option<Self> {
        match account {
            LoginState::Admin
            | LoginState::NoAccounts
            | LoginState::User { is_admin: true, .. } => {
                let listener = None; //flams_system::logger().listener();
                Some(Self {
                    listener,
                    #[cfg(feature = "hydrate")]
                    _running: RwSignal::new(false),
                    #[cfg(feature = "hydrate")]
                    socket: unreachable!(),
                })
            }
            _ => None,
        }
    }
    async fn next(&mut self) -> Option<QueueMessage> {
        loop {
            match &mut self.listener {
                None => tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.5)).await,
                Some(l) => return l.read().await.map(Into::into),
            }
        }
    }
    async fn handle_message(&mut self, msg: NonZeroU32) -> Option<QueueMessage> {
        let (lst, msg) = flams_system::building::queue_manager::QueueManager::get()
            .with_queue(msg.into(), |q| q.map(|q| (q.listener(), q.state_message())))?;
        self.listener = Some(lst);
        Some(msg.into())
    }
    async fn on_start(&mut self, _: &mut ws::AxumWS) {}
}

#[cfg(feature = "hydrate")]
impl ws::WebSocketClient<NonZeroU32, QueueMessage> for QueueSocket {
    fn new(ws: leptos::web_sys::WebSocket) -> Self {
        Self {
            #[cfg(not(feature = "docs-only"))]
            socket: ws,
            #[cfg(feature = "docs-only")]
            socket: (),
            _running: RwSignal::new(false),
            #[cfg(feature = "ssr")]
            listener: unreachable!(),
        }
    }
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket {
        #[cfg(not(feature = "docs-only"))]
        {
            &mut self.socket
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }
    #[allow(clippy::used_underscore_binding)]
    fn on_open(&self) -> Option<Box<dyn FnMut()>> {
        let running = self._running;
        Some(Box::new(move || {
            running.set(true);
        }))
    }
}

#[cfg(all(feature = "ssr", not(feature = "hydrate")))]
impl QueueSocket {
    fn run(_: AllQueues) {
        Self::force_start_server();
    }
}

#[cfg(feature = "hydrate")]
impl QueueSocket {
    fn run(queues: AllQueues) {
        use ws::WebSocketClient;
        Self::force_start_client(
            move |msg| {
                //tracing::warn!("Starting!");
                let current = queues.selected.get_untracked();
                queues.queues.with_untracked(|queues| {
                    let Some(queue) = queues.get(&current) else {
                        tracing::error!("Queue not found: {current}");
                        return;
                    };
                    Self::do_msg(*queue, msg);
                });
                if !queues.show.get_untracked() {
                    queues.show.set(true);
                }
                None
            },
            move |mut socket| {
                #[allow(clippy::used_underscore_binding)]
                Effect::new(move |_| {
                    if socket._running.get() {
                        let current = queues.selected.get_untracked();
                        socket.send(&current);
                    }
                });
            },
        );
    }
    fn do_msg(queue: RwSignal<QueueData>, msg: QueueMessage) {
        match msg {
            QueueMessage::Idle(entries) => queue.set(QueueData::Idle(RwSignal::new(entries))),
            QueueMessage::Started {
                running,
                queue: actual_queue,
                blocked,
                failed,
                done,
            } => queue.set(QueueData::Running(RunningQueue {
                running: RwSignal::new(running),
                queue: RwSignal::new(actual_queue),
                blocked: RwSignal::new(blocked),
                failed: RwSignal::new(failed),
                done: RwSignal::new(done),
                eta: WrappedEta(RwSignal::new(Eta::default())),
            })),
            QueueMessage::Finished { failed, done } => queue.set(QueueData::Finished(failed, done)),
            QueueMessage::TaskStarted { id, mut target } => queue.with_untracked(|queue| {
                if let QueueData::Running(RunningQueue {
                    queue,
                    running,
                    blocked,
                    ..
                }) = queue
                {
                    let mut worked = false;
                    queue.update(|v| {
                        let Some((i, _)) = v.iter().enumerate().find(|(_, e)| e.id == id) else {
                            return;
                        };
                        worked = true;
                        let e = v.remove(i);
                        e.steps
                            .update(|m| m.insert(std::mem::take(&mut target), TaskState::Running));
                        running.update(|running| running.push(e));
                    });
                    if !worked {
                        blocked.update(|v| {
                            let Some((i, _)) = v.iter().enumerate().find(|(_, e)| e.id == id)
                            else {
                                return;
                            };
                            worked = true;
                            let e = v.remove(i);
                            e.steps.update(|m| m.insert(target, TaskState::Running));
                            running.update(|running| running.push(e));
                        });
                    }
                }
            }),
            QueueMessage::TaskSuccess { id, target, eta } => queue.with_untracked(|queue| {
                if let QueueData::Running(RunningQueue {
                    queue,
                    running,
                    done,
                    eta: etasignal,
                    ..
                }) = queue
                {
                    etasignal.0.set(eta);
                    running.update(|v| {
                        let Some((i, _)) = v.iter().enumerate().find(|(_, e)| e.id == id) else {
                            return;
                        };
                        let e = v.remove(i);
                        e.steps.update(|m| m.insert(target, TaskState::Done));
                        if e.steps.with_untracked(|v| {
                            v.iter()
                                .any(|(_, v)| *v == TaskState::Queued || *v == TaskState::Blocked)
                        }) {
                            queue.update(|v| v.push(e));
                        } else {
                            done.update(|v| v.push(e));
                        }
                    });
                }
            }),
            QueueMessage::TaskFailed { id, target, eta } => queue.with_untracked(|queue| {
                if let QueueData::Running(RunningQueue {
                    running,
                    failed,
                    eta: etasignal,
                    ..
                }) = queue
                {
                    etasignal.0.set(eta);
                    running.update(|v| {
                        let Some((i, _)) = v.iter().enumerate().find(|(_, e)| e.id == id) else {
                            return;
                        };
                        let e = v.remove(i);
                        e.steps.update(|m| m.insert(target, TaskState::Failed));
                        failed.update(|v| v.push(e));
                    });
                }
            }),
            QueueMessage::TaskBlocked { id, target, eta } => queue.with_untracked(|queue| {
                if let QueueData::Running(RunningQueue {
                    running,
                    blocked,
                    eta: etasignal,
                    ..
                }) = queue
                {
                    etasignal.0.set(eta);
                    running.update(|v| {
                        let Some((i, _)) = v.iter().enumerate().find(|(_, e)| e.id == id) else {
                            return;
                        };
                        let e = v.remove(i);
                        e.steps.update(|m| m.insert(target, TaskState::Blocked));
                        blocked.update(|v| v.push(e));
                    });
                }
            }),
        }
    }
}

#[derive(Clone, Copy)]
struct AllQueues {
    show: RwSignal<bool>,
    selected: RwSignal<NonZeroU32>,
    queue_names: RwSignal<VecMap<NonZeroU32, String>>,
    queue_repos: RwSignal<VecMap<NonZeroU32, Option<Vec<RepoInfo>>>>,
    queues: RwSignal<VecMap<NonZeroU32, RwSignal<QueueData>>>,
}

impl AllQueues {
    fn new(ids: Vec<QueueInfo>) -> Self {
        let queues = RwSignal::new(
            ids.iter()
                .map(|v| (v.id, RwSignal::new(QueueData::Empty)))
                .collect(),
        );
        let selected = ids.first().map_or_else(
            || NonZeroU32::new(1).unwrap_or_else(|| unreachable!()),
            |v| v.id,
        );
        let mut queue_names = VecMap::default();
        let mut queue_repos = VecMap::default();
        for d in ids {
            queue_names.insert(d.id, d.name);
            queue_repos.insert(d.id, d.archives);
        }
        Self {
            show: RwSignal::new(false),
            selected: RwSignal::new(selected),
            queues,
            queue_names: RwSignal::new(queue_names),
            queue_repos: RwSignal::new(queue_repos),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
enum QueueData {
    Idle(RwSignal<Vec<Entry>>),
    Running(RunningQueue),
    Empty,
    Finished(Vec<Entry>, Vec<Entry>),
}

#[derive(Clone, Copy)] //,serde::Serialize,serde::Deserialize)]
#[allow(dead_code)]
struct RunningQueue {
    running: RwSignal<Vec<Entry>>,
    queue: RwSignal<Vec<Entry>>,
    blocked: RwSignal<Vec<Entry>>,
    failed: RwSignal<Vec<Entry>>,
    done: RwSignal<Vec<Entry>>,
    eta: WrappedEta,
}

#[derive(Clone, Copy)]
struct WrappedEta(RwSignal<ftml_ontology::utils::time::Eta>);

#[allow(clippy::cast_precision_loss)]
impl WrappedEta {
    fn into_view(self) -> impl IntoView {
        use ftml_component_utils::ProgressBar;
        inject_css(
            "flams-eta",
            r"
.flams-progress-bar {height:10px;}
    ",
        );
        let pctg = Memo::new(move |_| {
            let eta = self.0.get();
            ((eta.done as f64 / eta.total as f64) * 1000.0).round() / 1000.0
        });
        let time_left = move || {
            let eta = self.0.get();
            if eta.time_left == Delta::default() {
                "N/A".to_string()
            } else {
                eta.time_left.max_seconds().to_string()
            }
        };
        view! {
          <div style="width:500px;"><ProgressBar class="flams-progress-bar" value=pctg/>
            {move || (pctg.get() * 100.0).to_string().chars().take(4).collect::<String>()} "%; ca. "{time_left}" remaining"
          </div>
        }
    }
}
