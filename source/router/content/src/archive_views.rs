#![allow(clippy::must_use_candidate)]

use flams_backend_types::archive_json::ArchiveIndex;
use flams_web_utils::components::{LazySubtree, Leaf, Tree, wait_and_then_fn};
use ftml_component_utils::{Header, inject_css};
use ftml_uris::{
    ArchiveId, DocumentUri, FtmlUri, Uri, UriPath, UriWithArchive, UriWithPath,
    components::{UriComponentTuple, UriComponents},
};
use leptos::prelude::*;

#[component]
pub fn ArchiveView(comps: UriComponents) -> impl IntoView {
    let UriComponentTuple { uri, a, p, .. } = comps.into();
    inject_css(
        "flams-archive-block",
        ".flams-archive-block{ margin-left:10px;padding-top:0 !important;padding-bottom:0 !important;row-gap:0 !important;box-shadow:-3px 3px 5px -1px var(--colorBrandForeground1) !important;}",
    );
    wait_and_then_fn(
        move || archive_detail(uri.clone(), a.clone(), p.clone()),
        |a| a.into_view().into_any(),
    )
}

#[server(prefix = "/api/backend", endpoint = "archive_detail")]
async fn archive_detail(
    uri: Option<Uri>,
    a: Option<ArchiveId>,
    p: Option<String>,
) -> Result<ArchiveDetails, ServerFnError<String>> {
    use flams_math_archives::source_files::SourceEntryRef;
    use flams_math_archives::utils::path_ext::RelPath;
    use flams_math_archives::{
        Archive, MathArchive,
        backend::GlobalBackend,
        manager::{ArchiveOrGroup, ArchiveTree},
        source_files::SourceEntry,
    };
    use flams_web_utils::blocking_server_fn;
    fn convert(a: &ArchiveOrGroup, tree: &ArchiveTree) -> AorGEntry {
        match a {
            ArchiveOrGroup::Archive(a) => AorGEntry::Archive {
                id: a.clone(),
                index: tree.with_index(
                    flams_system::settings::Settings::get().external_url(),
                    |idx, _| idx.iter().find(|i| i.id() == a).cloned(),
                ),
            },
            ArchiveOrGroup::Group(gr) => AorGEntry::Group(gr.id.clone()),
        }
    }
    blocking_server_fn(move || {
        let (id, path) = match (uri, a, p) {
            (Some(Uri::Archive(a)), _, _) => (a.id, None),
            (Some(Uri::Path(a)), _, _) => (a.archive_id().clone(), a.path().cloned()),
            (None, Some(a), p) => (
                a,
                if let Some(p) = p {
                    if let Ok(p) = p.parse() {
                        Some(p)
                    } else {
                        return Err("Invalid path segment".to_string());
                    }
                } else {
                    None
                },
            ),
            _ => return Err("Invalid components".to_string()),
        };
        GlobalBackend.with_tree(move |tree| match tree.get_group_or_archive(&id) {
            Some(ArchiveOrGroup::Group(gr)) if path.is_none() => Ok(ArchiveDetails::Group {
                id,
                children: gr.children.iter().map(|e| convert(e, tree)).collect(),
            }),
            Some(ArchiveOrGroup::Archive(_)) => {
                let index = tree.with_index(
                    flams_system::settings::Settings::get().external_url(),
                    |idx, _| idx.iter().find(|i| i.id() == &id).cloned(),
                );
                let children = tree
                    .get(&id)
                    .and_then(|a| match a {
                        Archive::Local(l) => Some(l.with_sources(|s| {
                            let children = if let Some(p) = &path
                                && let Some(SourceEntryRef::Dir(d)) = s.find(RelPath::from_path(p))
                            {
                                &d.children
                            } else {
                                &s.children
                            };
                            children
                                .iter()
                                .map(|c| match c {
                                    SourceEntry::Dir(p) => DirOrFile::Dir(
                                        p.relative_path.clone().expect("should be impossible"),
                                    ),
                                    SourceEntry::File(sf) => DirOrFile::File {
                                        uri: DocumentUri::from_archive_relpath(
                                            l.uri().clone(),
                                            sf.relative_path.as_ref(),
                                        )
                                        .expect("should be impossible"),
                                        name: sf.relative_path.clone(),
                                    },
                                })
                                .collect()
                        })),
                        Archive::Ext(_, _) => None,
                    })
                    .unwrap_or_default();
                Ok(ArchiveDetails::Archive {
                    id,
                    index,
                    children,
                })
            }
            _ => Err(format!("Archive {id} not found")),
        })
    })
    .await
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::large_enum_variant)]
enum ArchiveDetails {
    Group {
        id: ArchiveId,
        children: Box<[AorGEntry]>,
    },
    Archive {
        id: ArchiveId,
        index: Option<ArchiveIndex>,
        children: Box<[DirOrFile]>,
    },
}
impl ArchiveDetails {
    fn do_children(self) -> (String, Option<Box<str>>, Option<Box<str>>, impl IntoView) {
        match self {
            Self::Group { id, children } => (
                id.to_string(),
                None,
                None,
                leptos::either::Either::Left(
                    children
                        .into_iter()
                        .map(AorGEntry::into_view)
                        .collect_view(),
                ),
            ),
            Self::Archive {
                id,
                index,
                children,
            } => {
                let (teaser, logo) = match index {
                    Some(
                        ArchiveIndex::Book {
                            teaser, thumbnail, ..
                        }
                        | ArchiveIndex::Paper {
                            thumbnail, teaser, ..
                        }
                        | ArchiveIndex::Library {
                            teaser, thumbnail, ..
                        }
                        | ArchiveIndex::Course {
                            teaser, thumbnail, ..
                        }
                        | ArchiveIndex::SelfStudy {
                            thumbnail, teaser, ..
                        },
                    ) => (teaser, thumbnail),
                    _ => (None, None),
                };
                (
                    id.to_string(),
                    teaser,
                    logo,
                    leptos::either::Either::Right(view!(<Tree>{
                        children
                            .into_iter()
                            .map(|d| d.into_view(id.clone()))
                            .collect_view()
                    }</Tree>)),
                )
            }
        }
    }
    fn into_view(self) -> impl IntoView {
        use flams_web_utils::components::{Layout, LayoutHeader};
        let (id, teaser, logo, children) = self.do_children();
        view! {
            <Layout>
                <LayoutHeader slot><h2>{id}</h2>
                    <div>
                        {teaser.map(|t| view!(<div inner_html=t.into_string()/>))}
                        {logo.map(|src| view!(
                            <div style="margin-left:auto;">
                                <img src=src.into_string() style="max-width:150px;max-height:150px;"/>
                            </div>
                        ))}

                    </div>
                </LayoutHeader>
                {children}
            </Layout>
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::large_enum_variant)]
enum AorGEntry {
    Group(ArchiveId),
    Archive {
        id: ArchiveId,
        index: Option<ArchiveIndex>,
    },
}
impl AorGEntry {
    fn into_view(self) -> impl IntoView {
        use ftml_component_utils::LazyCollapsible;
        use ftml_component_utils::{Block, BoldCaption, HeaderLeft, HeaderRight};
        let (title, id, teaser, logo) = match self {
            Self::Group(id) => (id.to_string(), id, None, None),
            Self::Archive { id, index } => {
                if let Some(i) = index {
                    (
                        format!("{} ({id})", i.title()),
                        id,
                        i.teaser().map(str::to_string),
                        i.thumbnail().map(str::to_string),
                    )
                } else {
                    (id.to_string(), id, None, None)
                }
            }
        };
        view! {
            <Block class="flams-archive-block">
                <Header slot><BoldCaption>{title}</BoldCaption></Header>
                <HeaderLeft slot>
                    <div inner_html=teaser style="font-size:small"/>
                </HeaderLeft>
                <HeaderRight slot>{logo.map(|s| view!(<img src=s style="max-width:100px;max-height:100px;"/>))}</HeaderRight>
                <LazyCollapsible>
                    <Header slot><span style="font-size:small;">"Contents"</span></Header>
                    {
                        let id = id.clone();
                        wait_and_then_fn(
                            move || archive_detail(None, Some(id.clone()), None),
                            |a| a.do_children().3.into_any(),
                        )
                    }
                </LazyCollapsible>
            </Block>
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::large_enum_variant)]
enum DirOrFile {
    Dir(UriPath),
    File { uri: DocumentUri, name: UriPath },
}

impl DirOrFile {
    fn into_view(self, id: ArchiveId) -> impl IntoView {
        use flams_web_utils::components::{Drawer, Trigger};
        use ftml_component_utils::{Button, ButtonAppearance};
        use leptos::either::Either::{Left, Right};
        match self {
            Self::Dir(path) => {
                let name = path
                    .as_ref()
                    .rsplit_once('/')
                    .map_or_else(|| path.to_string(), |(_, e)| e.to_string());
                let f = move || {
                    wait_and_then_fn(
                        move || archive_detail(None, Some(id.clone()), Some(path.to_string())),
                        |a| a.do_children().3.into_any(),
                    )
                };
                Left(view! {<LazySubtree>
                    <Header slot><ftml_component_utils::icons::FolderIcon/>" "{name}</Header>
                    {
                        (f.clone())()
                    }
                </LazySubtree>})
            }
            Self::File { uri, .. } => {
                let name = format!(" {} ({})", uri.name, uri.language);
                let namecl = name.clone();
                let link = format!("/?uri={}", uri.url_encoded());
                let comps = ftml_uris::components::DocumentUriComponents::Full(uri);
                Right(view! {<Leaf>
                <Drawer lazy=true>
                    <Trigger slot>
                        <ftml_component_utils::icons::FileIcon/>{name}
                    </Trigger>
                    <Header slot><a href=link target="_blank">
                      <Button appearance=ButtonAppearance::Subtle>{namecl}</Button>
                    </a></Header>
                    <div style="width:min-content">
                        <crate::components::Document doc=comps.clone()/>
                    </div>
                </Drawer>
                </Leaf>})
            }
        }
    }
}
