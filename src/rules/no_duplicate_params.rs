use std::collections::HashSet;
use rslint_parser::{SyntaxKind, SyntaxNodeExt};
use crate::{Rule, RuleContext};

pub struct NoDuplicateParams;

impl Rule for NoDuplicateParams {
    fn name(&self) -> &'static str {
        "no-duplicate-params"
    }

    fn check(&self, rule_context: &RuleContext) -> bool {
        for node in rule_context.root.descendants() {
            if node.kind() != SyntaxKind::PARAMETER_LIST {
                continue;
            }

            let mut param_set: HashSet<String> = HashSet::new();

            for token in node.tokens() {
                if token.kind() != SyntaxKind::IDENT {
                    continue;
                }

                let name = token.text().to_string();

                if !param_set.insert(name) {
                    return true;
                }
            }
        }

        false
    }
}