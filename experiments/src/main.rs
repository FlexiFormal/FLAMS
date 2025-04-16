use flams_lsp::state::LSPState;
use flams_ontology::uris::{DocumentURI, URIRefTrait};
use flams_system::backend::{
    archives::{
        source_files::{SourceDir, SourceEntry},
        Archive,
    },
    GlobalBackend,
};
use flams_utils::{prelude::TreeChildIter, time::measure, unwrap};
use git2::build::CheckoutBuilder;

pub fn main() {
    git_pull();
}

fn git_pull() {
    let path = std::path::Path::new("/home/jazzpirate/work/mh2/logic");
    let branch = "stex4";
    const NOTES_NS: &str = "refs/notes/flams";

    let repo = git2::Repository::open_ext(
        path,
        git2::RepositoryOpenFlags::NO_SEARCH.intersection(git2::RepositoryOpenFlags::NO_DOTGIT),
        std::iter::empty::<&std::ffi::OsStr>(),
    )
    .unwrap();

    // fetch
    let mut fetch = git2::FetchOptions::new();
    repo.find_remote("origin")
        .unwrap()
        .fetch(
            &[branch, &format!("+{NOTES_NS}:{NOTES_NS}")],
            Some(&mut fetch),
            None,
        )
        .unwrap();

    // latest commit
    let commit = repo
        .find_branch(&format!("origin/{branch}"), git2::BranchType::Remote)
        .unwrap()
        .get()
        .peel_to_commit()
        .unwrap();
    let fcomm: flams_git::Commit = commit.clone().into();
    println!("{fcomm:?}");
    let a_commit = repo.find_annotated_commit(commit.id()).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    let is_descendant = repo.graph_descendant_of(head.id(), commit.id()).unwrap();
    println!("Is descendant: {is_descendant}");

    let is_descendant = repo.graph_descendant_of(commit.id(), head.id()).unwrap();
    println!("Is descendant reverse: {is_descendant}");

    let anal = repo.merge_analysis(&[&a_commit]).unwrap();
    println!("Can fast-forward: {}", anal.0.is_fast_forward());
    if anal.0.is_fast_forward() {
        let headref = repo.reference(
            repo.head().unwrap().name().unwrap(),
            commit.id(),
            true,
            "Fast-forward",
        );
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
        return;
    }

    /*
    // merge
    let mut merge_options = git2::MergeOptions::new();
    let mut checkout = git2::build::CheckoutBuilder::new();
    merge_options
        .file_favor(git2::FileFavor::Theirs)
        .fail_on_conflict(false);

    repo.merge(
        &[&a_commit],
        Some(&mut merge_options),
        Some(&mut git2::build::CheckoutBuilder::new()),
    )
    .unwrap();
     */
}

fn git_urls() {
    tracing_subscriber::fmt().init();
    use git_url_parse::GitUrl;
    let url = GitUrl::parse("https://gl.mathhub.info/smglom/foo.git").expect("Failed to parse URL");
    tracing::info!("HTTPS: {url}\n{url:?}");
    let mut url2 =
        GitUrl::parse("git@gl.mathhub.info:smglom/foo.git").expect("Failed to parse URL");
    tracing::info!("HTTPS: {url2}\n{url2:?}");
    let gl = url::Url::parse("https://gl.mathhub.info").unwrap();
    tracing::info!("Top: {gl:?}\n = {}", gl.host_str().unwrap());
    let url3 = GitUrl::parse("http://192.168.1.1:7070/smglom/foo").unwrap();
    tracing::info!("local HTTPS: {url3:?}");
    let gl = url::Url::parse("http://192.168.1.1:7070").unwrap();
    tracing::info!("local top: {gl:?}\n = {}", gl.host_str().unwrap());
    url2 = url2.trim_auth();
    url2.scheme = git_url_parse::Scheme::Https;
    url2.scheme_prefix = true;
    if !url2.path.starts_with('/') {
        url2.path = format!("/{}", url2.path);
    }
    tracing::info!("Converted to HTTPS: {url2}\n{url2:?}");
}

pub fn linter() {
    let mut rt = tokio::runtime::Builder::new_multi_thread();
    rt.enable_all();
    rt.thread_stack_size(2 * 1024 * 1024);
    rt.build()
        .expect("Failed to initialize Tokio runtime")
        .block_on(linter_i());
}

async fn linter_i() {
    tracing_subscriber::fmt().init();
    let _ce = color_eyre::install();
    let mut spec = flams_system::settings::SettingsSpec::default();
    spec.lsp = true;
    flams_system::settings::Settings::initialize(spec);
    flams_system::backend::GlobalBackend::initialize();
    //flams_system::initialize(spec);
    let state = LSPState::default();
    tracing::info!("Waiting for stex to load...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    tracing::info!("Go!");
    let (_, t) = measure(move || {
        tracing::info!("Loading all archives");
        let mut files = Vec::new();
        for a in GlobalBackend::get().all_archives().iter() {
            if let Archive::Local(a) = a {
                a.with_sources(|d| {
                    for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
                        match e {
                            SourceEntry::File(f) => files.push((
                                f.relative_path
                                    .split('/')
                                    .fold(a.source_dir(), |p, s| p.join(s))
                                    .into(),
                                unwrap!(DocumentURI::from_archive_relpath(
                                    a.uri().owned(),
                                    &f.relative_path,
                                ).ok()),
                            )),
                            _ => {}
                        }
                    }
                })
            }
        }
        let len = files.len();
        tracing::info!("Linting {len} files");
        state.load_all(
            files.into_iter(), /*.enumerate().map(|(i,(path,uri))| {
                                 tracing::info!("{}/{len}: {}",i+1,path.display());
                                 (path,uri)
                               })*/
            |_, _| {},
        );
    });
    tracing::info!("initialized after {t}");
}
