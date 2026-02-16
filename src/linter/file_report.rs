use serde::Serialize;
use crate::diagnostics::Diagnostic;

#[derive(Serialize)]
pub struct FileReport {
    pub file: String,
    pub diagnostics: Vec<Diagnostic>,
}
