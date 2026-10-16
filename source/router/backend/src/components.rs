use crate::FileStates;
use flams_backend_types::archives::{ArchiveData, ArchiveGroupData, DirectoryData, FileData};
use flams_router_base::{LoginState, maybe_lazy};
use flams_router_buildqueue_base::{FormatOrTarget, select_queue, server_fns::enqueue};
use flams_utils::unwrap;
use flams_web_utils::components::{
    Header, LazySubtree, Leaf, Subtree, Tree, message_action, wait_and_then, wait_and_then_fn,
};
use ftml_dom::utils::css::inject_css;
use ftml_ontology::utils::time::Timestamp;
use ftml_uris::ArchiveId;
use leptos::prelude::*;
use std::num::NonZeroU32;

maybe_lazy!(ArchivesTop = archives_top());

//#[component]
pub fn archives_top() -> AnyView {
    wait_and_then_fn(
        || super::server_fns::group_entries(None),
        |(groups, archives)| {
            {
                let mut summary = flams_backend_types::archives::FileStateSummary::default();
                for g in &groups {
                    if let Some(s) = g.summary {
                        summary.merge(s);
                    }
                }
                for a in &archives {
                    if let Some(s) = a.summary {
                        summary.merge(s);
                    }
                }
                view!(<Tree><Subtree expanded=true>
            <Header slot>
                "All Archives "
                {badge(summary)}
                {dialog(move |signal| if signal.get() {
                  Some(wait_and_then(
                    move || super::server_fns::build_status(None,None),
                    move |state| modal(None,None,state,None)
                  ))
                } else {None})}
            </Header>
            <ArchivesAndGroups archives groups/>
        </Subtree></Tree>)
            }
            .into_any()
        },
    )
    .into_any()
}

#[component]
fn ArchivesAndGroups(groups: Vec<ArchiveGroupData>, archives: Vec<ArchiveData>) -> AnyView {
    view! {
      {groups.into_iter().map(group).collect_view()}
      {archives.into_iter().map(archive).collect_view()}
    }
    .into_any()
}

fn group(a: ArchiveGroupData) -> AnyView {
    let id = a.id.clone();
    let header = view!(
      <ftml_component_utils::icons::LibraryIcon/>" "
      {a.id.last().to_string()}
      {a.summary.map(badge)}
      {dialog(move |signal| if signal.get() {
        let id = id.clone();
        let title = id.clone();
        Some(wait_and_then(
          move || super::server_fns::build_status(Some(id.clone()),None),
          move |state| modal(Some(title),None,state,None)
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
            view!(<Tree><ArchivesAndGroups groups archives/></Tree>).into_any()
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn archive(a: ArchiveData) -> AnyView {
    let id = a.id.clone();
    let header = view!(
      <ftml_component_utils::icons::ClosedBookIcon/>" "
      {a.id.last().to_string()}
      {a.summary.map(badge)}
      {dialog(move |signal| if signal.get() {
        let id = id.clone();
        let title = id.clone();
        Some(wait_and_then(
          move || super::server_fns::build_status(Some(id.clone()),None),
          move |state| modal(Some(title),None,state,None)
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
            view!(<Tree>{dirs_and_files(&nid,dirs,files)}</Tree>).into_any()
          )
        }
      </LazySubtree>
    }.into_any()
}

fn dirs_and_files(archive: &ArchiveId, dirs: Vec<DirectoryData>, files: Vec<FileData>) -> AnyView {
    view! {
      {dirs.into_iter().map(|d| dir(archive.clone(),d)).collect_view()}
      {files.into_iter().map(|f| file(archive.clone(),f)).collect_view()}
    }
    .into_any()
}

fn dir(archive: ArchiveId, d: DirectoryData) -> AnyView {
    let pathstr = unwrap!(d.rel_path.split('/').last()).to_string();
    let id = archive.clone();
    let rel_path = d.rel_path.clone();
    let header = view!(
      <ftml_component_utils::icons::FolderIcon/>" "
      {pathstr}
      {d.summary.map(badge)}
      {dialog(move |signal| if signal.get() {
        let id = id.clone();
        let title = id.clone();
        let rel_path = rel_path.clone();
        Some(wait_and_then(
          move || super::server_fns::build_status(Some(id.clone()),None),
          move |state| modal(Some(title),Some(rel_path),state,None)
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
            view!(<Tree>{dirs_and_files(&archive,dirs,files)}</Tree>).into_any()
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn file(archive: ArchiveId, f: FileData) -> AnyView {
    use flams_web_utils::components::{Drawer, Header, Trigger};
    use ftml_component_utils::{Button, ButtonAppearance};

    let link = format!("/?a={archive}&rp={}", f.rel_path);
    let button = format!("[{archive}]/{}", f.rel_path);
    let comps = ftml_uris::components::DocumentUriComponents::RelPath {
        a: archive.clone(),
        rp: f.rel_path.clone(),
    };

    let pathstr = unwrap!(f.rel_path.split('/').next_back()).to_string();
    let header = view!(
      <Drawer lazy=true>
        <Trigger slot>
          <ftml_component_utils::icons::FileIcon/>" "
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
          move || super::server_fns::build_status(Some(id.clone()),Some(rp.clone())),
          move |state| modal(Some(title.clone()),Some(rel_path.clone()),state,Some(fmt.clone()))
        ))
      } else {None})}
    );
    view! {
      <Leaf>{header}</Leaf>
    }
    .into_any()
}

fn badge(state: crate::FileStateSummary) -> AnyView {
    use ftml_component_utils::{Badge, BadgeAppearance, BadgeColor};
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
    }.into_any()
}

fn dialog<V: IntoView + 'static>(
    children: impl Fn(RwSignal<bool>) -> V + Send + Clone + 'static,
) -> AnyView {
    use ftml_component_utils::{Dialog, DialogBody, DialogContent, DialogSurface};
    let clicked = RwSignal::new(false);
    (move || {
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
                "🛈"
              </span>
            })
        } else {
            None
        }
    })
    .into_any()
}

fn modal(
    archive: Option<ArchiveId>,
    path: Option<String>,
    states: FileStates,
    format: Option<String>,
) -> AnyView {
    use ftml_component_utils::{
        Block, BoldCaption, Button, ButtonSize, Divider, Header, HeaderRight, Table, TableCell,
        TableHeader, TableRow,
    };
    let do_clean = path.is_none();
    let title = path.as_ref().map_or_else(
        || {
            archive
                .as_ref()
                .map_or_else(|| "All Archives".to_string(), ArchiveId::to_string)
        },
        |path| format!("[{}]{path}", archive.as_ref().expect("unreachable")),
    );
    let targets = format.is_some();
    let queue_id = RwSignal::<Option<NonZeroU32>>::new(None);
    let act = message_action(
        move |(t, b, clean)| {
            enqueue(
                archive.clone(),
                t,
                path.clone(),
                Some(b),
                queue_id.get_untracked(),
                clean,
            )
        },
        |i| format!("{i} new build tasks queued"),
    );
    let clean_btn = move |f: String| {
        if do_clean {
            Some(view! {
                <Button size=ButtonSize::Small on_click=move |_|
                {act.dispatch((FormatOrTarget::Format(f.clone()),false,true));}
                >"clean"</Button>
            })
        } else {
            None
        }
    };
    view! {
      <div style="text-align:left"><Block>
        <HeaderRight slot>{format.map(|f| {
            let f2 = f.clone();
            let f3 = f.clone();
            view!{
            <Button size=ButtonSize::Small on_click=move |_|
                {act.dispatch((FormatOrTarget::Format(f.clone()),true,false));}
            >"stale"</Button>
            <Button size=ButtonSize::Small on_click=move |_|
                {act.dispatch((FormatOrTarget::Format(f2.clone()),false,false));}
            >"all"</Button>
            {clean_btn(f3)}
            }
        })}</HeaderRight>
          <Header slot>
            <BoldCaption>{title}</BoldCaption>
          </Header>
          <Divider/>
          {select_queue(queue_id)}
          <Table>
              <TableHeader slot>
                    <TableCell><BoldCaption>{if targets {"Target"} else {"Format"}}</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"New"</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"Stale"</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"Up to date"</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"Last built"</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"Last changed"</BoldCaption></TableCell>
                    <TableCell><BoldCaption>"Build"</BoldCaption></TableCell>
              </TableHeader>
              {states.0.into_iter().map(|(name,summary)| {
                let fmt1 = name.clone();
                let fmt2 = name.clone();
                let fmt3 = name.clone();
                view!{
                  <TableRow>
                    <TableCell><BoldCaption>{name}</BoldCaption></TableCell>
                    <TableCell>{summary.new}</TableCell>
                    <TableCell>{summary.stale}</TableCell>
                    <TableCell>{summary.up_to_date}</TableCell>
                    <TableCell>{if summary.last_built == Timestamp::zero() {"(Never)".to_string()} else {summary.last_built.to_string()}}</TableCell>
                    <TableCell>{if summary.last_changed == Timestamp::zero() {"(Never)".to_string()} else {summary.last_changed.to_string()}}</TableCell>
                    <TableCell><div style="display:flex;flex-direction:column;">
                      <Button size=ButtonSize::Small on_click=move |_|
                        {act.dispatch((if targets {todo!()} else {
                          FormatOrTarget::Format(fmt1.clone())
                        },true,false));}
                      >"stale"</Button>
                      <Button size=ButtonSize::Small on_click=move |_|
                        {act.dispatch((if targets {todo!()} else {
                          FormatOrTarget::Format(fmt2.clone())
                        },false,false));}
                      >"all"</Button>
                      {clean_btn(fmt3)}
                    </div></TableCell>
                  </TableRow>
                }
            }).collect_view()}
        </Table>
          </Block>
          </div>
    }.into_any()
}
