#[derive(Debug)]
pub struct ChangeListener<T: Clone> {
    pub inner: async_broadcast::Receiver<T>,
}
impl<T: Clone + Send + Sync> ChangeListener<T> {
    pub fn get(&mut self) -> Option<T> {
        match self.inner.try_recv() {
            Ok(t) => Some(t),
            Err(async_broadcast::TryRecvError::Empty | async_broadcast::TryRecvError::Closed) => {
                None
            }
            Err(e) => {
                tracing::error!("Error in ChangeListener::get: {:?}", e);
                None
            }
        }
    }
    pub async fn read(&mut self) -> Option<T> {
        self.inner.recv_direct().await.ok()
    }
}
impl<T: Clone> Clone for ChangeListener<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.new_receiver(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangeSender<T: Clone> {
    inner: async_broadcast::Sender<T>,
    // keeps the channel open:
    _recv: async_broadcast::InactiveReceiver<T>,
}
impl<T: Clone> ChangeSender<T> {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        let (mut s, r) = async_broadcast::broadcast(cap);
        s.set_overflow(true);
        Self {
            inner: s,
            _recv: r.deactivate(),
        }
    }
    pub fn send(&self, msg: T) {
        match self.inner.try_broadcast(msg) {
            Ok(_) | Err(async_broadcast::TrySendError::Inactive(_)) => (),
            _ => todo!(),
        }
    }

    #[inline]
    pub fn lazy_send<F: FnOnce() -> T>(&self, msg: F) {
        if self.inner.receiver_count() > 0 {
            self.send(msg());
        }
    }
    #[inline]
    #[must_use]
    pub fn listener(&self) -> ChangeListener<T> {
        ChangeListener {
            inner: self.inner.new_receiver(),
        }
    }
}
