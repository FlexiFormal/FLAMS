use std::num::NonZeroU32;

use immt_ontology::uris::ArchiveId;
use immt_utils::{time::{Delta, Eta}, vecmap::VecMap};
use immt_web_utils::inject_css;
use leptos::prelude::*;

use crate::utils::{from_server_clone, from_server_copy, ws::WebSocket};

#[server(
  prefix="/api/buildqueue",
  endpoint="get_queues",
)]
#[allow(clippy::unused_async)]
pub async fn get_queues() -> Result<Vec<(NonZeroU32,String)>,ServerFnError<String>> {
  use crate::users::LoginState;
  use immt_system::building::queue_manager::QueueManager;
  
  let login = LoginState::get();
  if login != LoginState::Admin && login != LoginState::NoAccounts {
    return Err(format!("Not logged in: {login:?}").into());
  }
  Ok(QueueManager::get().all_queues().into_iter().map(|(k,v)| (k.into(),v.to_string())).collect())
}

#[server(
  prefix="/api/buildqueue",
  endpoint="run",
)]
#[allow(clippy::unused_async)]
pub async fn run(id:NonZeroU32) -> Result<(),ServerFnError<String>> {
  use crate::users::LoginState;
  use immt_system::building::queue_manager::QueueManager;
  
  let login = LoginState::get();
  if login != LoginState::Admin && login != LoginState::NoAccounts {
    return Err(format!("Not logged in: {login:?}").into());
  }
  let Ok(()) = QueueManager::get().start_queue(id.into()) else {
      return Err(format!("Queue {id} not found").into())
  };
  Ok(())
}

#[server(
  prefix="/api/buildqueue",
  endpoint="log",
)]
#[allow(clippy::unused_async)]
pub async fn get_log(archive:ArchiveId,rel_path:String,target:String) -> Result<String,ServerFnError<String>> {
  use crate::users::LoginState;
  use std::path::PathBuf;
  use immt_system::backend::{Backend,GlobalBackend};
  let login = LoginState::get();
  if login != LoginState::Admin && login != LoginState::NoAccounts {
    return Err("Not logged in".to_string().into());
  }    
  let Some(target) = immt_system::formats::BuildTarget::get_from_str(&target) else {
    return Err(format!("Target {target} not found").into())
  };
  let path = GlobalBackend::get().with_archive(&archive, |a| {
    let Some(a) = a else { return Err::<PathBuf,String>(format!("Archive {archive} not found")) };
    Ok(a.get_log(&rel_path, target))
  })?;
  let v = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
  Ok(String::from_utf8_lossy(&v).to_string())
}


#[component]
pub fn QueuesTop() -> impl IntoView {
  use thaw::{TabList,Tab,Divider,Layout};
  use immt_web_utils::components::Spinner;
  from_server_copy(true,get_queues,|v| {
    let queues = AllQueues::new(v);
    QueueSocket::run(queues);
    provide_context(queues);
    inject_css("immt-fullscreen", ".immt-fullscreen { width:100%; height:calc(100% - 44px - 21px) }");
    view!{<Show when=move || queues.show.get() fallback=|| view!(<Spinner/>)>
      <TabList selected_value=queues.selected.get().to_string()>
        <For each=move || queues.queues.get() key=|e| e.0 children=move |(i,_)| view!{
          <Tab value=i.to_string()>{
            queues.queue_names.get().get(&i).unwrap_or_else(|| unreachable!()).clone()
          }</Tab>
        } />
      </TabList>
      <div style="margin:10px"><Divider/></div>
      <Layout class="immt-fullscreen">{move || {
        let curr = queues.selected.get();
        let ls = *queues.queues.get_untracked().get(&curr).unwrap_or_else(|| unreachable!());
        match ls.get() {
          QueueData::Idle(v) => {
              idle(curr,v).into_any()
          },
          QueueData::Running(r) => {
              running(r).into_any()
          }
          QueueData::Empty => view!(<div>"Other"</div>).into_any()
      }
    }}</Layout>
    </Show>}
  })
}

fn idle(id:NonZeroU32,ls:RwSignal<Vec<Entry>>) -> impl IntoView {
  use thaw::Button;
  let act = Action::<(),Result<(),ServerFnError<String>>>::new(move |()| run(id));
  view!{
    <div style="width:100%"><div style="position:fixed;right:20px">
        <Button on_click=move |_| {act.dispatch(());}>"Run"</Button>
    </div></div>
    <ol reversed style="margin-left:30px">
      <For each=move || ls.get() key=|e| e.id children=|e| e.as_view()/>
    </ol>
  }
}

fn running(queue:RunningQueue) -> impl IntoView {
  use immt_web_utils::components::{AnchorLink,Anchor,Header};
  use thaw::Layout;
  let RunningQueue {running,queue,blocked,failed,done,eta} = queue;
  view!{
    <div style="position:fixed;right:20px;z-index:5"><Anchor>
        <AnchorLink href="#running"><Header slot>"Running"</Header></AnchorLink>
        <AnchorLink href="#queued"><Header slot>"Queued"</Header></AnchorLink>
        <AnchorLink href="#blocked"><Header slot>"Blocked"</Header></AnchorLink>
        <AnchorLink href="#failed"><Header slot>"Failed"</Header></AnchorLink>
        <AnchorLink href="#finished"><Header slot>"Finished"</Header></AnchorLink>
    </Anchor></div>
    <Layout content_style="text-align:left;">
        {eta.into_view()}
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
    </Layout>
  }
}

#[derive(Clone)]
pub struct QueueSocket {
  #[cfg(feature="ssr")]
  #[cfg_attr(docsrs, doc(cfg(feature = "ssr")))]
  listener: Option<immt_utils::change_listener::ChangeListener<immt_system::building::QueueMessage>>,
  #[cfg(all(not(doc),feature="hydrate"))]
  socket: leptos::web_sys::WebSocket,
  #[cfg(doc)]
  socket: (),
  #[cfg(feature="hydrate")]
  _running:RwSignal<bool>
}
impl WebSocket<NonZeroU32,QueueMessage> for QueueSocket {
  const SERVER_ENDPOINT: &'static str = "/ws/queue";
}

#[cfg(feature="ssr")]
#[cfg_attr(docsrs, doc(cfg(feature = "ssr")))]
#[async_trait::async_trait]
impl crate::utils::ws::WebSocketServer<NonZeroU32,QueueMessage> for QueueSocket {
    async fn new(account:crate::users::LoginState,_db:crate::server::db::DBBackend) -> Option<Self> {
        use crate::users::LoginState;
        match account {
            LoginState::Admin | LoginState::NoAccounts => {
                let listener = None;//immt_system::logger().listener();
                Some(Self {
                    listener,
                    #[cfg(feature="hydrate")] _running:RwSignal::new(false),
                    #[cfg(feature="hydrate")] socket:unreachable!()
                })
            }
            _ => None
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
    async fn handle_message(&mut self,msg:NonZeroU32) -> Option<QueueMessage> {
      let (lst,msg) = immt_system::building::queue_manager::QueueManager::get()
        .get_queue(msg.into(), |q| 
          q.map(|q| (q.listener(),q.state_message()))
      )?;
      self.listener = Some(lst);
      Some(msg.into())
    }
    async fn on_start(&mut self,_:&mut axum::extract::ws::WebSocket) {}
}

#[cfg(feature="hydrate")]
#[cfg_attr(docsrs, doc(cfg(feature = "hydrate")))]
impl crate::utils::ws::WebSocketClient<NonZeroU32,QueueMessage> for QueueSocket {
    fn new(ws: leptos::web_sys::WebSocket) -> Self { Self{
        #[cfg(not(doc))]
        socket:ws,
        #[cfg(doc)]
        socket:(),
        _running:RwSignal::new(false),
        #[cfg(feature="ssr")] listener:unreachable!()
    } }
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket {&mut self.socket }
    fn on_open(&self) -> Option<Box<dyn FnMut()>> {
      let running = self._running;
      Some(Box::new(move || {
        running.set(true);
      }))
    }
}

#[cfg(all(feature="ssr",not(feature="hydrate")))]
impl QueueSocket {
  fn run(_:AllQueues) {
    Self::force_start_server();
  }
}

#[cfg(feature="hydrate")]
impl QueueSocket {
  fn run(queues:AllQueues) {
    use crate::utils::ws::WebSocketClient;
    Self::force_start_client(move |msg| {
      let current = queues.selected.get_untracked();
      queues.queues.with_untracked(|queues| {
        let Some(queue) = queues.get(&current) else {
          tracing::error!("Queue not found: {current}");
          return
        };
        Self::do_msg(*queue, msg);
      });
      if !queues.show.get_untracked() {
        queues.show.set(true);
      }
      None
    },move |mut socket| {
      Effect::new(move |_| {
        if socket._running.get() {
          let current = queues.selected.get();
          socket.send(&current);
        }
      });
    });
  }
  fn do_msg(queue:RwSignal<QueueData>,msg:QueueMessage) {
    match msg {
      QueueMessage::Idle(entries) =>
        queue.set(QueueData::Idle(RwSignal::new(entries))),
      QueueMessage::Started { running, queue:actual_queue, blocked, failed, done } =>
        queue.set(QueueData::Running(RunningQueue {
          running:RwSignal::new(running),
          queue:RwSignal::new(actual_queue),
          blocked:RwSignal::new(blocked),
          failed:RwSignal::new(failed),
          done:RwSignal::new(done),
          eta:WrappedEta(RwSignal::new(Eta::default()))
        })),
      QueueMessage::TaskStarted { id, target } =>
        queue.with_untracked(|queue| 
          if let QueueData::Running(RunningQueue {queue,running,..}) = queue {
            queue.update(|v| {
              let Some((i,_)) = v.iter().enumerate().find(|(_,e)| e.id == id) else {return};
              let e = v.remove(i);
              e.steps.update(|m| m.insert(target,TaskState::Running));
              running.update(|running| running.push(e));
            });
          }
        ),
      QueueMessage::TaskSuccess { id, target, eta } =>
        queue.with_untracked(|queue| 
          if let QueueData::Running(RunningQueue {queue,running,done,eta:etasignal,..}) = queue {
            etasignal.0.set(eta);
            running.update(|v| {
              let Some((i,_)) = v.iter().enumerate().find(|(_,e)| e.id == id) else {return};
              let e = v.remove(i);
              e.steps.update(|m| m.insert(target,TaskState::Done));
              if e.steps.with_untracked(|v| v.iter().any(
                |(_,v)| *v == TaskState::Queued || *v == TaskState::Blocked
              )) {
                queue.update(|v| v.push(e));
              } else {
                done.update(|v| v.push(e));
              }
            });
          }
        ),
      QueueMessage::TaskFailed { id, target, eta } =>
        queue.with_untracked(|queue| 
          if let QueueData::Running(RunningQueue {running,failed,eta:etasignal,..}) = queue {
            etasignal.0.set(eta);
            running.update(|v| {
              let Some((i,_)) = v.iter().enumerate().find(|(_,e)| e.id == id) else {return};
              let e = v.remove(i);
              e.steps.update(|m| m.insert(target,TaskState::Failed));
              failed.update(|v| v.push(e));
            });
          }
        )
    }
  }
}


#[derive(Clone,Copy)]
struct AllQueues {
    show:RwSignal<bool>,
    selected:RwSignal<NonZeroU32>,
    queue_names:RwSignal<VecMap<NonZeroU32,String>>,
    queues:RwSignal<VecMap<NonZeroU32,RwSignal<QueueData>>>
}

impl AllQueues {
  fn new(ids:Vec<(NonZeroU32,String)>) -> Self {
    let queues = RwSignal::new(ids.iter().map(|(id,_)| (*id,RwSignal::new(QueueData::Empty))).collect());
    let selected = ids.first().map_or_else(||NonZeroU32::new(1).unwrap_or_else(|| unreachable!()),|(i,_)| *i);
    let queue_names = RwSignal::new(ids.into());
      Self {show:RwSignal::new(false),selected:RwSignal::new(selected),queues,queue_names}
  }
}

#[derive(Clone)]
#[allow(dead_code)]
enum QueueData {
    Idle(RwSignal<Vec<Entry>>),
    Running(RunningQueue),
    Empty
}

#[derive(Clone,Copy)]//,serde::Serialize,serde::Deserialize)]
#[allow(dead_code)]
struct RunningQueue {
    running:RwSignal<Vec<Entry>>,
    queue:RwSignal<Vec<Entry>>,
    blocked:RwSignal<Vec<Entry>>,
    failed:RwSignal<Vec<Entry>>,
    done:RwSignal<Vec<Entry>>,
    eta:WrappedEta
}

#[derive(Clone,Copy)]
struct WrappedEta(RwSignal<Eta>);

#[allow(clippy::cast_precision_loss)]
impl WrappedEta {
  fn into_view(self) -> impl IntoView {
    use thaw::ProgressBar;
    inject_css("immt-eta", r"
.immt-progress-bar {height:10px;}
    ");
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
    view!{
      <div style="width:500px;"><ProgressBar class="immt-progress-bar" value=pctg/>
        {move || (pctg.get() * 100.0).to_string().chars().take(4).collect::<String>()} "%; ca. "{time_left}" remaining"
      </div>
    }
  }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,PartialEq,Eq)]
pub struct Entry{
  id:u32,
  archive:ArchiveId,
  rel_path:String,
  #[cfg(feature="hydrate")]
  steps:RwSignal<VecMap<String,TaskState>>,
  #[cfg(not(feature="hydrate"))]
  steps:VecMap<String,TaskState>,
}

impl Entry {

  #[cfg(not(feature="hydrate"))]
  fn as_view(&self) -> impl IntoView {
    view!{
      <li>{format!("[{}]{}",self.archive,self.rel_path)}</li>
    }
  }

  #[cfg(feature="hydrate")]
  fn as_view(&self) -> impl IntoView {
    use immt_web_utils::components::{Collapsible,Header};
    let title=format!("[{}]{}",self.archive,self.rel_path);
    let total = self.steps.with_untracked(|v| v.0.len());
    let steps = self.steps;
    let current = move || steps.with(|v| {
      v.iter().enumerate().find_map(|(i,(e,s))| if *s == TaskState::Done {None} else {
        Some((i+1,e.clone()))
      }).unwrap_or_else(|| (total,"Done".to_string()))
    });
    let rel_path = self.rel_path.clone();
    let archive = self.archive.clone();
    view!{
      <li><Collapsible>
        <Header slot>
          <b>{title}{move || {let (i,s) = current(); format!(" ({i}/{total}) {s}")}}</b>
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
  }
}

#[cfg(feature="ssr")]
impl From<immt_system::building::QueueEntry> for Entry {
  fn from(e:immt_system::building::QueueEntry) -> Self {
    #[cfg(feature="hydrate")]
    {unreachable!()}
    #[cfg(not(feature="hydrate"))]
    Self {
      id:e.id.into(),
      archive:e.archive,
      rel_path:e.rel_path.to_string(),
      steps:e.steps.into_iter().map(|(k,v)| (k.to_string(),v.into())).collect()
    }
  }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub enum TaskState {
    Running,Queued,Blocked,Done,Failed,None
}
impl TaskState {
  fn into_view(self,t:String,archive:&ArchiveId,rel_path:&str) -> impl IntoView {
    use immt_web_utils::components::{Collapsible,Header};
    match self {
      Self::Running => view!{<i style="color:yellow">{t}" (Running)"</i>}.into_any(),
      Self::Queued | Self::Blocked | Self::None => view!{<span style="color:gray">{t}" (...)"</span>}.into_any(),
      Self::Done => {
        let archive = archive.clone();
        let rel_path = rel_path.to_string();
        let tc = t.clone();
        view!{
          <Collapsible lazy=true>
            <Header slot><span style="color:green">{t}" (Done)"</span></Header>
            {
              let archive = archive.clone();
              let rel_path = rel_path.clone();
              let tc = tc.clone();
              from_server_clone(true, move || get_log(archive.clone(),rel_path.to_string(),tc.clone()), |s| {
              view!{<pre style="border:2px solid black;width:fit-content;padding:5px;font-size:smaller;">{s}</pre>}
            })}
          </Collapsible>
        }.into_any()
      },
      Self::Failed => {
        let archive = archive.clone();
        let rel_path = rel_path.to_string();
        let tc = t.clone();
        view!{
          <Collapsible lazy=true>
            <Header slot><span style="color:red">{t}" (Failed)"</span></Header>
            {
              let archive = archive.clone();
              let rel_path = rel_path.clone();
              let tc = tc.clone();
              from_server_clone(true, move || get_log(archive.clone(),rel_path.to_string(),tc.clone()), |s| {
              view!{<pre style="border:2px solid black;width:fit-content;padding:5px;font-size:smaller;">{s}</pre>}
            })}
          </Collapsible>
        }.into_any()
      }
    }
  }
}
#[cfg(feature="ssr")]
impl From<immt_system::building::TaskState> for TaskState {
  fn from(e:immt_system::building::TaskState) -> Self {
    use immt_system::building::TaskState;
    match e {
      TaskState::Running => Self::Running,
      TaskState::Queued => Self::Queued,
      TaskState::Blocked => Self::Blocked,
      TaskState::Done => Self::Done,
      TaskState::Failed => Self::Failed,
      TaskState::None => Self::None
    }
  }
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum QueueMessage {
    Idle(Vec<Entry>),
    Started {running:Vec<Entry>,queue:Vec<Entry>,blocked:Vec<Entry>,failed:Vec<Entry>,done:Vec<Entry>},
    TaskStarted {id:u32,target:String},
    TaskSuccess {id:u32,target:String,eta:Eta},
    TaskFailed {id:u32,target:String,eta:Eta}
}
#[cfg(feature="ssr")]
impl From<immt_system::building::QueueMessage> for QueueMessage {
  fn from(e:immt_system::building::QueueMessage) -> Self {
    use immt_system::building::QueueMessage;
    match e {
      QueueMessage::Idle(v) => Self::Idle(v.into_iter().map(Into::into).collect()),
      QueueMessage::Started {running,queue,blocked,failed,done} => Self::Started {
        running:running.into_iter().map(Into::into).collect(),
        queue:queue.into_iter().map(Into::into).collect(),
        blocked:blocked.into_iter().map(Into::into).collect(),
        failed:failed.into_iter().map(Into::into).collect(),
        done:done.into_iter().map(Into::into).collect()
      },
      QueueMessage::TaskStarted {id,target} => Self::TaskStarted {id:id.into(),target:target.to_string()},
      QueueMessage::TaskSuccess {id,target,eta} => Self::TaskSuccess {id:id.into(),target:target.to_string(),eta},
      QueueMessage::TaskFailed {id,target,eta} => Self::TaskFailed {id:id.into(),target:target.to_string(),eta}
    }
  }
}