use rslint_parser::SyntaxKind;
use crate::{Rule, RuleContext};

pub struct StrictEquality;

impl Rule for StrictEquality {
    fn name(&self) -> &'static str {
        "strict-equality"
    }

    fn check(&self, rule_context: &RuleContext) -> bool {
        for element in rule_context.root.descendants_with_tokens() {
            if element.kind() == SyntaxKind::EQ2 {
                return true;
            }
            
            if element.kind() == SyntaxKind::NEQ {
                return true;
            }
        }
        
        false
    }
}