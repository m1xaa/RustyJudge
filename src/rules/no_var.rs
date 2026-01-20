use rslint_parser::SyntaxKind;
use crate::rules::rule::{Rule, RuleContext};

pub struct NoVar;

impl Rule for NoVar {
    fn name(&self) -> &'static str {
        "no-var"
    }

    fn check(&self, rule_context: &RuleContext) -> bool {
        for node in rule_context.root.descendants() {
            if node.kind() == SyntaxKind::VAR_DECL {
                for child in node.children_with_tokens() {
                    if child.kind() == SyntaxKind::VAR_KW {
                        return true;
                    }
                }
            }
        }

        false
    }
}