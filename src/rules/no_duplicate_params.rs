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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{make_context, RuleContext};
    use rslint_parser::parse_text;

    #[test]
    fn no_duplicate_params_ok() {
        let ctx = make_context("function test(a, b, c) {}", Option::None);
        let rule = NoDuplicateParams;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_single_duplicate() {
        let ctx = make_context("function test(a, b, a) {}", Option::None);
        let rule = NoDuplicateParams;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_multiple_duplicates() {
        let ctx = make_context("function test(a, b, a, b) {}", Option::None);
        let rule = NoDuplicateParams;

        let diagnostics = rule.check(&ctx);

        // drugi a + drugi b
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn duplicates_are_scoped_per_function() {
        let ctx = make_context(
            "
            function one(a, a) {}
            function two(a, b) {}
            ", 
            Option::None
        );
        let rule = NoDuplicateParams;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_non_identifier_tokens() {
        let ctx = make_context("function test(a = 1, b = 2) {}", Option::None);
        let rule = NoDuplicateParams;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }
}
