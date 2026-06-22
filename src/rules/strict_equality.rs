use rslint_parser::SyntaxKind;
use crate::diagnostics::Diagnostic;
use crate::rules::{Rule, RuleContext};

pub struct StrictEquality;

impl Rule for StrictEquality {
    fn name(&self) -> &'static str {
        "strict-equality"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for element in ctx.root.descendants_with_tokens() {
            match element.kind() {
                SyntaxKind::EQ2 => {
                    diagnostics.push(
                        ctx.diagnostic_at(
                            element.text_range(),
                            "Use === instead of == for strict equality",
                        )
                    );
                }
                SyntaxKind::NEQ => {
                    diagnostics.push(
                        ctx.diagnostic_at(
                            element.text_range(),
                            "Use !== instead of != for strict equality",
                        )
                    );
                }
                _ => {}
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::make_context;
    use super::*;

    #[test]
    fn detects_double_equals() {
        let ctx = make_context("a == b;", None);
        let rule = StrictEquality;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_not_equals() {
        let ctx = make_context("a != b;", None);
        let rule = StrictEquality;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_strict_equals() {
        let ctx = make_context("a === b;", None);
        let rule = StrictEquality;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_strict_not_equals() {
        let ctx = make_context("a !== b;", None);
        let rule = StrictEquality;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_violations() {
        let ctx = make_context("
            if (a == b) {}
            if (c != d) {}
        ", None);

        let rule = StrictEquality;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn mixed_strict_and_non_strict() {
        let ctx = make_context("
            if (a == b) {}
            if (c === d) {}
            if (e != f) {}
        ", None);

        let rule = StrictEquality;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }
}

