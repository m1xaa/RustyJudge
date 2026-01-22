use rslint_parser::{SyntaxKind, SyntaxNode, SyntaxNodeExt};
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct NoEmptyBlock;

impl Rule for NoEmptyBlock {
    fn name(&self) -> &'static str {
        "no-empty-block"
    }

    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for node in rule_context.root.descendants() {
            if node.kind() != SyntaxKind::BLOCK_STMT {
                continue;
            }

            let tokens = node.tokens();
            let mut tokens_iter = tokens.iter();

            let l_curly = match tokens_iter.next() {
                Some(t) if t.kind() == SyntaxKind::L_CURLY => t,
                _ => continue,
            };

            let next = loop {
                match tokens_iter.next() {
                    Some(t) if t.kind() == SyntaxKind::WHITESPACE => continue,
                    Some(t) => break t,
                    None => continue,
                }
            };

            if next.kind() != SyntaxKind::R_CURLY {
                continue;
            }

            diagnostics.push(rule_context.diagnostic_at(l_curly.text_range(), "Empty block of code"))
        }

        diagnostics
    }
}