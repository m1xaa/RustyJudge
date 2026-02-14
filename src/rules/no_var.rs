use rslint_parser::SyntaxKind;
use crate::diagnostics::Diagnostic;
use crate::rules::rule::{Rule, RuleContext};

pub struct NoVar;

impl Rule for NoVar {
    fn name(&self) -> &'static str {
        "no-var"
    }

    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for node in rule_context.root.descendants() {
            if node.kind() != SyntaxKind::VAR_DECL {
                continue;
            }
            for child in node.children_with_tokens() {
                if child.kind() != SyntaxKind::VAR_KW {
                    continue;
                }
                diagnostics.push(
                    rule_context.diagnostic_at(
                        node.text_range(), 
                        "Avoid using var. Use let or const instead.")
                );
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_context;

    #[test]
    fn detects_single_var_declaration() {
        let ctx = make_context("var x = 5;", None);
        let rule = NoVar;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_let() {
        let ctx = make_context("let x = 5;", None);
        let rule = NoVar;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_flag_const() {
        let ctx = make_context("const x = 5;", None);
        let rule = NoVar;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_var_declarations() {
        let ctx = make_context("
            var a = 1;
            var b = 2;
        ", None);

        let rule = NoVar;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn detects_var_inside_function() {
        let ctx = make_context("
            function test() {
                var x = 10;
            }
        ", None);

        let rule = NoVar;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_no_declarations() {
        let ctx = make_context("console.log('hello');", None);
        let rule = NoVar;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }
}
