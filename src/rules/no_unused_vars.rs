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
