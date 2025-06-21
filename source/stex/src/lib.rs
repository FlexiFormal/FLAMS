//#![feature(lazy_type_alias)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod dependencies;
mod latex;
pub mod quickparse;
mod rustex;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use either::Either;
use eyre::Context;
use flams_ftml::{HTMLString, FTML_DOC, FTML_OMDOC};
use flams_ontology::uris::{ArchiveId, ArchiveURITrait, DocumentURI, PathURITrait, URIRefTrait};
use flams_system::{
    backend::{
        archives::{Archive, ArchiveOrGroup, LocalArchive},
        AnyBackend, Backend, GlobalBackend,
    },
    build_result, build_target,
    building::{BuildResult, BuildResultArtifact, BuildTask},
    flams_extension,
    formats::{CHECK, PDF},
    source_format,
};
use flams_utils::vecmap::VecSet;
pub use rustex::{OutputCont, RusTeX};

use crate::dependencies::STeXDependency;

source_format!(stex ["tex","ltx"] [
  PDFLATEX_FIRST => PDFLATEX => RUSTEX => FTML_OMDOC => CHECK]
   @ "(Semantically annotated) LaTeX"
   = dependencies::get_deps
);

build_target!(
  pdflatex_first [] => [AUX]
  @ "Run pdflatex and bibtex/biber/index once"
  = pdflatex_first
);

fn pdflatex_first(backend: &AnyBackend, task: &BuildTask) -> BuildResult {
    let Either::Left(path) = task.source() else {
        return BuildResult {
            log: Either::Left("Needs a physical file".to_string()),
            result: Err(Vec::new()),
        };
    };
    latex::clean(path);
    let log = path.with_extension("log");
    let mh = backend
        .mathhubs()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let ret = latex::pdflatex_and_bib(path, [("STEX_WRITESMS", "true"), ("MATHHUB", &mh)]);
    if ret.is_ok() {
        BuildResult {
            log: Either::Right(log),
            result: Ok(BuildResultArtifact::File(PDF, path.with_extension("pdf"))),
        }
    } else {
        BuildResult {
            log: Either::Right(log),
            result: Err(Vec::new()),
        }
    }
}

build_target!(
  pdflatex [AUX] => [PDF]
  @ "Run pdflatex a second time"
  = pdflatex_second
);

fn pdflatex_second(backend: &AnyBackend, task: &BuildTask) -> BuildResult {
    let Either::Left(path) = task.source() else {
        return BuildResult {
            log: Either::Left("Needs a physical file".to_string()),
            result: Err(Vec::new()),
        };
    };
    let log = path.with_extension("log");
    let mh = backend
        .mathhubs()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let ret = latex::pdflatex(path, [("STEX_USESMS", "true"), ("MATHHUB", &mh)]);
    if ret.is_ok() {
        BuildResult {
            log: Either::Right(log),
            result: Ok(BuildResultArtifact::File(PDF, path.with_extension("pdf"))),
        }
    } else {
        BuildResult {
            log: Either::Right(log),
            result: Err(Vec::new()),
        }
    }
}

build_target!(
  rustex [AUX] => [FTML_DOC]
  @ "Run RusTeX tex->html only"
  = rustex
);

fn rustex(backend: &AnyBackend, task: &BuildTask) -> BuildResult {
    // TODO make work with string as well
    let Either::Left(path) = task.source() else {
        return BuildResult {
            log: Either::Left("Needs a physical file".to_string()),
            result: Err(Vec::new()),
        };
    };
    let out = path.with_extension("rlog");
    let ocl = out.clone();
    let mh = backend
        .mathhubs()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let run = move || {
        RusTeX::get()
            .map_err(|()| "Could not initialize RusTeX".to_string())
            .and_then(|e| {
                std::panic::catch_unwind(move || {
                    e.run_with_envs(
                        path,
                        false,
                        [
                            ("STEX_USESMS".to_string(), "true".to_string()),
                            ("MATHHUB".to_string(), mh),
                        ],
                        Some(&ocl),
                    )
                })
                .map_err(|e| {
                    if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Ok(s) = e.downcast::<String>() {
                        *s
                    } else {
                        "Unknown RusTeX error".to_string()
                    }
                })
            })
    };
    #[cfg(debug_assertions)]
    let ret = {
        std::thread::scope(move |s| {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(s, run)
                .expect("foo")
                .join()
                .expect("foo")
        })
    };
    #[cfg(not(debug_assertions))]
    let ret = { run() };
    match ret {
        Err(s) => BuildResult {
            log: Either::Left(s),
            result: Err(Vec::new()),
        },
        Ok(Err(_)) => BuildResult {
            log: Either::Right(out),
            result: Err(Vec::new()),
        },
        Ok(Ok(s)) => {
            latex::clean(path);
            BuildResult {
                log: Either::Right(out),
                result: Ok(HTMLString::create(s)),
            }
        }
    }
}

build_result!(aux @ "LaTeX aux/bbl/toc files, as generated by pdflatex+bibtex/biber/mkindex");

flams_extension!(stex_ext = RusTeX::initialize);

lazy_static::lazy_static! {
    static ref OPTIONS : regex::Regex = unsafe{ regex::Regex::new(
        r"\\(?<cmd>documentclass|usepackage|RequirePackage)(?<opts>\[[^\]]*\])?\{(?<name>notesslides|stex|hwexam|problem)\}"
    ).unwrap_unchecked() };
    static ref LIBS: regex::Regex = unsafe{ regex::Regex::new(
        r"\\libinput\{"
    ).unwrap_unchecked()};
}

macro_rules! err {
    ($fmt:expr) => {return Err(eyre::eyre!($fmt))};
    ($fmt:expr, $($args:tt)*) => {return Err(eyre::eyre!($fmt,$($args)*))};
    ($e:expr => $fmt:expr) => { $e.wrap_err($fmt)?};
    ($e:expr => $fmt:expr, $($args:tt)*) => { $e.wrap_err_with(|| format!($fmt,$($args)*))?};
}

pub fn export_standalone(doc: &DocumentURI, file: &Path, target_dir: &Path) -> eyre::Result<()> {
    use std::fmt::Write;
    if !file.extension().is_some_and(|e| e == "tex") {
        err!("Not a .tex file: {}", file.display());
    }

    // safe, because we earlier checked that it has extension .tex => it has a file name
    let file_name = unsafe { file.file_name().unwrap_unchecked() };

    let mh_path = target_dir.join("mathhub");
    err!(
        std::fs::create_dir_all(&mh_path) =>
        "Invalid target directory: {}",
        mh_path.display()
    );
    let archive = doc.archive_id();

    let mh = flams_system::settings::Settings::get()
        .mathhubs
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let Ok(()) = latex::pdflatex_and_bib(file, [("STEX_WRITESMS", "true"), ("MATHHUB", &mh)])
    else {
        err!(
            "failed to build {}\nCheck .log file for details",
            file.display()
        );
    };

    let sms = file.with_extension("sms");
    let sms_target = target_dir.join(file_name).with_extension("sms");
    err!(std::fs::copy(&sms, &sms_target) => "Failed to copy file {}",sms.display() );

    let orig_txt = err!(
        std::fs::read_to_string(file) =>
        "failed to open file {}",
        file.display()
    );
    let Some(begin) = orig_txt.find("\\begin{document}") else {
        err!("No \\begin{{document}} found!")
    };
    let mut txt = orig_txt[..begin].to_string();
    //orig_txt.truncate(begin);
    let rel_path = if let Some(p) = doc.path() {
        format!("{p}/{}", file_name.display())
    } else {
        file_name.display().to_string()
    };
    err!(
        write!(
            txt,
            "\n\\begin{{document}}\n  \\inputref[{archive}]{{{rel_path}}}\n\\end{{document}}"
        ) =>
        "impossible",
    );

    let mut matched = false;
    let txt = OPTIONS.replace(&txt, |cap: &regex::Captures<'_>| {
        matched = true;
        // This is safe, because the named groups are necessary components of the regex, so a match
        // entails they are defined.
        let (cmd, name) = unsafe {
            (
                cap.name("cmd").unwrap_unchecked().as_str(),
                cap.name("name").unwrap_unchecked().as_str(),
            )
        };
        if let Some(opts) = cap.name("opts") {
            format!(
                "\\{cmd}[{},mathhub=./mathhub,usesms]{{{name}}}",
                &opts.as_str()[1..opts.as_str().len() - 1]
            )
        } else {
            format!("\\{cmd}[mathhub=./mathhub,usesms]{{{name}}}")
        }
    });
    if !matched {
        err!(
            "No sTeX \\documentclass or \\package found in {}",
            file.display()
        );
    }
    let rep = format!("\\libinput[{archive}]{{");
    let txt = LIBS.replace_all(&txt, &rep);

    let tex_target = target_dir.join(file_name);
    err!(std::fs::write(&tex_target, txt.as_bytes()) => "Failed to write to file {}",tex_target.display());

    copy("stex.sty", &target_dir)?;
    copy("stex-logo.sty", &target_dir)?;
    copy("stex-backend-pdflatex.cfg", &target_dir)?;
    copy("stex-highlighting.sty", &target_dir)?;
    copy("stexthm.sty", &target_dir)?;
    // stex-compat?

    let mut todos = vec![(orig_txt, file.to_owned(), doc.clone())];
    let mut archives = VecSet(Vec::with_capacity(4));
    while let Some((txt, f, d)) = todos.pop() {
        if !archives.0.contains(d.archive_id()) {
            archives.0.push(d.archive_id().clone());
            do_archive(d.archive_id(), &mh_path)?;
        }
        // by construction, the files in todos have a file name
        let name = unsafe { f.file_name().unwrap_unchecked() };
        let target_file = if let Some(p) = d.path() {
            mh_path
                .join(d.archive_id().to_string())
                .join("source")
                .join(p.to_string()) //.join(name)
        } else {
            mh_path.join(d.archive_id().to_string()).join("source") //.join(name)
        };
        err!(std::fs::create_dir_all(&target_file) => "Failed to create directory {}",target_file.display());
        let target_file = target_file.join(name);
        err!(std::fs::copy(&f, target_file) => "Failed to copy file {}",f.display());
        for dep in dependencies::parse_deps(&txt, &f, &d, &GlobalBackend::get().to_any()) {
            match dep {
                STeXDependency::Inputref { archive, filepath } => {
                    let archive = archive.as_ref().unwrap_or(d.archive_id());
                    let Some((d, f)) = GlobalBackend::get().with_local_archive(archive, |a| {
                        a.and_then(|a| {
                            let f = a.path().join("source").join(&*filepath);
                            let d = DocumentURI::from_archive_relpath(a.uri().owned(), &*filepath)
                                .ok()?;
                            Some((d, f))
                        })
                    }) else {
                        err!("Could not find document for file {}", f.display())
                    };
                    let txt = err!(
                        std::fs::read_to_string(&f) =>
                        "failed to open file {}",
                        f.display()
                    );
                    todos.push((txt, f, d));
                }
                STeXDependency::Img { archive, filepath } => {
                    let archive = archive.as_ref().unwrap_or(d.archive_id());
                    let Some(source) = GlobalBackend::get().with_local_archive(archive, |a| {
                        a.map(|a| a.path().join("source").join(&*filepath))
                    }) else {
                        err!("Could not find image file {}", f.display())
                    };
                    let img_target = mh_path
                        .join(archive.to_string())
                        .join("source")
                        .join(&*filepath);
                    if !source.exists() {
                        err!("img file not found: {}", source.display())
                    }
                    // safe, because file exists and is not root
                    let parent = unsafe { img_target.parent().unwrap_unchecked() };
                    err!(std::fs::create_dir_all(&parent) => "Error creating directory {}",parent.display());
                    err!(std::fs::copy(&source,&img_target) => "Error copying {}",img_target.display());
                }
                STeXDependency::ImportModule { .. }
                | STeXDependency::UseModule { .. }
                | STeXDependency::Module { .. } => (),
            }
        }
    }

    Ok(())
}

fn copy(name: &str, to: &Path) -> eyre::Result<()> {
    let Some(sty) = tex_engine::engine::filesystem::kpathsea::KPATHSEA.which(name) else {
        err!("No {name} found")
    };
    let sty_target = to.join(name);
    err!(std::fs::copy(sty, sty_target) =>"Failed to copy {name}");
    Ok(())
}

fn do_archive(id: &ArchiveId, target: &Path) -> eyre::Result<()> {
    GlobalBackend::get().manager().with_tree(|t| {
        let mut steps = id.steps();
        let Some(mut current) = steps.next() else {
            err!("empty archive ID");
        };
        let mut ls = &t.groups;
        loop {
            let Some(a) = ls.iter().find(|a| a.id().last_name() == current) else {
                err!("archive not found: {id}");
            };
            match a {
                ArchiveOrGroup::Archive(_) => {
                    if steps.next().is_some() {
                        err!("archive not found: {id}");
                    }
                    let Some(Archive::Local(a)) = t.get(id) else {
                        err!("Not a local archive: {id}")
                    };
                    return do_manifest(a, target);
                }
                ArchiveOrGroup::Group(g) => {
                    let Some(next) = steps.next() else {
                        err!("archive not found: {id}");
                    };
                    current = next;
                    ls = &g.children;
                    if let Some(ArchiveOrGroup::Archive(a)) =
                        g.children.iter().find(|a| a.id().is_meta())
                    {
                        let Some(Archive::Local(a)) = t.get(a) else {
                            err!("archive not found: {a}");
                        };
                        do_manifest(a, target)?;
                    }
                }
            }
        }
    })
}

fn do_manifest(a: &LocalArchive, target: &Path) -> eyre::Result<()> {
    let archive_target = target.join(a.id().to_string());
    let manifest_target = archive_target.join("META-INF/MANIFEST.MF");
    if manifest_target.exists() {
        return Ok(());
    }
    let manifest_source = a.path().join("META-INF/MANIFEST.MF");
    if !manifest_source.exists() {
        err!(
            "MANIFEST.MF of {} not found (at {})",
            a.id(),
            manifest_source.display()
        );
    }
    // safe, because by construction, file has a parent
    let meta_inf = unsafe { manifest_target.parent().unwrap_unchecked() };
    err!(std::fs::create_dir_all(&meta_inf) => "Failed to create directory {}",meta_inf.display());
    err!(std::fs::copy(&manifest_source, &manifest_target) => "failed to copy {} to {}",manifest_source.display(),manifest_target.display());

    let lib_source = a.path().join("lib");
    if lib_source.exists() {
        let lib_target = archive_target.join("lib");
        flams_utils::fs::copy_dir_all(&lib_source, &lib_target)?;
    }
    Ok(())
}
/*
#[cfg(test)]
#[rstest::rstest]
fn standalone_test() {
    tracing_subscriber::fmt().init();
    flams_system::settings::Settings::initialize(flams_system::settings::SettingsSpec::default());
    flams_system::backend::GlobalBackend::initialize();

    let target_dir = Path::new("/home/jazzpirate/temp/test");
    let doc = "https://mathhub.info?a=Papers/24-cicm-views-in-alea&d=paper&l=en"
        .parse()
        .unwrap();
    let file =
        Path::new("/home/jazzpirate/work/MathHub/Papers/24-cicm-views-in-alea/source/paper.tex");
    export_standalone(&doc, &file, target_dir).unwrap()
}
 */

/*
#[cfg(test)]
#[rstest::rstest]
fn test() {
    fn print<T>() {
        tracing::info!(
            "Size of {}:{}",
            std::any::type_name::<T>(),
            std::mem::size_of::<T>()
        )
    }
    tracing_subscriber::fmt().init();
    print::<ArchiveId>();
    print::<flams_ontology::uris::BaseURI>();
    print::<flams_ontology::uris::ArchiveURI>();
    print::<flams_ontology::uris::PathURI>();
    print::<flams_ontology::uris::ModuleURI>();
    print::<flams_ontology::uris::DocumentURI>();
    print::<flams_ontology::uris::SymbolURI>();
    print::<flams_ontology::uris::DocumentElementURI>();
}
 */
