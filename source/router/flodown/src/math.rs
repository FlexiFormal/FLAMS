use ftml_uris::ModuleUri;
use leptos::prelude::*;
use std::{collections::BTreeMap, sync::atomic::AtomicUsize};

#[cfg(feature = "ssr")]
use flams_router_base::ws::WSSocket;

#[cfg(feature = "ssr")]
pub(crate) static TEX_SPAN: std::sync::LazyLock<tracing::Span> =
    std::sync::LazyLock::new(|| tracing::info_span!(target:"mathjx",parent:None,"mathjx"));

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct TeXSocket {
    socket: WSSocket<(usize, TeXMath), (usize, Result<String, String>)>,
    tex: std::sync::Arc<flams_stex::math::RusTeXMath>,
}

#[cfg(feature = "ssr")]
impl flams_router_base::ws::WSServerSocket<(usize, TeXMath), (usize, Result<String, String>)>
    for TeXSocket
{
    #[inline]
    fn span(&self) -> Option<&'static tracing::Span> {
        Some(&*TEX_SPAN)
    }

    async fn new(
        socket: flams_router_base::ws::WSSocket<(usize, TeXMath), (usize, Result<String, String>)>,
    ) -> Self {
        tokio::task::spawn_blocking(move || {
            TEX_SPAN.in_scope(move || Self {
                socket,
                tex: std::sync::Arc::new(flams_stex::math::RusTeXMath::default()),
            })
        })
        .await
        .expect("this should not happen")
    }
    async fn handle(&self, (i, msg): (usize, TeXMath)) -> bool {
        let s = self.clone();
        tokio::task::spawn_blocking(move || {
            TEX_SPAN.in_scope(move || {
                tracing::info!("Received {:?}", msg);
                if let Some(r) = match msg {
                    TeXMath::UseModule(m) => {
                        s.tex.add_usemodule(&m);
                        None
                    }
                    TeXMath::Inline(il) => {
                        let r = s.tex.run(&format!("${il}$"));
                        Some((i, r))
                    }
                    TeXMath::Block(bl) => {
                        let r = s.tex.run(&format!("\\[{bl}\\]")).map(|mut s| {
                            s.insert_str(0, "<div class=\"rustex-display\">");
                            s.push_str("</div>");
                            s
                        });
                        Some((i, r))
                    }
                } {
                    tracing::info!("Returning {:?}", r);
                    s.socket.send(r);
                }
                true
            })
        })
        .await
        .expect("this should not happen")
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug)]
enum MaybeMath {
    Done(flams_stex::math::RusTeXMath),
    Pending(Vec<(usize, TeXMath)>),
}

#[derive(Debug, Clone)]
pub struct MathSocket {
    #[cfg(all(feature = "hydrate", not(feature = "docs-only")))]
    socket: send_wrapper::SendWrapper<leptos::web_sys::WebSocket>,
    #[cfg(all(feature = "hydrate", feature = "docs-only"))]
    socket: (),
    #[cfg(feature = "ssr")]
    rustex: std::sync::Arc<parking_lot::Mutex<MaybeMath>>,
    #[cfg(feature = "ssr")]
    queue: std::sync::Arc<parking_lot::Mutex<Vec<(usize, Result<String, String>)>>>,
    #[cfg(feature = "hydrate")]
    initialized: std::sync::Arc<parking_lot::Mutex<Option<Vec<(usize, TeXMath)>>>>,
}

impl flams_router_base::ws::WebSocket<(usize, TeXMath), (usize, Result<String, String>)>
    for MathSocket
{
    const SERVER_ENDPOINT: &str = "/ws/mathjx";
    const TIMEOUT: f32 = 10.0;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TeXMath {
    Inline(String),
    Block(String),
    UseModule(ModuleUri),
}

#[cfg(feature = "ssr")]
impl MathSocket {
    fn run_arg(
        slf: &flams_stex::math::RusTeXMath,
        i: usize,
        tm: TeXMath,
    ) -> Option<(usize, Result<String, String>)> {
        use ftml_ontology::utils::time::measure;
        match tm {
            TeXMath::UseModule(m) => {
                //println!("Doing UseModule {m}");
                slf.add_usemodule(&m);
                None
            }
            TeXMath::Inline(il) => {
                //println!("Doing ${il}$");
                let r = slf.run(&format!("${il}$")); //(r, t) = measure(|| slf.run(&format!("${il}$")));
                //println!("Done ${il}$ after {t}");
                Some((i, r))
            }
            TeXMath::Block(bl) => {
                //println!("Doing $${bl}$$");
                let r=//(r, t) = measure(|| {
                    slf.run(&format!("\\[{bl}\\]")).map(|mut s| {
                        s.insert_str(0, "<div class=\"rustex-display\">");
                        s.push_str("</div>");
                        s
                        //})
                });
                //println!("Done $${bl}$$ after {t}");
                Some((i, r))
            }
        }
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl flams_router_base::ws::WebSocketServer<(usize, TeXMath), (usize, Result<String, String>)>
    for MathSocket
{
    async fn new(
        account: flams_router_base::LoginState,
        _db: flams_database::DBBackend,
    ) -> Option<Self> {
        use flams_router_base::LoginState;
        match account {
            LoginState::Admin | LoginState::NoAccounts | LoginState::User { .. } => {
                let rustex =
                    std::sync::Arc::new(parking_lot::Mutex::new(MaybeMath::Pending(Vec::new())));
                let queue = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
                let rst = rustex.clone();
                let q2 = queue.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    let r = flams_stex::math::RusTeXMath::default();
                    loop {
                        let mut lock = rst.lock();
                        //let mut queue = q2.lock();
                        if let MaybeMath::Pending(v) = &mut *lock
                            && !v.is_empty()
                        {
                            let (i, msg) = v.remove(0);
                            drop(lock);
                            if let Some(s) = Self::run_arg(&r, i, msg) {
                                q2.lock().push(s);
                            }
                        } else {
                            *lock = MaybeMath::Done(r);
                            break;
                        }
                    }
                });
                Some(Self {
                    rustex,
                    queue,
                    #[cfg(feature = "hydrate")]
                    socket: unreachable!(),
                })
            }
            _ => None,
        }
    }

    #[inline]
    async fn next(&mut self) -> Option<(usize, Result<String, String>)> {
        let v = {
            let mut lock = self.queue.lock();
            if !lock.is_empty() {
                Some(lock.remove(0))
            } else {
                None
            }
        };
        if let Some(v) = v {
            Some(v)
        } else {
            // hack to avoid async channels - 3.5 > Self::TIMEOUT
            tokio::time::sleep(std::time::Duration::from_secs_f32(3.5)).await;
            None
        }
    }

    async fn handle_message(
        &mut self,
        (i, msg): (usize, TeXMath),
    ) -> Option<(usize, Result<String, String>)> {
        let rs = self.rustex.clone();
        tokio::task::spawn_blocking(move || {
            let mut lock = rs.lock();
            match &mut *lock {
                MaybeMath::Done(rs) => {
                    //let rs = rs.clone();
                    //drop(lock);
                    Self::run_arg(rs, i, msg)
                }
                MaybeMath::Pending(v) => {
                    //println!("Deferring {msg:?}");
                    v.push((i, msg));
                    None
                }
            }
        })
        .await
        .ok()
        .flatten()
    }
}

#[cfg(feature = "hydrate")]
impl flams_router_base::ws::WebSocketClient<(usize, TeXMath), (usize, Result<String, String>)>
    for MathSocket
{
    fn new(ws: leptos::web_sys::WebSocket) -> Self {
        Self {
            #[cfg(not(feature = "docs-only"))]
            socket: send_wrapper::SendWrapper::new(ws),
            #[cfg(feature = "docs-only")]
            socket: (),
            initialized: std::sync::Arc::new(parking_lot::Mutex::new(Some(Vec::new()))),
            #[cfg(feature = "ssr")]
            rustex: unreachable!(),
            #[cfg(feature = "ssr")]
            queue: unreachable!(),
        }
    }
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket {
        #[cfg(not(feature = "docs-only"))]
        {
            &mut self.socket
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }
    fn on_open(&self) -> Option<Box<dyn FnMut()>> {
        let mut slf = self.clone();
        Some(Box::new(move || {
            let v = slf.initialized.lock().take();
            if let Some(v) = v {
                for msg in v {
                    //tracing::warn!("Sending message: {msg:?}");
                    slf.send(&msg);
                }
            }
        }) as _)
    }
}

#[derive(Debug, Clone)]
struct MathState {
    counter: std::sync::Arc<AtomicUsize>,
    values: std::sync::Arc<
        parking_lot::Mutex<BTreeMap<usize, RwSignal<Option<Result<String, String>>>>>,
    >,
    hash: std::sync::Arc<
        dashmap::DashMap<
            TeXMath,
            RwSignal<Option<Result<String, String>>>,
            rustc_hash::FxBuildHasher,
        >,
    >,
}

#[cfg(feature = "hydrate")]
impl MathSocket {
    pub fn add_module(uri: ModuleUri) {
        #[cfg(not(feature = "docs-only"))]
        {
            use flams_router_base::ws::WebSocketClient;
            let slf = expect_context::<leptos::prelude::RwSignal<Self>>();
            let state = expect_context::<MathState>();
            let msg = TeXMath::UseModule(uri);
            if state.hash.contains_key(&msg) {
                return;
            }
            state.hash.insert(msg.clone(), RwSignal::new(None));
            let counter = state
                .counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            slf.update_untracked(move |slf| {
                {
                    if let Some(v) = &mut *slf.initialized.lock() {
                        v.push((counter, msg));
                        return;
                    }
                }
                //tracing::warn!("Sending message: {msg:?}");
                slf.send(&(counter, msg));
            })
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }

    pub fn inline_math(s: &str) -> (usize, RwSignal<Option<Result<String, String>>>) {
        #[cfg(not(feature = "docs-only"))]
        {
            use flams_router_base::ws::WebSocketClient;
            let state = expect_context::<MathState>();
            let counter = state
                .counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let msg = TeXMath::Inline(s.trim().to_string());
            if let Some(sig) = state.hash.get(&msg) {
                return (counter, *sig);
            }
            let sig = RwSignal::new(None);
            state.hash.insert(msg.clone(), sig);
            let slf = expect_context::<leptos::prelude::RwSignal<Self>>();
            slf.update_untracked(move |slf| {
                state.values.lock().insert(counter, sig);
                {
                    if let Some(v) = &mut *slf.initialized.lock() {
                        v.push((counter, msg));
                        return;
                    }
                }
                //tracing::warn!("Sending message: {msg:?}");
                slf.send(&(counter, msg));
            });
            (counter, sig)
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }

    pub fn block_math(s: &str) -> (usize, RwSignal<Option<Result<String, String>>>) {
        #[cfg(not(feature = "docs-only"))]
        {
            use flams_router_base::ws::WebSocketClient;
            let state = expect_context::<MathState>();
            let counter = state
                .counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let msg = TeXMath::Block(s.trim().to_string());
            if let Some(sig) = state.hash.get(&msg) {
                return (counter, *sig);
            }
            let sig = RwSignal::new(None);
            state.hash.insert(msg.clone(), sig);
            let slf = expect_context::<leptos::prelude::RwSignal<Self>>();
            slf.update_untracked(move |slf| {
                state.values.lock().insert(counter, sig);
                state.values.lock().insert(counter, sig);
                {
                    if let Some(v) = &mut *slf.initialized.lock() {
                        v.push((counter, msg));
                        return;
                    }
                }
                //tracing::warn!("Sending message: {msg:?}");
                slf.send(&(counter, msg));
            });
            (counter, sig)
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }

    pub fn run() {
        #[cfg(not(feature = "docs-only"))]
        {
            use flams_router_base::ws::WebSocketClient;
            let values = std::sync::Arc::new(parking_lot::Mutex::default());
            let hash = std::sync::Arc::new(dashmap::DashMap::default());
            provide_context(MathState {
                counter: std::sync::Arc::default(),
                values: values.clone(),
                hash: hash.clone(),
            });
            let slf = Self::start(move |(i, msg)| {
                //tracing::warn!("Received message {i}: {msg:?}");
                if let Some(v) = values.lock().remove(&i) {
                    let _ = v.try_set(Some(msg));
                } else {
                    tracing::error!("Unexpected websocket message");
                };
                None
            })
            .expect("failed to open websocket");
            leptos::prelude::provide_context(leptos::prelude::RwSignal::new(slf));
        }
        #[cfg(feature = "docs-only")]
        {
            unreachable!()
        }
    }
}
