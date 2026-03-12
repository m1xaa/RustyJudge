
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

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

            let _ = match dot.next_token() {
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

#[cfg(test)]
mod tests {
    use crate::rules::make_context;
    use super::*;


    #[test]
    fn detects_console_log_call() {
        let ctx = make_context("console.log('hello');", Option::None);
        let rule = NoConsoleLog;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_other_console_methods() {
        let ctx = make_context("console.error('fail');", Option::None);
        let rule = NoConsoleLog;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_similar_but_not_exact_pattern() {
        let ctx = make_context("myconsole.log('hello');", Option::None);
        let rule = NoConsoleLog;

        let diagnostics = rule.check(&ctx);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_console_logs() {
        let ctx = make_context(
            "console.log('a');\nconsole.log('b');",
            Option::None
        );
        let rule = NoConsoleLog;
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn detects_console_log_without_call_parentheses() {
        let ctx = make_context("console.log;", Option::None);
        let rule = NoConsoleLog;

        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
    }
}

