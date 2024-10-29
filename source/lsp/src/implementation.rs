#![allow(clippy::cognitive_complexity)]

use std::ops::ControlFlow;

use super::{IMMTLSPServer,ServerWrapper};
use async_lsp::{lsp_types as lsp, LanguageServer, ResponseError};
use futures::future::BoxFuture;

macro_rules! impl_request {
  ($name:ident = $struct:ident) => {
      #[must_use]
      fn $name(&mut self, params: <lsp::request::$struct as lsp::request::Request>::Params) -> Res<<lsp::request::$struct as lsp::request::Request>::Result> {
          tracing::info!("LSP: {params:?}");
          Box::pin(std::future::ready(Err(
              ResponseError::new(
                  async_lsp::ErrorCode::METHOD_NOT_FOUND,
                  format!("No such method: {}", <lsp::request::$struct as lsp::request::Request>::METHOD)
              )
          )))
      }
  }
}

macro_rules! impl_notification {
  (! $name:ident = $struct:ident) => {
      #[must_use]
      fn $name(&mut self, params: <lsp::notification::$struct as lsp::notification::Notification>::Params) -> Self::NotifyResult {
          tracing::info!("LSP: {params:?}");
          ControlFlow::Continue(())
      }
  };
  ($name:ident = $struct:ident) => {
      #[must_use]
      fn $name(&mut self, params: <lsp::notification::$struct as lsp::notification::Notification>::Params) -> Self::NotifyResult {
          tracing::info!("LSP: {params:?}");
          ControlFlow::Break(Err(async_lsp::Error::Routing(format!(
              "Unhandled notification: {}",
              <lsp::notification::$struct as lsp::notification::Notification>::METHOD,
          ))))
      }
  }
}

type Res<T> = BoxFuture<'static,Result<T,ResponseError>>;

impl<T:IMMTLSPServer> LanguageServer for ServerWrapper<T> {
  type Error = ResponseError;
  type NotifyResult = ControlFlow<async_lsp::Result<()>>;


  fn initialize(
    &mut self,
    _params: lsp::InitializeParams,
) -> Res<lsp::InitializeResult> {
    Box::pin(async move {
        tracing::info!("LSP: initialize");
        Ok(lsp::InitializeResult {
            capabilities: lsp::ServerCapabilities {
                position_encoding: Some(lsp::PositionEncodingKind::UTF16),
                text_document_sync: Some(lsp::TextDocumentSyncCapability::Options(lsp::TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(lsp::TextDocumentSyncKind::INCREMENTAL),
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    save: Some(lsp::TextDocumentSyncSaveOptions::Supported(true)),
                })),
                semantic_tokens_provider:Some(lsp::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    lsp::SemanticTokensRegistrationOptions {
                        text_document_registration_options:tdro(),
                        semantic_tokens_options:lsp::SemanticTokensOptions {
                            work_done_progress_options:lsp::WorkDoneProgressOptions { work_done_progress:Some(true) },
                            range:Some(true),
                            full:Some(lsp::SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            legend:lsp::SemanticTokensLegend {
                                token_types:vec![], // TODO
                                token_modifiers:vec![] // TODO
                            }
                        },
                        static_registration_options:lsp::StaticRegistrationOptions { id:Some("stex-sem-tokens".to_string()) }
                    }
                )),
                moniker_provider:Some(lsp::OneOf::Right(lsp::MonikerServerCapabilities::RegistrationOptions(
                    lsp::MonikerRegistrationOptions {
                        text_document_registration_options:tdro(),
                        moniker_options:lsp::MonikerOptions { work_done_progress_options:lsp::WorkDoneProgressOptions { work_done_progress:Some(true) } }
                    }
                ))),
                document_symbol_provider: Some(lsp::OneOf::Right(lsp::DocumentSymbolOptions {
                    label:Some("iMMT".to_string()),
                    work_done_progress_options:lsp::WorkDoneProgressOptions { work_done_progress:Some(true) }
                })),
                workspace_symbol_provider: Some(lsp::OneOf::Right(lsp::WorkspaceSymbolOptions {
                    work_done_progress_options:lsp::WorkDoneProgressOptions { work_done_progress:Some(true) },
                    resolve_provider:Some(true)
                })),
                workspace:Some(lsp::WorkspaceServerCapabilities {
                    workspace_folders:Some(lsp::WorkspaceFoldersServerCapabilities { supported:Some(true),change_notifications:Some(lsp::OneOf::Right("immt-change-listener".to_string())) }),
                    file_operations:Some(lsp::WorkspaceFileOperationsServerCapabilities {
                        did_create:Some(lsp::FileOperationRegistrationOptions { filters:vec![
                            lsp::FileOperationFilter {scheme:Some("file".to_string()),pattern:lsp::FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(lsp::FileOperationPatternKind::File),
                                options:None
                            }}
                        ]}),
                        did_rename:Some(lsp::FileOperationRegistrationOptions { filters:vec![
                            lsp::FileOperationFilter {scheme:Some("file".to_string()),pattern:lsp::FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(lsp::FileOperationPatternKind::File),
                                options:None
                            }}
                        ]}),
                        did_delete:Some(lsp::FileOperationRegistrationOptions { filters:vec![
                            lsp::FileOperationFilter {scheme:Some("file".to_string()),pattern:lsp::FileOperationPattern {
                                glob:"**/*.tex".to_string(),
                                matches:Some(lsp::FileOperationPatternKind::File),
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
    fn initialized(&mut self, _params: lsp::InitializedParams) -> Self::NotifyResult {
        tracing::info!("LSP: initialized");
        self.inner.initialized();
        /*
         */
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
    fn did_open(&mut self, params: lsp::DidOpenTextDocumentParams) -> Self::NotifyResult {
        let document = params.text_document;
        tracing::info!("URI: {}",document.uri);
        tracing::info!("language: {}",document.language_id);
        tracing::info!("version: {}",document.version);
        tracing::info!("text:\n{}",document.text);
        ControlFlow::Continue(())
    }
    #[must_use]
    //impl_notification!(! did_change = DidChangeTextDocument);
    fn did_change(&mut self, params: lsp::DidChangeTextDocumentParams) -> Self::NotifyResult {
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
    fn document_symbol(&mut self, params: lsp::DocumentSymbolParams) -> Res<Option<lsp::DocumentSymbolResponse>> {
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
        fn semantic_tokens_full(&mut self, params: lsp::SemanticTokensParams) -> Res<Option<lsp::SemanticTokensResult>> {
            tracing::info_span!("semantic_tokens_full").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}",params.work_done_progress_params);
                tracing::info!("partial_results: {:?}",params.partial_result_params);
                tracing::info!("uri: {}",params.text_document.uri);
                Box::pin(std::future::ready(Ok(None)))
            })
        }

        #[must_use]
        // impl_request!(semantic_tokens_full_delta = SemanticTokensFullDeltaRequest);
        fn semantic_tokens_full_delta(&mut self, params: lsp::SemanticTokensDeltaParams) -> Res<Option<lsp::SemanticTokensFullDeltaResult>> {
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
        fn semantic_tokens_range(&mut self, params: lsp::SemanticTokensRangeParams) -> Res<Option<lsp::SemanticTokensRangeResult>> {
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





fn tdro() -> lsp::TextDocumentRegistrationOptions {
  lsp::TextDocumentRegistrationOptions {
      document_selector:Some(vec![
          lsp::DocumentFilter { language:Some("tex".to_string()),scheme:Some("file".to_string()),pattern:None },
          lsp::DocumentFilter { language:Some("latex".to_string()),scheme:Some("file".to_string()),pattern:None },
      ])
  }
}