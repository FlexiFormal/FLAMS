use std::path::{Path, PathBuf};

use crate::{
    Archive, MathArchive,
    backend::LocalBackend,
    document_file::DocumentFile,
    formats::SourceFormatId,
    mathhub::mathhubs,
    source_files::FileStates,
    utils::{
        AsyncEngine,
        errors::{BackendError, ManifestParseError, NewArchiveError},
        path_ext::{PathExt, RelPath},
    },
};
use flams_backend_types::archive_json::{ArchiveIndex, Institution};
use ftml_ontology::{
    domain::modules::Module,
    utils::{RefTree, TreeChild, awaitable::AsyncCache},
};
use ftml_uris::{ArchiveId, ArchiveUri, BaseUri, DocumentUri, ModuleUri, UriPath, UriWithArchive};

#[derive(Debug)]
pub struct ArchiveManager {
    pub(crate) tree: parking_lot::RwLock<ArchiveTree>,
    pub(crate) modules: AsyncCache<ModuleUri, Module, BackendError>,
    pub(crate) documents: AsyncCache<DocumentUri, triomphe::Arc<DocumentFile>, BackendError>,
    #[cfg(feature = "rdf")]
    triple_store: crate::triple_store::RDFStore,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self {
            tree: parking_lot::RwLock::new(ArchiveTree::default()),
            modules: AsyncCache::default(),
            documents: AsyncCache::default(),
            #[cfg(feature = "rdf")]
            triple_store: crate::triple_store::RDFStore::default(),
        }
    }
}

impl ArchiveManager {
    #[inline]
    #[must_use]
    pub fn all_archives(&self) -> impl std::ops::Deref<Target = [Archive]> + '_ {
        parking_lot::RwLockReadGuard::map(self.tree.read(), |s| s.archives.as_slice())
    }

    #[cfg(feature = "rdf")]
    #[inline]
    #[must_use]
    pub const fn triple_store(&self) -> &crate::triple_store::RDFStore {
        &self.triple_store
    }

    #[inline]
    pub fn with_tree<R>(&self, f: impl FnOnce(&ArchiveTree) -> R) -> R {
        f(&self.tree.read())
    }

    pub fn reinit<R>(
        &self,
        f: impl FnOnce(&mut ArchiveTree) -> R,
        paths: &[&Path],
        external_url: &str,
    ) -> R {
        let ls = self.tree.read().load(paths, external_url, false);
        let mut tree = self.tree.write();
        let r = f(&mut tree);
        tree.archives.clear();
        tree.top.clear();
        self.modules.clear();
        self.documents.clear();
        self.triple_store.clear();
        for a in ls.into_iter().flatten() {
            tree.insert(
                a,
                #[cfg(feature = "rdf")]
                &self.triple_store,
            );
        }
        r
    }
    /*
    pub(crate) fn load_document(
        &self,
        archive: &ArchiveUri,
        path: Option<&UriPath>,
        language: Language,
        name: &SimpleUriName,
    ) -> Option<UncheckedDocument> {
        self.with_archive(archive.archive_id(), |a| {
            let Some(a) = a else {
                return Err(crate::BackendError::ArchiveNotFound);
            };
            a.load_document(path, name, language)
        })
    }
     */

    pub(crate) fn load_module(
        &self,
        archive: &ArchiveUri,
        path: Option<&UriPath>,
        name: &str,
    ) -> Result<Module, crate::BackendError> {
        self.with_archive(archive.archive_id(), |a| {
            let Some(a) = a else {
                return Err(crate::BackendError::ArchiveNotFound);
            };
            a.load_module(path, name)
        })
    }
    pub(crate) fn load_module_async<A: AsyncEngine>(
        &self,
        archive: &ArchiveUri,
        path: Option<&UriPath>,
        name: &str,
    ) -> impl Future<Output = Result<Module, crate::BackendError>> + 'static + use<A> {
        self.with_archive(archive.archive_id(), |a| {
            let Some(a) = a else {
                return either::Left(std::future::ready(Err(
                    crate::BackendError::ArchiveNotFound,
                )));
            };
            either::Right(a.load_module_async::<A>(path, name))
        })
    }

    /// # Errors
    pub fn load_one(
        &self,
        manifest: &Path,
        rel_path: RelPath,
        external_url: &str,
    ) -> Result<(), ManifestParseError> {
        let a = crate::manifest::parse_manifest(manifest, rel_path, external_url)?;
        if let Archive::Local(a) = &a {
            a.update_sources();
        }
        self.tree.write().insert(
            a,
            #[cfg(feature = "rdf")]
            &self.triple_store,
        );
        Ok(())
    }

    pub fn load(&self, paths: &[&Path], external_url: &str) {
        let ls = self.tree.read().load(paths, external_url, true);
        let mut lock = self.tree.write();
        for a in ls.into_iter().flatten() {
            lock.insert(
                a,
                #[cfg(feature = "rdf")]
                &self.triple_store,
            );
        }
    }

    /// # Errors
    /// # Panics
    pub fn new_archive(
        &self,
        id: &ArchiveId,
        base_uri: &BaseUri,
        format: SourceFormatId,
        default_file: &str,
        content: &str,
        external_url: &str,
    ) -> Result<PathBuf, NewArchiveError> {
        use std::io::Write;
        let mh = *mathhubs().first().ok_or(NewArchiveError::NoMathHub)?;
        let meta_inf = id
            .steps()
            .fold(mh.to_path_buf(), |p, s| p.join(s))
            .join("META-INF");
        // SAFETY: we constructed the path as a descendant of mh
        let root = unsafe { meta_inf.parent().unwrap_unchecked() };
        macro_rules! err {
            ($p:pat = $expr:expr;$id:ident) => {
                #[allow(clippy::let_unit_value)]
                let $p = match $expr {
                    Ok(v) => v,
                    Err(e) => return Err(NewArchiveError::$id(root.to_path_buf(), e)),
                };
            };
        }
        macro_rules! dump {
            ($f:expr; $($t:tt)*) => {
                err!(f = std::fs::File::create(&$f);Write);
                if let Err(e) = write!(std::io::BufWriter::new(f),$($t)*) {
                    return Err(NewArchiveError::Write($f, e));
                }
            };
        }
        err!(() = std::fs::create_dir_all(&meta_inf);CreateDir);
        let manifest = meta_inf.join("MANIFEST.MF");
        err!(mf = std::fs::File::create_new(&manifest);Write);
        if let Err(e) = write!(
            std::io::BufWriter::new(mf),
            "id: {id}\nurl-base: {base_uri}\nformat: {}",
            format.name
        ) {
            return Err(NewArchiveError::Write(manifest, e));
        }
        dump!(root.join(".gitignore");"{}",include_str!("gitignore_template.txt"));

        let lib = root.join("lib");
        err!(() = std::fs::create_dir_all(&lib);CreateDir);
        let preamble = lib.join("preamble.tex");
        dump!(preamble;"% preamble code for stex");

        let source = root.join("source");
        err!(() = std::fs::create_dir_all(&source);CreateDir);
        let default = source.join(default_file);
        dump!(default;"{}",content);
        self.load_one(&manifest, RelPath::from_id(id), external_url)
            .expect("this is a bug");
        Ok(root.to_path_buf())
    }
}

#[derive(Debug, Default)]
pub struct ArchiveTree {
    pub archives: Vec<Archive>,
    pub top: Vec<ArchiveOrGroup>,
    pub index: (Vec<Institution>, Vec<ArchiveIndex>),
}

#[derive(Debug)]
pub enum ArchiveOrGroup {
    Archive(ArchiveId),
    Group(ArchiveGroup),
}

#[derive(Debug)]
pub struct ArchiveGroup {
    pub id: ArchiveId,
    pub children: Vec<ArchiveOrGroup>,
    pub state: FileStates,
}

pub trait MaybeTriple: Send {
    #[cfg(feature = "rdf")]
    fn add_triple(&mut self, quad: impl FnOnce() -> ulo::rdf_types::Triple);
}
impl MaybeTriple for () {
    #[cfg(feature = "rdf")]
    #[inline]
    fn add_triple(&mut self, _: impl FnOnce() -> ulo::rdf_types::Triple) {}
}
#[cfg(feature = "rdf")]
impl<F> MaybeTriple for F
where
    F: FnMut(ulo::rdf_types::Triple) + Send,
{
    #[inline]
    fn add_triple(&mut self, quad: impl FnOnce() -> ulo::rdf_types::Triple) {
        self(quad());
    }
}

impl ArchiveTree {
    #[must_use]
    pub fn state(&self) -> FileStates {
        let mut r = FileStates::default();
        for aog in &self.top {
            match aog {
                ArchiveOrGroup::Archive(a) => {
                    if let Some(Archive::Local(a)) = self.get(a) {
                        r.merge_all(&a.file_state.read().state);
                    }
                }
                ArchiveOrGroup::Group(g) => r.merge_all(&g.state),
            }
        }
        r
    }

    #[must_use]
    pub fn get_group_or_archive(&self, id: &ArchiveId) -> Option<&ArchiveOrGroup> {
        let mut steps = id.steps().peekable();
        let mut curr = &self.top;
        while let Some(step) = steps.next() {
            let e = curr
                .binary_search_by_key(&step, |e| e.id().last())
                .ok()
                .map(|i| &curr[i])?;
            if steps.peek().is_none() {
                return Some(e);
            }
            if let ArchiveOrGroup::Group(g) = e {
                curr = &g.children;
            } else {
                return None;
            }
        }
        None
    }

    #[must_use]
    pub fn get(&self, id: &ArchiveId) -> Option<&Archive> {
        self.archives
            .binary_search_by_key(&id, |a: &Archive| a.id())
            .ok()
            .map(|i| &self.archives[i])
    }

    #[allow(clippy::linkedlist)]
    fn load(
        &self,
        paths: &[&Path],
        external_url: &str,
        skip_existent: bool,
    ) -> std::collections::LinkedList<Vec<Archive>> {
        use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
        use spliter::ParallelSpliterator;
        paths
            .par_iter()
            .flat_map(|p| {
                crate::archive_iter::ManifestIterator::new(p)
                    .par_split()
                    .into_par_iter()
                    .map(move |r| (p, r))
                    .filter_map(|(mh, p)| {
                        // SAFETY: manifest file is grandchild of root directory of archive
                        let top_dir =
                            unsafe { p.parent().unwrap_unchecked().parent().unwrap_unchecked() };
                        let rel_path = top_dir.relative_to(mh)?;
                        let Some(id): Option<ArchiveId> = rel_path.parse().ok() else {
                            tracing::warn!("invalid archive id: {rel_path}");
                            return None;
                        };
                        if skip_existent && self.get(&id).is_some() {
                            return None;
                        }
                        match crate::manifest::parse_manifest(&p, rel_path, external_url) {
                            Ok(r) => Some(r),
                            Err(e) => {
                                tracing::warn!("{e} in {rel_path}");
                                None
                            }
                        }
                    })
                    .map(|a| {
                        if let Archive::Local(a) = &a {
                            a.update_sources();
                        }
                        a
                    })
            })
            .collect_vec_list()
        /*for n in news.into_iter().flatten() {
            self.insert(n, &mut f);
        }*/
    }

    fn insert(
        &mut self,
        archive: Archive,
        #[cfg(feature = "rdf")] triple_store: &crate::triple_store::RDFStore,
    ) {
        #[cfg(feature = "rdf")]
        let mut triples = vec![{
            use ftml_uris::FtmlUri;
            ulo::triple!(<(archive.uri().to_iri())>: ulo:library)
        }];

        let id = archive.id().clone();
        let rel_path = RelPath::from_id(&id);
        let steps = if let Some((group, _)) = rel_path.split_last() {
            group.steps()
        } else {
            match self
                .archives
                .binary_search_by_key(&&id, |a: &Archive| a.id())
            {
                Ok(i) => self.archives[i] = archive,
                Err(i) => self.archives.insert(i, archive),
            }
            match self
                .top
                .binary_search_by_key(&id.as_ref(), |v| v.id().last())
            {
                Ok(i) => self.top[i] = ArchiveOrGroup::Archive(id),
                Err(i) => self.top.insert(i, ArchiveOrGroup::Archive(id)),
            }
            return;
        };
        let mut curr = &mut self.top;
        let mut curr_name_len = 0;
        let mut group = &id;
        for step in steps {
            if curr_name_len == 0 {
                curr_name_len += step.len();
            } else {
                curr_name_len += step.len() + 1;
            }
            let curr_name = &id.as_ref()[..curr_name_len];
            match curr.binary_search_by_key(&step, |v| v.id().last()) {
                Ok(i) => {
                    let ArchiveOrGroup::Group(g) = &mut curr[i]
                    // TODO maybe reachable?
                    else {
                        unreachable!()
                    };
                    if let Archive::Local(a) = &archive {
                        g.state.merge_all(a.file_state.read().state());
                    }
                    group = &g.id;
                    curr = &mut g.children;
                }
                Err(i) => {
                    let mut state = FileStates::default();
                    if let Archive::Local(a) = &archive {
                        state.merge_all(a.file_state.read().state());
                    }
                    let g = ArchiveGroup {
                        // SAFETY: known to be valid
                        id: unsafe { curr_name.parse().unwrap_unchecked() },
                        children: Vec::new(),
                        state,
                    };
                    curr.insert(i, ArchiveOrGroup::Group(g));
                    let ArchiveOrGroup::Group(g) = &mut curr[i] else {
                        unreachable!()
                    };
                    #[cfg(feature = "rdf")]
                    {
                        use ftml_uris::FtmlUri;
                        let iri = (archive.uri().base.clone() & g.id.clone()).to_iri();
                        if *group != id {
                            let parent = (archive.uri().base.clone() & group.clone()).to_iri();
                            triples.push(ulo::triple!(<(parent)> ulo:contains <(iri.clone())>));
                        }
                        triples.push(ulo::triple!(<(iri)>: ulo:library_group));
                    }
                    curr = &mut g.children;
                }
            }
        }

        #[cfg(feature = "rdf")]
        {
            use ftml_uris::FtmlUri;
            let parent = (archive.uri().base.clone() & group.clone()).to_iri();
            triples.push(ulo::triple!(<(parent)> ulo:contains <(archive.uri().to_iri())>));
            let global = ulo::rdf_types::NamedNodeRef::new_unchecked("flams://archives");
            triple_store.add_quads(triples.into_iter().map(|t| t.in_graph(global)));
        }

        match self
            .archives
            .binary_search_by_key(&&id, |a: &Archive| a.id())
        {
            Ok(i) => self.archives[i] = archive,
            Err(i) => self.archives.insert(i, archive),
        }
        match curr.binary_search_by_key(&id.last(), |v| v.id().last()) {
            Ok(i) => curr[i] = ArchiveOrGroup::Archive(id),
            Err(i) => curr.insert(i, ArchiveOrGroup::Archive(id)),
        }
    }
}

impl ArchiveOrGroup {
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &ArchiveId {
        match self {
            Self::Archive(id) => id,
            Self::Group(g) => &g.id,
        }
    }
}

impl<'a> TreeChild<'a> for &'a ArchiveOrGroup {
    fn tree_children(self) -> impl Iterator<Item = Self> {
        match self {
            ArchiveOrGroup::Archive(_) => either::Either::Left(std::iter::empty()),
            ArchiveOrGroup::Group(g) => either::Either::Right(g.children.iter()),
        }
    }
}

impl RefTree for ArchiveTree {
    type Child<'a>
        = &'a ArchiveOrGroup
    where
        Self: 'a;
    #[inline]
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        self.top.iter()
    }
}
impl RefTree for ArchiveGroup {
    type Child<'a>
        = &'a ArchiveOrGroup
    where
        Self: 'a;
    #[inline]
    fn tree_children(&self) -> impl Iterator<Item = Self::Child<'_>> {
        self.children.iter()
    }
}
