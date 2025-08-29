pub mod errors;
pub mod ignore_source;
pub mod lazy_file;
pub mod path_ext;

pub trait AsyncEngine: 'static {
    fn background(f: impl FnOnce() + Send + 'static);
    fn block_on<R: Send + Sync + 'static>(
        f: impl FnOnce() -> R + Send + Sync + 'static,
    ) -> impl std::future::Future<Output = R> + Send + Sync;
}

pub struct SyncEngine;

impl AsyncEngine for SyncEngine {
    #[inline]
    fn background(f: impl FnOnce() + Send + 'static) {
        let _ = std::thread::spawn(f);
    }
    fn block_on<R: Send + Sync>(
        f: impl FnOnce() -> R + Send + Sync,
    ) -> impl std::future::Future<Output = R> + Send + Sync {
        std::future::ready(f())
    }
}
