use leptos::*;
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::uris::archives::ArchiveId;
use immt_core::uris::ArchiveURI;
use immt_core::utils::filetree::SourceFile;
use crate::accounts::if_logged_in;

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

//stylance::import_crate_style!(treeview,"style/components/treeview.scss");
#[allow(non_upper_case_globals)]
mod treeview {
    pub const immt_treeview: &str = "immt-treeview";
    pub const immt_treeview_inner: &str = "immt-treeview-inner";
    pub const immt_treeview_li: &str = "immt-treeview-li";
    pub const immt_treeview_summary: &str = "immt-treeview-summary";
    pub const immt_treeview_group: &str = "immt-treeview-group";
    pub const immt_treeview_archive: &str = "immt-treeview-archive";
    pub const immt_treeview_directory_open: &str = "immt-treeview-directory-open";
    pub const immt_treeview_directory_closed: &str = "immt-treeview-directory-closed";
    pub const immt_treeview_file_up_to_date: &str = "immt-treeview-file-up-to-date";
    pub const immt_treeview_file_stale: &str = "immt-treeview-file-stale";
    pub const immt_treeview_file_deleted: &str = "immt-treeview-file-deleted";
    pub const immt_treeview_file_new: &str = "immt-treeview-file-new";
    pub const immt_treeview_left_margin: &str = "immt-treeview-left-margin";
}
use treeview::*;
use crate::components::content::{SHtmlComponent,Document};
use crate::components::queue::enqueue;

#[component]
pub fn ArchivesTop() -> impl IntoView {
    view!(
        <Await future = || get_archives(None) let:archives blocking=true><ul class=immt_treeview>{match archives {
                Ok(archives) => archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(uri,state) => view!{<li class=immt_treeview_li><Archive uri=*uri state=state.clone()/></li>},
                    ArchiveOrGroup::Group(id,state) => view!{<li class=immt_treeview_li><ArchiveGroup id=id.clone() state=state.clone()/></li>}
                }).collect::<Vec<_>>(),
                _ => Vec::new()
            }}
        </ul></Await>
    )
}

#[island]
pub fn ArchiveOrGroups(path:ArchiveId) -> impl IntoView {
    use thaw::*;
    //console_log!("Loading archive/group {path}");
    //let pathcl = path.clone();
    let resource = create_local_resource(move || Some(path.clone()), get_archives);
    move || view!{
        <Suspense fallback=|| view!(<Spinner size=SpinnerSize::Tiny/>)>
        <ul class=immt_treeview_inner>{
            match resource.get() {
                Some(Ok(archives)) => {archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(uri,state) => view!{<li class=immt_treeview_li><Archive uri=uri state/></li>},
                    ArchiveOrGroup::Group(id,state) => view!{<li class=immt_treeview_li><ArchiveGroup id state/></li>}
                }).collect::<Vec<_>>()},
                _ => Vec::new()
            }
        }</ul>
        </Suspense>
    }
}

fn badge(new:u32,stale:u32) -> impl IntoView {
    use thaw::{Badge,BadgeVariant};
    let f = move || view!(<div style="margin-left:10px;display:inline-block;">
            <Badge variant=BadgeVariant::Success value=new>
            <Badge variant=BadgeVariant::Warning value=stale>""
            </Badge></Badge>
        </div>);
    if_logged_in(f,|| view!(<div/>))
}


#[island]
fn ArchiveGroup(id:ArchiveId, state: AllStates) -> impl IntoView {
    use thaw::*;
    let (expanded, set_expanded) = create_signal(false);
    let (clicked,click) = create_signal(false);
    let i = id.clone();
    let i2 = id.clone();
    let fullstate = state;
    //console_log!("{}={}",id.num(),id);
    let state = fullstate.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=move |_| {set_expanded.update(|b| *b = !*b)}>
            <span><Icon icon=icondata_bi::BiLibraryRegular/>" "{i2.as_str().split('/').last().unwrap().to_string()}</span>
            <Modal show=(clicked,click)><GroupModal id=i state=fullstate/></Modal>
            <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
            <span class=immt_treeview_left_margin/>
            {move || badge(state.new,state.stale.0)}
        </summary>
        <span>{move || {if expanded.get() {
            Some(view!{<ArchiveOrGroups path=id.clone()/>})
            } else { None }
        }}</span>
    </details>}
}

#[island]
fn Archive(uri:ArchiveURI, state: AllStates) -> impl IntoView {
    use thaw::*;
    let (expanded, set_expanded) = create_signal(false);
    let (clicked,click) = create_signal(false);
    //console_log!("{}={}",id.num(),id);
    let s = state.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=move |_| set_expanded.update(|b| *b = !*b)>
            <span><Icon icon=icondata_bi::BiBookSolid/>" "
                {uri.id().as_str().split('/').last().unwrap().to_string()}
            </span>
            <Modal show=(clicked,click)><ArchiveModal id=uri.id() state=state/></Modal>
            <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
            <span class=immt_treeview_left_margin/>
            {badge(s.new,s.stale.0)}
        </summary>
        {move || if expanded.get() {
            Some(view!{<DirOrFiles archive=uri inner=true/>})
            } else { None }
        }
    </details>}
}

#[island]
fn DirOrFiles(archive:ArchiveURI, #[prop(optional)] path:String, inner:bool) -> impl IntoView {
    let cls = if inner { immt_treeview_inner } else { immt_treeview };
    let path = if path.is_empty() {None} else {Some(path)};
    //console_log!("{}={}",archive.num(),archive);
    view!{<ul class=cls>{crate::components::with_spinner(
        move || path.clone(),
        move |p| get_files_in(archive.id(),p),
        archive,move |archive,archives| {
        archives.into_iter().map(|a| match a{
            DirOrFile::File(file) => view!{<li class=immt_treeview_li><SourceFile archive=archive.clone() file/></li>},
            DirOrFile::Dir(path,state) => view!{<li class=immt_treeview_li><Directory archive=archive.clone() path state/></li>}
        }).collect::<Vec<_>>()
    })}</ul>}
}


#[island]
fn Directory(archive:ArchiveURI, path:String, state: AllStates) -> impl IntoView {
    use thaw::*;
    let (expanded, set_expanded) = create_signal(false);
    let on_click = move |_| set_expanded.update(|b| *b = !*b);
    let (clicked,click) = create_signal(false);
    let p = path.clone();
    let p2 = path.clone();
    let fullstate = state;
    let state = fullstate.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=on_click>
            <span><Icon icon=icondata_bi::BiFolderRegular/>" "
                {path.split('/').last().unwrap().to_string()}
            </span>
            <Modal show=(clicked,click)><DirectoryModal archive=archive.id() path=p state=fullstate/></Modal>
            <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
            <span class=immt_treeview_left_margin/>
            {badge(state.new,state.stale.0)}
        </summary>
        {move || if expanded.get() {
            Some(view!{<DirOrFiles archive=archive.clone() path=p2.clone() inner=true/>})
            } else { None }
        }
    </details>}
}

#[island]
fn SourceFile(archive:ArchiveURI,file:SourceFile) -> impl IntoView {
    use thaw::*;
    use crate::components::IFrame;
    let show = create_rw_signal(false);

    let mut name = format!("{} [{}]",file.relative_path.split('/').last().unwrap(),file.format);
    let cls = match file.build_state {
        BuildState::UpToDate {last_built,..} => {
            name.push_str(&format!(" (last built: {})",last_built));
            immt_treeview_file_up_to_date
        },
        BuildState::Stale {last_built,..} => {
            name.push_str(&format!(" (last built: {})",last_built));
            immt_treeview_file_stale
        },
        BuildState::Deleted => immt_treeview_file_deleted,
        BuildState::New => immt_treeview_file_new
    };
    let (clicked,click) = create_signal(false);
    let p = file.relative_path.clone();
    view!{
        <span class=cls on:click=move |_| show.set(true)><Icon icon=icondata_bi::BiFileRegular/>" "{name.clone()}</span>
        <Drawer title=format!("[{archive}]/{}",file.relative_path) show placement=DrawerPlacement::Right width="70%">
            /*<DrawerHeader>
                <DrawerHeaderTitle></DrawerHeaderTitle>
            </DrawerHeader>*/
        {
            move || if show.get() {
                view!(<IFrame src=format!("?a={}&rp={}",archive.id(),p) ht="calc(100vh - 110px)".to_string()/>)
            } else {view!(<span/>).into_view()}
        }</Drawer>
        <Modal show=(clicked,click)><FileModal archive=archive.id() file/></Modal>
        <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
    }
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
        ModalType::Group => (create_action(move |b| {
            enqueue(None,Some(onclickid.clone()),onclicktgt.clone(),None,*b)
        }),id.to_string()),
        ModalType::Archive => (create_action(move |b| {
            enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),None,*b)
        }),id.to_string()),
        ModalType::Directory(p) => {
            let s = format!("[{}]/{}",id,p);
            (create_action(move |b| {
            enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b)
        }),s)
        },
        ModalType::File(p) => {
            let p = p.relative_path;
            let s = format!("[{}]/{}",id,p);
            (create_action(move |b| {
                enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),Some(p.clone()),*b)
            }),s)
        },
    };

    view!{
        <div class="immt-treeview-file-card"><Card title>
            <CardHeaderExtra slot><Tag>{target_str}</Tag></CardHeaderExtra>
            {if_logged_in(
                move || {
                    view!{<span>
                    <button on:click=move |_| act.dispatch(false)>"build all stale/new"</button>
                    <button on:click=move |_| act.dispatch(true)>"force build all"</button>
                    </span>}
                },
                || view!{<span/>}
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
