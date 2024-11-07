use immt_lsp::{async_lsp::{client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer, ClientSocket, LanguageClient}, LSPState, ProgressCallbackServer};

use immt_ontology::uris::{DocumentURI, URIRefTrait};
use immt_system::{backend::{archives::{source_files::{SourceDir, SourceEntry}, Archive}, GlobalBackend}, settings::Settings};
use immt_utils::{prelude::TreeChildIter, time::measure};
use tower::ServiceBuilder;
use tracing::Level;
use immt_lsp::async_lsp::lsp_types as lsp;

use crate::users::LoginState;

//struct TickEvent;
struct STDIOLSPServer {
  client:ClientSocket,
  state:LSPState,
  on_port:tokio::sync::watch::Receiver<Option<u16>>,
  workspaces:Vec<(String,lsp::Url)>
}

impl STDIOLSPServer {
  fn load_all(&self) {
    use rayon::prelude::*;
    let client = self.client.clone();
    let state = self.state.clone();
    let workspaces = self.workspaces.clone();
    let _ = tokio::task::spawn_blocking(move || {
      let (_,t) = measure(move || {
        let mut files = Vec::new();
        for a in GlobalBackend::get().all_archives().iter() {
          if let Archive::Local(a) =a { 
            a.with_sources(|d| for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
              match e {
                SourceEntry::File(f) => files.push((
                  f.relative_path.split('/').fold(a.source_dir(),|p,s| p.join(s)),
                  DocumentURI::from_archive_relpath(a.uri().owned(), &f.relative_path)
              )),
                _ => {}
              }
            })
          }
        }
        ProgressCallbackServer::with(client,"Initializing".to_string(),Some(files.len() as _),|p| {
          /*files.par_iter().for_each(|(file,uri)| {
            //p.update(file.display().to_string(), Some(1));
            state.load(&file,&uri);
          });*/
          for (file,uri) in files {
            p.update(file.display().to_string(), Some(1));
            state.load(&file,&uri,|data| {
              let lock = data.lock();
              if !lock.diagnostics.is_empty() {
                let mut client = p.client();
                if let Ok(uri) = lsp::Url::from_file_path(&file) { 
                  client.publish_diagnostics(lsp::PublishDiagnosticsParams {
                    uri,version:None,diagnostics:lock.diagnostics.iter().map(immt_lsp::to_diagnostic).collect()
                  });
                }
              }
            });
          }
          let mathhubs = &Settings::get().mathhubs;
          for (name,uri) in &workspaces {
            tracing::info!("workspace: {name}@{uri}");
          }
        });
      });
      tracing::info!("initialized after {t}");
    });
  }
}

impl immt_lsp::IMMTLSPServer for STDIOLSPServer {
  #[inline]
  fn client_mut(&mut self) -> &mut ClientSocket {
    &mut self.client
  }
  #[inline]
  fn client(&self) -> &ClientSocket {
    &self.client
  }
  #[inline]
  fn state(&self) -> &LSPState {
    &self.state
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
    self.load_all();
  }

  fn initialize<I:Iterator<Item=(String,lsp::Url)> + Send + 'static>(&mut self,workspaces:I) {
    self.workspaces = workspaces.collect();
  }
}

impl STDIOLSPServer {
  #[allow(clippy::let_and_return)]
  fn new_router(client:ClientSocket,on_port:tokio::sync::watch::Receiver<Option<u16>>) -> Router<immt_lsp::ServerWrapper<Self>> {
    let /*mut*/ router = Router::from_language_server(immt_lsp::ServerWrapper::new(Self {client,on_port,state:LSPState::default(),workspaces:Vec::new()}));
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
impl lsp::notification::Notification for ServerURL {
    type Params = String;
    const METHOD : &str = "immt/serverURL";
}


struct WSLSPServer {
  client:ClientSocket,
  state:LSPState
}

impl immt_lsp::IMMTLSPServer for WSLSPServer {
  #[inline]
  fn client_mut(&mut self) -> &mut ClientSocket {
    &mut self.client
  }
  #[inline]
  fn client(&self) -> &ClientSocket {
    &self.client
  }
  #[inline]
  fn state(&self) -> &LSPState {
    &self.state
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
    LoginState::NoAccounts | LoginState::Admin => immt_lsp::ws::upgrade(ws, |c| WSLSPServer { client: c, state:LSPState::default() }),
    _ => {
      let mut res = axum::response::Response::new(axum::body::Body::empty());
      *(res.status_mut()) = http::StatusCode::UNAUTHORIZED;
      res
    }
  }
}