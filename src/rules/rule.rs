use rslint_parser::SyntaxNode;

pub trait Rule {
    fn name(&self) -> &'static str;
    fn check(&self, rule_context: &RuleContext) -> bool;  
    // for now return bool if not ok
    // will add diagnostics later
}

pub struct RuleContext {
    pub root: SyntaxNode,
    pub source: String
}

impl RuleContext {
    pub fn new(root: SyntaxNode, source: String) -> Self {
        RuleContext { root, source }
    }
}