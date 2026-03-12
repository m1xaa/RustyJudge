mod parser;
mod cli;
mod rules;
mod diagnostics;
mod linter;
mod run_cli;
mod lsp;
mod cfg;

pub use run_cli::run_cli;
pub use lsp::run_lsp_server;
