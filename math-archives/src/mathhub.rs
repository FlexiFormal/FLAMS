use crate::{Archive, utils::path_ext::PathExt};
use std::path::{Path, PathBuf};

#[allow(clippy::doc_markdown)]
/// The default MathHub directories on the user's file system; determined from environment
/// variables, or the path stated in `~/.mathhub/mathhub.path`, or `~/MathHub`.
///
/// # Panics
/// If it fails to do any of those
#[must_use]
pub fn default_mathhubs() -> Vec<PathBuf> {
    if let Ok(f) = std::env::var("MATHHUB") {
        return f.split(',').map(|s| PathBuf::from(s.trim())).collect();
    }
    if let Some(d) = simple_home_dir::home_dir() {
        let p = d.join(".mathhub").join("mathhub.path");
        if let Ok(f) = std::fs::read_to_string(p) {
            return f
                .split('\n')
                .map(|s: &str| PathBuf::from(s.trim()))
                .collect();
        }
        return vec![d.join("MathHub")];
    }
    panic!(
        "No MathHub directory found and default ~/MathHub not accessible!\n\
Please set the MATHHUB environment variable or create a file ~/.mathhub/mathhub.path containing \
the path to the MathHub directory."
    )
}

static MH: std::sync::OnceLock<&'static [&'static Path]> = std::sync::OnceLock::new();

/// The mathhub directories used by this run. Static, initilized as [`default_mathhubs`]
/// on first access. Can be set *before* any call using [`set_mathhubs`].
pub fn mathhubs() -> &'static [&'static Path] {
    MH.get_or_init(|| {
        &*Box::leak(
            default_mathhubs()
                .into_iter()
                .map(|p| &*Box::leak(p.into_boxed_path()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    })
}

/// Sets the mathhub directories used by tis run. May only be used *before* any call
/// to `mathhubs`.
///
/// # Errors
/// If already set
#[allow(clippy::result_unit_err)]
pub fn set_mathhubs(paths: impl IntoIterator<Item = PathBuf>) -> Result<(), ()> {
    if MH.get().is_some() {
        return Err(());
    }
    MH.get_or_init(|| {
        &*Box::leak(
            paths
                .into_iter()
                .map(|p| &*Box::leak(p.into_boxed_path()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    });
    Ok(())
}

pub static MATHHUBS: std::sync::LazyLock<&'static [&'static Path]> = std::sync::LazyLock::new(
    || {
        if let Ok(f) = std::env::var("MATHHUB") {
            return Box::leak(
                f.split(',')
                    .map(|s| &*PathBuf::from(s.trim()).leak())
                    .collect::<Box<[_]>>(),
            );
        }
        if let Some(d) = simple_home_dir::home_dir() {
            let p = d.join(".mathhub").join("mathhub.path");
            if let Ok(f) = std::fs::read_to_string(p) {
                return Box::leak(
                    f.split('\n')
                        .map(|s| &*PathBuf::from(s.trim()).leak())
                        .collect::<Box<[_]>>(),
                );
            }
            return Box::leak(Box::new([&*d.join("MathHub").leak()]));
        }
        panic!(
            "No MathHub directory found and default ~/MathHub not accessible!\n\
    Please set the MATHHUB environment variable or create a file ~/.mathhub/mathhub.path containing \
    the path to the MathHub directory."
        )
    },
);

pub fn load_all_archives() -> impl rayon::iter::ParallelIterator<Item = Archive> {
    //impl orx_parallel::ParIter<Item = Archive> {
    use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
    use spliter::ParallelSpliterator;
    crate::mathhub::MATHHUBS.par_iter().flat_map(|p| {
        crate::archive_iter::ManifestIterator::new(p)
            .par_split()
            .into_par_iter()
            .map(move |r| (p, r))
            .filter_map(|(mh, p)| {
                // SAFETY: manifest file is grandchild of root directory of archive
                let parent = unsafe { p.parent().unwrap_unchecked().parent().unwrap_unchecked() };

                let rel_path = parent.relative_to(mh)?;
                match crate::manifest::parse_manifest(&p, rel_path) {
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
    /*
    //use orx_parallel::IterIntoParIter;
    crate::mathhub::MATHHUBS
        .iter()
        .flat_map(|p| crate::archive_iter::ManifestIterator::new(p).map(move |r| (p, r)))
        .iter_into_par()
        .filter_map(|(mh, p)| {
            // SAFETY: manifest file is grandson of root directory of archive
            let parent = unsafe { p.parent().unwrap_unchecked().parent().unwrap_unchecked() };
            let Ok(diff) = parent.strip_prefix(mh) else {
                return None;
            };
            crate::manifest::parse_manifest(&p, &diff.as_slash_str(), external_url)
        })
    */
}

#[test]
fn all_archives() {
    use crate::source_format;
    use ftml_ontology::utils::time::measure;
    use rayon::iter::*;
    source_format!(STEX {
        name: "stex",
        file_extensions: &["tex", "ltx"],
        description: "foo",
        dependencies: |_| Vec::new(),
        targets: &[]
    });

    let _ = tracing_subscriber::fmt().try_init();
    let (i, t) = measure(|| load_all_archives().count());
    tracing::info!("Loaded {i} archives in {t}");
    tracing::info!("Memory: {}", ftml_uris::get_memory_state());
}
