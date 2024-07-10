use leptos::*;
use immt_core::building::buildstate::{BuildState, AllStates};
use immt_core::uris::archives::{ArchiveId, ArchiveURI};
use immt_core::utils::filetree::SourceFile;
use crate::console_log;
use crate::accounts::{if_logged_in, LoginState};

#[cfg(feature = "server")]
mod server {
    use immt_api::backend::archives::Archive;
    use immt_core::ontology::archives::ArchiveGroup as AGroup;
    use immt_controller::{BaseController,ControllerTrait,controller};
    use super::{ArchiveOrGroup, DirOrFile};
    use immt_core::building::buildstate::AllStates;
    use immt_core::prelude::*;
    use immt_core::uris::archives::ArchiveId;
    use immt_core::utils::filetree::{SourceDir, SourceDirEntry, SourceFile};

    //#[cfg(feature="async")]
    pub fn get_archive_children(prefix:Option<&str>) -> Option<Vec<ArchiveOrGroup>> {
        controller().archives().with_tree(|toptree| {
            let (tree,has_meta) = match prefix {
                Some(prefix) => match toptree.find_group_or_archive(prefix)? {
                    AGroup::Group{children,has_meta,..} => (children.as_slice(),*has_meta),
                    _ => return None
                },
                None => (toptree.groups(),false)
            };
            let mut children = tree.iter().filter_map(|v| match v {
                AGroup::Archive(id) => {
                    if let Some(Archive::Physical(ma)) = toptree.find_archive(id) {
                        Some(ArchiveOrGroup::Archive(id.to_string(), ma.state().clone()))
                    } else {None}
                },
                AGroup::Group{id,state,..} => Some(ArchiveOrGroup::Group(id.to_string(),state.clone()))
            }).collect::<Vec<_>>();
            if has_meta {
                let name = format!("{}/meta-inf",prefix.unwrap());
                let id = ArchiveId::new(name.as_str());
                children.insert(0,ArchiveOrGroup::Archive(name, if let Some(Archive::Physical(ma)) = toptree.find_archive(
                    &id
                ) {
                    ma.state().clone()
                } else {AllStates::default()}));
            }
            Some(children)
        })
    }
    /*
    #[cfg(not(feature="async"))]
    pub fn get_archive_children(prefix:Option<&str>) -> Option<Vec<ArchiveOrGroup>> {
        controller().archives().with_tree(|toptree| {
            let (tree,has_meta) = match prefix {
                Some(prefix) => match toptree.find_group_or_archive(prefix)? {
                    AGroup::Group{children,has_meta,..} => (children.as_slice(),*has_meta),
                    _ => return None
                },
                None => (toptree.groups(),false)
            };
            let mut children = tree.iter().filter_map(|v| match v {
                AGroup::Archive(id) => {
                    if let Some(Archive::Physical(ma)) = toptree.find_archive(id) {
                        Some(ArchiveOrGroup::Archive(id.to_string(), ma.state().clone()))
                    } else {None}
                },
                AGroup::Group{id,state,..} => Some(ArchiveOrGroup::Group(id.to_string(),state.clone()))
            }).collect::<Vec<_>>();
            if has_meta {
                children.insert(0,ArchiveOrGroup::Archive(format!("{}/meta-inf",prefix.unwrap()), AllStates::default()));
            }
            Some(children)
        })
    }

     */

    pub fn get_dir_children(archive:&str,path:Option<&str>) -> Option<Vec<DirOrFile>> {
        controller().archives().find(archive,|a| {
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
    /*
    #[cfg(not(feature="async"))]
    pub fn get_dir_children(archive:&str,path:Option<&str>) -> Option<Vec<DirOrFile>> {
        controller().archives().find(archive,|a| {
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

     */
}


#[server(
    prefix="/api/backend",
    endpoint="archives",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_archives(prefix:Option<String>) -> Result<Vec<ArchiveOrGroup>,ServerFnError<String>> {
    match server::get_archive_children(prefix.as_deref()) {
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
pub async fn get_files_in(archive:String,prefix:Option<String>) -> Result<Vec<DirOrFile>,ServerFnError<String>> {
    match server::get_dir_children(archive.as_str(),prefix.as_deref()) {
        Some(v) => Ok(v),
        _ => Err(ServerFnError::WrappedServerError(format!("No archive {} found!",prefix.unwrap())))
    }
}

#[derive(Clone,Debug,serde::Serialize, serde::Deserialize)]
pub enum ArchiveOrGroup {
    Archive(String, AllStates),
    Group(String, AllStates)
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
use crate::components::queue::enqueue;

#[component]
pub fn ArchivesTop() -> impl IntoView {
    use thaw::*;
    view!(
        <Await future = || get_archives(None) let:archives blocking=true><ul class=immt_treeview>{match archives {
                Ok(archives) => archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(id,state) => view!{<li class=immt_treeview_li><Archive id=id.clone() state=state.clone()/></li>},
                    ArchiveOrGroup::Group(id,state) => view!{<li class=immt_treeview_li><ArchiveGroup id=id.clone() state=state.clone()/></li>}
                }).collect::<Vec<_>>(),
                _ => Vec::new()
            }}
        </ul></Await>
    )
}

#[island]
pub fn ArchiveOrGroups(path:String) -> impl IntoView {
    use thaw::*;
    //console_log!("Loading archive/group {path}");
    let pathcl = path.clone();
    let resource = create_local_resource(move || Some(path.clone()), get_archives);
    move || view!{
        <Suspense fallback=|| view!(<Spinner size=SpinnerSize::Tiny/>)>
        <ul class=immt_treeview_inner>{
            match resource.get() {
                Some(Ok(archives)) => {archives.into_iter().map(|a| match a{
                    ArchiveOrGroup::Archive(id,state) => view!{<li class=immt_treeview_li><Archive id state/></li>},
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
    //f
    /*view!{
        <IfLoggedIn>{f}</IfLoggedIn>
    }*/
}


#[island]
fn ArchiveGroup(id:String, state: AllStates) -> impl IntoView {
    use thaw::Modal;
    let (expanded, set_expanded) = create_signal(false);
    let (clicked,click) = create_signal(false);
    let i = id.clone();
    let i2 = id.clone();
    let state = state.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=move |_| {set_expanded.update(|b| *b = !*b)}>
            <span class=immt_treeview_group>{i2.split('/').last().unwrap().to_string()}</span>
            <Modal show=(clicked,click)><GroupModal id=i/></Modal>
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
fn Archive(id:String, state: AllStates) -> impl IntoView {
    use thaw::Modal;
    let (expanded, set_expanded) = create_signal(false);
    let (clicked,click) = create_signal(false);
    let i = id.clone();
    let i2 = id.clone();
    let s = state.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=move |_| set_expanded.update(|b| *b = !*b)>
            <span class=immt_treeview_archive>
                {id.split('/').last().unwrap().to_string()}
            </span>
            <Modal show=(clicked,click)><ArchiveModal id=i state=state/></Modal>
            <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
            <span class=immt_treeview_left_margin/>
            {badge(s.new,s.stale.0)}
        </summary>
        {move || if expanded.get() {
            Some(view!{<DirOrFiles archive=i2.clone() inner=true/>})
            } else { None }
        }
    </details>}
}

#[island]
fn DirOrFiles(archive:String, #[prop(optional)] path:String, inner:bool) -> impl IntoView {
    let cls = if inner { immt_treeview_inner } else { immt_treeview };
    let path = if path.is_empty() {None} else {Some(path)};
    let a = archive.clone();
    view!{<ul class=cls>{crate::components::with_spinner(
        move || (a.clone(),path.clone()),
        |(a,p)| get_files_in(a,p),
        archive,move |archive,archives| {
        archives.into_iter().map(|a| match a{
            DirOrFile::File(file) => view!{<li class=immt_treeview_li><SourceFile archive=archive.clone() file/></li>},
            DirOrFile::Dir(path,state) => view!{<li class=immt_treeview_li><Directory archive=archive.clone() path state/></li>}
        }).collect::<Vec<_>>()
    })}</ul>}
}


#[island]
fn Directory(archive:String, path:String, state: AllStates) -> impl IntoView {
    use thaw::Modal;
    let (expanded, set_expanded) = create_signal(false);
    let on_click = move |_| set_expanded.update(|b| *b = !*b);
    let (clicked,click) = create_signal(false);
    let a = archive.clone();
    let p = path.clone();
    let p2 = path.clone();
    let state = state.summary();
    view!{<details>
        <summary class=immt_treeview_summary on:click=on_click>
            <span class=move || {if expanded.get() {immt_treeview_directory_open} else {immt_treeview_directory_closed}}>
                {path.split('/').last().unwrap().to_string()}
            </span>
            <Modal show=(clicked,click)><DirectoryModal archive=a path=p/></Modal>
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
fn SourceFile(archive:String,file:SourceFile) -> impl IntoView {
    use thaw::Modal;
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
    view!{
        <span class=cls>{name}</span>
        <Modal show=(clicked,click)><FileModal archive file/></Modal>
        <span on:click=move |_| {click.set(true)} style="cursor: help;">" 🛈"</span>
    }
}

#[island]
fn GroupModal(id:String) -> impl IntoView {
    use thaw::*;
    view!{
        <div class="immt-treeview-file-card"><Card title=id>
            <CardHeaderExtra slot><Tag>"TODO"</Tag></CardHeaderExtra>
            <Table>
                <thead>
                    <tr><td>"Build Target"</td><td>"Last Built"</td></tr>
                </thead>
                <tbody>
                    <tr><td><b>"Last Built"</b></td><td>"TODO"</td></tr>
                </tbody>
            </Table>
        </Card></div>
    }
}

#[island]
fn ArchiveModal(id:String, state: AllStates) -> impl IntoView {
    use thaw::*;
    //console_log!("ArchiveModal: {:?}",state);
    let targets = state.targets().map(|(i,_)| i).collect::<Vec<_>>();
    let onclicktgt = *targets.first().unwrap();
    let target_str = targets.iter().map(|t| t.to_string()).collect::<String>();
    let onclickid = id.clone();
    let act = create_action(move |b| {
        //console_log!("building [{onclickid}]: {onclicktgt} ({})",*b);
        enqueue(Some(onclickid.clone()),None,onclicktgt.clone(),None,*b)
    });
    view!{
        <div class="immt-treeview-file-card"><Card title=id>
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
}

#[island]
fn DirectoryModal(archive:String,path:String) -> impl IntoView {
    use thaw::*;
    let title = format!("[{}]/{}",archive,path);
    view!{
        <div class="immt-treeview-file-card"><Card title>
            <CardHeaderExtra slot><Tag>"TODO"</Tag></CardHeaderExtra>
            <Table>
                <thead>
                    <tr><td>"Build Target"</td><td>"Last Built"</td></tr>
                </thead>
                <tbody>
                    <tr><td><b>"Last Built"</b></td><td>"TODO"</td></tr>
                </tbody>
            </Table>
        </Card></div>
    }
}

#[island]
fn FileModal(archive:String,file:SourceFile) -> impl IntoView {
    use thaw::*;
    let title = format!("[{}]/{}",archive,file.relative_path);
    let format = file.format.to_string();
    view!{//<Space vertical=true>
        <div class="immt-treeview-file-card"><Card title>
            <CardHeaderExtra slot><Tag>{format}</Tag></CardHeaderExtra>
            <Table>
                <thead>
                    <tr><td>"Build Target"</td><td>"Last Built"</td></tr>
                </thead>
                <tbody>
                    <tr><td><b>"Last Built"</b></td><td>{
                        match file.build_state {
                            BuildState::UpToDate {last_built,..} => view!(<Tag variant=TagVariant::Success>"Up to date ("{last_built.to_string()}")"</Tag>),
                            BuildState::Stale {last_built,..} => view!(<Tag variant=TagVariant::Warning>"Changed ("{last_built.to_string()}")"</Tag>),
                            BuildState::Deleted =>view!(<Tag variant=TagVariant::Error>"Deleted"</Tag>),
                            BuildState::New =>view!(<Tag>"New"</Tag>)
                        }
                    }</td></tr>
                </tbody>
            </Table>
            /*<CardFooter slot>"footer"</CardFooter>*/
        </Card></div>
    /*</Space>*/}
}
