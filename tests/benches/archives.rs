use std::path::Path;
use std::time::Duration;
use criterion::{black_box, Criterion, criterion_group, criterion_main};
use immt_api::backend::manager::{ArchiveIterator, ArchiveLoaderAsync};
use immt_api::building::targets::BuildFormat;
use immt_api::core::formats::FormatId;



pub fn iterator(c: &mut Criterion) {
    let fmt = [BuildFormat::new(FormatId::new("stex"),&["tex","ltx"])];
    c.bench_function("iterator", |b| b.iter(|| {
        let iter = ArchiveIterator::new(
            Path::new("/home/jazzpirate/work/MathHub"),
            &fmt
        );
        iter.count();
    }));
}

pub fn rayon(c:&mut Criterion) {
    use immt_api::par::*;
    let fmt = [BuildFormat::new(FormatId::new("stex"),&["tex","ltx"])];
    c.bench_function("par_iterator", |b| b.iter(|| {
        let iter = ArchiveIterator::new(
            Path::new("/home/jazzpirate/work/MathHub"),
            &fmt
        ).par_split().into_par_iter();
        iter.count();
    }));
}

pub fn multithreaded(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("multithreaded", |b| b.to_async(&rt).iter(|| async {
        ArchiveLoaderAsync::load(
            Path::new("/home/jazzpirate/work/MathHub"),
            &[BuildFormat::new(FormatId::new("stex"),&["tex","ltx"])],
            black_box(|a,_| true)
        ).await;
    }));
}

pub fn tokio_rayon(c: &mut Criterion) {
    use immt_api::par::*;
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("tokio_rayon", |b| b.to_async(&rt).iter(|| async {tokio_rayon::spawn(move || {
        let fmt = [BuildFormat::new(FormatId::new("stex"),&["tex","ltx"])];
        let iter = ArchiveIterator::new(
            Path::new("/home/jazzpirate/work/MathHub"),
            &fmt
        ).par_split().into_par_iter();
        iter.count()}).await;
    }));
}


criterion_group!(
    name = load_archives;
    config = Criterion::default().significance_level(0.01).measurement_time(Duration::from_secs(10));
    targets = iterator,rayon,multithreaded,tokio_rayon
);
criterion_main!(load_archives);