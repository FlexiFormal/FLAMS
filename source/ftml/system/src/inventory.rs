use std::{path::Path, sync::Arc};

#[cfg(feature = "tokio")]
use flams_ontology::uris::{ArchiveURI, ArchiveURITrait};
use flams_ontology::{
    content::modules::OpenModule,
    narration::{
        documents::{DocumentStyles, OpenDocument},
        problems::CognitiveDimension,
        DocumentElement,
    },
    uris::{DocumentURI, ModuleURI, SymbolURI, URIRefTrait},
    DocumentRange, Unchecked,
};
use flams_system::{
    backend::archives::{
        source_files::{FileState, SourceDir, SourceFile},
        ArchiveBase, ArchiveTrait, ExternalArchive, Fut, RepositoryData,
    },
    formats::SourceFormat,
};
use flams_utils::{unwrap, vecmap::VecMap, CSS};
use parking_lot::RwLock;
use rustc_hash::FxBuildHasher;

#[derive(Debug)]
struct Frags {
    css: Vec<CSS>,
    body: DocumentRange,
    frag: VecMap<Box<str>, DocumentRange>,
}

#[derive(Debug)]
struct CachedData {
    dir: RwLock<SourceDir>,
    docs: dashmap::DashMap<DocumentURI, OpenDocument<Unchecked>, FxBuildHasher>,
    mods: dashmap::DashMap<ModuleURI, OpenModule<Unchecked>, FxBuildHasher>,
    frags: dashmap::DashMap<
        (
            Option<flams_ontology::uris::Name>,
            flams_ontology::uris::NameStep,
            flams_ontology::languages::Language,
        ),
        Frags,
        FxBuildHasher,
    >,
}

#[derive(Debug)]
pub struct InventoriedArchive {
    data: RepositoryData,
    top_path: Arc<Path>,
    remote_url: Arc<url::Url>,
    //pub(super) ignore: IgnoreSource,
    cache: Arc<CachedData>,
    is_managed: std::sync::OnceLock<Option<git_url_parse::GitUrl>>,
}

impl ArchiveBase for InventoriedArchive {
    #[inline]
    fn data(&self) -> &RepositoryData {
        &self.data
    }
    #[inline]
    fn files(&self) -> &parking_lot::RwLock<SourceDir> {
        &self.cache.dir
    }
    #[cfg(feature = "git")]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
        let gl = flams_system::settings::Settings::get()
            .gitlab_url
            .as_ref()?;
        self.is_managed
            .get_or_init(|| {
                let Ok(repo) = flams_git::repos::GitRepo::open(&*self.top_path) else {
                    return None;
                };
                gl.host_str().and_then(|s| repo.is_managed(s))
            })
            .as_ref()
    }
    #[cfg(not(feature = "git"))]
    fn is_managed(&self) -> Option<&git_url_parse::GitUrl> {
        None
    }
    fn update_sources(
        &self,
        _: &flams_utils::change_listener::ChangeSender<flams_system::backend::BackendChange>,
    ) {
        #[cfg(feature = "tokio")]
        {
            use flams_ontology::uris::URIRefTrait;

            flams_system::async_bg(update_async(
                self.cache.clone(),
                self.uri().owned(),
                self.remote_url.clone(),
            ));
        }
        #[cfg(not(feature = "tokio"))]
        {
            update_sync(&self.cache, &self.uri(), &self.remote_url);
        }
    }
}

impl InventoriedArchive {
    fn get<R>(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        and_then: impl FnOnce(String) -> R,
    ) -> Option<R> {
        let url: url::Url = if let Some(path) = path {
            self.remote_url.join(&format!("{}/", path)).ok()?
        } else {
            (&*self.remote_url).clone()
        };
        let url = url.join(&format!("{name}.html")).ok()?;
        reqwest::blocking::get(url)
            .ok()
            .and_then(|r| r.text().ok().map(and_then))
    }
    fn get_a<R>(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        and_then: impl FnOnce(String) -> R + Send + 'static,
    ) -> Fut<Option<R>> {
        let url: url::Url = if let Some(path) = path {
            self.remote_url.join(&format!("{}/", path)).ok()?
        } else {
            (&*self.remote_url).clone()
        };
        let url = url.join(&format!("{name}.html")).ok()?;
        Some(Box::pin(async move {
            let r = reqwest::get(url).await.ok()?;
            r.text().await.ok().map(and_then)
        }))
    }
}

impl ArchiveTrait for InventoriedArchive {
    #[inline]
    fn load_html_full(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        _: flams_ontology::languages::Language,
    ) -> Option<String> {
        self.get(path, name, |s| s)
    }
    #[inline]
    fn load_html_full_async(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        _: flams_ontology::languages::Language,
    ) -> Fut<Option<String>> {
        self.get_a(path, name, |s| s)
    }
    fn load_html_body(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        language: flams_ontology::languages::Language,
        full: bool,
    ) -> Option<(Vec<flams_utils::CSS>, String)> {
        let idx = (path.cloned(), name.clone(), language);
        let f = &*self.cache.frags.get(&idx)?;
        let css = f.css.clone();
        let body = f.body;
        self.get(path, name, move |s| {
            (css, s[body.start..body.end].to_string())
        })
    }
    fn load_html_body_async(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        language: flams_ontology::languages::Language,
        full: bool,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        let idx = (path.cloned(), name.clone(), language);
        let f = &*self.cache.frags.get(&idx)?;
        let css = f.css.clone();
        let body = f.body;
        self.get_a(path, name, move |s| {
            (css, s[body.start..body.end].to_string())
        })
    }

    fn load_html_fragment(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        language: flams_ontology::languages::Language,
        range: DocumentRange,
    ) -> Option<(Vec<flams_utils::CSS>, String)> {
        let idx = (path.cloned(), name.clone(), language);
        let f = &*self.cache.frags.get(&idx)?;
        let css = f.css.clone();
        self.get(path, name, move |s| {
            (css, s[range.start..range.end].to_string())
        })
    }

    fn load_html_fragment_async(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        language: flams_ontology::languages::Language,
        range: DocumentRange,
    ) -> Fut<Option<(Vec<CSS>, String)>> {
        let idx = (path.cloned(), name.clone(), language);
        let f = &*self.cache.frags.get(&idx)?;
        let css = f.css.clone();
        self.get_a(path, name, move |s| {
            (css, s[range.start..range.end].to_string())
        })
    }

    fn load_reference_blob(
        &self,
        _path: Option<&flams_ontology::uris::Name>,
        _name: &flams_ontology::uris::NameStep,
        _language: flams_ontology::languages::Language,
        _range: DocumentRange,
    ) -> eyre::Result<Vec<u8>> {
        Err(eyre::eyre!(
            "Not implemented (yet): References in Inventory"
        ))
    }
    fn load_document(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
        language: flams_ontology::languages::Language,
    ) -> Option<flams_ontology::narration::documents::UncheckedDocument> {
        let path = if let Some(path) = path {
            self.uri().owned() % path.clone()
        } else {
            self.uri().owned().into()
        };
        let doc = path & (name.clone(), language);
        self.cache.docs.get(&doc).as_deref().cloned()
    }
    fn load_module(
        &self,
        path: Option<&flams_ontology::uris::Name>,
        name: &flams_ontology::uris::NameStep,
    ) -> Option<OpenModule<Unchecked>> {
        let path = if let Some(path) = path {
            self.uri().owned() % path.clone()
        } else {
            self.uri().owned().into()
        };
        let md = path | name.clone();
        self.cache.mods.get(&md).as_deref().cloned()
    }
}
impl ExternalArchive for InventoriedArchive {}
impl InventoriedArchive {
    pub fn create_new(mut repo: RepositoryData, path: &Path) -> Option<Box<dyn ExternalArchive>> {
        let Some(indx) = repo.attributes.0.iter().position(|(i, _)| **i == *"index") else {
            tracing::error!("Inventoried Archive requires `index` field in MANIFEST.MF");
            return None;
        };
        let (_, remote_url) = repo.attributes.remove_index(indx);

        let Ok(remote_url) = remote_url.parse() else {
            tracing::error!("Illegal URL: {}", remote_url);
            return None;
        };

        Some(Box::new(Self {
            remote_url: std::sync::Arc::new(remote_url),
            top_path: path.clone().into(),
            data: repo,
            cache: std::sync::Arc::new(CachedData {
                dir: Default::default(),
                docs: Default::default(),
                mods: Default::default(),
                frags: Default::default(),
            }),
            is_managed: Default::default(),
        }) as _)
    }
}

/*
lazy_static::lazy_static! {
    static ref REL_LINK: regex::Regex = unwrap!(regex::Regex::new(
        r#"<a\s+[^>]*?href\s*=\s*["']([^"']+)["']"#
    ).ok());
}
 */

#[cfg(feature = "tokio")]
#[tracing::instrument(level = "info",
     target = "archives",
     name = "Scraping",
     fields(archive = %archive.archive_id()),
     skip_all
 )]
async fn update_async(
    data: std::sync::Arc<CachedData>,
    archive: ArchiveURI,
    url: std::sync::Arc<url::Url>,
) {
    if let Err(e) = update_async_i(data, archive, url).await {
        tracing::error!("{e}");
    }
}

#[tracing::instrument(level = "info",
     target = "archives",
     name = "Scraping",
     fields(archive = %archive.archive_id()),
     skip_all
 )]
fn update_sync(data: &CachedData, archive: &ArchiveURI, url: &url::Url) {
    if let Err(e) = update_sync_i(data, archive, url) {
        tracing::error!("{e}");
    }
}

#[cfg(feature = "tokio")]
async fn update_async_i(
    data: std::sync::Arc<CachedData>,
    archive: ArchiveURI,
    url: std::sync::Arc<url::Url>,
) -> Result<(), eyre::Report> {
    let client = reqwest::Client::new();
    let inventory: Inventory = get_async(&client, (*url).clone()).await?;
    let (mut docs, mut mods) = tokio::task::spawn_blocking(move || {
        do_los(
            inventory.learning_objects,
            inventory.definienda,
            &mut *data.dir.write(),
            &archive,
        )
    })
    .await??;

    /*for d in &docs {
        println!("{d:#?}");
    }
    println!("------------------------------------------------------------------");
    for m in &mods {
        println!("{m:#?}");
    }*/

    Ok(())
}

fn update_sync_i(
    data: &CachedData,
    archive: &ArchiveURI,
    url: &url::Url,
) -> Result<(), eyre::Report> {
    let client = reqwest::blocking::Client::new();
    let inventory: Inventory = get_sync(&client, (*url).clone())?;
    let (docs, mods) = do_los(
        inventory.learning_objects,
        inventory.definienda,
        &mut *data.dir.write(),
        archive,
    )?;

    Ok(())
}

fn do_los(
    v: Vec<InvLO>,
    defs: Vec<Def>,
    dir: &mut SourceDir,
    uri: &ArchiveURI,
) -> Result<
    (
        Vec<(OpenDocument<Unchecked>, Vec<(String, Vec<SymbolURI>)>)>,
        Vec<OpenModule<Unchecked>>,
    ),
    eyre::Report,
> {
    use flams_ontology::uris::ArchiveURITrait;

    let mut tds = Vec::new();
    for lo in v {
        let urlstr = lo.url.strip_suffix(".html").unwrap_or(&lo.url);
        let uri = DocumentURI::from_archive_relpath(uri.clone(), urlstr)?;
        tds.push((
            OpenDocument::<Unchecked> {
                uri,
                title: None,
                elements: Vec::new(),
                styles: DocumentStyles::default(),
            },
            Vec::new(),
        ));
        dir.insert(SourceFile {
            relative_path: lo.url.into(),
            format: crate::FTML,
            target_state: VecMap::default(),
            format_state: FileState::New,
        });
    }

    let mut modsv = Vec::new();
    for Def { symbol, url } in defs {
        let (url, id) = url.rsplit_once('#').unwrap_or((&url, ""));
        let url = url.strip_suffix(".html").unwrap_or(url);
        let doc = DocumentURI::from_archive_relpath(uri.clone(), url)?;
        if let Some(d) = tds.iter_mut().find(|(d, _)| d.uri == doc) {
            let v = if let Some((_, v)) = d.1.iter_mut().find(|(a, _)| a == id) {
                v
            } else {
                d.1.push((id.to_string(), Vec::new()));
                &mut unwrap!(d.1.last_mut()).1
            };
            v.push(symbol.clone());
        }
        if symbol.archive_uri() == uri.archive_uri() {
            use flams_ontology::{
                content::{
                    declarations::{symbols::ArgSpec, OpenDeclaration},
                    modules::OpenModule,
                },
                uris::ContentURITrait,
                Unchecked,
            };

            let md = match modsv
                .iter_mut()
                .find(|m: &&mut OpenModule<Unchecked>| m.uri == *symbol.module())
            {
                Some(m) => m,
                None => {
                    let m = OpenModule {
                        uri: symbol.module().owned(),
                        meta: None,
                        signature: None,
                        elements: Vec::new(),
                    };
                    modsv.push(m);
                    unwrap!(modsv.last_mut())
                }
            };
            md.elements.push(OpenDeclaration::Symbol(
                flams_ontology::content::declarations::symbols::Symbol {
                    uri: symbol.clone(),
                    arity: ArgSpec::default(),
                    macroname: None,
                    role: Default::default(),
                    tp: None,
                    df: None,
                    assoctype: None,
                    reordering: None,
                },
            ));
        }
    }

    let index_uri = DocumentURI::from_archive_relpath(uri.clone(), "index")?;
    let index = match tds.iter_mut().find(|d| d.0.uri == index_uri) {
        Some(i) => &mut i.0,
        None => {
            tds.push((
                OpenDocument {
                    uri: index_uri,
                    title: None,
                    elements: Vec::new(),
                    styles: DocumentStyles::default(),
                },
                Vec::new(),
            ));
            &mut unwrap!(tds.last_mut()).0
        }
    };
    index.elements = modsv
        .iter()
        .map(|m| DocumentElement::Module {
            range: DocumentRange { start: 0, end: 0 },
            module: m.uri.clone().into(),
            children: Vec::new(),
        })
        .collect();
    /*let modsv = modsv
    .into_iter()
    .map(|m| m.check(&mut GlobalBackend::get().as_checker()))
    .collect();*/

    Ok((tds, modsv))
}

#[cfg(feature = "tokio")]
async fn get_async<R: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: url::Url,
) -> Result<R, eyre::Report> {
    Ok(client
        .execute(client.get(url).build()?)
        .await?
        .json()
        .await?)
}
fn get_sync<R: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: url::Url,
) -> Result<R, eyre::Report> {
    Ok(client.execute(client.get(url).build()?)?.json()?)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Inventory {
    learning_objects: Vec<InvLO>,
    #[serde(default)]
    definienda: Vec<Def>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Def {
    symbol: SymbolURI,
    url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct InvLO {
    #[serde(default)]
    objectives: Vec<InvPair>,
    #[serde(default)]
    prerequisites: Vec<InvPair>,
    url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct InvPair {
    dimension: CognitiveDimension,
    symbol: SymbolURI,
}
