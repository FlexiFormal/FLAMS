use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use string_interner::{StringInterner,backend::StringBackend,symbol::SymbolU16};

lazy_static! {
    static ref RWL: parking_lot::RwLock<StringInterner<StringBackend<SymbolU16>>> = parking_lot::RwLock::new(StringInterner::new());
    static ref MUTEX: parking_lot::Mutex<StringInterner<StringBackend<SymbolU16>>> = parking_lot::Mutex::new(StringInterner::new());
    static ref IDS:[String;65530] = {
        let mut ids = array_init::array_init::<_,_,65530>(|_| String::new());
        for i in 0..65530 {
            ids[i] = i.to_string();
        }
        ids
    };
    static ref TABLE: symbol_table::SymbolTable = symbol_table::SymbolTable::new();
}

pub fn rwlock(c: &mut Criterion) {
    c.bench_function("rwlock", |b| b.iter(|| black_box({
        let mut keys = Vec::with_capacity(65530);
        let mut strings = Vec::with_capacity(65530);
        for s in IDS.iter() {
            keys.push(black_box(&RWL).write().get_or_intern(s));
        }
        for _ in 0..10 {
            for k in keys.iter().copied() {
                strings.push(black_box(&RWL).read().resolve(k).unwrap().to_string());
            }
            strings.clear();
        }
    })));
}
pub fn mutex(c: &mut Criterion) {
    c.bench_function("mutex", |b| b.iter(|| black_box({
        let mut keys = Vec::with_capacity(65530);
        let mut strings = Vec::with_capacity(65530);
        for s in IDS.iter() {
            keys.push(black_box(&MUTEX).lock().get_or_intern(s));
        }
        for _ in 0..10 {
            for k in keys.iter().copied() {
                strings.push(black_box(&MUTEX).lock().resolve(k).unwrap().to_string());
            }
            strings.clear();
        }
    })));
}

pub fn table(c: &mut Criterion) {
    c.bench_function("table", |b| b.iter(|| black_box({
        let mut keys = Vec::with_capacity(65530);
        let mut strings = Vec::with_capacity(65530);
        for s in IDS.iter() {
            keys.push(black_box(&TABLE).intern(s));
        }
        for _ in 0..10 {
            for k in keys.iter().copied() {
                strings.push(black_box(&TABLE).resolve(k).to_string());
            }
            strings.clear();
        }
    })));
}

pub fn arc(c: &mut Criterion) {
    c.bench_function("arc", |b| b.iter(|| black_box({
        let mut keys = Vec::with_capacity(65530);
        let mut strings = Vec::with_capacity(65530);
        for s in IDS.iter() {
            keys.push(std::sync::Arc::new(s.clone()));
        }
        for _ in 0..10 {
            for k in keys.iter().cloned() {
                strings.push(k);
            }
            strings.clear();
        }
    })));
}

pub fn triomphe(c: &mut Criterion) {
    c.bench_function("triomphe", |b| b.iter(|| black_box({
        let mut keys = Vec::with_capacity(65530);
        let mut strings = Vec::with_capacity(65530);
        for s in IDS.iter() {
            keys.push(triomphe::Arc::new(s.clone()));
        }
        for _ in 0..10 {
            for k in keys.iter().cloned() {
                strings.push(k);
            }
            strings.clear();
        }
    })));
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.01).measurement_time(Duration::from_secs(10));
    targets = rwlock,mutex,table,arc,triomphe
);
criterion_main!(benches);