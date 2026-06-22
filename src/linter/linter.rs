use std::error::Error;

use crate::cfg::build_semantic_model_from_root;
use crate::diagnostics::Diagnostic;
use crate::parser::parse_source;
use crate::rules::{Rule, RuleContext};

pub struct Linter {
    rules: Vec<Box<dyn Rule + Send + Sync>>,
}

impl Linter {
    pub fn with_rules(rules: Vec<Box<dyn Rule + Send + Sync>>) -> Self {
        Self { rules }
    }

    pub fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            diagnostics.extend(rule.check(ctx));
        }

        diagnostics
    }
}

pub fn lint_file(
    source: String,
    file_name: String,
    max_line_length: usize,
    linter: &Linter,
) -> Result<Vec<Diagnostic>, Box<dyn Error>> {
    let ast = parse_source(&source)?;
    let semantic = build_semantic_model_from_root(ast.clone()).ok();

    let context = RuleContext::new(
        file_name,
        ast,
        source,
        max_line_length,
        semantic,
    );

    Ok(linter.run(&context))
}