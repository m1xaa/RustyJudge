use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct MaxLineLength;

impl Rule for MaxLineLength {
    fn name(&self) -> &'static str {
        "max-line-length"
    }

    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (row_idx, line) in rule_context.source.lines().enumerate() {
            if line.len() > rule_context.max_line_length {
                diagnostics.push(Diagnostic::new(
                    row_idx + 1,
                    rule_context.max_line_length + 1,
                    format!(
                        "Line exceeds maximum length of {} characters",
                        rule_context.max_line_length
                    ),
                ));
            }
        }

        diagnostics
    }
}
