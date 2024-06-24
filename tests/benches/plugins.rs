use std::path::Path;
use std::time::Duration;
use criterion::{black_box, Criterion, criterion_group, criterion_main};
use libloading::{Library, Symbol};
use immt_api::extensions::{ExtensionDeclaration, MMTExtension};

struct Controller {
    triples: Vec<immt_api::core::ontology::rdf::terms::NamedNode>
}
impl immt_api::controller::Controller for Controller {
    fn test(&mut self) -> &mut Vec<immt_api::core::ontology::rdf::terms::NamedNode> {
        &mut self.triples
    }
}
/*
pub fn direct(c: &mut Criterion) {
    let plugin = test_plugin::TestPlugin::new();
    c.bench_function("direct", |b| b.iter(|| plugin.test(black_box(&mut Controller { triples: Vec::new()}))));
}

pub fn ffi(c:&mut Criterion) {
    let libfile = Path::new(immt_tests::MAIN_DIR).join("target/x86_64-unknown-linux-gnu/release/libtest_plugin.so");
    let lib = unsafe{ Library::new(libfile).unwrap() };

    let registry : Symbol<*mut ExtensionDeclaration> = unsafe{lib.get(b"extension_declaration").unwrap()};
    let plugin = unsafe{(registry.as_ref().unwrap().register)()};
    c.bench_function("ffi", |b| b.iter(|| plugin.test(black_box(&mut Controller { triples: Vec::new()}))));
}

pub fn direct2(c: &mut Criterion) {
    let plugin = test_plugin::TestPlugin::new();
    c.bench_function("rustex direct", |b| b.iter(|| plugin.test2(black_box(&mut Controller { triples: Vec::new()}))));
}

pub fn ffi2(c:&mut Criterion) {
    let libfile = Path::new(immt_tests::MAIN_DIR).join("target/x86_64-unknown-linux-gnu/release/libtest_plugin.so");
    let lib = unsafe{ Library::new(libfile).unwrap() };

    let registry : Symbol<*mut ExtensionDeclaration> = unsafe{lib.get(b"extension_declaration").unwrap()};
    let plugin = unsafe{(registry.as_ref().unwrap().register)()};
    c.bench_function("rustex ffi", |b| b.iter(|| plugin.test2(black_box(&mut Controller { triples: Vec::new()}))));
}

criterion_group!(
    name = plugins;
    config = Criterion::default().significance_level(0.01).measurement_time(Duration::from_secs(10));
    targets = direct,ffi
);
criterion_group!(
    name = rustex;
    config = Criterion::default().significance_level(0.01).sample_size(10);
    targets = direct2,ffi2
);
criterion_main!(plugins,rustex);

*/