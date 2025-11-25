use flams_utils::parking_lot;
use std::marker::PhantomData;

#[cfg(feature = "ssr")]
pub use axum::extract::ws::Message as WSMessage;
#[cfg(feature = "ssr")]
pub use axum::extract::ws::WebSocket as AxumWS;
#[cfg(feature = "ssr")]
pub use flams_database::DBBackend;

#[cfg(feature = "hydrate")]
#[derive(Debug)]
pub struct WSClient<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> {
    socket: leptos::web_sys::WebSocket,
    _phantom: PhantomData<(ClientMsg, ServerMsg)>,
    queue: std::sync::Arc<parking_lot::Mutex<Option<Vec<String>>>>,
}

#[cfg(feature = "hydrate")]
impl<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> Clone for WSClient<ClientMsg, ServerMsg>
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            socket: self.socket.clone(),
            queue: self.queue.clone(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "hydrate")]
impl<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> WSClient<ClientMsg, ServerMsg>
{
    pub fn send(&self, msg: &ClientMsg) {
        let Ok(s) = serde_json::to_string(msg) else {
            tracing::error!("Error serializing websocket message");
            return;
        };
        {
            if let Some(v) = &mut *self.queue.lock() {
                v.push(s);
                return;
            }
        }

        if let Err(e) = self.socket.send_with_str(&s) {
            tracing::error!("Error sending websocket message: {}", js_to_string(e));
        }
    }

    #[inline]
    pub fn new(endpoint: &str, mut handler: impl FnMut(ServerMsg) + 'static) -> Option<Self> {
        Self::new_i(
            endpoint,
            Box::new(move |s| {
                let mut deserializer = serde_json::Deserializer::from_str(&s);
                deserializer.disable_recursion_limit();
                let value = ServerMsg::deserialize(&mut deserializer);
                match value {
                    Ok(msg) => handler(msg),
                    Err(e) => {
                        tracing::error!("{e}");
                    }
                }
            }),
        )
    }
    fn new_i(endpoint: &str, mut handler: Box<dyn FnMut(String) + 'static>) -> Option<Self> {
        use leptos::wasm_bindgen::JsCast;
        use leptos::wasm_bindgen::prelude::Closure;
        let ws = match leptos::web_sys::WebSocket::new(endpoint) {
            Ok(ws) => ws,
            Err(e) => {
                tracing::error!("Error creating websocket: {}", js_to_string(e));
                return None;
            }
        };
        let ws2 = ws.clone();
        let callback =
            Closure::<dyn FnMut(_)>::new(move |event| callback(&ws2, &mut *handler, event));
        ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback.forget();

        let r = Self {
            socket: ws,
            queue: std::sync::Arc::default(),
            _phantom: PhantomData,
        };
        let ws = r.socket.clone();
        let queue = r.queue.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_: leptos::web_sys::MessageEvent| {
            if let Some(queue) = { queue.lock().take() } {
                for s in queue {
                    let _ = ws.send_with_str(&s);
                }
            }
        });
        r.socket.set_onopen(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
        Some(r)
    }
}

#[cfg(feature = "hydrate")]
#[allow(clippy::needless_pass_by_value)]
fn callback(
    ws: &leptos::web_sys::WebSocket,
    handler: &mut dyn FnMut(String),
    event: leptos::web_sys::MessageEvent,
) {
    let Some(data) = event.data().as_string() else {
        tracing::error!("Not a string: {}", js_to_string(event.data()));
        return;
    };
    if data == "ping" {
        if let Err(e) = ws.send_with_str("pong") {
            tracing::error!("Error sending websocket message: {}", js_to_string(e));
        }
    } else {
        handler(data);
    }
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct WSSocket<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> {
    socket: tokio::sync::mpsc::UnboundedSender<ServerMsg>,
    _phantom: PhantomData<(ClientMsg, ServerMsg)>,
}

#[cfg(feature = "ssr")]
pub trait WSServerSocket<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
>: Sized + Sync + 'static
{
    const TIMEOUT: f32 = 10.0;

    fn new(socket: WSSocket<ClientMsg, ServerMsg>) -> impl Future<Output = Self> + Send;
    fn handle(&self, msg: ClientMsg) -> impl Future<Output = bool> + Send;
    fn span(&self) -> Option<&'static tracing::Span>;

    fn allow_user(state: crate::LoginState) -> bool {
        true
    }

    async fn handler(
        auth_session: axum_login::AuthSession<flams_database::DBBackend>,
        ws: axum::extract::WebSocketUpgrade,
    ) -> axum::response::Response
    where
        Self: Send,
    {
        let login = match &auth_session.backend.admin {
            None => crate::LoginState::NoAccounts,
            Some(_) => match auth_session.user {
                None => crate::LoginState::None,
                Some(flams_database::DBUser {
                    id: 0, username, ..
                }) if username == "admin" => crate::LoginState::Admin,
                Some(u) => crate::LoginState::User {
                    name: u.username,
                    avatar: u.avatar_url.unwrap_or_default(),
                    is_admin: u.is_admin,
                },
            },
        };
        if !Self::allow_user(login) {
            let mut res = axum::response::Response::new(axum::body::Body::empty());
            *(res.status_mut()) = axum::http::StatusCode::UNAUTHORIZED;
            return res;
        }
        ws.on_upgrade(move |mut ws| async move {
            use axum::body::Bytes;
            let timeout = std::time::Duration::from_secs_f32(Self::TIMEOUT);
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

            let socket = WSSocket {
                socket: sender.clone(),
                _phantom: PhantomData,
            };
            let slf = std::sync::Arc::new(Self::new(socket).await);
            let span = slf.span().map(tracing::Span::enter);
            let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            loop {
                if cancel.load(std::sync::atomic::Ordering::Acquire) {
                    tracing::info!("Dropping websocket");
                    return
                }
                tokio::select! {
                    () = tokio::time::sleep(timeout) => {
                        if ws.send(axum::extract::ws::Message::Ping(Bytes::new())).await.is_err() {
                            tracing::info!("Error sending ping; Dropping websocket");
                            return
                        }
                    },
                    msg = receiver.recv() => {
                        match msg {
                            None => {
                                tracing::info!("Receiver closed; Dropping websocket");
                                return
                            },
                            Some(msg) => {
                                if let Ok(msg) = serde_json::to_string(&msg) && {
                                    tracing::info!("Returning {}",msg);
                                    ws.send(axum::extract::ws::Message::Text(msg.into())).await.is_err()
                                } {
                                    tracing::info!("Error serializing result; Dropping websocket");
                                    return
                                }
                            }
                        }
                    }
                    msg = ws.recv() => {
                        match msg {
                            None => {
                                tracing::info!("Received None-message from client; Dropping websocket");
                                return
                            },
                            Some(Ok(axum::extract::ws::Message::Ping(_))) => {
                                if ws.send(axum::extract::ws::Message::Pong(Bytes::new())).await.is_err() {
                                    tracing::info!("Ping not returned; Dropping websocket");
                                    return;
                                }
                            }
                            Some(Ok(axum::extract::ws::Message::Text(msg))) => {
                                tracing::info!("Received {}",msg);
                                let cancel = cancel.clone();
                                let slf = slf.clone();
                                #[allow(clippy::let_underscore_future)]
                                let _ = tokio::task::spawn(async move {
                                    match serde_json::from_str(&msg) {
                                        Ok(msg) => {
                                            if !slf.handle(msg).await {
                                                tracing::info!("handle returned false; Dropping websocket");
                                                cancel.store(true, std::sync::atomic::Ordering::Release);
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Error: {e:?}");
                                        }
                                    }
                                });
                            }
                            _ => ()
                        }
                    }
                }
            }
        })
    }
}

/*
impl<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> Clone for WSServer<ClientMsg,ServerMsg> {
    fn clone(&self) -> Self {
        Self {
            socket:self.socket.clone()
        }
    }
}
*/

#[cfg(feature = "ssr")]
impl<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
> WSSocket<ClientMsg, ServerMsg>
{
    #[inline]
    pub fn send(&self, msg: ServerMsg) {
        let _ = self.socket.send(msg);
    }
}

#[cfg(feature = "hydrate")]
fn js_to_string(e: leptos::wasm_bindgen::JsValue) -> String {
    use leptos::web_sys::js_sys::Object;
    Object::from(e).to_string().into()
}

#[cfg(feature = "hydrate")]
pub trait WebSocketClient<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
>: WebSocket<ClientMsg, ServerMsg>
{
    fn new(ws: leptos::web_sys::WebSocket) -> Self;
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket;

    fn send(&mut self, msg: &ClientMsg) {
        let Ok(s) = serde_json::to_string(msg) else {
            tracing::error!("Error serializing websocket message");
            return;
        };
        if let Err(e) = self.socket().send_with_str(&s) {
            tracing::error!("Error sending websocket message: {}", js_to_string(e));
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn callback(
        ws: &leptos::web_sys::WebSocket,
        handle: &mut impl FnMut(ServerMsg) -> Option<ClientMsg>,
        event: leptos::web_sys::MessageEvent,
    ) {
        let Some(data) = event.data().as_string() else {
            tracing::error!("Not a string: {}", js_to_string(event.data()));
            return;
        };
        if data == "ping" {
            if let Err(e) = ws.send_with_str("pong") {
                tracing::error!("Error sending websocket message: {}", js_to_string(e));
            }
        } else {
            let mut deserializer = serde_json::Deserializer::from_str(&data);
            deserializer.disable_recursion_limit();
            let value = ServerMsg::deserialize(&mut deserializer);
            let ret = match value {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("{e}");
                    return;
                }
            };
            if let Some(a) = handle(ret) {
                let Ok(s) = serde_json::to_string(&a) else {
                    tracing::error!("Error serializing websocket message");
                    return;
                };
                if let Err(e) = ws.send_with_str(&s) {
                    tracing::error!("Error sending websocket message: {}", js_to_string(e));
                }
            }
        }
    }

    fn start(mut handle: impl (FnMut(ServerMsg) -> Option<ClientMsg>) + 'static) -> Option<Self> {
        use leptos::wasm_bindgen::JsCast;
        use leptos::wasm_bindgen::prelude::Closure;
        let ws = match leptos::web_sys::WebSocket::new(Self::SERVER_ENDPOINT) {
            Ok(ws) => ws,
            Err(e) => {
                tracing::error!("Error creating websocket: {}", js_to_string(e));
                return None;
            }
        };
        let ws2 = ws.clone();
        let callback =
            Closure::<dyn FnMut(_)>::new(move |event| Self::callback(&ws2, &mut handle, event));
        ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        let mut r = Self::new(ws);
        callback.forget();
        if let Some(mut f) = r.on_open() {
            let callback = Closure::<dyn FnMut(_)>::new(move |_: leptos::web_sys::MessageEvent| {
                f();
            });
            r.socket()
                .set_onopen(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
        Some(r)
    }

    fn on_open(&self) -> Option<Box<dyn FnMut()>> {
        None
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
pub trait WebSocketServer<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
>: WebSocket<ClientMsg, ServerMsg>
{
    async fn new(account: crate::LoginState, db: flams_database::DBBackend) -> Option<Self>;
    async fn next(&mut self) -> Option<ServerMsg>;
    async fn handle_message(&mut self, msg: ClientMsg) -> Option<ServerMsg>;
    async fn on_start(&mut self, _socket: &mut axum::extract::ws::WebSocket) {}

    async fn ws_handler(
        auth_session: axum_login::AuthSession<flams_database::DBBackend>,
        ws: axum::extract::WebSocketUpgrade,
    ) -> axum::response::Response
    where
        Self: Send,
    {
        let login = match &auth_session.backend.admin {
            None => crate::LoginState::NoAccounts,
            Some(_) => match auth_session.user {
                None => crate::LoginState::None,
                Some(flams_database::DBUser {
                    id: 0, username, ..
                }) if username == "admin" => crate::LoginState::Admin,
                Some(u) => crate::LoginState::User {
                    name: u.username,
                    avatar: u.avatar_url.unwrap_or_default(),
                    is_admin: u.is_admin,
                },
            },
        };
        Self::new(login, auth_session.backend).await.map_or_else(
            || {
                let mut res = axum::response::Response::new(axum::body::Body::empty());
                *(res.status_mut()) = axum::http::StatusCode::UNAUTHORIZED;
                res
            },
            |conn| ws.on_upgrade(move |socket| conn.on_upgrade(socket)),
        )
    }

    async fn on_upgrade(mut self, mut socket: axum::extract::ws::WebSocket)
    where
        Self: Send,
    {
        use axum::body::Bytes;
        if socket
            .send(axum::extract::ws::Message::Ping(Bytes::new()))
            .await
            .is_err()
        {
            return;
        }
        let timeout = std::time::Duration::from_secs_f32(Self::TIMEOUT);
        self.on_start(&mut socket).await;
        loop {
            tokio::select! {
                () = tokio::time::sleep(timeout) => {
                    if socket.send(axum::extract::ws::Message::Ping(Bytes::new())).await.is_err() {
                        return
                    }
                },
                msg = self.next() => {
                    if let Some(msg) = msg {
                    if let Ok(msg) = serde_json::to_string(&msg) {
                        if socket.send(axum::extract::ws::Message::Text(msg.into())).await.is_err() {
                            return
                        }
                    }
                } else {return}
                },
                o = socket.recv() => {
                    match o {
                        None => {
                            break
                        },
                        Some(msg) => match msg {
                            Ok(axum::extract::ws::Message::Ping(_)) => {
                                if socket.send(axum::extract::ws::Message::Pong(Bytes::new())).await.is_err() {
                                    break
                                }
                            },
                            Ok(axum::extract::ws::Message::Text(msg)) => {
                                if let Ok(msg) = serde_json::from_str(&msg) {
                                    if let Some(reply) = self.handle_message(msg).await {
                                        if let Ok(reply) = serde_json::to_string(&reply) {
                                            if socket.send(axum::extract::ws::Message::Text(reply.into())).await.is_err() {
                                                break
                                            }
                                        }
                                    }
                                }
                            },
                            _ => ()
                        },
                    }
                },
            }
        }
    }
}

pub trait WebSocket<
    ClientMsg: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
    ServerMsg: serde::Serialize + std::fmt::Debug + for<'a> serde::Deserialize<'a> + Send,
>: Sized + 'static
{
    const TIMEOUT: f32 = 10.0;
    const SERVER_ENDPOINT: &'static str;

    #[cfg(feature = "ssr")]
    fn force_start_server() {
        //let (signal_read,_) = signal(false);
        //let _res = Effect::new(move |_| {
        //    let _ = signal_read.get();
        //});
    }

    #[cfg(feature = "hydrate")]
    fn force_start_client(
        handle: impl (FnMut(ServerMsg) -> Option<ClientMsg>) + 'static + Clone,
        mut on_start: impl FnMut(Self) + 'static,
    ) where
        Self: WebSocketClient<ClientMsg, ServerMsg>,
    {
        //let (signal_read,_) = signal(false);
        let _res = leptos::prelude::Effect::new(move |_| {
            //let _ = signal_read.get();
            if let Some(r) = Self::start(handle.clone()) {
                on_start(r);
            }
        });
    }
}
