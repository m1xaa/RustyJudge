use std::collections::HashMap;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;
use crate::linter::{lint_file, Linter};
use crate::rules::default_rules;
use crate::diagnostics::Diagnostic as RjDiagnostic;

pub async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}


struct Backend {
    client: Client,
    docs: RwLock<HashMap<Url, String>>,
    linter: Linter,
    max_line_length: usize,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            docs: RwLock::new(HashMap::new()),
            linter: Linter::with_rules(default_rules()),
            max_line_length: 120,
        }
    }

    async fn lint_and_publish(&self, uri: &Url) {
        let Some(source) = self.docs.read().await.get(uri).cloned() else { return; };

        let file_name = uri.to_string();

        let diags = lint_file(
            source,
            file_name,
            self.max_line_length,
            &self.linter,
        ).unwrap_or_else(|e| vec![RjDiagnostic::new(1, 1, format!("Syntax error: {e}"))]);


        let lsp_diags: Vec<Diagnostic> = diags.iter().map(to_lsp_diag).collect();
        self.client.publish_diagnostics(uri.clone(), lsp_diags, None).await;
    }
}

fn to_lsp_diag(d: &RjDiagnostic) -> Diagnostic {
    let line = d.row.saturating_sub(1) as u32;
    let col = d.col.saturating_sub(1) as u32;

    Diagnostic {
        range: Range {
            start: Position { line, character: col },
            end: Position { line, character: col + 1 },
        },
        severity: Some(DiagnosticSeverity::WARNING),
        source: Some("rusty-judge".to_string()),
        message: d.message().to_string(),
        ..Diagnostic::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {

    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                    ..Default::default()
                })),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "rusty-judge-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "RustyJudge LSP initialized").await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.docs.write().await.insert(uri.clone(), text);
        self.lint_and_publish(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            self.docs.write().await.insert(uri.clone(), change.text);
            self.lint_and_publish(&uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.lint_and_publish(&params.text_document.uri).await;
    }
}
