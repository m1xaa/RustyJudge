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
pub use cfg::{build_semantic_model_from_root, compute_liveness, print_liveness, print_statement_liveness};
pub use parser::*;
