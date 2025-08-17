use std::path::{Path, PathBuf};

use crate::{Archive, utils::path_ext::PathExt};

pub static MATHHUBS: std::sync::LazyLock<Box<[Box<Path>]>> = std::sync::LazyLock::new(|| {
    if let Ok(f) = std::env::var("MATHHUB") {
        return f
            .split(',')
            .map(|s| PathBuf::from(s.trim()).into_boxed_path())
            .collect();
    }
    if let Some(d) = simple_home_dir::home_dir() {
        let p = d.join(".mathhub").join("mathhub.path");
        if let Ok(f) = std::fs::read_to_string(p) {
            return f
                .split('\n')
                .map(|s| PathBuf::from(s.trim()).into_boxed_path())
                .collect();
        }
        return Box::new([d.join("MathHub").into_boxed_path()]);
    }
    panic!(
        "No MathHub directory found and default ~/MathHub not accessible!\n\
    Please set the MATHHUB environment variable or create a file ~/.mathhub/mathhub.path containing \
    the path to the MathHub directory."
    )
});

pub fn load_all_archives(external_url: &str) -> impl rayon::iter::ParallelIterator<Item = Archive> {
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
                crate::manifest::parse_manifest(&p, parent.relative_to(mh)?, external_url)
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
        targets: &[]
    });

    let _ = tracing_subscriber::fmt().try_init();
    let (i, t) = measure(|| load_all_archives("foo").count());
    tracing::info!("Loaded {i} archives in {t}");
    tracing::info!("Memory: {}", ftml_uris::get_memory_state());
}
