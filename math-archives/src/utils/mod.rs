pub mod errors;
pub mod ignore_source;
pub mod lazy_file;
pub mod path_ext;

pub trait AsyncEngine: 'static {
    fn background(f: impl FnOnce() + Send + 'static);
    fn block_on<R: Send + Sync + 'static>(
        f: impl FnOnce() -> R + Send + Sync + 'static,
    ) -> impl std::future::Future<Output = R> + Send + Sync;
    fn exec_after(delay: std::time::Duration, f: impl FnOnce() + Send + 'static);
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
    fn exec_after(delay: std::time::Duration, f: impl FnOnce() + Send + 'static) {
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            f();
        });
    }
}

pub struct AllSyncEngine;
impl AsyncEngine for AllSyncEngine {
    #[inline]
    fn background(f: impl FnOnce() + Send + 'static) {
        f();
    }
    fn block_on<R: Send + Sync>(
        f: impl FnOnce() -> R + Send + Sync,
    ) -> impl std::future::Future<Output = R> + Send + Sync {
        std::future::ready(f())
    }
    fn exec_after(delay: std::time::Duration, f: impl FnOnce() + Send + 'static) {
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            f();
        });
    }
}
