pub(crate) mod errors;

//use std::future::Future;
use async_trait::async_trait;
use leptos::prelude::*;
use crate::console_log;


#[async_trait]
pub(crate) trait WebSocket<
    ClientMsg:serde::Serialize+for<'a>serde::Deserialize<'a>+Send,
    ServerMsg:serde::Serialize+std::fmt::Debug+for<'a>serde::Deserialize<'a>+Send
>:Sized+'static {
    const TIMEOUT: f32 = 10.0;
    const SERVER_ENDPOINT:&'static str;
    #[cfg(feature="server")]
    async fn ws_handler(
        auth_session: axum_login::AuthSession<crate::accounts::AccountManager>,
        axum::extract::State(state): axum::extract::State<crate::server::AppState>,
        ws:axum::extract::WebSocketUpgrade,
        // do I even need this?
        agent:Option<axum_extra::TypedHeader<axum_extra::headers::UserAgent>>
    ) -> axum::response::Response where Self:Send {
        let login = crate::accounts::login_status_with_session(Some(&auth_session),|| Some(state.db.clone())).await;
        //println!("Login status: {login:?}");
        let login = login.unwrap_or(crate::accounts::LoginState::None);
        if let Some(conn) = Self::new(login,state.db).await {
            ws.on_upgrade(move |socket| conn.on_upgrade(socket))
        } else {
            let mut res = axum::response::Response::new(axum::body::Body::empty());
            *(res.status_mut()) = http::StatusCode::UNAUTHORIZED;
            res
        }
    }
    #[cfg(feature="server")]
    async fn on_upgrade(mut self,mut socket:axum::extract::ws::WebSocket) where Self:Send {
        if !socket.send(axum::extract::ws::Message::Ping(vec!())).await.is_ok() {
            return
        }
        let timeout = std::time::Duration::from_secs_f32(Self::TIMEOUT);
        self.on_start(&mut socket).await;
        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => if !socket.send(axum::extract::ws::Message::Ping(vec!())).await.is_ok() {
                    return
                },
                msg = self.next() => if let Some(msg) = msg {
                    if let Ok(msg) = leptos::serde_json::to_string(&msg) {
                        if !socket.send(axum::extract::ws::Message::Text(msg)).await.is_ok() {
                            return
                        }
                    }
                } else {return},
                o = socket.recv() => match o {
                    None => break,
                    Some(msg) => match msg {
                        Ok(axum::extract::ws::Message::Ping(_)) => {
                            if !socket.send(axum::extract::ws::Message::Pong(vec!())).await.is_ok() {
                                break
                            }
                        },
                        Ok(axum::extract::ws::Message::Text(msg)) => {
                            if let Ok(msg) = leptos::serde_json::from_str(&msg) {
                                if let Some(reply) = self.handle_message(msg).await {
                                    if let Ok(reply) = leptos::serde_json::to_string(&reply) {
                                        if !socket.send(axum::extract::ws::Message::Text(reply)).await.is_ok() {
                                            break
                                        }
                                    }
                                }
                            }
                        },
                        _ => ()
                    },
                },
            }
        }
    }

    /*#[cfg(feature="server")]
    fn start(mut handle:impl (FnMut(ServerMsg) -> Option<ClientMsg>)+'static) -> Self {
        unreachable!()
    }*/

    #[allow(unused_variables)]
    fn force_start(handle:impl (FnMut(ServerMsg) -> Option<ClientMsg>)+'static+Clone) {
        let (signal_read,_signal_write) = signal(false);
        let _res = Effect::new(move |_| {
            let _ = signal_read.get();
            #[cfg(feature="client")]
            let _ = Self::start(handle.clone());
        });
    }

    #[cfg(feature="client")]
    fn start(mut handle:impl (FnMut(ServerMsg) -> Option<ClientMsg>)+'static) -> Self {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        let ws = leptos::web_sys::WebSocket::new(Self::SERVER_ENDPOINT).unwrap();
        let ws2 = ws.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |event: leptos::web_sys::MessageEvent| {
            let data = event.data().as_string().unwrap();
            //console_log!("Here: {data}");
            if data == "ping" {
                ws2.send_with_str("pong").unwrap();
            } else {
                let mut deserializer = serde_json::Deserializer::from_str(&data);
                deserializer.disable_recursion_limit();
                let value = ServerMsg::deserialize(&mut deserializer);
                let ret = match value {
                    Ok(msg) => msg,
                    Err(e) => {
                        console_log!("Error: {e}");
                        return
                        //panic!("Fooo");
                    }
                };
                if let Some(a) = handle(ret) {
                    ws2.send_with_str(&serde_json::to_string(&a).unwrap()).unwrap();
                }
            }
        });
        ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
        Self::new(ws)
    }
    #[cfg(feature="server")]
    async fn on_start(&mut self,_socket:&mut axum::extract::ws::WebSocket) {}
    #[cfg(feature="server")]
    async fn new(account:crate::accounts::LoginState,db:sea_orm::DatabaseConnection) -> Option<Self>;
    #[cfg(feature="client")]
    fn new(ws: leptos::web_sys::WebSocket) -> Self;
    #[cfg(feature="server")]
    async fn next(&mut self) -> Option<ServerMsg>;
    #[cfg(feature="server")]
    async fn handle_message(&mut self,msg:ClientMsg) -> Option<ServerMsg>;
    #[cfg(feature="client")]
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket;
    #[cfg(feature="client")]
    fn send(&mut self,msg:&ClientMsg) {
        self.socket().send_with_str(&serde_json::to_string(msg).unwrap()).unwrap();
    }
}

#[macro_export]
macro_rules! socket {
    ($ident:ident<$tpC:ty> => {$($struc:tt)*} {$($server:tt)*}) => {
        #[cfg(feature="server")]
        socket!(@server $ident;$tpS;$tpC;{$($struc)*};$($server)*);
        #[cfg(feature="client")]
        socket!(@client $ident;$tpS;$tpC;$path);
    };
}
/*
pub(crate) fn callback<F:Future<Output=()> + 'static>(millis:u32,f:impl Fn() -> F + 'static + Clone) -> Effect<gloo_timers::callback::Interval> {
    create_effect(move |_| {
        let f = f.clone();
        gloo_timers::callback::Interval::new(millis, move ||
            spawn_local((f.clone())())
        )
    })
}


pub(crate) fn if_logged_in_client<R>(yes:impl FnOnce() -> R,no:impl FnOnce() -> R) -> R {
    let login = expect_context::<RwSignal<LoginState>>();
    crate::console_log!("Here: Checking login in {} mode: {:?}",target(),login.get_untracked());
    match login.get() {
        LoginState::Admin => yes(),
        _ => no()
    }
}

#[cfg(feature="server")]
pub(crate) async fn if_logged_in_server<R>(yes:impl Future<Output=R>,no:impl Future<Output=R>) -> R {
    #[cfg(feature="accounts")]
    {
        match crate::accounts::login_status().await {
            Ok(LoginState::Admin) => yes.await,
            _ => no.await
        }
    }
    #[cfg(all(feature="server",not(feature="accounts")))]
    { yes.await }
}

#[cfg(feature="accounts")]
mod accounts {
    use std::collections::HashMap;
    use std::future::Future;
    use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
    use actix_web::cookie::time::Duration;
    use tokio::sync::RwLock;
    use immt_core::utils::triomphe::Arc;
    #[derive(Clone,Default)]
    pub(crate) struct PseudoRedis(Arc<RwLock<HashMap<String,(i64,HashMap<String,String>)>>>);
    impl PseudoRedis {
        fn clear(mut lock:tokio::sync::RwLockWriteGuard<HashMap<String,(i64,HashMap<String,String>)>>) {
            let now = chrono::Utc::now().timestamp_millis();
            lock.retain(|_,(t,_)| *t>now);
        }
    }
    impl SessionStore for PseudoRedis {
        fn load(&self, session_key: &SessionKey) -> impl Future<Output=Result<Option<HashMap<String,String>>, LoadError>> {
            async move {
                let map = self.0.read().await;
                let ret = map.get(session_key.as_ref()).map(|(_,m)| m).cloned();
                Ok(ret)
            }
        }

        fn save(&self, session_state: HashMap<String,String>, ttl: &Duration) -> impl Future<Output=Result<SessionKey, SaveError>> {
            async move {
                let id = md5::compute(format!("({},{:?})", chrono::Utc::now().timestamp_millis(),session_state)).0.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                let key = SessionKey::try_from(id).map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;
                let mut map = self.0.write().await;
                let timeout = chrono::Utc::now().timestamp_millis()+(ttl.whole_milliseconds() as i64);
                map.insert(key.as_ref().to_string(), (timeout,session_state));
                Self::clear(map);
                Ok(key)
            }
        }
        fn update(&self, session_key: SessionKey, session_state: HashMap<String,String>, ttl: &Duration) -> impl Future<Output=Result<SessionKey, UpdateError>> {
            async move {
                let mut map = self.0.write().await;
                let timeout = chrono::Utc::now().timestamp_millis()+(ttl.whole_milliseconds() as i64);
                map.insert(session_key.as_ref().to_string(), (timeout,session_state));
                Self::clear(map);
                Ok(session_key)
            }
        }
        fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> impl Future<Output=Result<(), anyhow::Error>> {
            async move {
                let mut map = self.0.write().await;
                if let Some((t,_)) = map.get_mut(session_key.as_ref()) {
                    let timeout = chrono::Utc::now().timestamp_millis()+(ttl.whole_milliseconds() as i64);
                    *t = timeout;
                    Self::clear(map);
                    Ok(())
                } else {Err(anyhow::anyhow!("Session not found"))}
            }
        }
        fn delete(&self, session_key: &SessionKey) -> impl Future<Output=Result<(), anyhow::Error>> {
            async move {
                let mut map = self.0.write().await;
                map.remove(session_key.as_ref());
                Ok(())
            }
        }
    }
}
#[cfg(feature="accounts")]
pub(crate) use accounts::*;
use crate::accounts::LoginState;


#[macro_export]
macro_rules! socket {
    ($ident:ident<$tpS:ty,$tpC:ty> @ $path:literal => {$($struc:tt)*} {$($server:tt)*}) => {
        #[cfg(feature="server")]
        socket!(@server $ident;$tpS;$tpC;{$($struc)*};$($server)*);
        #[cfg(feature="client")]
        socket!(@client $ident;$tpS;$tpC;$path);
    };
    (@server $ident:ident;$tpS:ty;$tpC:ty;{$($struc:tt)*};$($server:tt)*) => {
        pub struct $ident {
            $($struc)*
        }
        /*
        impl $ident {
            pub async fn start(r:actix_web::HttpRequest,stream:actix_web::web::Payload) -> impl actix_web::Responder {
                use $crate::utils::ws::WS;
                let ctrl = Box::pin(r.app_data::<crate::server::Controller>().unwrap().clone());
                actix_web_actors::ws::start(Self::new(std::time::Instant::now(),ctrl),&r,stream)
            }
        }*/
        impl $crate::utils::ws::WS<$tpS,$tpC> for $ident {
            $($server)*
        }
        impl actix::Actor for $ident {
            type Context = actix_web_actors::ws::WebsocketContext<Self>;
            fn started(&mut self, ctx: &mut Self::Context) {
                use $crate::utils::ws::WS;
                self.actor_started(ctx)
            }
        }
        impl actix::StreamHandler<std::result::Result<actix_web_actors::ws::Message,actix_web_actors::ws::ProtocolError>> for $ident {
            fn handle(&mut self, msg: std::result::Result<actix_web_actors::ws::Message,actix_web_actors::ws::ProtocolError>, ctx: &mut Self::Context) {
                use $crate::utils::ws::WS;
                self.handle_stream(msg,ctx)
            }
        }
    };
    (@client $ident:ident;$tpS:ty;$tpC:ty;$path:literal) => {
        pub struct $ident();
        impl $crate::utils::ws::WS<$tpS,$tpC> for $ident {
            const PATH: &'static str = $path;
        }
    };
}

#[cfg(feature="server")]
pub mod ws {
    use std::time::Instant;
    use actix::prelude::*;
    use actix_web_actors::ws;

    pub trait WS<S:for<'de> serde::Deserialize<'de>,C:serde::Serialize>:Actor<Context=ws::WebsocketContext<Self>> + StreamHandler<Result<ws::Message,ws::ProtocolError>> {
        const TIMEOUT: f32 = 10.0;
        const EVERY: f32 = 2.0;
        fn new(now:Instant,r:&actix_web::HttpRequest) -> Self;
        fn last_ping(&mut self) -> &mut Instant;
        fn every(&mut self,first:bool, ctx: &mut Self::Context) -> Option<C> { None }
        fn handle_message(&mut self,_message:S) -> Option<C> { None }
        fn run<F:FnMut(C) -> Option<S>>(_f:F) {}

        fn on_start(&mut self,_ctx:&mut Self::Context) {}
        fn actor_started(&mut self, ctx: &mut Self::Context) {
            let interval = std::time::Duration::from_secs_f32(Self::EVERY);
            let timeout = std::time::Duration::from_secs_f32(Self::TIMEOUT);
            self.on_start(ctx);
            ctx.run_interval(interval, move |this,ctx| {
                if Instant::now().duration_since(*this.last_ping()) > timeout {
                    ctx.stop();return
                }
                ctx.ping(b"");
                let mut first = true;
                while let Some(msg) = this.every(first,ctx) {
                    first = false;
                    if let Ok(msg) = leptos::serde_json::to_string(&msg) {
                        ctx.text(msg)
                    }
                }
            });
        }
        fn handle_stream(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
            *self.last_ping() = Instant::now();
            match msg {
                Ok(ws::Message::Ping(m)) => {
                    ctx.pong(&m)
                },
                Ok(ws::Message::Text(msg)) => {
                    if let Ok(msg) = leptos::serde_json::from_str(&msg) {
                        if let Some(reply) = self.handle_message(msg) {
                            if let Ok(reply) = leptos::serde_json::to_string(&reply) {
                                ctx.text(reply)
                            }
                        }
                    }
                },
                Ok(ws::Message::Binary(_)) => (),
                Ok(ws::Message::Close(_)) => ctx.stop(),
                _ => (),
            }
        }

        async fn start(r:actix_web::HttpRequest,stream:actix_web::web::Payload) -> impl actix_web::Responder {
            ws::start(Self::new(Instant::now(),&r),&r,stream)
        }
    }
}

#[cfg(feature="client")]
pub mod ws {
    use leptos::{serde_json, web_sys};
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;

    pub(crate) trait WS<S:serde::Serialize,C:for<'de> serde::Deserialize<'de>> {
        const PATH: &'static str;
        fn run<F:FnMut(C) -> Option<S> + 'static>(mut f:F) {
            let ws = leptos::web_sys::WebSocket::new(Self::PATH).unwrap();
            let ws2 = ws.clone();
            let callback = Closure::<dyn FnMut(_)>::new(move |event: leptos::web_sys::MessageEvent| {
                let data = event.data().as_string().unwrap();
                if data == "ping" {
                    ws2.send_with_str("pong").unwrap();
                } else {
                    let ret = serde_json::from_str::<C>(&data).unwrap();
                    if let Some(a) = f(ret) {
                        ws2.send_with_str(&serde_json::to_string(&a).unwrap()).unwrap();
                    }
                }
            });
            ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
    }
}


#[cfg(feature="client")]
pub mod client {
    use std::ops::Deref;
    use wasm_bindgen::JsCast;
    use leptos::*;
    use leptos::web_sys::Node;

    #[macro_export]
    macro_rules! append {
        ($elem:ident <- $($tks:tt)*) => {
            if let Some(r) = $elem.get_untracked() {
                web_sys::Node::append_child(&r,wasm_bindgen::JsCast::dyn_ref(&*template!{$($tks)*}).unwrap()).unwrap();
            }
        }
    }

    pub trait HTMLExt {
        fn append<D:Deref<Target=Node>>(&self,elem:D);
    }
    impl<E:html::ElementDescriptor+Deref<Target=web_sys::HtmlElement>> HTMLExt for NodeRef<E> {
        fn append<D:Deref<Target=Node>>(&self,elem:D) {
            if let Some(r) = self.get_untracked() {
                r.append_child(elem.dyn_ref().unwrap()).unwrap();
            }
        }
    }
}

 */