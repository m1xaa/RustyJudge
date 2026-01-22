use rslint_parser::SyntaxKind;
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct NoConsoleLog;

impl Rule for NoConsoleLog {
    fn name(&self) -> &'static str {
        "no-console-log"
    }

    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for element in rule_context.root.descendants_with_tokens() {
            let token = match element.as_token() {
                Some(t) => t,
                None => continue,
            };

            if token.text() != "console" {
                continue;
            }

            let dot = match token.next_token() {
                Some(t) if t.text() == "." => t,
                _ => continue,
            };

            let log = match dot.next_token() {
                Some(t) if t.text() == "log" => t,
                _ => continue,
            };

            diagnostics.push(
                rule_context.diagnostic_at(token.text_range(), "Unexpected console.log call.\
                Remove debug logging before production")
            );
        }

        diagnostics
    }
}
