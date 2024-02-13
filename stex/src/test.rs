use std::fmt::Display;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tracing::info;
use immt_api::archives::{ArchiveData, ArchiveGroupBase, ArchiveGroupT, ArchiveId};
use immt_api::formats::FormatStore;
use immt_api::Str;
use immt_system::utils::{measure, measure_average};
use immt_system::utils::parse::{ParseReader, ParseStr};
use immt_api::utils::problems::ProblemHandler as PH;
use immt_system::backend::archives::ArchiveGroup;
use immt_system::utils::problems::ProblemHandler;
use immt_system::utils::sourcerefs::SourceOffsetLineCol;
use crate::{EXTENSIONS, ID, STeXExtension};
use crate::quickparse::latex::LaTeXToken;
use crate::quickparse::tokenizer::LetterGroup;

fn test_setup() -> ProblemHandler {
    use tracing_subscriber::layer::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
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
        .try_init();
    ProblemHandler::default()
}

#[inline(never)]
fn check_file_string_tokens<P:PH>(path:&Path,handler:&P) {
    let contents = std::fs::read_to_string(path).unwrap();
    let i = ParseStr::<SourceOffsetLineCol>::new(&contents);
    let tokenizer = crate::quickparse::tokenizer::TeXTokenizer::<_,_,LetterGroup>::new(i,Some(path),handler);
    let v = tokenizer.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_string_parsed<P:PH>(path:&Path,handler:&P) {
    let contents = std::fs::read_to_string(path).unwrap();
    let i = ParseStr::<SourceOffsetLineCol>::new(&contents);
    let parser = crate::quickparse::latex::LaTeXParser::<_,_,LaTeXToken<_,_>>::new(i,Some(path),handler);
    let v = parser.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_tokens<P:PH>(path:&Path,handler:&P) {
    let f = std::fs::File::open(path).unwrap();
    let i = ParseReader::<_,SourceOffsetLineCol>::new(BufReader::new(f));
    let tokenizer = crate::quickparse::tokenizer::TeXTokenizer::<_,_,LetterGroup>::new(i,Some(path),handler);
    let v = tokenizer.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[inline(never)]
fn check_file_parsed<P:PH>(path:&Path,handler:&P) {
    let f = std::fs::File::open(path).unwrap();
    let i = ParseReader::<_,SourceOffsetLineCol>::new(BufReader::new(f));
    let parser = crate::quickparse::latex::LaTeXParser::<_,_,LaTeXToken<_,_>>::new(i,Some(path),handler);
    let v = parser.collect::<Vec<_>>();
    //println!("Done: {}",v.len())
}

#[test]
fn check_metathy() {
    let handler = test_setup();
    check_file_string_tokens(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"),&handler);
    check_file_string_parsed(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"),&handler);
    check_file_tokens(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"),&handler);
    check_file_parsed(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"),&handler);
}
#[test]
fn check_latex_ltx() {
    let _handler = test_setup();
    measure_average("latex.ltx string",50,|| {
        check_file_string_tokens(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"),&());
        check_file_string_parsed(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"),&());
    });
    measure_average("latex.ltx file",50,|| {
        check_file_tokens(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"),&());
        check_file_parsed(Path::new("/home/jazzpirate/TeXLive/texmf-dist/tex/latex/base/latex.ltx"),&());
    })
}

fn get_mh<P:PH>(handler:&P) -> ArchiveGroup {
    let mut top = ArchiveGroup::new(ArchiveId::new(Str::from("")));
    let path = Path::new("/home/jazzpirate/work/MathHub");
    let mut store = FormatStore::default();
    store.register(immt_api::formats::Format::new(ID,EXTENSIONS,Box::new(STeXExtension)));
    top.base_mut().archives = ArchiveGroupT::load_dir(path,&store,handler).into();
    top
}

#[derive(Clone,Default)]
struct Counter {
    inner:Arc<AtomicUsize>//::new(AtomicUsize::new(0))
}
impl PH for Counter {
    fn add(&self, kind: &'static str, message: impl Display) {
        self.inner.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
    }
}


#[test]
fn check_mh() {
    use rayon::iter::ParallelIterator;
    let handler = test_setup();
    let mh = get_mh(&handler);

    let ph = Counter::default();
    let mut counter = 0;
    measure("str_sync",|| {
        mh.archives().for_each(|a| {
            let p = a.path();
            a.iter_sources((),|f,_| {
                counter += 1;
                let p = f.path_in_archive(p);
                check_file_string_parsed(&p,&ph);
            })
        });
    });
    info!("Checked {} files ({} problems)",counter,ph.inner.load(std::sync::atomic::Ordering::Relaxed));

    let ph = Counter::default();
    let mut counter = 0;
    measure("file_sync",|| {
        mh.archives().for_each(|a| {
            let p = a.path();
            a.iter_sources((),|f,_| {
                counter += 1;
                let p = f.path_in_archive(p);
                check_file_parsed(&p,&ph);
            })
        });
    });
    info!("Checked {} files ({} problems)",counter,ph.inner.load(std::sync::atomic::Ordering::Relaxed));

    let ph = Counter::default();
    let counter = Arc::new(AtomicUsize::new(0));
    measure("str",|| {
        mh.archives_par().for_each(|a| {
            let p = a.path();
            a.iter_sources_par(|iter| {
                iter.for_each(|s| {
                    counter.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
                    let p = s.path_in_archive(p);
                    check_file_string_parsed(&p,&ph);
                })
            })
        });
    });
    info!("Checked {} files ({} problems)",counter.load(std::sync::atomic::Ordering::Relaxed),ph.inner.load(std::sync::atomic::Ordering::Relaxed));

    let ph = Counter::default();
    let counter = Arc::new(AtomicUsize::new(0));
    measure("file",|| {
        mh.archives_par().for_each(|a| {
            let p = a.path();
            a.iter_sources_par(|iter| {
                iter.for_each(|s| {
                    counter.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
                    let p = s.path_in_archive(p);
                    check_file_parsed(&p,&ph);
                })
            })
        });
    });
    info!("Checked {} files ({} problems)",counter.load(std::sync::atomic::Ordering::Relaxed),ph.inner.load(std::sync::atomic::Ordering::Relaxed));
}