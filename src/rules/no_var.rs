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