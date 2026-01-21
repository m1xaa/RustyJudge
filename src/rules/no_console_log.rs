use rslint_parser::SyntaxKind;
use crate::{Rule, RuleContext};

pub struct NoConsoleLog;

impl Rule for NoConsoleLog {
    fn name(&self) -> &'static str {
        "no-console-log"
    }

    fn check(&self, rule_context: &RuleContext) -> bool {
        for element in rule_context.root.descendants_with_tokens() {
            let token = match element.as_token() {
                Some(t) => t,
                None => continue,
            };
            
            if token.text() != "console" {
                continue;
            }
            
            let dot = match token.next_token() {
                Some(t) if t.text() == "." => t,
                _ => continue,
            };
            
            let log = match dot.next_token() {
                Some(t) if t.text() == "log" => t,
                _ => continue,
            };
            
            return true;
        }

        false
    }

}