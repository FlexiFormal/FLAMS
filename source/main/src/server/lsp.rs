use immt_lsp::async_lsp::{client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer, ClientSocket};

use immt_system::settings::Settings;
use tower::ServiceBuilder;
use tracing::Level;
use immt_lsp::async_lsp::lsp_types as types;

use crate::users::LoginState;

//struct TickEvent;
struct STDIOLSPServer {
  client:ClientSocket,
  on_port:tokio::sync::watch::Receiver<Option<u16>>
}

impl immt_lsp::IMMTLSPServer for STDIOLSPServer {
  #[inline]
  fn client(&mut self) -> &mut ClientSocket {
    &mut self.client
  }
  fn initialized(&mut self) {
    let v = *self.on_port.borrow();
    if v.is_some() {
      if let Err(r) = self.client.notify::<ServerURL>(ServerURL::get()) {
          tracing::error!("failed to send notification: {}", r);
      }
    } else {
      let mut port = self.on_port.clone();
      let client = self.client.clone();
      tokio::spawn(async move {
        let _ = port.wait_for(|e| e.map_or(false,|_| {
          if let Err(r) = client.notify::<ServerURL>(ServerURL::get()) {
            tracing::error!("failed to send notification: {}", r);
          };
          true
      })).await;
      });
    }
  }
}

impl STDIOLSPServer {
  #[allow(clippy::let_and_return)]
  fn new_router(client:ClientSocket,on_port:tokio::sync::watch::Receiver<Option<u16>>) -> Router<immt_lsp::ServerWrapper<Self>> {
    let /*mut*/ router = Router::from_language_server(immt_lsp::ServerWrapper::new(Self {client,on_port}));
    //router.event(Self::on_tick);
    router
  }
  /*fn on_tick(&mut self,_:TickEvent) -> ControlFlow<async_lsp::Result<()>> {
    tracing::info!("tick");
    self.counter += 1;
    ControlFlow::Continue(())
  }*/
}

#[allow(clippy::future_not_send)]
/// #### Panics
pub async fn lsp(on_port:tokio::sync::watch::Receiver<Option<u16>>) {
  let (server,_client) = immt_lsp::async_lsp::MainLoop::new_server(|client| {
    /*tokio::spawn({
      let client = client.clone();
      async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs_f32(0.5));
        loop {
          interval.tick().await;
          if client.emit(TickEvent).is_err() {
            break
          }
        }
      }
    });*/
    ServiceBuilder::new()
      .layer(TracingLayer::default())
      .layer(LifecycleLayer::default())
      .layer(CatchUnwindLayer::default())
      .layer(ConcurrencyLayer::default())
      .layer(ClientProcessMonitorLayer::new(client.clone()))
      .service(STDIOLSPServer::new_router(client,on_port))
  });

  tracing_subscriber::fmt()
    .with_max_level(Level::INFO)//if debug {Level::TRACE} else {Level::INFO})
    .with_ansi(false)
    .with_target(true)
    .with_writer(std::io::stderr)
    .init();
  #[cfg(unix)]
  let (stdin,stdout) = (
    immt_lsp::async_lsp::stdio::PipeStdin::lock_tokio().expect("Failed to lock stdin"),
    immt_lsp::async_lsp::stdio::PipeStdout::lock_tokio().expect("Failed to lock stdout")
  );
  #[cfg(not(unix))]
  let (stdin,stdout) = (
    tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
    tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout())
  );

  server.run_buffered(stdin, stdout).await.expect("Failed to run server");
}


struct ServerURL;
impl ServerURL {
    fn get() -> String {
        let settings = Settings::get();
        format!("http://{}:{}",settings.ip,settings.port)
    }
}
impl types::notification::Notification for ServerURL {
    type Params = String;
    const METHOD : &str = "immt/serverURL";
}


struct WSLSPServer {
  client:ClientSocket
}

impl immt_lsp::IMMTLSPServer for WSLSPServer {
  #[inline]
  fn client(&mut self) -> &mut ClientSocket {
    &mut self.client
  }
}

pub(crate) async fn register(
  auth_session: axum_login::AuthSession<crate::server::db::DBBackend>,
  ws:axum::extract::WebSocketUpgrade,
) -> axum::response::Response {
  let login = match &auth_session.backend.admin {
    None => LoginState::NoAccounts,
    Some(_) => match auth_session.user {
        None => LoginState::None,
        Some(crate::users::User{id:0,username,..}) if username == "admin" => LoginState::Admin,
        Some(u) => LoginState::User(u.username)
    }
  };
  match login {
    LoginState::NoAccounts | LoginState::Admin => immt_lsp::ws::upgrade(ws, |c| WSLSPServer { client: c }),
    _ => {
      let mut res = axum::response::Response::new(axum::body::Body::empty());
      *(res.status_mut()) = http::StatusCode::UNAUTHORIZED;
      res
    }
  }
}