use std::fmt;
use rslint_parser::{parse_text, SyntaxNode, TextRange};
use crate::diagnostics::Diagnostic;

pub trait Rule {
    fn name(&self) -> &'static str;
    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic>;
}

pub struct RuleContext {
    pub file_name: String,
    pub root: SyntaxNode,
    pub source: String,
    pub max_line_length: usize,
}

impl RuleContext {
    pub fn new(file_name: String, root: SyntaxNode, source: String, max_line_length: usize) -> Self {
        RuleContext { file_name, root, source, max_line_length }
    }

    pub fn diagnostic_at(
        &self,
        range: TextRange,
        message: impl Into<String>,
    ) -> Diagnostic {
        let offset: usize = range.start().into();
        let (line, col) = self.offset_to_row_col(offset);

        Diagnostic::new(
            line + 1,
            col + 1,
            message.into(),
        )
    }

    fn offset_to_row_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;

        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (line, col)
    }
}

impl fmt::Display for RuleContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {}", self.file_name)
    }
}


pub fn make_context(source: &str, max_len: Option<usize>) -> RuleContext {
    let max_len = max_len.unwrap_or(120);

    let parse = parse_text(source, 0).to_syntax();
    assert!(parse.errors().is_empty());

    let syntax = parse.syntax();
    RuleContext::new("test.js".to_string(), syntax, source.to_string(), max_len)
}
