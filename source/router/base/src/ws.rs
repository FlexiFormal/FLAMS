#[cfg(feature = "ssr")]
pub use axum::extract::ws::Message as WSMessage;
#[cfg(feature = "ssr")]
pub use axum::extract::ws::WebSocket as AxumWS;
#[cfg(feature = "ssr")]
pub use flams_database::DBBackend;

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
        handle: &mut impl (FnMut(ServerMsg) -> Option<ClientMsg>),
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
                tracing::error!(
                    "Error creating websockecrate::users::t: {}",
                    js_to_string(e)
                );
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
        if socket
            .send(axum::extract::ws::Message::Ping(Vec::new()))
            .await
            .is_err()
        {
            return;
        }
        let timeout = std::time::Duration::from_secs_f32(Self::TIMEOUT);
        self.on_start(&mut socket).await;
        loop {
            tokio::select! {
                () = tokio::time::sleep(timeout) => if socket.send(axum::extract::ws::Message::Ping(Vec::new())).await.is_err() {
                    return
                },
                msg = self.next() => if let Some(msg) = msg {
                    if let Ok(msg) = serde_json::to_string(&msg) {
                        if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                            return
                        }
                    }
                } else {return},
                o = socket.recv() => match o {
                    None => break,
                    Some(msg) => match msg {
                        Ok(axum::extract::ws::Message::Ping(_)) => {
                            if socket.send(axum::extract::ws::Message::Pong(Vec::new())).await.is_err() {
                                break
                            }
                        },
                        Ok(axum::extract::ws::Message::Text(msg)) => {
                            if let Ok(msg) = serde_json::from_str(&msg) {
                                if let Some(reply) = self.handle_message(msg).await {
                                    if let Ok(reply) = serde_json::to_string(&reply) {
                                        if socket.send(axum::extract::ws::Message::Text(reply)).await.is_err() {
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
