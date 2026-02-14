pub mod parser;
mod cli;
mod rules;
mod diagnostics;
mod linter;

pub use parser::parse_js_file;
pub use cli::parse_cli;
pub use rules::*;
pub use linter::run_cli;