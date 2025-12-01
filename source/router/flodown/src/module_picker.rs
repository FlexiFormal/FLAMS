use flams_backend_types::archives::{ArchiveData, ArchiveGroupData, DirectoryData};
use flams_web_utils::components::{
    Header, LazySubtree, Leaf, Tree, wait_and_then, wait_and_then_fn,
};
use ftml_uris::{ArchiveId, ModuleUri, NamedUri};
use leptos::prelude::*;

pub fn picker(sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>) -> impl IntoView {
    wait_and_then_fn(
        || flams_router_backend::server_fns::group_entries(None),
        move |(groups, archives)| archives_and_groups(groups, archives, sig).into_any(),
    )
}

fn archives_and_groups(
    groups: Vec<ArchiveGroupData>,
    archives: Vec<ArchiveData>,
    sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>,
) -> impl IntoView {
    view! {
      {groups.into_iter().map(|g| group(g,sig)).collect_view()}
      {archives.into_iter().map(|m| archive(m,sig)).collect_view()}
    }
}

fn group(a: ArchiveGroupData, sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>) -> AnyView {
    let header = view!(
      <thaw::Icon icon=icondata_bi::BiLibraryRegular/>" "
      {a.id.last().to_string()}
    );
    let id = a.id;
    let f = move || flams_router_backend::server_fns::group_entries(Some(id.clone()));
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          wait_and_then(f.clone(),
          move |(groups,archives)|
            view!(<Tree>{archives_and_groups(groups, archives,sig)}</Tree>).into_any()
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn archive(a: ArchiveData, sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>) -> AnyView {
    let header = view!(
      <thaw::Icon icon=icondata_bi::BiBookSolid/>" "
      {a.id.last().to_string()}
    );
    let id = a.id;
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          let id = id.clone();
          let nid = id.clone();
          wait_and_then(move || flams_router_backend::server_fns::archive_modules(id.clone(),None),move |(dirs,mods)|
            view!(<Tree>{dirs_and_mods(&nid,dirs,mods,sig)}</Tree>).into_any()
          )
        }
      </LazySubtree>
    }.into_any()
}

fn dirs_and_mods(
    archive: &ArchiveId,
    dirs: Vec<DirectoryData>,
    mods: Vec<ModuleUri>,
    sig: RwSignal<std::collections::HashSet<ModuleUri, rustc_hash::FxBuildHasher>>,
) -> AnyView {
    view! {
      {dirs.into_iter().map(|d| dir(archive.clone(),d,sig)).collect_view()}
      {mods.into_iter().map(|m| module(m,sig)).collect_view()}
    }
    .into_any()
}

fn dir(
    archive: ArchiveId,
    d: DirectoryData,
    sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>,
) -> AnyView {
    let pathstr = unsafe { d.rel_path.split('/').last().unwrap_unchecked() }.to_string();
    let header = view!(
      <thaw::Icon icon=icondata_bi::BiFolderRegular/>" "
      {pathstr}
    );
    let id = archive.clone();
    let rel_path = d.rel_path;
    let f = move || {
        flams_router_backend::server_fns::archive_modules(id.clone(), Some(rel_path.clone()))
    };
    view! {
      <LazySubtree>
        <Header slot>{header}</Header>
        {
          let archive = archive.clone();
          wait_and_then(
              f.clone(),
              move |(dirs,mods)|
            view!(<Tree>{dirs_and_mods(&archive,dirs,mods,sig)}</Tree>).into_any()
          )
        }
      </LazySubtree>
    }
    .into_any()
}

fn module(uri: ModuleUri, sig: RwSignal<rustc_hash::FxHashSet<ModuleUri>>) -> AnyView {
    use thaw::Checkbox;
    let name = uri.name().last().to_string();
    let selected = RwSignal::new(false);
    let uricl = uri.clone();
    let mut changed = false;
    let _ = Effect::new(move || {
        sig.track();
        selected.track();
        if changed {
            changed = false;
            return;
        }
        if sig.with(|s| s.contains(&uricl) && !selected.get_untracked()) {
            selected.set(true);
            changed = true;
        }
        if selected.get() && !changed {
            changed = true;
            sig.update(|s| {
                s.insert(uri.clone());
            });
        }
    });
    view!(
        <Leaf><Checkbox checked=selected label=name/></Leaf>
    )
    .into_any()
}
