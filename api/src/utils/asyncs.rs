use std::future::Future;

pub fn background<F:FnOnce() + Send + 'static>(f:F) {
    let f = in_span(|| f());
    rayon::spawn(f);
}

pub fn in_span<F:FnOnce() -> R,R>(f:F) -> impl FnOnce() -> R {
    let span = tracing::Span::current();
    move || {
        let _span = span.enter();
        f()
    }
}


#[derive(Debug)]
pub struct ChangeListener<T:Clone>{
    pub inner: async_broadcast::Receiver<T>,
}
impl<T:Clone> ChangeListener<T> {
    pub fn get(&mut self) -> Option<T> {
        match self.inner.try_recv() {
            Ok(t) => Some(t),
            Err(async_broadcast::TryRecvError::Empty | async_broadcast::TryRecvError::Closed) => None,
            Err(e) => {
                tracing::error!("Error in ChangeListener::get: {:?}",e);
                None
            }
        }
    }
}
impl<T:Clone> Clone for ChangeListener<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.new_receiver() }
    }
}
#[derive(Debug,Clone)]
pub struct ChangeSender<T:Clone>{
    inner: async_broadcast::Sender<T>,
    // keeps the channel open:
    recv: async_broadcast::InactiveReceiver<T>,
}
impl<T:Clone> ChangeSender<T> {
    pub fn new(cap:usize) -> Self {
        let (mut s,r) = async_broadcast::broadcast(cap);
        s.set_overflow(true);
        Self { inner: s,recv:r.deactivate() }
    }
    pub fn send(&self, msg:T) {
        match self.inner.try_broadcast(msg) {
            Ok(_) | Err(async_broadcast::TrySendError::Inactive(_)) => (),
            _ => todo!()
        }
    }
    
    #[inline(always)]
    pub fn lazy_send<F:FnOnce() -> T>(&self,msg:F) {
        if self.inner.receiver_count() > 0 {
            self.send(msg())
        }
    }
    pub fn listener(&self) -> ChangeListener<T> {
        ChangeListener { inner: self.inner.new_receiver() }
    }
}

//#[cfg(feature = "tokio")]
pub mod lock {
    use std::ops::Deref;

    pub struct Lock<T> {
        sync_lock: parking_lot::RwLock<T>,
        //async_lock: tokio::sync::RwLock<()>,
    }
    impl<T> Lock<T> {
        pub fn new(t:T) -> Self {
            Self { sync_lock: parking_lot::RwLock::new(t) }//, async_lock: tokio::sync::RwLock::new(()) }
        }
        pub fn read<R,F:FnOnce(ReadLock<T>) -> R>(&self,f:F) -> R {
            let lock = self.sync_lock.read();
            f(ReadLock{lock,orig:&self.sync_lock})
        }
        pub fn write<R,F:FnOnce(&mut T) -> R>(&self,f:F) -> R {
            let mut lock = self.sync_lock.write();
            f(&mut lock)
        }
        /*
        pub async fn read_async<R,Fu:std::future::Future<Output = R>,F:FnOnce(AsyncReadLock<T>) -> Fu>(&self,f:F) -> R {
            let tlock = self.async_lock.read().await;
            let plock = self.sync_lock.read();
            f(AsyncReadLock{plock,tlock,porig:&self.sync_lock,torig:&self.async_lock}).await
        }
        pub async fn write_async<R,Fu:std::future::Future<Output = R>,F:FnOnce(&mut T) -> Fu>(&self,f:F) -> R {
            let tlock = self.async_lock.write().await;
            let mut plock = self.sync_lock.write();
            let r = f(&mut plock).await;
            drop(plock);drop(tlock);
            r
        }
        
         */
    }
    pub struct ReadLock<'a,T>{
        lock:parking_lot::RwLockReadGuard<'a,T>,
        orig: &'a parking_lot::RwLock<T>,
    }
    impl<'a,T> ReadLock<'a,T> {
        pub fn write_temporarily<R,F:FnOnce(&mut T) -> R>(self,f:F) -> (Self,R) {
            drop(self.lock);
            let mut lock = self.orig.write();
            let r = f(&mut lock);
            drop(lock);
            let self_ = self.orig.read();
            (ReadLock{lock:self_,orig:self.orig},r)
        }
        pub fn write<R,F:FnOnce(&mut T) -> R>(self,f:F) -> R {
            drop(self.lock);
            let mut lock = self.orig.write();
            f(&mut lock)
        }
    }
/*
    pub struct AsyncReadLock<'a,T>{
        plock:parking_lot::RwLockReadGuard<'a,T>,
        tlock:tokio::sync::RwLockReadGuard<'a,()>,
        porig: &'a parking_lot::RwLock<T>,
        torig: &'a tokio::sync::RwLock<()>,
    }
    impl<'a,T> AsyncReadLock<'a, T> {
        pub async fn write_temporarily<R,Fu:std::future::Future<Output = R>,F:FnOnce(&mut T) -> Fu>(self,f:F) -> (Self,R) {
            drop(self.plock);
            drop(self.tlock);
            let tlock = self.torig.write().await;
            let mut plock = self.porig.write();
            let r = f(&mut plock).await;
            drop(plock);drop(tlock);
            let tlock = self.torig.read().await;
            let plock = self.porig.read();
            (AsyncReadLock {plock,tlock,porig:self.porig,torig:self.torig}, r)
        }
        pub async fn write<R,Fu:std::future::Future<Output = R>,F:FnOnce(&mut T) -> Fu>(self,f:F) -> R {
            drop(self.plock);
            drop(self.tlock);
            let tlock = self.torig.write().await;
            let mut plock = self.porig.write();
            let r = f(&mut plock).await;
            drop(plock);drop(tlock);
            r
        }
    }*/
    
    impl<T> Deref for ReadLock<'_,T> {
        type Target = T;
        fn deref(&self) -> &Self::Target { &self.lock }
    }
}