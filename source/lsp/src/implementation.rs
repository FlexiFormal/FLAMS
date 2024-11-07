#![allow(clippy::cognitive_complexity)]

use std::ops::ControlFlow;

use crate::{to_diagnostic, LSPStore};

use super::{IMMTLSPServer,ServerWrapper};
use async_lsp::{lsp_types::{self as lsp}, LanguageClient, LanguageServer, ResponseError};
use futures::{future::BoxFuture, FutureExt};

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
  };
  (! $name:ident = $struct:ident => ($default:expr)) => {
      #[must_use]
      fn $name(&mut self, params: <lsp::request::$struct as lsp::request::Request>::Params) -> Res<<lsp::request::$struct as lsp::request::Request>::Result> {
          tracing::info!("LSP: {params:?}");
          Box::pin(std::future::ready(Ok($default)))
      }
  };
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
        params: lsp::InitializeParams,
    ) -> Res<lsp::InitializeResult> {
        tracing::info!("LSP: initialize");
        self.inner.initialize(
            params.workspace_folders
            .unwrap_or_default().into_iter()
            .map(|f| (f.name,f.uri))
        );
        Box::pin(std::future::ready({
            Ok(lsp::InitializeResult {
                capabilities: super::capabilities::capabilities(),
                server_info:None
            })
        }))
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
    impl_notification!(! did_change_workspace_folders = DidChangeWorkspaceFolders);
    impl_notification!(! did_change_configuration = DidChangeConfiguration);
    impl_notification!(! did_change_watched_files = DidChangeWatchedFiles);
    impl_notification!(! did_create_files = DidCreateFiles);
    impl_notification!(! did_rename_files = DidRenameFiles);
    impl_notification!(! did_delete_files = DidDeleteFiles);

    // textDocument/
    #[must_use]
    //impl_notification!(! did_open = DidOpenTextDocument);
    fn did_open(&mut self, params: lsp::DidOpenTextDocumentParams) -> Self::NotifyResult {
        let document = params.text_document;
        tracing::trace!("URI: {}, language: {}, version: {}, text: \n{}",
            document.uri,
            document.language_id,
            document.version,
            document.text
        );
        self.inner.state().insert(document.uri,document.text);
        ControlFlow::Continue(())
    }
    #[must_use]
    #[allow(clippy::let_underscore_future)]
    //impl_notification!(! did_change = DidChangeTextDocument);
    fn did_change(&mut self, params: lsp::DidChangeTextDocumentParams) -> Self::NotifyResult {
        let document = params.text_document;
        if let Some(d) = self.inner.state().get(&document.uri) {
            for change in params.content_changes {
                tracing::trace!("URI: {},version: {}, text: \"{}\", range: {:?}",
                    document.uri,
                    document.version,
                    change.text,
                    change.range
                );
                d.delta(change.text, change.range);
            }
            let mut client = self.inner.client().clone();
            let _ = tokio::spawn(d.with_annots(LSPStore(self.inner.state().clone()),move |a| {
                let r = lsp::PublishDiagnosticsParams {
                    uri: document.uri,
                    diagnostics:  a.diagnostics.iter().map(to_diagnostic).collect(),
                    version: None,
                };
                let _ = client.publish_diagnostics(r);
            }));
            //d.with_text(|t| tracing::info!("new text:\n{}",t));
        } else {
            tracing::warn!("document not found: {}",document.uri);
        }
        ControlFlow::Continue(())
    }

    impl_notification!(! will_save = WillSaveTextDocument);
    impl_notification!(! did_save = DidSaveTextDocument);
    impl_notification!(! did_close = DidCloseTextDocument);

    // window/
        // workDoneProgress/
        impl_notification!(work_done_progress_cancel = WorkDoneProgressCancel);

    // $/
    impl_notification!(! set_trace = SetTrace);
    impl_notification!(! cancel_request = Cancel);
    impl_notification!(! progress = Progress);


    // Requests -----------------------------------------------

    // textDocument/

    #[must_use]
    // impl_request!(document_symbol = DocumentSymbolRequest);
    fn document_symbol(&mut self, params: lsp::DocumentSymbolParams) -> Res<Option<lsp::DocumentSymbolResponse>> {
        tracing::trace_span!("document_symbol").in_scope(move || {
            tracing::trace!("uri: {},work_done_progress_params: {:?}, partial_results: {:?}",
                params.text_document.uri,
                params.work_done_progress_params,
                params.partial_result_params
            );
            let p = params.work_done_progress_params.work_done_token.map(
                |tk| self.get_progress(tk)
            );
            self.inner.state().get_symbols(&params.text_document.uri,p)
                .map_or_else(|| Box::pin(std::future::ready(Ok(None))) as _,
                |f| Box::pin(f.map(Result::Ok)) as _
                )
        })
    }

    #[must_use]
    // impl_request!(! document_diagnostic = DocumentDiagnosticRequest => (lsp::DocumentDiagnosticReportResult::Report(lsp::DocumentDiagnosticReport::Full(lsp::RelatedFullDocumentDiagnosticReport::default()))));
    fn document_diagnostic(&mut self, params: lsp::DocumentDiagnosticParams) -> Res<lsp::DocumentDiagnosticReportResult> {
        fn default() -> lsp::DocumentDiagnosticReportResult { lsp::DocumentDiagnosticReportResult::Report(
            lsp::DocumentDiagnosticReport::Full(
                lsp::RelatedFullDocumentDiagnosticReport::default()
            )
        )}
        tracing::trace_span!("document_diagnostics").in_scope(move || {
            tracing::trace!("work_done_progress_params: {:?}, partial_results: {:?}, position: {:?}, context: {:?}",
                params.work_done_progress_params,
                params.partial_result_params,
                params.text_document,
                params.identifier
            );

            let p = params.work_done_progress_params.work_done_token.map(
                |tk| self.get_progress(tk)
            );
            self.inner.state().get_diagnostics(&params.text_document.uri,p)
                .map_or_else(|| Box::pin(std::future::ready(Ok(default()))) as _,
                |f| Box::pin(f.map(Result::Ok)) as _
            )
        })
    }

    #[must_use]
    //impl_request!(references = References);
    fn references(&mut self, params: lsp::ReferenceParams) -> Res<Option<Vec<lsp::Location>>> {
        tracing::info_span!("references").in_scope(move || {
            tracing::info!("work_done_progress_params: {:?}, partial_results: {:?}, position: {:?}, context: {:?}",
                params.work_done_progress_params,
                params.partial_result_params,
                params.text_document_position,
                params.context
            );
            if let Some(p) = params.work_done_progress_params.work_done_token {
                self.get_progress(p).finish();
            }
            if let Some(p) = params.partial_result_params.partial_result_token {
                self.get_progress(p).finish();
            }
            Box::pin(std::future::ready(Ok(Some(Vec::new()))))
        })
    }


    #[must_use]
    //impl_request!(! document_link = DocumentLinkRequest => (None));
    fn document_link(&mut self, params: lsp::DocumentLinkParams) -> Res<Option<Vec<lsp::DocumentLink>>> {
        tracing::info_span!("document_link").in_scope(move || {
            tracing::info!("uri: {},work_done_progress_params: {:?}, partial_results: {:?}",
                params.text_document.uri,
                params.work_done_progress_params,
                params.partial_result_params
            );
            let p = params.work_done_progress_params.work_done_token.map(
                |tk| self.get_progress(tk)
            );
            self.inner.state().get_links(&params.text_document.uri,p)
                .map_or_else(|| Box::pin(std::future::ready(Ok(None))) as _,
                |f| Box::pin(f.map(Result::Ok)) as _
                )
        })
    }

    impl_request!(! workspace_diagnostic = WorkspaceDiagnosticRequest => (lsp::WorkspaceDiagnosticReportResult::Report(lsp::WorkspaceDiagnosticReport {items:Vec::new()})));
/*
    #[must_use]
    fn workspace_diagnostic(&mut self, params: lsp::WorkspaceDiagnosticParams) -> Res<lsp::WorkspaceDiagnosticReportResult> {
        tracing::info_span!("workspace_diagnostics").in_scope(move || {
            tracing::info!("work_done_progress_params: {:?}, partial_results: {:?}, identifier: {:?}, previous_results_id: {:?}",
                params.work_done_progress_params,
                params.partial_result_params,
                params.identifier,
                params.previous_result_ids
            );
            if let Some(_token) = params.partial_result_params.partial_result_token {
                if self.ws_diagnostics.load(Ordering::Relaxed) {
                    self.ws_diagnostics.store(false, Ordering::Relaxed);
                    return Box::pin(std::future::ready(Ok(
                        lsp::WorkspaceDiagnosticReportResult::Partial(lsp::WorkspaceDiagnosticReportPartialResult {
                            items:Vec::new()
                        })
                    )))
                }
                
                self.ws_diagnostics.store(true, Ordering::Relaxed);
                return Box::pin(std::future::ready(Ok(
                    lsp::WorkspaceDiagnosticReportResult::Report(lsp::WorkspaceDiagnosticReport {
                        items:Vec::new()
                    })
                )))
            }
    
            /*
            if let Some(p) = params.work_done_progress_params.work_done_token {
                self.get_progress(p).finish_delay();
            }
            if let Some(p) = params.partial_result_params.partial_result_token {
                self.get_progress(p).finish_delay();
            }
            */
            Box::pin(std::future::ready(Ok(
                lsp::WorkspaceDiagnosticReportResult::Report(lsp::WorkspaceDiagnosticReport {
                    items:Vec::new()
                })
            )))
        })
    }
*/


    impl_request!(! code_action = CodeActionRequest => (None));
    impl_request!(! inlay_hint = InlayHintRequest => (None));
    impl_request!(! code_lens = CodeLensRequest => (None));
    impl_request!(! document_highlight = DocumentHighlightRequest => (None));
    impl_request!(! folding_range = FoldingRangeRequest => (None));
    impl_request!(! hover = HoverRequest => (None));
    impl_request!(! definition = GotoDefinition => (None));
    
    impl_request!(implementation = GotoImplementation);
    impl_request!(type_definition = GotoTypeDefinition);
    impl_request!(declaration = GotoDeclaration);
    impl_request!(document_color = DocumentColor);
    impl_request!(color_presentation = ColorPresentationRequest);
    impl_request!(selection_range = SelectionRangeRequest);
    impl_request!(moniker = MonikerRequest);
    impl_request!(inline_value = InlineValueRequest);
    impl_request!(on_type_formatting = OnTypeFormatting);
    impl_request!(range_formatting = RangeFormatting);
    impl_request!(formatting = Formatting);
    impl_request!(prepare_rename = PrepareRenameRequest);
    impl_request!(rename = Rename);
    impl_request!(prepare_type_hierarchy = TypeHierarchyPrepare);
    impl_request!(will_save_wait_until = WillSaveWaitUntil);
    impl_request!(completion = Completion);
    impl_request!(signature_help = SignatureHelpRequest);
    impl_request!(linked_editing_range = LinkedEditingRange);
    impl_request!(prepare_call_hierarchy = CallHierarchyPrepare);
        // semanticTokens/
        #[must_use]
        // impl_request!(semantic_tokens_full = SemanticTokensFullRequest);
        fn semantic_tokens_full(&mut self, params: lsp::SemanticTokensParams) -> Res<Option<lsp::SemanticTokensResult>> {
            tracing::info_span!("semantic_tokens_full").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}, partial_results: {:?}, uri: {}",
                    params.work_done_progress_params,
                    params.partial_result_params,
                    params.text_document.uri
                );
                let p = params.work_done_progress_params.work_done_token.map(
                    |tk| self.get_progress(tk)
                );
                self.inner.state().get_semantic_tokens(&params.text_document.uri,p,None)
                    .map_or_else(|| Box::pin(std::future::ready(Ok(None))) as _,
                    |f| Box::pin(f.map(|r| Ok(r.map(|r| lsp::SemanticTokensResult::Tokens(r))))) as _
                    )
            })
        }

        #[must_use]
        // impl_request!(semantic_tokens_full_delta = SemanticTokensFullDeltaRequest);
        fn semantic_tokens_full_delta(&mut self, params: lsp::SemanticTokensDeltaParams) -> Res<Option<lsp::SemanticTokensFullDeltaResult>> {
            tracing::info_span!("semantic_tokens_full_delta").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}, partial_results: {:?}, previous_result_id: {:?}, uri:{}",
                    params.work_done_progress_params,
                    params.partial_result_params,
                    params.previous_result_id,
                    params.text_document.uri
                );
                Box::pin(std::future::ready(Ok(None)))
            })
        }

        #[must_use]
        // impl_request!(semantic_tokens_range = SemanticTokensRangeRequest);
        fn semantic_tokens_range(&mut self, params: lsp::SemanticTokensRangeParams) -> Res<Option<lsp::SemanticTokensRangeResult>> {
            tracing::info_span!("semantic_tokens_range").in_scope(|| {
                tracing::info!("work_done_progress_params: {:?}, partial_results: {:?}, range: {:?}, uri:{}",
                    params.work_done_progress_params,
                    params.partial_result_params,
                    params.range,
                    params.text_document.uri
                );
                let p = params.work_done_progress_params.work_done_token.map(
                    |tk| self.get_progress(tk)
                );
                self.inner.state().get_semantic_tokens(&params.text_document.uri,p,Some(params.range))
                    .map_or_else(|| Box::pin(std::future::ready(Ok(None))) as _,
                    |f| Box::pin(f.map(|r| Ok(r.map(|r| lsp::SemanticTokensRangeResult::Tokens(r))))) as _
                    )
            })
        }

    // callHierarchy/
    impl_request!(incoming_calls = CallHierarchyIncomingCalls);
    impl_request!(outgoing_calls = CallHierarchyOutgoingCalls);

    // workspace/
    impl_request!(will_create_files = WillCreateFiles);
    impl_request!(will_rename_files = WillRenameFiles);
    impl_request!(will_delete_files = WillDeleteFiles);
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