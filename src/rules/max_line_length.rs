
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

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

#[cfg(test)]
mod tests {
    use rslint_parser::parse_text;
    use crate::rules::make_context;
    use super::*;


    #[test]
    fn no_diagnostic_when_lines_within_limit() {
        let ctx = make_context("let x = 5;", Option::from(20));
        let rule = MaxLineLength;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_line_exceeding_limit() {
        let ctx = make_context("12345678901", Option::from(10));
        let rule = MaxLineLength;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_multiple_long_lines() {
        let source = "short\n123456\nok\n1234567";
        let ctx = make_context(source, Option::from(5));
        let rule = MaxLineLength;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn uses_overridden_max_length_from_context() {
        let ctx = make_context("123456", Option::from(3));
        let rule = MaxLineLength;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }
}

