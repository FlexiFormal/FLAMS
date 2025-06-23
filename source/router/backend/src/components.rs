use std::num::NonZeroU32;

use flams_ontology::{
    archive_json::{ArchiveData, ArchiveGroupData, DirectoryData, FileData},
    file_states::FileStateSummary,
    uris::ArchiveId,
};
use flams_router_base::LoginState;
use flams_router_buildqueue_base::{FormatOrTarget, select_queue, server_fns::enqueue};
use flams_utils::{time::Timestamp, unwrap};
use flams_web_utils::{
    components::{
        Header, LazySubtree, Leaf, Tree, message_action, wait_and_then, wait_and_then_fn,
    },
    inject_css,
};
use leptos::prelude::*;

use crate::FileStates;

#[component]
pub fn ArchivesTop() -> impl IntoView {
    wait_and_then_fn(
        || super::server_fns::group_entries(None),
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
        Some(wait_and_then(
          move || super::server_fns::build_status(id.clone(),None),
          move |state| modal(title,None,state,None)
        ))
      } else {None})}
    );
    let id = a.id;
    let f = move || super::server_fns::group_entries(Some(id.clone()));
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          wait_and_then(f.clone(),
          |(groups,archives)|
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
        Some(wait_and_then(
          move || super::server_fns::build_status(id.clone(),None),
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
          wait_and_then(move || super::server_fns::archive_entries(id.clone(),None),move |(dirs,files)|
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
) -> impl IntoView + 'static + use<> {
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
        Some(wait_and_then(
          move || super::server_fns::build_status(id.clone(),None),
          move |state| modal(title,Some(rel_path),state,None)
        ))
      } else {None})}
    );
    let id = archive.clone();
    let rel_path = d.rel_path;
    let f = move || super::server_fns::archive_entries(id.clone(), Some(rel_path.clone()));
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          let archive = archive.clone();
          wait_and_then(
              f.clone(),
              move |(dirs,files)|
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
        flams_router_base::uris::DocURIComponents::RelPath(archive.clone(), f.rel_path.clone());

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
        let rel_path = f.rel_path.clone();
        let title = archive.clone();
        let rp = rel_path.clone();
        let fmt = f.format.clone();
        Some(wait_and_then_fn(
          move || super::server_fns::build_status(id.clone(),Some(rp.clone())),
          move |state| modal(title.clone(),Some(rel_path.clone()),state,Some(fmt.clone()))
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
    };
    inject_css("flams-filecard", include_str!("filecards.css"));
    let title = path
        .as_ref()
        .map_or_else(|| archive.to_string(), |path| format!("[{archive}]{path}"));
    //let toaster = ToasterInjection::expect_context();
    let targets = format.is_some();
    let queue_id = RwSignal::<Option<NonZeroU32>>::new(None);
    let act = message_action(
        move |(t, b)| {
            enqueue(
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
