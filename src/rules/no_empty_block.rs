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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_context;

    #[test]
    fn detects_simple_empty_block() {
        let ctx = make_context("{}", None);
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn detects_empty_block_with_whitespace() {
        let ctx = make_context("{   }", None);
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_non_empty_block() {
        let ctx = make_context("{ let x = 1; }", None);
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_multiple_empty_blocks() {
        let ctx = make_context(
            "
            {}
            function test() {}
            ",
            None
        );
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);
        
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_blocks_with_comments() {
        let ctx = make_context("{ /* comment */ }", None);
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);
        
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn detects_empty_block_in_if_statement() {
        let ctx = make_context("if (true) {}", None);
        let rule = NoEmptyBlock;

        let diagnostics = rule.check(&ctx);

        assert_eq!(diagnostics.len(), 1);
    }
}
