use immt_lsp::{annotations::to_diagnostic,async_lsp::{client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer, ClientSocket, LanguageClient}, state::LSPState, IMMTLSPServer, ProgressCallbackServer};

use immt_ontology::uris::{DocumentURI, URIRefTrait};
use immt_system::{backend::{archives::{source_files::{SourceDir, SourceEntry}, Archive}, GlobalBackend}, settings::Settings};
use immt_utils::{prelude::TreeChildIter, time::measure};
use tower::ServiceBuilder;
use tracing::Level;
use immt_lsp::async_lsp::lsp_types as lsp;

use crate::users::LoginState;


static GLOBAL_STATE: std::sync::OnceLock<LSPState> = std::sync::OnceLock::new();

//struct TickEvent;
pub struct STDIOLSPServer {
  client:ClientSocket,
  on_port:tokio::sync::watch::Receiver<Option<u16>>,
  workspaces:Vec<(String,lsp::Url)>
}

impl STDIOLSPServer {
  #[inline]
  pub fn global_state() -> Option<&'static LSPState> {
    GLOBAL_STATE.get()
  }
  fn load_all(&self) {
    let client = self.client.clone();
    let state = self.state().clone();
    for (name,uri) in &self.workspaces {
      tracing::info!("workspace: {name}@{uri}");
    }
    let _ = tokio::task::spawn_blocking(move || {
      state.load_mathhubs(client);
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
    Self::global_state().unwrap_or_else(|| unreachable!())
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
    tracing::info!("Using {} threads",tokio::runtime::Handle::current().metrics().num_workers());
    //#[cfg(not(debug_assertions))]
    {self.load_all();}
  }

  fn initialize<I:Iterator<Item=(String,lsp::Url)> + Send + 'static>(&mut self,workspaces:I) {
    self.workspaces = workspaces.collect();
  }
}

impl STDIOLSPServer {
  #[allow(clippy::let_and_return)]
  fn new_router(client:ClientSocket,on_port:tokio::sync::watch::Receiver<Option<u16>>) -> Router<immt_lsp::ServerWrapper<Self>> {
    let _ = GLOBAL_STATE.set(LSPState::default());
    let server = immt_lsp::ServerWrapper::new(Self {client,on_port,workspaces:Vec::new()});
    server.router()
    //let /*mut*/ router = Router::from_language_server();
    //router.event(Self::on_tick);
    //router
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

  let debug = immt_system::settings::Settings::get().debug;

  tracing_subscriber::fmt()
    .with_max_level(Level::INFO)//(if debug {Level::TRACE} else {Level::INFO})
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
        format!("http://{}:{}",settings.ip,settings.port())
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
        Some(u) => LoginState::User{name:u.username,avatar:u.avatar_url.unwrap_or_default(),is_admin:u.is_admin}
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

#[tokio::test]
async fn linter() {
  tracing_subscriber::fmt().init();
  let _ce = color_eyre::install();
  let mut spec = immt_system::settings::SettingsSpec::default();
  spec.lsp = true;
  immt_system::initialize(spec);
  let state = LSPState::default();
  let _ = GLOBAL_STATE.set(state.clone());
  tracing::info!("Waiting for stex to load...");
  std::thread::sleep(std::time::Duration::from_secs(6));
  tracing::info!("Go!");
  let (_,t) = measure(move || {
    tracing::info!("Loading all archives");
    let mut files = Vec::new();
    for a in GlobalBackend::get().all_archives().iter() {
      if let Archive::Local(a) =a { 
        a.with_sources(|d| for e in <_ as TreeChildIter<SourceDir>>::dfs(d.children.iter()) {
          match e {
            SourceEntry::File(f) => files.push((
              f.relative_path.split('/').fold(a.source_dir(),|p,s| p.join(s)).into(),
              DocumentURI::from_archive_relpath(a.uri().owned(), &f.relative_path)
          )),
            _ => {}
          }
        })
      }
    }
    let len = files.len();
    tracing::info!("Linting {len} files");
    state.load_all(files.into_iter()/*.enumerate().map(|(i,(path,uri))| {
      tracing::info!("{}/{len}: {}",i+1,path.display());
      (path,uri)
    })*/, |_,_| {});
  });
  tracing::info!("initialized after {t}");
}