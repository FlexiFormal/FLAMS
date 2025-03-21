use std::num::NonZeroU32;

use flams_ontology::{
    archive_json::{ArchiveData, ArchiveGroupData, DirectoryData, FileData},
    file_states::FileStateSummary,
    uris::{ArchiveId, ArchiveURI, ArchiveURITrait, URIOrRefTrait},
};
use flams_router_login::LoginState;
use flams_utils::{time::Timestamp, unwrap, vecmap::VecMap};
use flams_web_utils::{
    components::{Header, LazySubtree, Leaf, Tree},
    inject_css,
};
use leptos::prelude::*;

use crate::utils::{from_server_clone, from_server_copy};

use super::buildqueue::FormatOrTarget;

#[server(prefix = "/api/backend", endpoint = "group_entries")]
#[allow(clippy::unused_async)]
pub async fn group_entries(
    r#in: Option<ArchiveId>,
) -> Result<(Vec<ArchiveGroupData>, Vec<ArchiveData>), ServerFnError<String>> {
    use flams_system::backend::archives::{Archive, ArchiveOrGroup as AoG};
    let login = LoginState::get_server();

    tokio::task::spawn_blocking(move || {
        let allowed = matches!(
            login,
            LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. }
        );
        flams_system::backend::GlobalBackend::get().with_archive_tree(|tree| {
            let v = match r#in {
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
    .unwrap_or_else(|e| Err(e.to_string().into()))
}

#[server(prefix = "/api/backend", endpoint = "archive_entries")]
pub async fn archive_entries(
    archive: ArchiveId,
    path: Option<String>,
) -> Result<(Vec<DirectoryData>, Vec<FileData>), ServerFnError<String>> {
    use either::Either;
    use flams_system::backend::{archives::source_files::SourceEntry, Backend};
    let login = LoginState::get_server();

    tokio::task::spawn_blocking(move || {
        let allowed = matches!(
            login,
            LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. }
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
                            return Err(
                                format!("Directory {p} not found in archive {archive}").into()
                            )
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
    .unwrap_or_else(|e| Err(e.to_string().into()))
}

#[server(prefix = "/api/backend", endpoint = "archive_dependencies")]
pub async fn archive_dependencies(
    archive: ArchiveId,
) -> Result<Vec<ArchiveId>, ServerFnError<String>> {
    use flams_system::backend::Backend;
    tokio::task::spawn_blocking(move || {
        let Some(iri) = flams_system::backend::GlobalBackend::get()
            .with_archive(&archive, |a| a.map(|a| a.uri().to_iri()))
        else {
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

        Ok(res
            .into_uris::<ArchiveURI>()
            .map(|uri| uri.archive_id().clone())
            .collect())
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string().into()))
    .map_err(Into::into)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct FileStates(VecMap<String, FileStateSummary>);

#[cfg(feature = "ssr")]
impl From<flams_system::backend::archives::source_files::FileStates> for FileStates {
    fn from(value: flams_system::backend::archives::source_files::FileStates) -> Self {
        Self(
            value
                .formats
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

#[cfg(feature = "ssr")]
impl
    From<
        &VecMap<
            flams_system::formats::BuildTargetId,
            flams_system::backend::archives::source_files::FileState,
        >,
    > for FileStates
{
    fn from(
        value: &VecMap<
            flams_system::formats::BuildTargetId,
            flams_system::backend::archives::source_files::FileState,
        >,
    ) -> Self {
        use flams_system::backend::archives::source_files::FileState;
        Self(
            value
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        match v {
                            FileState::New => FileStateSummary {
                                new: 1,
                                ..Default::default()
                            },
                            FileState::Stale(s) => FileStateSummary {
                                stale: 1,
                                last_built: s.last_built,
                                last_changed: s.last_changed,
                                ..Default::default()
                            },
                            FileState::UpToDate(s) => FileStateSummary {
                                up_to_date: 1,
                                last_built: s.last_built,
                                ..Default::default()
                            },
                            FileState::Deleted => FileStateSummary {
                                deleted: 1,
                                ..Default::default()
                            },
                        },
                    )
                })
                .collect(),
        )
    }
}

#[server(prefix = "/api/backend", endpoint = "build_status")]
#[allow(clippy::unused_async)]
pub async fn build_status(
    archive: ArchiveId,
    path: Option<String>,
) -> Result<FileStates, ServerFnError<String>> {
    use either::Either;
    use flams_system::backend::archives::{Archive, ArchiveOrGroup as AoG};
    use flams_system::backend::Backend;
    let login = LoginState::get_server();

    tokio::task::spawn_blocking(move || {
        let allowed = matches!(
            login,
            LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. }
        );
        if !allowed {
            return Err("Not logged in".to_string().into());
        }
        path.map_or_else(
            || {
                flams_system::backend::GlobalBackend::get().with_archive_tree(|tree| {
                    match tree.find(&archive) {
                        None => Err(format!("Archive {archive} not found").into()),
                        Some(AoG::Archive(id)) => {
                            let Some(Archive::Local(archive)) = tree.get(id) else {
                                return Err(format!("Archive {archive} not found").into());
                            };
                            Ok(archive.file_state().into())
                        }
                        Some(AoG::Group(g)) => Ok(g.state.clone().into()),
                    }
                })
            },
            |path| {
                flams_system::backend::GlobalBackend::get().with_local_archive(&archive, |a| {
                    let Some(a) = a else {
                        return Err(format!("Archive {archive} not found").into());
                    };
                    a.with_sources(|d| match d.find(&path) {
                        Some(Either::Left(d)) => Ok(d.state.clone().into()),
                        Some(Either::Right(f)) => Ok((&f.target_state).into()),
                        None => {
                            Err(format!("Directory {path} not found in archive {archive}").into())
                        }
                    })
                })
            },
        )
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string().into()))
}

//use flate2::*;
//use tar::*;
//use tokio::stream::
//GARBL

#[server(prefix="/api/backend",endpoint="download",
  input=server_fn::codec::GetUrl,
  output=server_fn::codec::Streaming
)]
pub async fn archive_stream(
    id: ArchiveId,
) -> Result<leptos::server_fn::codec::ByteStream, ServerFnError> {
    use flams_system::backend::Backend;
    use futures::TryStreamExt;
    /*
       struct Wrap<R:tokio::io::AsyncRead>(tokio_util::io::ReaderStream<R>);
       impl<R:tokio::io::AsyncRead> futures::Stream for Wrap<R> {
         type Item = Result<tokio_util::bytes::Bytes,ServerFnError>;
         fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
             <_ as futures::Stream>::poll_next(unsafe{self.map_unchecked_mut(|f| &mut f.0)},cx).map_err(|e| ServerFnError::new(e.to_string()))
         }
       }
    */
    let stream = flams_system::backend::GlobalBackend::get()
        .with_local_archive(&id, |a| a.map(|a| a.zip()))
        .ok_or_else(|| ServerFnError::new(format!("No archive with id {id} found!")))?;
    //.await.ok_or_else(|| ServerFnError::new(format!("Error bundling {id}")))?;
    Ok(leptos::server_fn::codec::ByteStream::new(
        stream.map_err(|e| ServerFnError::new(e.to_string())),
    ))

    //let f = tokio::fs::File::open(&fut).await.map_err(|e| ServerFnError::new(format!("Error reading file: {e}")))?;

    /*Ok(leptos::server_fn::codec::ByteStream::new(
      Wrap(tokio_util::io::ReaderStream::new(
        tokio::io::BufReader::new(f)
      ))
    ))*/
    //use tokio_util::io::ReaderStream;
    //tar::Builder::new(w);
}

#[component]
pub fn ArchivesTop() -> impl IntoView {
    from_server_copy(
        false,
        || group_entries(None),
        |(groups, archives)| view!(<Tree><ArchivesAndGroups archives groups/></Tree>),
    )
}

#[component]
fn ArchivesAndGroups(groups: Vec<ArchiveGroupData>, archives: Vec<ArchiveData>) -> impl IntoView {
    view! {
      {groups.into_iter().map(group).collect_view()}
      {archives.into_iter().map(archive).collect_view()}
    }
}

fn group(a: ArchiveGroupData) -> impl IntoView {
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
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          //let id = id.clone();
          from_server_clone(false,f.clone(),|(groups,archives)|
            view!(<Tree><ArchivesAndGroups groups archives/></Tree>)
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn archive(a: ArchiveData) -> impl IntoView {
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
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          let id = id.clone();
          let nid = id.clone();
          from_server_clone(false,move || archive_entries(id.clone(),None),move |(dirs,files)|
            view!(<Tree>{dirs_and_files(&nid,dirs,files)}</Tree>)
          )
        }
      </LazySubtree>
    }
}

fn dirs_and_files(
    archive: &ArchiveId,
    dirs: Vec<DirectoryData>,
    files: Vec<FileData>,
) -> impl IntoView {
    view! {
      {dirs.into_iter().map(|d| dir(archive.clone(),d)).collect_view()}
      {files.into_iter().map(|f| file(archive.clone(),f)).collect_view()}
    }
}

fn dir(archive: ArchiveId, d: DirectoryData) -> impl IntoView {
    let pathstr = unwrap!(d.rel_path.split('/').last()).to_string();
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
    let f = move || archive_entries(id.clone(), Some(rel_path.clone()));
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          let archive = archive.clone();
          from_server_clone(false,f.clone(),move |(dirs,files)|
            view!(<Tree>{dirs_and_files(&archive,dirs,files)}</Tree>)
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn file(archive: ArchiveId, f: FileData) -> impl IntoView {
    use flams_web_utils::components::{Drawer, Header, Trigger};
    use thaw::{Button, ButtonAppearance};

    let link = format!("/?a={archive}&rp={}", f.rel_path);
    let button = format!("[{archive}]/{}", f.rel_path);
    let comps =
        flams_router_content::uris::DocURIComponents::RelPath(archive.clone(), f.rel_path.clone());

    let pathstr = unwrap!(f.rel_path.split('/').last()).to_string();
    let header = view!(
      <Drawer lazy=true>
        <Trigger slot>
          <thaw::Icon icon=icondata_bi::BiFileRegular/>" "
          {pathstr}
        </Trigger>
        <Header slot><a href=link target="_blank">
          <Button appearance=ButtonAppearance::Subtle>{button}</Button>
        </a></Header>
        <div style="width:min-content"><flams_router_content::components::Document doc=comps.clone()/></div>
      </Drawer>
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
    view! {
      <Leaf>{header}</Leaf>
    }
}

fn badge(state: FileStateSummary) -> impl IntoView {
    use thaw::{Badge, BadgeAppearance, BadgeColor};
    view! {
      {if state.new == 0 {None} else {Some(view!(
        " "<Badge class="flams-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Success>{state.new}</Badge>
      ))}}
      {if state.stale == 0 {None} else {Some(view!(
        " "<Badge class="flams-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Warning>{state.stale}</Badge>
      ))}}
      {if state.deleted == 0 {None} else {Some(view!(
        " "<Badge class="flams-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Danger>{state.deleted}</Badge>
      ))}}
    }
}

fn dialog<V: IntoView + 'static>(
    children: impl Fn(RwSignal<bool>) -> V + Send + Clone + 'static,
) -> impl IntoView {
    use thaw::{Dialog, DialogBody, DialogContent, DialogSurface, Icon};
    let clicked = RwSignal::new(false);
    move || {
        if matches!(
            LoginState::get(),
            LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. }
        ) {
            let children = (children.clone())(clicked);
            Some(view! {
              <Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
              {children}
              </DialogContent></DialogBody></DialogSurface></Dialog>
              <span on:click=move |_| {clicked.set(true)} style="cursor: help;">
                <Icon icon=icondata_ai::AiInfoCircleOutlined/>
              </span>
            })
        } else {
            None
        }
    }
}

fn modal(
    archive: ArchiveId,
    path: Option<String>,
    states: FileStates,
    format: Option<String>,
) -> impl IntoView {
    use thaw::{
        Button, ButtonSize, Caption1Strong, Card, CardHeader, CardHeaderAction, Divider, Table,
        ToasterInjection,
    }; //,CardHeaderDescription
    inject_css("flams-filecard", include_str!("filecards.css"));
    let title = path
        .as_ref()
        .map_or_else(|| archive.to_string(), |path| format!("[{archive}]{path}"));
    let toaster = ToasterInjection::expect_context();
    let targets = format.is_some();
    let queue_id = RwSignal::<Option<NonZeroU32>>::new(None);
    let act = flams_web_utils::components::message_action(
        move |(t, b)| {
            super::buildqueue::enqueue(
                archive.clone(),
                t,
                path.clone(),
                Some(b),
                queue_id.get_untracked(),
            )
        },
        |i| format!("{i} new build tasks queued"),
    );
    view! {
      <div class="flams-treeview-file-card"><Card>
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
          {select_queue(queue_id)}
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

pub(crate) fn select_queue(queue_id: RwSignal<Option<NonZeroU32>>) -> impl IntoView {
    move || {
        let user = LoginState::get();
        if matches!(user, LoginState::NoAccounts) {
            return None;
        }
        let r = Resource::new(|| (), move |()| super::buildqueue::get_queues());
        Some(
            view! {<Suspense fallback = || view!(<flams_web_utils::components::Spinner/>)>{move || {
              match r.get() {
                None => leptos::either::EitherOf3::A(view!(<flams_web_utils::components::Spinner/>)),
                Some(Err(e)) => leptos::either::EitherOf3::B(flams_web_utils::components::display_error(e.to_string().into())),
                Some(Ok(queues)) => leptos::either::EitherOf3::C(view!{<div><div style="width:fit-content;margin-left:auto;">{do_queues(queue_id,queues)}</div></div>})
              }
            }}</Suspense>},
        )
    }
}

fn do_queues(
    queue_id: RwSignal<Option<NonZeroU32>>,
    v: Vec<super::buildqueue::QueueInfo>,
) -> impl IntoView {
    use thaw::Select;
    inject_css("flams-select-queue", include_str!("select_queue.css"));
    let queues = if v.is_empty() {
        vec![(0u32, "New Build Queue".to_string())]
    } else {
        v.into_iter()
            .map(|q| (q.id.get(), q.name))
            .chain(std::iter::once((0u32, "New Build Queue".to_string())))
            .collect()
    };
    let value = RwSignal::new(unwrap!(queues.first()).clone().1);
    let qc = queues.clone();
    let _ = Effect::new(move |_| {
        let queue = value.get();
        if queue == "New Build Queue" {
            queue_id.update_untracked(|v| *v = None);
        } else {
            let idx = unwrap!(? qc.iter().find_map(|(id,name)| if *name == queue {Some(*id)} else {None}));
            queue_id.update_untracked(|v| *v = Some(unwrap!(NonZeroU32::new(idx))));
        }
    });
    view! {
      <span style="font-style:italic;">"Build Queue: "
      <Select value class="flams-select-queue">{
        queues.into_iter().map(|(id,name)| view!{
          <option value=name.clone()>{name.clone()}</option>
        }).collect_view()
      }</Select></span>
    }
}
