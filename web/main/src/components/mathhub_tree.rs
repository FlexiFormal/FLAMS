use leptos::prelude::*;
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::uris::archives::ArchiveId;
use immt_core::uris::ArchiveURI;
use immt_core::utils::filetree::SourceFile;
use crate::accounts::if_logged_in;
use crate::components::queue::enqueue;
use crate::css;
use crate::utils::errors::IMMTError;

#[cfg(feature = "server")]
mod server {
    use immt_api::backend::archives::{Archive, Storage};
    use immt_core::ontology::archives::ArchiveGroup as AGroup;
    use immt_controller::{BaseController,ControllerTrait,controller};
    use super::{ArchiveOrGroup, DirOrFile};
    use immt_core::building::buildstate::AllStates;
    use immt_core::prelude::*;
    use immt_core::uris::archives::ArchiveId;
    use immt_core::utils::filetree::{SourceDir, SourceDirEntry, SourceFile};

    //#[cfg(feature="async")]
    pub fn get_archive_children(prefix:Option<ArchiveId>) -> Option<Vec<ArchiveOrGroup>> {
        controller().backend().archive_manager().with_tree(|toptree| {
            let (tree,has_meta) = match prefix {
                Some(prefix) => match toptree.find_group_or_archive(prefix)? {
                    AGroup::Group{children,has_meta,..} => (children.as_slice(),*has_meta),
                    _ => return None
                },
                None => (toptree.groups(),false)
            };
            let mut children = tree.iter().filter_map(|v| match v {
                AGroup::Archive(uri) => {
                    if let Some(Archive::Physical(ma)) = toptree.find_archive(uri.id()) {
                        Some(ArchiveOrGroup::Archive(*uri, ma.state().clone()))
                    } else {None}
                },
                AGroup::Group{id,state,..} => Some(ArchiveOrGroup::Group(id.clone(),state.clone()))
            }).collect::<Vec<_>>();
            if has_meta {
                let uri = toptree.find_archive(ArchiveId::new(&format!("{}/meta-inf",prefix.unwrap())))?.uri();
                children.insert(0,ArchiveOrGroup::Archive(uri, if let Some(Archive::Physical(ma)) = toptree.find_archive(uri.id()) {
                    ma.state().clone()
                } else {AllStates::default()}));
            }
            Some(children)
        })
    }

    pub fn get_dir_children(archive:ArchiveId,path:Option<&str>) -> Option<Vec<DirOrFile>> {
        controller().backend().get_archive(archive,|a| {
            if let Archive::Physical(ma) = a? {
                let sf = ma.source_files()?;
                let dir = match path {
                    Some(path) => match sf.find_entry(path)? {
                        SourceDirEntry::Dir(d) => d.children.as_slice(),
                        _ => return None
                    },
                    None => ma.source_files()?
                };
                Some(dir.iter().map(|v| match v {
                    SourceDirEntry::File(f) => DirOrFile::File(f.clone()),
                    SourceDirEntry::Dir( SourceDir{relative_path,data,..}) => DirOrFile::Dir(relative_path.to_string(),data.clone())
                }).collect())
            } else { None }
        })
    }
}


#[server(
    prefix="/api/backend",
    endpoint="archives",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_archives(prefix:Option<ArchiveId>) -> Result<Vec<ArchiveOrGroup>,ServerFnError<String>> {
    match server::get_archive_children(prefix) {
        Some(v) => Ok(v),
        _ => Err(ServerFnError::WrappedServerError(format!("No archive {} found!",prefix.unwrap())))
    }
}

#[server(
    prefix="/api/backend",
    endpoint="files_in",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_files_in(archive:ArchiveId,prefix:Option<String>) -> Result<Vec<DirOrFile>,ServerFnError<String>> {
    match server::get_dir_children(archive,prefix.as_deref()) {
        Some(v) => Ok(v),
        _ => Err(ServerFnError::WrappedServerError(format!("No archive {} found!",prefix.unwrap())))
    }
}


#[derive(Clone,Debug,serde::Serialize, serde::Deserialize)]
pub enum ArchiveOrGroup {
    Archive(ArchiveURI, AllStates),
    Group(ArchiveId, AllStates)
}

#[derive(Clone,Debug,serde::Serialize, serde::Deserialize)]
pub enum DirOrFile {
    Dir(String, AllStates),
    File(SourceFile)
}

#[component]
pub fn ArchivesTop() -> impl IntoView {
    use crate::components::*;
    crate::components::wait_blocking(||get_archives(None),|archives| if let Ok(archives) = archives {
        view!(
            <h2>"MathHub"</h2>
            <Tree>{
                archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(uri,state) => view!(<Archive uri state/>).into_any(),
                    ArchiveOrGroup::Group(id,state) => view!(<ArchiveGroup id state/>).into_any()
                }).collect_view()
            }</Tree>
        ).into_any()
    } else {"".into_any()})
}


#[component]
pub fn ArchiveOrGroups(path:ArchiveId) -> impl IntoView {
    use crate::components::*;
    crate::components::wait(move ||get_archives(Some(path)),|archives| if let Ok(archives) = archives {
        archives.into_iter().map(|a| match a{
            ArchiveOrGroup::Archive(uri,state) => view!(<Archive uri state/>).into_any(),
            ArchiveOrGroup::Group(id,state) => view!(<ArchiveGroup id state/>).into_any()
        }).collect_view().into_any()
    } else {"".into_any()})
}

#[component]
fn DirOrFiles(archive:ArchiveURI, #[prop(optional)] path:String) -> impl IntoView {
    let path = if path.is_empty() {None} else {Some(path)};
    crate::components::wait(move ||get_files_in(archive.id(),path.clone()),move |dirs| if let Ok(dirs) = dirs {
        dirs.into_iter().map(|a| match a{
            DirOrFile::File(file) => view!(<SourceFile archive file/>).into_any(),
            DirOrFile::Dir(path,state) => view!(<Directory archive path state/>).into_any()
        }).collect_view().into_any()
    } else {"".into_any()})
}

macro_rules! item {
    ($header:block $(modal=$modal:block)? $(badges=$badges:block)? => $onclick:block) => {{
        use thaw::{Icon,Dialog,DialogSurface,DialogBody,DialogContent};
        use crate::components::{WHeader,Subtree};
        let clicked = RwSignal::new(false);
        view!{<Subtree lazy=true>
                <WHeader slot>{$header}
                $(<Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
                   {$modal}
                </DialogContent></DialogBody></DialogSurface></Dialog>)?
                " "<span on:click=move |_| {clicked.set(true)} style="cursor: help;">
                    <Icon icon=icondata_ai::AiInfoCircleOutlined/>
                </span>
                $({$badges})?
            </WHeader>
            $onclick
        </Subtree>}
    }};
}

#[component]
fn ArchiveGroup(id:ArchiveId, state: AllStates) -> impl IntoView {
    use thaw::Icon;
    let summary = state.summary();
    item!(
        {view!(<span><Icon icon=icondata_bi::BiLibraryRegular/>/*{icon(icondata_bi::BiLibraryRegular)}*/
            " "{id.as_str().split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<GroupModal id state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<ArchiveOrGroups path=id/>)}
    )
}

#[component]
fn Archive(uri:ArchiveURI, state: AllStates) -> impl IntoView {
    use thaw::Icon;
    let summary = state.summary();
    item!(
        {view!(<span><Icon icon=icondata_bi::BiBookSolid/>/*{icon(icondata_bi::BiBookSolid)}*/
            " "{uri.id().as_str().split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<ArchiveModal id=uri.id() state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<DirOrFiles archive=uri/>)}
    )
}

#[component]
fn Directory(archive:ArchiveURI, path:String, state: AllStates) -> impl IntoView {
    use thaw::Icon;
    let summary = state.summary();
    let path2 = path.clone();
    item!(
        {view!(<span><Icon icon=icondata_bi::BiFolderRegular/>/*{icon(icondata_bi::BiFolderRegular)}*/
            " "{path.split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<DirectoryModal archive=archive.id() path state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<DirOrFiles archive path=path2.clone()/>)}
    )
}

#[component]
fn SourceFile(archive:ArchiveURI,file:SourceFile) -> impl IntoView {
    use thaw::*;
    use crate::components::{WideDrawer,Trigger,WHeader,IFrame,Leaf};
    let mut name = format!("{} [{}]",file.relative_path.split('/').last().unwrap(),file.format);
    let cls = match file.build_state {
        BuildState::UpToDate {last_built,..} => {
            name.push_str(&format!(" (last built: {})",last_built));
            "immt-treeview-file-up-to-date"
        },
        BuildState::Stale {last_built,..} => {
            name.push_str(&format!(" (last built: {})",last_built));
            "immt-treeview-file-stale"
        },
        BuildState::Deleted => "immt-treeview-file-deleted",
        BuildState::New => "immt-treeview-file-new"
    };
    let clicked = RwSignal::new(false);
    let path = file.relative_path.clone();
    let pathh = path.clone();
    view!{<Leaf>
        <WideDrawer lazy=true>
            <Trigger slot><span class=cls>
                <Icon icon=icondata_bi::BiFileRegular/>" "{name}
            </span></Trigger>
            <WHeader slot>
                <a href=format!("/?a={}&rp={pathh}",archive.id()) target="_blank">{
                    let path = pathh.clone();
                    view!(<Button appearance=ButtonAppearance::Subtle>
                        "["{archive.id().to_string()}"]/"{path}
                    </Button>)
                }</a>
            </WHeader>
            <IFrame src=format!("?a={}&rp={path}",archive.id()) ht="calc(100vh - 110px)".to_string()/>
        </WideDrawer>
        <Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
            <FileModal archive=archive.id() file/>
        </DialogContent></DialogBody></DialogSurface></Dialog>
        " "<span on:click=move |_| {clicked.set(true)} style="cursor: help;">
            <Icon icon=icondata_ai::AiInfoCircleOutlined/>
        </span>
    </Leaf>}
}


#[derive(Clone,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
enum ModalType {
    Group,Archive,Directory(String),File(SourceFile)
}

#[component]
fn EntryModal(id:ArchiveId,state:AllStates,tp:ModalType) -> impl IntoView {
    use thaw::*;
    let targets = state.targets().map(|(i,_)| i).collect::<Vec<_>>();
    let onclicktgt = *targets.first().unwrap();
    let target_str = targets.iter().map(|t| t.to_string()).collect::<String>();
    let onclickid = id.clone();

    let toaster = ToasterInjection::expect_context();

    async fn mapfut<F:std::future::Future<Output=Result<usize,ServerFnError<IMMTError>>>>(toaster: ToasterInjection,f:F) {
        match f.await {
            Ok(u) => toaster.dispatch_toast(view!{
                    <MessageBar intent=MessageBarIntent::Success><MessageBarBody>
                        {u}" new build tasks queued"
                    </MessageBarBody></MessageBar>
                }.into_any(),ToastOptions::default().with_position(ToastPosition::Top)),
            Err(e) => toaster.dispatch_toast(view!{
                <MessageBar intent=MessageBarIntent::Error><MessageBarBody>
                    "Error queueing build jobs:"<br/>{e.to_string()}
                </MessageBarBody></MessageBar>
            }.into_any(),ToastOptions::default().with_position(ToastPosition::Top))
        }
    }

    let (act,title) = match tp {
        ModalType::Group => (Action::new(move |b| {
            mapfut(toaster,enqueue(None,Some(onclickid.clone()),onclicktgt.clone(),None,*b))
        }),id.to_string()),
        ModalType::Archive => (Action::new(move |b| {
            mapfut(toaster,enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),None,*b))
        }),id.to_string()),
        ModalType::Directory(p) => {
            let s = format!("[{}]/{}",id,p);
            (Action::new(move |b| {
            mapfut(toaster,enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b))
        }),s)
        },
        ModalType::File(p) => {
            let p = p.relative_path;
            let s = format!("[{}]/{}",id,p);
            (Action::new(move |b| {
                mapfut(toaster,enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b))
            }),s)
        },
    };

    view!{
        <div class="immt-treeview-file-card"><Card>
            <CardHeader>
                {title}
                <CardHeaderDescription slot><Tag>{target_str}</Tag></CardHeaderDescription>
            </CardHeader>
            {if_logged_in(
                move || {
                    view!{<span>
                    <button on:click=move |_| {act.dispatch(false);}>"build all stale/new"</button>
                    <button on:click=move |_| {act.dispatch(true);}>"force build all"</button>
                    </span>}.into_any()
                },
                || "".into_any()
            )}
            <Table>
                <thead>
                    <tr><td>"Build Target"</td><td>"Last Built"</td></tr>
                </thead>
                <tbody>
                    <For each=move || targets.clone() key=|t| t.to_string() children=move |t|
                        view!{<tr><td>{t.to_string()}</td><td>"TODO"</td></tr>}
                    />
                </tbody>
            </Table>
        </Card></div>
    }
    /*
    match file.build_state {
        BuildState::UpToDate {last_built,..} => view!(<Tag variant=TagVariant::Success>"Up to date ("{last_built.to_string()}")"</Tag>),
        BuildState::Stale {last_built,..} => view!(<Tag variant=TagVariant::Warning>"Changed ("{last_built.to_string()}")"</Tag>),
        BuildState::Deleted =>view!(<Tag variant=TagVariant::Error>"Deleted"</Tag>),
        BuildState::New =>view!(<Tag>"New"</Tag>)
    }
     */

}


#[component]
fn GroupModal(id:ArchiveId,state:AllStates) -> impl IntoView {
    move || view!(<EntryModal id=id.clone() state=state.clone() tp=ModalType::Group/>)
}

#[component]
fn ArchiveModal(id:ArchiveId, state: AllStates) -> impl IntoView {
    move || view!(<EntryModal id=id.clone() state=state.clone() tp=ModalType::Archive/>)
}

#[component]
fn DirectoryModal(archive:ArchiveId,path:String,state:AllStates) -> impl IntoView {
    move || view!(<EntryModal id=archive.clone() state=state.clone() tp=ModalType::Directory(path.clone())/>)
}

#[component]
fn FileModal(archive:ArchiveId,file:SourceFile) -> impl IntoView {
    let mut state = AllStates::default();
    state.merge(&file.build_state,file.format);
    view!(<EntryModal id=archive.clone() state tp=ModalType::File(file.clone())/>)
}

pub(crate) fn badge(new:u32,stale:u32) -> impl IntoView {
    use thaw::{Badge,BadgeAppearance,BadgeColor};
    let new = if new == 0 { None } else { Some(new) };
    let stale = if stale == 0 { None } else { Some(stale) };
    if_logged_in(move || Some(view!(
        {new.map(|new| view!(" "<Badge class="immt-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Success>{new}</Badge>))}
        {stale.map(|stale| view!(" "<Badge class="immt-mathhub-badge" appearance=BadgeAppearance::Outline color=BadgeColor::Warning>{stale}</Badge>))}
    )) ,|| None)
}