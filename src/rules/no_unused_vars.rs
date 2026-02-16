use std::collections::HashMap;
use rslint_parser::{SyntaxKind, SyntaxNodeExt, SyntaxToken};
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct NoUnusedVars;

impl Rule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let mut declared: HashMap<String, SyntaxToken> = HashMap::new();
        let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

        for node in ctx.root.descendants() {
            match node.kind() {
                SyntaxKind::NAME => {
                    let parent = match node.parent() {
                        Some(p) => p,
                        None => continue,
                    };

                    let name = node.text().to_string();

                    let is_decl = parent
                        .ancestors()
                        .any(|a| matches!(
                            a.kind(),
                            SyntaxKind::VAR_DECL | SyntaxKind::PARAMETER_LIST
                        ));

                    if is_decl {
                        declared.entry(name).or_insert_with(|| {
                            node.first_token().unwrap()
                        });
                    }
                }

                SyntaxKind::NAME_REF => {
                    let name = node.text().to_string();
                    used.insert(name);
                }

                _ => {}
            }
        }

        for (name, token) in declared {
            if !used.contains(&name) {
                diagnostics.push(
                    ctx.diagnostic_at(
                        token.text_range(),
                        format!("Unused variable '{}'", name),
                    )
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
    fn detects_unused_var() {
        let ctx = make_context("var x = 5;", None);
        let rule = NoUnusedVars;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn does_not_flag_used_var() {
        let ctx = make_context("
            var x = 5;
            console.log(x);
        ", None);

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_unused_vars() {
        let ctx = make_context("
            var a = 1;
            var b = 2;
            var c = 3;
            console.log(a);
        ", None);

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn parameters_count_as_declared() {
        let ctx = make_context("
            function test(a, b) {
                console.log(a);
            }
        ", None);

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn parameter_used_not_flagged() {
        let ctx = make_context("
            function test(a) {
                return a;
            }
        ", None);

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn usage_before_declaration_still_counts_as_used() {
        let ctx = make_context("
            console.log(x);
            var x = 5;
        ", None);

        let rule = NoUnusedVars;
        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }
}
