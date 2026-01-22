use rslint_parser::SyntaxKind;
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct StrictEquality;

impl Rule for StrictEquality {
    fn name(&self) -> &'static str {
        "strict-equality"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for element in ctx.root.descendants_with_tokens() {
            match element.kind() {
                SyntaxKind::EQ2 => {
                    diagnostics.push(
                        ctx.diagnostic_at(
                            element.text_range(),
                            "Use === instead of == for strict equality",
                        )
                    );
                }
                SyntaxKind::NEQ => {
                    diagnostics.push(
                        ctx.diagnostic_at(
                            element.text_range(),
                            "Use !== instead of != for strict equality",
                        )
                    );
                }
                _ => {}
            }
        }

        diagnostics
    }
}
