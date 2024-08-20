use leptos::prelude::*;
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::uris::archives::ArchiveId;
use immt_core::uris::ArchiveURI;
use immt_core::utils::filetree::SourceFile;
use crate::accounts::if_logged_in;
use crate::components::queue::enqueue;

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
    crate::components::wait_blocking(||get_archives(None),|archives| if let Ok(archives) = archives {
        view!(
            <h2>"MathHub"</h2>
            <ul class="immt-treeview">{
                archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(uri,state) => view!(<Archive uri state/>).into_any(),
                    ArchiveOrGroup::Group(id,state) => view!(<ArchiveGroup id state/>).into_any()
                }).collect::<Vec<_>>()
            }</ul>
        ).into_any()
    } else {"".into_any()})
}


#[island]
pub fn ArchiveOrGroups(path:ArchiveId) -> impl IntoView {
    crate::components::wait(move ||get_archives(Some(path)),|archives| if let Ok(archives) = archives {
        view!(
            <ul class="immt-treeview-inner">{
                archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(uri,state) => view!(<Archive uri state/>).into_any(),
                    ArchiveOrGroup::Group(id,state) => view!(<ArchiveGroup id state/>).into_any()
                }).collect::<Vec<_>>()
            }</ul>
        ).into_any()
    } else {"".into_any()})
}

#[island]
fn DirOrFiles(archive:ArchiveURI, #[prop(optional)] path:String) -> impl IntoView {
    let path = if path.is_empty() {None} else {Some(path)};
    crate::components::wait(move ||get_files_in(archive.id(),path.clone()),move |dirs| if let Ok(dirs) = dirs {
        view!(
            <ul class="immt-treeview-inner">{
                dirs.into_iter().map(|a| match a{
                    DirOrFile::File(file) => view!(<SourceFile archive file/>).into_any(),
                    DirOrFile::Dir(path,state) => view!(<Directory archive path state/>).into_any()
                }).collect::<Vec<_>>()
            }</ul>
        ).into_any()
    } else {"".into_any()})
}

macro_rules! item {
    ($header:block $(modal=$modal:block)? $(badges=$badges:block)? => $onclick:block) => {{
        //use thaw::{*,Icon as NoNotThisOne};
        use crate::components::icon;
        let expanded = RwSignal::new(false);
        let clicked = RwSignal::new(false);
        view!{<li class="immt-treeview-li"><details>
            <summary class="immt-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {$header}
                /*$(<Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
                   {$modal}
                </DialogContent></DialogBody></DialogSurface></Dialog>)?*/
                " "<span on:click=move |_| {clicked.set(true)} style="cursor: help;">{icon(icondata_ai::AiInfoCircleOutlined)}</span>
                $({$badges})?
            </summary>
            {move || {if expanded.get() {Some($onclick)} else { None }}}
        </details></li>}
    }};
}

#[island]
fn ArchiveGroup(id:ArchiveId, state: AllStates) -> impl IntoView {
    let summary = state.summary();
    item!(
        {view!(<span>{icon(icondata_bi::BiLibraryRegular)}" "{id.as_str().split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<GroupModal id state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<ArchiveOrGroups path=id/>)}
    )
}

#[island]
fn Archive(uri:ArchiveURI, state: AllStates) -> impl IntoView {
    let summary = state.summary();
    item!(
        {view!(<span>{icon(icondata_bi::BiBookSolid)}" "{uri.id().as_str().split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<ArchiveModal id=uri.id() state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<DirOrFiles archive=uri/>)}
    )
}

#[island]
fn Directory(archive:ArchiveURI, path:String, state: AllStates) -> impl IntoView {
    let summary = state.summary();
    item!(
        {view!(<span>{icon(icondata_bi::BiFolderRegular)}" "{path.split('/').last().unwrap().to_string()}</span>)}
        modal={view!(<DirectoryModal archive=archive.id() path state/>)}
        badges={badge(summary.new,summary.stale.0)}
        => {view!(<DirOrFiles archive path=path.clone()/>)}
    )
}

#[island]
fn SourceFile(archive:ArchiveURI,file:SourceFile) -> impl IntoView {
    use thaw::*;
    use crate::components::{IFrame,icon};
    let open = RwSignal::new(false);

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
    let p = file.relative_path.clone();
    view!{<li class="immt-treeview-li">
        <span class=cls on:click=move |_| open.set(true)>
            {icon(icondata_bi::BiFileRegular)}" "{name.clone()}
        </span>
        {crate::components::drawer(open,
            Some(view!(<a href=format!("/?a={}&rp={p}",archive.id()) target="_blank"><button class="thaw-breadcrumb-button">{
                format!("[{}]/{}",archive.id(),&p)
            }</button></a>)),
            move || view!{
            /*<div style="text-align:center;width:100%"><b>
                <a href=format!("/?a={}&rp={p}",archive.id())>{
                    format!("[{}]/{}",archive.id(),&p)
                }</a>
            </b></div>
            <Divider/>*/
            {let p = p.clone();move || if open.get() {
                //view!("Here!").into_any()
                view!(<IFrame src=format!("?a={}&rp={}",archive.id(),p.clone()) ht="calc(100vh - 110px)".to_string()/>).into_any()
            } else {"".into_any()} }
        })}
        /*
        <OverlayDrawer /*title=format!("[{archive}]/{}",file.relative_path)*/ open position=DrawerPosition::Right size=DrawerSize::Full>
            /*<DrawerHeader>
                <DrawerHeaderTitle></DrawerHeaderTitle>
            </DrawerHeader>*/
            //<Divider>
                <div style="text-align:center;width:100%"><b>
                    <a href=format!("/?a={}&rp={p}",archive.id())>{
                        format!("[{}]/{}",archive.id(),&p)
                    }</a>
                </b></div>
                <Divider/>
            //</Divider>

        {
            move || if open.get() {
                view!(<IFrame src=format!("?a={}&rp={}",archive.id(),p) ht="calc(100vh - 110px)".to_string()/>).into_any()
            } else {"".into_any()}
        }</OverlayDrawer>
         */
        /*<Dialog open=clicked><DialogSurface><DialogBody><DialogContent>
            <FileModal archive=archive.id() file/>
        </DialogContent></DialogBody></DialogSurface></Dialog>*/
        " "<span on:click=move |_| {clicked.set(true)} style="cursor: help;">{icon(icondata_ai::AiInfoCircleOutlined)}</span>
        //<span on:click=move |_| {clicked.set(true)} style="cursor: help;">" 🛈"</span>
    </li>}
}

#[derive(Clone,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
enum ModalType {
    Group,Archive,Directory(String),File(SourceFile)
}

#[island]
fn EntryModal(id:ArchiveId,state:AllStates,tp:ModalType) -> impl IntoView {
    use thaw::*;
    let targets = state.targets().map(|(i,_)| i).collect::<Vec<_>>();
    let onclicktgt = *targets.first().unwrap();
    let target_str = targets.iter().map(|t| t.to_string()).collect::<String>();
    let onclickid = id.clone();
    let (act,title) = match tp {
        ModalType::Group => (Action::new(move |b| {
            enqueue(None,Some(onclickid.clone()),onclicktgt.clone(),None,*b)
        }),id.to_string()),
        ModalType::Archive => (Action::new(move |b| {
            enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),None,*b)
        }),id.to_string()),
        ModalType::Directory(p) => {
            let s = format!("[{}]/{}",id,p);
            (Action::new(move |b| {
            enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b)
        }),s)
        },
        ModalType::File(p) => {
            let p = p.relative_path;
            let s = format!("[{}]/{}",id,p);
            (Action::new(move |b| {
                enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b)
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


#[island]
fn GroupModal(id:ArchiveId,state:AllStates) -> impl IntoView {
    move || view!(<EntryModal id=id.clone() state=state.clone() tp=ModalType::Group/>)
}

#[island]
fn ArchiveModal(id:ArchiveId, state: AllStates) -> impl IntoView {
    move || view!(<EntryModal id=id.clone() state=state.clone() tp=ModalType::Archive/>)
}

#[island]
fn DirectoryModal(archive:ArchiveId,path:String,state:AllStates) -> impl IntoView {
    move || view!(<EntryModal id=archive.clone() state=state.clone() tp=ModalType::Directory(path.clone())/>)
}

#[island]
fn FileModal(archive:ArchiveId,file:SourceFile) -> impl IntoView {
    let mut state = AllStates::default();
    state.merge(&file.build_state,file.format);
    view!(<EntryModal id=archive.clone() state tp=ModalType::File(file.clone())/>)
}

pub(crate) fn badge(new:u32,stale:u32) -> impl IntoView {
    use thaw::{BadgeAppearance,BadgeColor};
    use crate::components::Badge;
    let new = if new == 0 { None } else { Some(new) };
    let stale = if stale == 0 { None } else { Some(stale) };
    if_logged_in(|| Some(view!(
        {new.map(|new| view!(" "<Badge appearance=BadgeAppearance::Outline color=BadgeColor::Success>{new}</Badge>))}
        {stale.map(|stale| view!(" "<Badge appearance=BadgeAppearance::Outline color=BadgeColor::Warning>{stale}</Badge>))}
    )) ,|| None)
}