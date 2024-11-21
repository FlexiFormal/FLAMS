use immt_ontology::{file_states::FileStateSummary, uris::ArchiveId};
use immt_utils::{time::Timestamp, vecmap::VecMap};
use leptos::prelude::*;
use immt_web_utils::{components::{Header, Leaf, Subtree, Tree}, inject_css};

use crate::{users::LoginState, utils::{from_server_clone, from_server_copy}};

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct ArchiveData {
  pub id:ArchiveId,
  pub summary:Option<FileStateSummary>,
}
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct ArchiveGroupData {
  pub id:ArchiveId,
  pub summary:Option<FileStateSummary>,
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct DirectoryData {
  pub rel_path:String,
  pub summary:Option<FileStateSummary>,
}
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct FileData {
  pub rel_path:String,
  pub format:String
  //pub summary:Option<FileStateSummary>,
}

#[server(prefix="/api/backend",endpoint="group_entries")]
#[allow(clippy::unused_async)]
pub async fn group_entries(r#in:Option<ArchiveId>) -> Result<(Vec<ArchiveGroupData>,Vec<ArchiveData>),ServerFnError<String>> {
  use immt_system::backend::archives::{Archive,ArchiveOrGroup as AoG};
  use immt_system::backend::Backend;
  use crate::users::LoginState;

  let login = LoginState::get();
  let allowed = login == LoginState::Admin || login == LoginState::NoAccounts;
  immt_system::backend::GlobalBackend::get().with_archive_tree(|tree| {
    let v = match r#in {
      None => &tree.groups,
      Some(id) => match tree.find(&id) {
        Some(AoG::Group(g)) => &g.children,
        _ => return Err(format!("Archive Group {id} not found").into())
      }
    };
    let mut groups = Vec::new();
    let mut archives = Vec::new();
    for a in v {
      match a {
        AoG::Archive(id) => {
          let summary = if allowed {
            tree.get(id).and_then(|a| 
              if let Archive::Local(a) = a {
                Some(a.state_summary())
              } else { None }
            )
          } else {None};
          archives.push(ArchiveData{id: id.clone(),summary});
        }
        AoG::Group(g) => {
          let summary = if allowed {
            Some(g.state.summarize())
          } else {None};
          groups.push(ArchiveGroupData{id: g.id.clone(),summary});
        }
      }
    }
    Ok((groups,archives))
  })
}


#[server(prefix="/api/backend",endpoint="archive_entries")]
#[allow(clippy::unused_async)]
pub async fn archive_entries(archive:ArchiveId,path:Option<String>) -> Result<(Vec<DirectoryData>,Vec<FileData>),ServerFnError<String>> {
  use crate::users::LoginState;
  use either::Either;
  use immt_system::backend::{Backend,archives::source_files::SourceEntry};

  let login = LoginState::get();
  let allowed = login == LoginState::Admin || login == LoginState::NoAccounts;
  immt_system::backend::GlobalBackend::get().with_local_archive(&archive, |a| {
    let Some(a) = a else { return Err(format!("Archive {archive} not found").into()) };
    a.with_sources(|d| {
      let d = match path {
        None => d,
        Some(p) => match d.find(&p) {
          Some(Either::Left(d)) => d,
          _ => return Err(format!("Directory {p} not found in archive {archive}").into())
        }
      };
      let mut ds = Vec::new();
      let mut fs = Vec::new();
      for d in &d.children {
        match d {
          SourceEntry::Dir(d) => ds.push(DirectoryData{
              rel_path:d.relative_path.to_string(),
              summary: if allowed {Some(d.state.summarize())} else {None}
          }),
          SourceEntry::File(f) => fs.push(FileData {
            rel_path:f.relative_path.to_string(),
            format:f.format.to_string()
          })
        }
      }
      Ok((ds,fs))
    })
  })
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Default,serde::Serialize, serde::Deserialize)]
pub struct FileStates(
    VecMap<String,FileStateSummary>
);

#[cfg(feature="ssr")]
impl From<immt_system::backend::archives::source_files::FileStates> for FileStates {
  fn from(value: immt_system::backend::archives::source_files::FileStates) -> Self {
    Self(value.formats.into_iter().map(|(k,v)| (k.to_string(),v)).collect())
  }
}

#[cfg(feature="ssr")]
impl From<&VecMap<immt_system::formats::BuildTargetId,immt_system::backend::archives::source_files::FileState>> for FileStates {
  fn from(value: &VecMap<immt_system::formats::BuildTargetId,immt_system::backend::archives::source_files::FileState>) -> Self {
    use immt_system::backend::archives::source_files::FileState;
    Self(value.iter().map(|(k,v)| (k.to_string(),
      match v {
        FileState::New => FileStateSummary {
          new:1,
          ..Default::default()
        },
        FileState::Stale(s) => FileStateSummary {
          stale:1,
          last_built: s.last_built,
          last_changed: s.last_changed,
          ..Default::default()
        },
        FileState::UpToDate(s) => FileStateSummary {
          up_to_date:1,
          last_built: s.last_built,
          ..Default::default()
        },
        FileState::Deleted => FileStateSummary {
          deleted:1,
          ..Default::default()
        }
      }
  )).collect())
  }
}


#[server(prefix="/api/backend",endpoint="build_status")]
#[allow(clippy::unused_async)]
pub async fn build_status(archive:ArchiveId,path:Option<String>) -> Result<FileStates,ServerFnError<String>> {
  use immt_system::backend::archives::{Archive,ArchiveOrGroup as AoG};
  use crate::users::LoginState;
  use either::Either;
  use immt_system::backend::Backend;

  let login = LoginState::get();
  if login != LoginState::Admin && login != LoginState::NoAccounts {
    return Err("Not logged in".to_string().into());
  }
  path.map_or_else(
    || immt_system::backend::GlobalBackend::get().with_archive_tree(|tree| 
      match tree.find(&archive) {
        None => Err(format!("Archive {archive} not found").into()),
        Some(AoG::Archive(id)) => {
          let Some(Archive::Local(archive)) = tree.get(id) else {
            return Err(format!("Archive {archive} not found").into())
          };
          Ok(archive.file_state().into())
        }
        Some(AoG::Group(g)) => Ok(g.state.clone().into()),
      }
    ),
    |path| immt_system::backend::GlobalBackend::get().with_local_archive(&archive, |a| {
      let Some(a) = a else { return Err(format!("Archive {archive} not found").into()) };
      a.with_sources(|d|
        match d.find(&path) {
          Some(Either::Left(d)) => Ok(d.state.clone().into()),
          Some(Either::Right(f)) => Ok((&f.target_state).into()),
          None => Err(format!("Directory {path} not found in archive {archive}").into())
        }
      )
    })
  )
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum FormatOrTarget {
  Format(String),
  Targets(Vec<String>)
}

#[server(
  prefix="/api/buildqueue",
  endpoint="enqueue"
)]
#[allow(clippy::unused_async)]
pub async fn enqueue(archive:ArchiveId,
  target:FormatOrTarget,
  path:Option<String>,stale_only:Option<bool>
) -> Result<usize,ServerFnError<String>> {
  use immt_system::{formats::FormatOrTargets,building::queue_manager::QueueManager};
  use immt_system::backend::archives::ArchiveOrGroup as AoG;
  use immt_system::formats::{SourceFormat,BuildTarget};
  use immt_system::backend::Backend;

  let login = LoginState::get();
  if login != LoginState::Admin && login != LoginState::NoAccounts {
    return Err("Not logged in".to_string().into());
  }
  let queues = QueueManager::get();
  let stale_only = stale_only.unwrap_or(true);
  #[allow(clippy::option_if_let_else)]
  let tgts: Vec<_> = match &target {
    FormatOrTarget::Targets(t) => {
      let Some(v) = t.iter().map(|s| BuildTarget::get_from_str(s)).collect::<Option<Vec<_>>>() else {
        return Err(ServerFnError::MissingArg("Invalid target".into()))
      };
      v
    }
    FormatOrTarget::Format(_) => Vec::new()
  };
  let fot = match target {
    FormatOrTarget::Format(f) => FormatOrTargets::Format(
      SourceFormat::get_from_str(&f).map_or_else(
        || Err(ServerFnError::MissingArg("Invalid format".into())),
        Ok
      )?
    ),
    FormatOrTarget::Targets(_) => FormatOrTargets::Targets(tgts.as_slice())
  };
  let group = immt_system::backend::GlobalBackend::get().with_archive_tree(|tree| -> Result<bool,ServerFnError<String>>
    {match tree.find(&archive) {
      Some(AoG::Archive(_)) => Ok(false),
      Some(AoG::Group(_)) => Ok(true),
      None => Err(format!("Archive {archive} not found").into()),
    }}
  )?;
  if group && path.is_some() {
    return Err(ServerFnError::MissingArg("Must specify either an archive with optional path or a group".into())) 
  }

  queues.with_global(|queue|
    if group { Ok(queue.enqueue_group(&archive, fot, stale_only))} else {
      Ok(queue.enqueue_archive(&archive, fot, stale_only,path.as_deref()))
    }
  )
}


#[component]
pub fn ArchivesTop() -> impl IntoView {
  from_server_copy(false, || group_entries(None), |(groups,archives)|
    view!(<Tree><ArchivesAndGroups archives groups/></Tree>)
  )
}

#[component]
fn ArchivesAndGroups(groups:Vec<ArchiveGroupData>,archives:Vec<ArchiveData>) -> impl IntoView {
  view!{
    {groups.into_iter().map(group).collect_view()}
    {archives.into_iter().map(archive).collect_view()}
  }
}

fn group(a:ArchiveGroupData) -> impl IntoView {
  let id = a.id.clone();
  let header = view!(
    <thaw::Icon icon=icondata_bi::BiLibraryRegular/>" "
    {a.id.last_name().to_string()}
    {a.summary.map(badge)}
    {dialog(move |signal| if signal.get() {
      let id = id.clone();
      let title = id.clone();
      Some(from_server_clone(false,
        move || build_status(id.clone(),None),
        move |state| modal(title,None,state,None)
      ))
    } else {None})}
  );
  let id = a.id;
  let f = move || group_entries(Some(id.clone()));
  view!{
    <Subtree lazy=true>
      <Header slot>{header}</Header>
      {
        //let id = id.clone();
        from_server_clone(false,f.clone(),|(groups,archives)|
          view!(<Tree><ArchivesAndGroups groups archives/></Tree>)
        )
      }
    </Subtree>
  }
}

fn archive(a:ArchiveData) -> impl IntoView {
  let id = a.id.clone();
  let header = view!(
    <thaw::Icon icon=icondata_bi::BiBookSolid/>" "
    {a.id.last_name().to_string()}
    {a.summary.map(badge)}
    {dialog(move |signal| if signal.get() {
      let id = id.clone();
      let title = id.clone();
      Some(from_server_clone(false,
        move || build_status(id.clone(),None),
        move |state| modal(title,None,state,None)
      ))
    } else {None})}
  );
  let id = a.id;
  view!{
    <Subtree lazy=true>
      <Header slot>{header}</Header>
      {
        let id = id.clone();
        let nid = id.clone();
        from_server_clone(false,move || archive_entries(id.clone(),None),move |(dirs,files)|
          view!(<Tree>{dirs_and_files(&nid,dirs,files)}</Tree>)
        )
      }
    </Subtree>
  }
}

fn dirs_and_files(archive:&ArchiveId,dirs:Vec<DirectoryData>,files:Vec<FileData>) -> impl IntoView {
  view!{
    {dirs.into_iter().map(|d| dir(archive.clone(),d)).collect_view()}
    {files.into_iter().map(|f| file(archive.clone(),f)).collect_view()}
  }
}


fn dir(archive:ArchiveId,d:DirectoryData) -> impl IntoView {
  let pathstr = d.rel_path.split('/').last().unwrap_or_else(|| unreachable!()).to_string();
  let id = archive.clone();
  let rel_path = d.rel_path.clone();
  let header = view!(
    <thaw::Icon icon=icondata_bi::BiFolderRegular/>" "
    {pathstr}
    {d.summary.map(badge)}
    {dialog(move |signal| if signal.get() {
      let id = id.clone();
      let title = id.clone();
      let rel_path = rel_path.clone();
      Some(from_server_clone(false,
        move || build_status(id.clone(),None),
        move |state| modal(title,Some(rel_path),state,None)
      ))
    } else {None})}
  );
  let id = archive.clone();
  let rel_path = d.rel_path;
  let f = move || archive_entries(id.clone(),Some(rel_path.clone()));
  view!{
    <Subtree lazy=true>
      <Header slot>{header}</Header>
      {
        let archive = archive.clone();
        from_server_clone(false,f.clone(),move |(dirs,files)|
          view!(<Tree>{dirs_and_files(&archive,dirs,files)}</Tree>)
        )
      }
    </Subtree>
  }
}

fn file(archive:ArchiveId,f:FileData) -> impl IntoView {
  use immt_web_utils::components::{DrawerThaw,Header,Trigger};
  use thaw::{Button,ButtonAppearance};

  let link = format!("/?a={archive}&rp={}",f.rel_path);
  let button = format!("[{archive}]/{}",f.rel_path);
  let comps = crate::router::content::uris::DocURIComponents::RelPath(archive.clone(),f.rel_path.clone());

  let pathstr = f.rel_path.split('/').last().unwrap_or_else(|| unreachable!()).to_string();
  let header = view!(
    <DrawerThaw lazy=true>
      <Trigger slot>
        <thaw::Icon icon=icondata_bi::BiFileRegular/>" "
        {pathstr}
      </Trigger>
      <Header slot><a href=link target="_blank">
        <Button appearance=ButtonAppearance::Subtle>{button}</Button>
      </a></Header>
      <crate::router::content::Document doc=comps.clone()/>
    </DrawerThaw>
    {dialog(move |signal| if signal.get() {
      let id = archive.clone();
      let title = archive.clone();
      let rel_path = f.rel_path.clone();
      let rp = rel_path.clone();
      let fmt = f.format.clone();
      Some(from_server_clone(false,
        move || build_status(id.clone(),Some(rp.clone())),
        move |state| modal(title,Some(rel_path),state,Some(fmt))
      ))
    } else {None})}
  );
  view!{
    <Leaf>{header}</Leaf>
  }
}

fn badge(state:FileStateSummary) -> impl IntoView {
  use thaw::{Badge,BadgeAppearance,BadgeColor};
  view!{
    {if state.new == 0 {None} else {Some(view!(
      " "<Badge class="immt-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Success>{state.new}</Badge>
    ))}}
    {if state.stale == 0 {None} else {Some(view!(
      " "<Badge class="immt-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Warning>{state.stale}</Badge>
    ))}}
    {if state.deleted == 0 {None} else {Some(view!(
      " "<Badge class="immt-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Danger>{state.deleted}</Badge>
    ))}}
  }
}

fn dialog<V:IntoView + 'static>(children:impl Fn(RwSignal<bool>) -> V + Send + Clone + 'static) -> impl IntoView {
  use thaw::{Dialog,DialogSurface,DialogBody,DialogContent,Icon};
  let login = expect_context::<RwSignal<LoginState>>();
  let clicked = RwSignal::new(false);
  move || if matches!(login.get(),LoginState::Admin | LoginState::NoAccounts) {
    let children = (children.clone())(clicked);
    Some(view!{
      <Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
      {children}
      </DialogContent></DialogBody></DialogSurface></Dialog>
      <span on:click=move |_| {clicked.set(true)} style="cursor: help;">
        <Icon icon=icondata_ai::AiInfoCircleOutlined/>
      </span>
    })
  } else { None }
}

async fn run_build(id:ArchiveId,target:FormatOrTarget,path:Option<String>,stale_only:bool,toaster:thaw::ToasterInjection) {
  use thaw::{ToastOptions,ToastPosition,MessageBar,MessageBarIntent,MessageBarBody};
  match enqueue(id,target,path,Some(stale_only)).await {
    Ok(i) => toaster.dispatch_toast(
      move || view!{
        <MessageBar intent=MessageBarIntent::Success><MessageBarBody>
            {i}" new build tasks queued"
        </MessageBarBody></MessageBar>}.into_any(), 
      ToastOptions::default().with_position(ToastPosition::Top)
    ),
    Err(e) => toaster.dispatch_toast(
      || view!{
        <MessageBar intent=MessageBarIntent::Error><MessageBarBody>
            {e.to_string()}
        </MessageBarBody></MessageBar>}.into_any(), 
      ToastOptions::default().with_position(ToastPosition::Top)
    )
  }
}

fn modal(archive:ArchiveId,path:Option<String>,states:FileStates,format:Option<String>) -> impl IntoView {
  use thaw::{ToasterInjection,Card,CardHeader,CardHeaderAction,Table,Caption1Strong,Button,ButtonSize,Divider};//,CardHeaderDescription
  inject_css("immt-filecard", include_str!("filecards.css"));
  let title = path.as_ref().map_or_else(
    || archive.to_string(),
    |path| format!("[{archive}]{path}")
  );
  let toaster = ToasterInjection::expect_context();
  let targets = format.is_some();
  let act = Action::new(move |(t,b):&(FormatOrTarget,bool)| {
    run_build(archive.clone(),t.clone(),path.clone(),*b,toaster)
  });
  view!{
    <div class="immt-treeview-file-card"><Card>
        <CardHeader>
          <Caption1Strong>{title}</Caption1Strong>
          <CardHeaderAction slot>{format.map(|f| {
            let f2 = f.clone();
            view!{
              <Button size=ButtonSize::Small on_click=move |_| 
                {act.dispatch((FormatOrTarget::Format(f.clone()),true));}
              >"stale"</Button>
              <Button size=ButtonSize::Small on_click=move |_| 
                {act.dispatch((FormatOrTarget::Format(f2.clone()),false));}
              >"all"</Button>
            }
          })}</CardHeaderAction>
        </CardHeader>
        <Divider/>
        <Table>
            <thead>
                <tr>
                  <td><Caption1Strong>{if targets {"Target"} else {"Format"}}</Caption1Strong></td>
                  <td><Caption1Strong>"New"</Caption1Strong></td>
                  <td><Caption1Strong>"Stale"</Caption1Strong></td>
                  <td><Caption1Strong>"Up to date"</Caption1Strong></td>
                  <td><Caption1Strong>"Last built"</Caption1Strong></td>
                  <td><Caption1Strong>"Last changed"</Caption1Strong></td>
                  <td><Caption1Strong>"Build"</Caption1Strong></td>
                </tr>
            </thead>
            <tbody>
            {states.0.iter().map(|(name,summary)| {
              let name = name.clone();
              let fmt1 = name.clone();
              let fmt2 = name.clone();
              view!{
                <tr>
                  <td><Caption1Strong>{name}</Caption1Strong></td>
                  <td>{summary.new}</td>
                  <td>{summary.stale}</td>
                  <td>{summary.up_to_date}</td>
                  <td>{if summary.last_built == Timestamp::zero() {"(Never)".to_string()} else {summary.last_built.to_string()}}</td>
                  <td>{if summary.last_changed == Timestamp::zero() {"(Never)".to_string()} else {summary.last_changed.to_string()}}</td>
                  <td><div>
                    <Button size=ButtonSize::Small on_click=move |_| 
                      {act.dispatch((if targets {todo!()} else {
                        FormatOrTarget::Format(fmt1.clone())
                      },true));}
                    >"stale"</Button>
                    <Button size=ButtonSize::Small on_click=move |_| 
                      {act.dispatch((if targets {todo!()} else {
                        FormatOrTarget::Format(fmt2.clone())
                      },false));}
                    >"all"</Button>
                  </div></td>
                </tr>
              }
          }).collect_view()}
            </tbody>
        </Table>

    </Card></div>
  }
}