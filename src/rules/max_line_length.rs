use crate::{Rule, RuleContext};

pub struct MaxLineLength;

impl Rule for MaxLineLength {
    fn name(&self) -> &'static str {
        "max-line-length"
    }

    fn check(&self, rule_context: &RuleContext) -> bool {
        rule_context.source
            .lines()
            .any(|line| line.len() > rule_context.max_line_length)
    }
}