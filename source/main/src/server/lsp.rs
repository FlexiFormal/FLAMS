#![allow(clippy::cognitive_complexity)]

use std::ops::ControlFlow;

use async_lsp::{client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer, lsp_types::{request::Request, DocumentFilter, DocumentSymbolOptions, FileOperationFilter, FileOperationPattern, FileOperationPatternKind, FileOperationRegistrationOptions, MonikerOptions, MonikerRegistrationOptions, MonikerServerCapabilities, PositionEncodingKind, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensRegistrationOptions, SemanticTokensServerCapabilities, StaticRegistrationOptions, TextDocumentRegistrationOptions, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, WorkDoneProgressOptions, WorkspaceFileOperationsServerCapabilities, WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities, WorkspaceSymbolOptions}, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer, ClientSocket, ErrorCode, LanguageServer, ResponseError};
use futures::future::BoxFuture;
use immt_system::settings::Settings;
use tower::ServiceBuilder;
use tracing::Level;
use async_lsp::lsp_types as types;

//struct TickEvent;
struct IMMTLSPServer {
  client:ClientSocket,
  on_port:tokio::sync::watch::Receiver<Option<u16>>
}

impl IMMTLSPServer {
  #[allow(clippy::let_and_return)]
  fn new_router(client:ClientSocket,on_port:tokio::sync::watch::Receiver<Option<u16>>) -> Router<Self> {
    let /*mut*/ router = Router::from_language_server(Self {client,on_port});
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
  let (server,_client) = async_lsp::MainLoop::new_server(|client| {
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
      .service(IMMTLSPServer::new_router(client,on_port))
  });

  tracing_subscriber::fmt()
    .with_max_level(Level::INFO)//if debug {Level::TRACE} else {Level::INFO})
    .with_ansi(false)
    .with_target(true)
    .with_writer(std::io::stderr)
    .init();
  #[cfg(unix)]
  let (stdin,stdout) = (
    async_lsp::stdio::PipeStdin::lock_tokio().expect("Failed to lock stdin"),
    async_lsp::stdio::PipeStdout::lock_tokio().expect("Failed to lock stdout")
  );
  #[cfg(not(unix))]
  let (stdin,stdout) = (
    tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
    tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout())
  );

  server.run_buffered(stdin, stdout).await.expect("Failed to run server");
}


/*

use std::ops::DerefMut;

use async_lsp::ServerSocket;

async fn register(
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
  todo!()

}


struct STeXLSP {
  inner:ServerSocket
}


struct WS {
  inner:axum::extract::ws::WebSocket,
  read_buf:Vec<u8>
}
impl Deref for WS {
  type Target = axum::extract::ws::WebSocket;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
impl DerefMut for WS {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl WS {
  fn actually_poll(
    self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
      buf: &mut [u8],
  ) -> std::task::Poll<std::io::Result<usize>> {
    use axum::extract::ws::Message;

    let len = buf.len();

    futures::Stream::poll_next(self.as_mut(), cx).map(|r| match r {
      None => Ok(0),
      Some(Err(e)) => Err(e),
      Some(Ok(Message::Text(s))) => {
        let slen = s.len();
        if slen > len {
          buf.copy_from_slice(&s[..len]);
          self.read_buf.extend_from_slice(&s[len..]);
          return Ok(len)
        }
        buf[..slen].copy_from_slice(&s);
        Ok(slen)
      }
      Some(Ok(Message::Binary(b))) => {
        let slen = b.len();
        if slen > len {
          buf.copy_from_slice(&b[..len]);
          self.read_buf.extend_from_slice(&b[len..]);
          return Ok(len)
        }
        buf[..slen].copy_from_slice(&b);
        Ok(slen)
      }
      Some()
    })
  }
}

impl futures::AsyncRead for WS {
  fn poll_read(
      self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
      buf: &mut [u8],
  ) -> std::task::Poll<std::io::Result<usize>> {
      let slf = self.get_mut();
      if slf.read_buf.is_empty() {
        drop(slf);
        return self.actually_poll(cx, buf)
      }

      let buf_len = buf.len();
      let len = slf.read_buf.len();
      if len > buf_len {
        buf.copy_from_slice(&slf.read_buf[..buf_len]);
        slf.read_buf.drain(..buf_len);
        return std::task::Poll::Ready(Ok(len))
      }
      buf_len[..len].copy_from_slice(&slf.read_buf);
      slf.read_buf.clear();
      match self.actually_poll(cx, &mut buf[len..]) {
        std::task::Poll::Ready(Ok(n)) => std::task::Poll::Ready(Ok(len + n)),
        _ => std::task::Poll::Ready(Ok(len))
      }
  }
}

impl futures::AsyncBufRead for WS {
  fn poll_fill_buf(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<&[u8]>> {
    loop {
      match self.get_mut().actually_poll(cx, &mut []) {
        std::task::Poll::Ready(Ok(0)) => (),
        std::task::Poll::Ready(Ok(_)) => unreachable!(),
        std::task::Poll::Ready(Err(e)) => return std::task::Poll::Ready(Err(e)),
        std::task::Poll::Pending => break
      }
    }
    let buf = self.as_ref().read_buf.as_slice();
    if buf.is_empty() {
      std::task::Poll::Pending
    } else {
      std::task::Poll::Ready(Ok(buf))
    }
  }
  fn consume(self: std::pin::Pin<&mut Self>, amt: usize) {
    self.get_mut().read_buf.drain(..amt);
  }
}

impl futures::AsyncWrite for WS {
  fn poll_write(
      self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
      buf: &[u8],
  ) -> std::task::Poll<std::io::Result<usize>> {
    use futures::{Stream,StreamExt,Sink,SinkExt};
      SinkExt::
  }
}
  */


macro_rules! impl_request {
    ($name:ident = $struct:ident) => {
        #[must_use]
        fn $name(&mut self, params: <types::request::$struct as types::request::Request>::Params) -> Res<<types::request::$struct as types::request::Request>::Result> {
            tracing::info!("LSP: {params:?}");
            Box::pin(std::future::ready(Err(
                ResponseError::new(
                    ErrorCode::METHOD_NOT_FOUND,
                    format!("No such method: {}", <types::request::$struct>::METHOD)
                )
            )))
        }
    }
}

macro_rules! impl_notification {
    (! $name:ident = $struct:ident) => {
        #[must_use]
        fn $name(&mut self, params: <types::notification::$struct as types::notification::Notification>::Params) -> Self::NotifyResult {
            tracing::info!("LSP: {params:?}");
            ControlFlow::Continue(())
        }
    };
    ($name:ident = $struct:ident) => {
        #[must_use]
        fn $name(&mut self, params: <types::notification::$struct as types::notification::Notification>::Params) -> Self::NotifyResult {
            tracing::info!("LSP: {params:?}");
            ControlFlow::Break(Err(async_lsp::Error::Routing(format!(
                "Unhandled notification: {}",
                <types::notification::$struct as types::notification::Notification>::METHOD,
            ))))
        }
    }
}

fn tdro() -> TextDocumentRegistrationOptions {
    TextDocumentRegistrationOptions {
        document_selector:Some(vec![
            DocumentFilter { language:Some("tex".to_string()),scheme:Some("file".to_string()),pattern:None },
            DocumentFilter { language:Some("latex".to_string()),scheme:Some("file".to_string()),pattern:None },
        ])
    }
}

impl LanguageServer for IMMTLSPServer {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        _params: types::InitializeParams,
    ) -> Res<types::InitializeResult> {
        Box::pin(async move {
        tracing::info!("LSP: initialize");
        Ok(types::InitializeResult {
            capabilities: types::ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                })),
                semantic_tokens_provider:Some(SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions {
                        text_document_registration_options:tdro(),
                        semantic_tokens_options:SemanticTokensOptions {
                            work_done_progress_options:WorkDoneProgressOptions { work_done_progress:Some(true) },
                            range:Some(true),
                            full:Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            legend:SemanticTokensLegend {
                                token_types:vec![], // TODO
                                token_modifiers:vec![] // TODO
                            }
                        },
                        static_registration_options:StaticRegistrationOptions { id:Some("stex-sem-tokens".to_string()) }
                    }
                )),
                moniker_provider:Some(types::OneOf::Right(MonikerServerCapabilities::RegistrationOptions(
                    MonikerRegistrationOptions {
                        text_document_registration_options:tdro(),
                        moniker_options:MonikerOptions { work_done_progress_options:WorkDoneProgressOptions { work_done_progress:Some(true) } }
                    }
                ))),
                document_symbol_provider: Some(types::OneOf::Right(DocumentSymbolOptions {
                    label:Some("iMMT".to_string()),
                    work_done_progress_options:WorkDoneProgressOptions { work_done_progress:Some(true) }
                })),
                workspace_symbol_provider: Some(types::OneOf::Right(WorkspaceSymbolOptions {
                    work_done_progress_options:WorkDoneProgressOptions { work_done_progress:Some(true) },
                    resolve_provider:Some(true)
                })),
                workspace:Some(WorkspaceServerCapabilities {
                    workspace_folders:Some(WorkspaceFoldersServerCapabilities { supported:Some(true),change_notifications:Some(types::OneOf::Right("immt-change-listener".to_string())) }),
                    file_operations:Some(WorkspaceFileOperationsServerCapabilities {
                        did_create:Some(FileOperationRegistrationOptions { filters:vec![
                            FileOperationFilter {scheme:Some("file".to_string()),pattern:FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(FileOperationPatternKind::File),
                                options:None
                            }}
                        ]}),
                        did_rename:Some(FileOperationRegistrationOptions { filters:vec![
                            FileOperationFilter {scheme:Some("file".to_string()),pattern:FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(FileOperationPatternKind::File),
                                options:None
                            }}
                        ]}),
                        did_delete:Some(FileOperationRegistrationOptions { filters:vec![
                            FileOperationFilter {scheme:Some("file".to_string()),pattern:FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(FileOperationPatternKind::File),
                                options:None
                            }}
                        ]}),
                        will_create:None,
                        will_delete:None,
                        will_rename:None
                    })
                }),

                selection_range_provider: None,
                hover_provider: None,
                completion_provider: None,
                signature_help_provider: None,
                definition_provider: None,
                type_definition_provider: None,
                implementation_provider: None,
                references_provider: None,
                document_highlight_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: None,
                document_link_provider: None,
                color_provider:None,
                folding_range_provider:None,
                declaration_provider:None,
                execute_command_provider:None,
                call_hierarchy_provider:None,
                linked_editing_range_provider:None,
                inline_value_provider:None,
                inlay_hint_provider:None,
                diagnostic_provider:None,
                //inline_completion_provider:None,
                experimental:None
            },
            server_info:None
        })
        })
    }

    #[must_use]
    fn shutdown(
        &mut self,
        (): (),
    ) -> Res<()> {
        tracing::info!("LSP: shutdown");
        Box::pin(std::future::ready(Ok(())))
    }

    // Notifications -------------------------------------------

    #[must_use]
    //impl_notification!(! initialized = Initialized);
    fn initialized(&mut self, _params: types::InitializedParams) -> Self::NotifyResult {
        tracing::info!("LSP: initialized");
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
      ControlFlow::Continue(())
    }

    impl_notification!(! exit = Exit);

    // workspace/
    impl_notification!(did_change_workspace_folders = DidChangeWorkspaceFolders);
    impl_notification!(did_change_configuration = DidChangeConfiguration);
    impl_notification!(did_change_watched_files = DidChangeWatchedFiles);
    impl_notification!(did_create_files = DidCreateFiles);
    impl_notification!(did_rename_files = DidRenameFiles);
    impl_notification!(did_delete_files = DidDeleteFiles);

    // textDocument/
    #[must_use]
    //impl_notification!(! did_open = DidOpenTextDocument);
    fn did_open(&mut self, params: types::DidOpenTextDocumentParams) -> Self::NotifyResult {
        let document = params.text_document;
        tracing::info!("URI: {}",document.uri);
        tracing::info!("language: {}",document.language_id);
        tracing::info!("version: {}",document.version);
        tracing::info!("text:\n{}",document.text);
        ControlFlow::Continue(())
    }
    #[must_use]
    //impl_notification!(! did_change = DidChangeTextDocument);
    fn did_change(&mut self, params: types::DidChangeTextDocumentParams) -> Self::NotifyResult {
        let document = params.text_document;
        tracing::info!("URI: {}",document.uri);
        tracing::info!("version: {}",document.version);
        for change in params.content_changes {
            tracing::info!("text:\n{}",change.text);
            tracing::info!("range: {:?}",change.range);
        }
        ControlFlow::Continue(())
    }

    impl_notification!(will_save = WillSaveTextDocument);
    impl_notification!(! did_save = DidSaveTextDocument);
    impl_notification!(! did_close = DidCloseTextDocument);

    // window/
        // workDoneProgress/
        impl_notification!(work_done_progress_cancel = WorkDoneProgressCancel);

    // $/
    impl_notification!(! set_trace = SetTrace);
    impl_notification!(cancel_request = Cancel);
    impl_notification!(progress = Progress);


    // Requests -----------------------------------------------

    // textDocument/

    #[must_use]
    // impl_request!(document_symbol = DocumentSymbolRequest);
    fn document_symbol(&mut self, params: types::DocumentSymbolParams) -> Res<Option<types::DocumentSymbolResponse>> {
        tracing::info_span!("document_symbol").in_scope(|| {
            tracing::info!("work_done_progress_params: {:?}",params.work_done_progress_params);
            tracing::info!("partial_results: {:?}",params.partial_result_params);
            tracing::info!("uri: {}",params.text_document.uri);
            Box::pin(std::future::ready(Ok(None)))
        })
    }

    impl_request!(implementation = GotoImplementation);
    impl_request!(type_definition = GotoTypeDefinition);
    impl_request!(declaration = GotoDeclaration);
    impl_request!(definition = GotoDefinition);
    impl_request!(document_color = DocumentColor);
    impl_request!(color_presentation = ColorPresentationRequest);
    impl_request!(folding_range = FoldingRangeRequest);
    impl_request!(selection_range = SelectionRangeRequest);
    impl_request!(moniker = MonikerRequest);
    impl_request!(inline_value = InlineValueRequest);
    impl_request!(inlay_hint = InlayHintRequest);
    impl_request!(document_highlight = DocumentHighlightRequest);
    impl_request!(on_type_formatting = OnTypeFormatting);
    impl_request!(range_formatting = RangeFormatting);
    impl_request!(formatting = Formatting);
    impl_request!(prepare_rename = PrepareRenameRequest);
    impl_request!(rename = Rename);
    impl_request!(prepare_type_hierarchy = TypeHierarchyPrepare);
    impl_request!(document_diagnostic = DocumentDiagnosticRequest);
    impl_request!(will_save_wait_until = WillSaveWaitUntil);
    impl_request!(completion = Completion);
    impl_request!(hover = HoverRequest);
    impl_request!(signature_help = SignatureHelpRequest);
    impl_request!(references = References);
    impl_request!(code_action = CodeActionRequest);
    impl_request!(linked_editing_range = LinkedEditingRange);
    impl_request!(document_link = DocumentLinkRequest);
    impl_request!(code_lens = CodeLensRequest);
    impl_request!(prepare_call_hierarchy = CallHierarchyPrepare);
        // semanticTokens/
        #[must_use]
        // impl_request!(semantic_tokens_full = SemanticTokensFullRequest);
        fn semantic_tokens_full(&mut self, params: types::SemanticTokensParams) -> Res<Option<types::SemanticTokensResult>> {
            tracing::info_span!("semantic_tokens_full").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}",params.work_done_progress_params);
                tracing::info!("partial_results: {:?}",params.partial_result_params);
                tracing::info!("uri: {}",params.text_document.uri);
                Box::pin(std::future::ready(Ok(None)))
            })
        }

        #[must_use]
        // impl_request!(semantic_tokens_full_delta = SemanticTokensFullDeltaRequest);
        fn semantic_tokens_full_delta(&mut self, params: types::SemanticTokensDeltaParams) -> Res<Option<types::SemanticTokensFullDeltaResult>> {
            tracing::info_span!("semantic_tokens_full_delta").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}",params.work_done_progress_params);
                tracing::info!("partial_results: {:?}",params.partial_result_params);
                tracing::info!("previous_result_id: {:?}",params.previous_result_id);
                tracing::info!("uri: {}",params.text_document.uri);
                Box::pin(std::future::ready(Ok(None)))
            })
        }

        #[must_use]
        // impl_request!(semantic_tokens_range = SemanticTokensRangeRequest);
        fn semantic_tokens_range(&mut self, params: types::SemanticTokensRangeParams) -> Res<Option<types::SemanticTokensRangeResult>> {
            tracing::info_span!("semantic_tokens_range").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}",params.work_done_progress_params);
                tracing::info!("partial_results: {:?}",params.partial_result_params);
                tracing::info!("range: {:?}",params.range);
                tracing::info!("uri: {}",params.text_document.uri);
                Box::pin(std::future::ready(Ok(None)))
            })
        }

    // callHierarchy/
    impl_request!(incoming_calls = CallHierarchyIncomingCalls);
    impl_request!(outgoing_calls = CallHierarchyOutgoingCalls);

    // workspace/
    impl_request!(will_create_files = WillCreateFiles);
    impl_request!(will_rename_files = WillRenameFiles);
    impl_request!(will_delete_files = WillDeleteFiles);
    impl_request!(workspace_diagnostic = WorkspaceDiagnosticRequest);
    impl_request!(symbol = WorkspaceSymbolRequest);
    impl_request!(execute_command = ExecuteCommand);

    // typeHierarchy/
    impl_request!(supertypes = TypeHierarchySupertypes);
    impl_request!(subtypes = TypeHierarchySubtypes);

    // inlayHint/
    impl_request!(inlay_hint_resolve = InlayHintResolveRequest);

    // completionItem/
    impl_request!(completion_item_resolve = ResolveCompletionItem);

    // codeAction/
    impl_request!(code_action_resolve = CodeActionResolveRequest);

    // workspaceSymbol/
    impl_request!(workspace_symbol_resolve = WorkspaceSymbolResolve);

    // codeLens/
    impl_request!(code_lens_resolve = CodeLensResolve);

    // documentLink/
    impl_request!(document_link_resolve = DocumentLinkResolve);
}

type Res<T> = BoxFuture<'static,Result<T,ResponseError>>;


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