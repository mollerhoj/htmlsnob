use htmlsnob::{format, lint};
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    state: Mutex<State>,
    client: Client,
}

#[derive(Debug)]
struct State {
    ast: Vec<htmlsnob::ast::Node>,
    config: htmlsnob::config::Config,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(false)),
                // Configure sync to run only on save
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::NONE),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File opened!")
            .await;

        // Run initial diagnostics when file is opened
        self.check_document(&params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File saved!")
            .await;

        // Only check the document when it's saved
        if let Some(text) = params.text {
            self.check_document(&params.text_document.uri, &text).await;
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.client
            .log_message(MessageType::INFO, "Formatting document requested")
            .await;

        // Get document content from file
        let text = std::fs::read_to_string(params.text_document.uri.to_file_path().unwrap())
            .expect("Failed to read file");

        // Format the entire document
        let state = self.state.lock().await;
        let formatted_text = format(&state.ast, &state.config);
        drop(state);

        // If no changes were made, return None
        if formatted_text == text {
            return Ok(None);
        }

        // TEST: Let's always just return None
        Ok(None)

        //// Create a TextEdit that replaces the entire document
        //let edit = TextEdit {
        //    range: Range {
        //        start: Position {
        //            line: 0,
        //            character: 0,
        //        },
        //        end: Position {
        //            // This effectively covers the entire document
        //            line: u32::MAX, // A very large number to ensure the entire document is covered
        //            character: 0,
        //        },
        //    },
        //    new_text: formatted_text,
        //};

        //Ok(Some(vec![edit]))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    async fn check_document(&self, uri: &Url, content: &str) {
        let template_language =
            htmlsnob::template_language::TemplateLanguage::from_filename(uri.path());

        let mut state = self.state.lock().await;
        state.config.options.template_language = template_language;
        let (ast, warnings) = lint(content, &mut state.config);
        state.ast = ast;
        drop(state);

        // Convert the linter diagnostics to LSP diagnostics
        let diagnostics = warnings
            .into_iter()
            .flat_map(|warning| {
                warning
                    .areas
                    .iter()
                    .map(|area| {
                        // Create a diagnostic for each range
                        Diagnostic {
                            range: Self::area_to_range(area),
                            severity: Self::warning_severity_to_diagnostic_severity(
                                &warning.severity,
                            ),
                            source: Some("htmlsnob-lsp".to_string()),
                            message: warning.message.clone(),
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Send the diagnostics to the client
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    fn area_to_range(area: &htmlsnob::ast::Area) -> Range {
        Range {
            start: Position {
                line: u32::try_from(area.start.line).unwrap(),
                character: u32::try_from(area.start.column).unwrap(),
            },
            end: Position {
                line: u32::try_from(area.end.line).unwrap(),
                character: u32::try_from(area.end.column).unwrap(),
            },
        }
    }

    fn warning_severity_to_diagnostic_severity(
        warning_severity: &htmlsnob::WarningSeverity,
    ) -> Option<DiagnosticSeverity> {
        match warning_severity {
            htmlsnob::WarningSeverity::ERROR => Some(DiagnosticSeverity::ERROR),
            htmlsnob::WarningSeverity::WARNING => Some(DiagnosticSeverity::WARNING),
            htmlsnob::WarningSeverity::INFORMATION => Some(DiagnosticSeverity::INFORMATION),
            htmlsnob::WarningSeverity::HINT => Some(DiagnosticSeverity::HINT),
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let registry = htmlsnob_rules::registry();

    // Load config from file
    let default_config = std::fs::read_to_string("default_config/default_config.toml")
        .expect("Failed to read config file");
    let attribute_name_whitelist =
        std::fs::read_to_string("default_config/attribute_name_whitelist.toml")
            .expect("Failed to read config file");
    let class_order = std::fs::read_to_string("default_config/class_order.toml")
        .expect("Failed to read config file");

    let config_string = format!(
        "{}\n{}\n{}",
        default_config, attribute_name_whitelist, class_order
    );

    let config = htmlsnob::config::Config::from_toml(&config_string, &registry);
    let (service, socket) = LspService::new(|client| Backend {
        state: Mutex::new(State {
            ast: Vec::new(),
            config: config,
        }),
        client,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
