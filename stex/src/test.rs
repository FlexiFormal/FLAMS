use std::fmt::Display;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::Layer;
use immt_api::archives::{ArchiveData, ArchiveGroupBase, ArchiveGroupT, ArchiveId};
use immt_api::formats::FormatStore;
use immt_system::utils::{measure, measure_average};
use immt_system::utils::parse::{ParseReader, ParseStr};
use immt_system::backend::archives::ArchiveGroup;
use immt_system::utils::sourcerefs::SourceOffsetLineCol;
use crate::{EXTENSIONS, ID, STeXExtension};
use crate::quickparse::latex::LaTeXToken;

fn test_setup(filter:bool) {
    use tracing_subscriber::layer::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let fmt = tracing_subscriber::fmt::Layer::default()
        .compact()
        .with_ansi(true)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        //.with_max_level(tracing::Level::INFO)
        //.with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_filter(LevelFilter::INFO)
        //.with_filter(Targets::new().with_target("tex-linter",LevelFilter::OFF))
        ;
    if filter {
        let _ = tracing_subscriber::Registry::default()
            .with(
                fmt.with_filter(Targets::new().with_default(LevelFilter::INFO).with_target("source_file",LevelFilter::OFF))
            )
            .try_init();
    } else {
        let _ = tracing_subscriber::Registry::default().with(fmt).try_init();
    }
/*
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        //.pretty()
        .with_ansi(true)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with(Targets::new().with_target("tex-linter",LevelFilter::OFF))
        .try_init();

 */
}

#[inline(never)]
fn check_file_string_tokens(path:&Path) {
    let contents = std::fs::read_to_string(path).unwrap();
    let i = ParseStr::<SourceOffsetLineCol>::new(&contents);
    let tokenizer = crate::quickparse::tokenizer::TeXTokenizer::new(i,Some(path));
    let v = tokenizer.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_string_parsed(path:&Path) {
    let contents = std::fs::read_to_string(path).unwrap();
    let i = ParseStr::<SourceOffsetLineCol>::new(&contents);
    let parser = crate::quickparse::latex::LaTeXParser::<_,LaTeXToken<_,_>>::new(i,Some(path));
    let v = parser.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_tokens(path:&Path) {
    let f = std::fs::File::open(path).unwrap();
    let i = ParseReader::<_,SourceOffsetLineCol>::new(BufReader::new(f));
    let tokenizer = crate::quickparse::tokenizer::TeXTokenizer::new(i,Some(path));
    let v = tokenizer.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_parsed(path:&Path) {
    let f = std::fs::File::open(path).unwrap();
    let i = ParseReader::<_,SourceOffsetLineCol>::new(BufReader::new(f));
    let parser = crate::quickparse::latex::LaTeXParser::<_,LaTeXToken<_,_>>::new(i,Some(path));
    let v = parser.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[test]
fn check_metathy() {
    test_setup(false);
    check_file_string_tokens(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"));
    check_file_string_parsed(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"));
    check_file_tokens(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"));
    check_file_parsed(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"));
}
#[test]
fn check_latex_ltx() {
    test_setup(false);
    measure_average("latex.ltx string",50,|| {
        check_file_string_tokens(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"));
        check_file_string_parsed(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"));
    });
    measure_average("latex.ltx file",50,|| {
        check_file_tokens(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"));
        check_file_parsed(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"));
    })
}

fn get_mh() -> ArchiveGroup {
    let mut top = ArchiveGroup::new(ArchiveId::new(""));
    let path = Path::new("/home/jazzpirate/work/MathHub");
    let mut store = FormatStore::default();
    store.register(immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(STeXExtension)));
    top.base_mut().archives = ArchiveGroupT::load_dir(path,&store).into();
    top
}

#[test]
fn check_mh() {
    use rayon::iter::ParallelIterator;
    test_setup(true);
    let mh = get_mh();

    let mut counter = 0;
    measure_average("str_sync",10,|| {
        mh.archives().for_each(|a| {
            let p = a.path();
            a.iter_sources((),|f,_| {
                counter += 1;
                let p = f.path_in_archive(p);
                check_file_string_parsed(&p);
            })
        });
    });
    info!("Checked {} files",counter);

    let mut counter = 0;
    measure_average("file_sync",10,|| {
        mh.archives().for_each(|a| {
            let p = a.path();
            a.iter_sources((),|f,_| {
                counter += 1;
                let p = f.path_in_archive(p);
                check_file_parsed(&p);
            })
        });
    });
    info!("Checked {} files",counter);

    let counter = Arc::new(AtomicUsize::new(0));
    measure_average("str",10,|| {
        mh.archives_par().for_each(|a| {
            let p = a.path();
            a.iter_sources_par(|iter| {
                iter.for_each(|s| {
                    counter.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
                    let p = s.path_in_archive(p);
                    check_file_string_parsed(&p);
                })
            })
        });
    });
    info!("Checked {} files",counter.load(std::sync::atomic::Ordering::Relaxed));

    let counter = Arc::new(AtomicUsize::new(0));
    measure_average("file",10,|| {
        mh.archives_par().for_each(|a| {
            let p = a.path();
            a.iter_sources_par(|iter| {
                iter.for_each(|s| {
                    counter.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
                    let p = s.path_in_archive(p);
                    check_file_parsed(&p);
                })
            })
        });
    });
    info!("Checked {} files",counter.load(std::sync::atomic::Ordering::Relaxed));
}