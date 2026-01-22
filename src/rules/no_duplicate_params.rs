use std::collections::HashSet;
use rslint_parser::{SyntaxKind, SyntaxNodeExt};
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct NoDuplicateParams;

impl Rule for NoDuplicateParams {
    fn name(&self) -> &'static str {
        "no-duplicate-params"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for param_list in ctx.root.descendants() {
            if param_list.kind() != SyntaxKind::PARAMETER_LIST {
                continue;
            }

            let mut seen: HashSet<String> = HashSet::new();

            for token in param_list.tokens() {
                if token.kind() != SyntaxKind::IDENT {
                    continue;
                }

                let name = token.text().to_string();

                if !seen.insert(name.clone()) {
                    diagnostics.push(
                        ctx.diagnostic_at(
                            token.text_range(),
                            format!(
                                "Duplicate parameter name '{}' in function declaration",
                                name
                            ),
                        )
                    );
                }
            }
        }

        diagnostics
    }
}
